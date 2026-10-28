//! Reads page-level signals from a fetched page: X-Robots-Tag, robots meta, TDMRep header and meta.

use std::sync::Arc;

use crawler_core::signals::html_meta_extractor::HtmlMetaExtractor;
use crawler_core::{
    BodyArchive, CanonicalHeaders, Excerpt, Hasher, ParsedSignal, RobotsTagParser, RobotsTagSource,
    SignalContext, SignalFailure, SignalKind, SignalObservation, SignalParser, SignalStatus,
    TdmrepParser, TdmrepSource, Timestamp,
};

use crate::fetch_response::FetchResponse;

/// Header names read for X-Robots-Tag.
const VEC_STR_ROBOTS_HEADERS: [&str; 1] = ["x-robots-tag"];
/// Header names read for TDMRep.
const VEC_STR_TDMREP_HEADERS: [&str; 2] = ["tdm-reservation", "tdm-policy"];

/// One page-level source: kind, whether it exists on the page, and what it parsed to.
struct PageSource {
    /// Signal kind.
    e_kind: SignalKind,
    /// Whether the header or meta element exists.
    b_present: bool,
    /// Parser output.
    e_parsed: ParsedSignal,
    /// Canonical header lines, for header kinds.
    opt_o_excerpt: Option<Excerpt>,
    /// Hash of the exact bytes the parser read.
    opt_str_sha256: Option<String>,
    /// `EvidenceNotArchived` when the header block could not be stored.
    opt_e_failure: Option<SignalFailure>,
}

/// Turns one page response into four observations.
///
/// Header kinds are parsed from the canonical text of their headers (`CanonicalHeaders`), which
/// is archived; `content_sha256` resolves to exactly those bytes. If the archive refuses them,
/// the kind is unreachable with `evidence_not_archived` (fail closed). Meta kinds are parsed from
/// the capped page body; their `content_sha256` is the page body hash (`fetch.body_sha256`), which
/// resolves once page bodies are stored (F7/F9). The archive reference of a header block is
/// released at once, like an uncached signal file, until receipts hold references (F6).
pub struct PageSignalReader {
    /// Product token for agent-scoped directives.
    str_product_token: String,
    /// Content-addressed store for header blocks.
    o_archive: Arc<dyn BodyArchive>,
}

impl PageSignalReader {
    /// Creates a reader for a product token.
    pub fn new(str_product_token: &str, o_archive: Arc<dyn BodyArchive>) -> Self {
        Self {
            str_product_token: str_product_token.to_owned(),
            o_archive,
        }
    }

    /// Returns the X-Robots-Tag, robots meta, TDMRep header, and TDMRep meta observations.
    pub fn observations(
        &self,
        o_response: &FetchResponse,
        dt_observed_at: Timestamp,
    ) -> Vec<SignalObservation> {
        self.sources(o_response)
            .into_iter()
            .map(|o_source| self.observation(o_source, o_response, &dt_observed_at))
            .collect()
    }

    /// Parses the four page-level sources of a response: the two header kinds, then the two
    /// meta kinds.
    fn sources(&self, o_response: &FetchResponse) -> Vec<PageSource> {
        let str_path = url::Url::parse(&o_response.str_final_url)
            .map(|url_f| url_f.path().to_owned())
            .unwrap_or_default();
        let o_context = SignalContext::new(&self.str_product_token, &str_path);
        let mut vec_o_sources = self.header_sources(o_response, &o_context);
        vec_o_sources.extend(self.meta_sources(o_response, &o_context));
        vec_o_sources
    }

    /// Returns the X-Robots-Tag and TDMRep header sources.
    fn header_sources(
        &self,
        o_response: &FetchResponse,
        o_context: &SignalContext,
    ) -> Vec<PageSource> {
        vec![
            self.header_source(
                o_response,
                SignalKind::XRobotsTag,
                &VEC_STR_ROBOTS_HEADERS,
                o_context,
            ),
            self.header_source(
                o_response,
                SignalKind::TdmrepHeader,
                &VEC_STR_TDMREP_HEADERS,
                o_context,
            ),
        ]
    }

    /// Returns the robots meta and TDMRep meta sources.
    fn meta_sources(
        &self,
        o_response: &FetchResponse,
        o_context: &SignalContext,
    ) -> Vec<PageSource> {
        let str_body_sha256 = Hasher::new().sha256_hex(&o_response.by_body);
        let vec_str_names: Vec<String> = HtmlMetaExtractor::new()
            .extract(&o_response.by_body)
            .into_iter()
            .map(|o_m| o_m.0)
            .collect();
        [SignalKind::MetaRobots, SignalKind::TdmrepMeta]
            .into_iter()
            .map(|e_kind| {
                self.meta_source(
                    o_response,
                    (e_kind, &str_body_sha256),
                    &vec_str_names,
                    o_context,
                )
            })
            .collect()
    }

    /// Parses a header-based source from its canonical header text and archives that text.
    fn header_source(
        &self,
        o_response: &FetchResponse,
        e_kind: SignalKind,
        vec_str_names: &[&str],
        o_context: &SignalContext,
    ) -> PageSource {
        let str_block = CanonicalHeaders::new().render(&o_response.vec_o_headers, vec_str_names);
        let b_present = !str_block.is_empty();
        let o_source = PageSource {
            e_kind,
            b_present,
            e_parsed: self
                .parser_for(e_kind)
                .parse(str_block.as_bytes(), o_context),
            opt_o_excerpt: b_present.then(|| Excerpt::new(&str_block)),
            opt_str_sha256: None,
            opt_e_failure: None,
        };
        self.archived(o_source, str_block.as_bytes())
    }

    /// Stores the bytes a header source was parsed from, failing closed when that is refused.
    fn archived(&self, o_source: PageSource, by_block: &[u8]) -> PageSource {
        match self.o_archive.store(by_block) {
            Ok(str_sha256) => {
                self.o_archive.release(&str_sha256);
                PageSource {
                    opt_str_sha256: Some(str_sha256),
                    ..o_source
                }
            }
            Err(_) => PageSource {
                opt_e_failure: Some(SignalFailure::EvidenceNotArchived),
                opt_o_excerpt: None,
                ..o_source
            },
        }
    }

    /// Parses a meta-based source from the capped page body, whose hash is the evidence.
    fn meta_source(
        &self,
        o_response: &FetchResponse,
        o_kind_hash: (SignalKind, &str),
        vec_str_meta_names: &[String],
        o_context: &SignalContext,
    ) -> PageSource {
        let (e_kind, str_body_sha256) = o_kind_hash;
        let str_token = self.str_product_token.to_ascii_lowercase();
        let b_present = vec_str_meta_names.iter().any(|str_name| match e_kind {
            SignalKind::MetaRobots => str_name == "robots" || *str_name == str_token,
            _ => str_name == "tdm-reservation" || str_name == "tdm-policy",
        });
        PageSource {
            e_kind,
            b_present,
            e_parsed: self
                .parser_for(e_kind)
                .parse(&o_response.by_body, o_context),
            opt_o_excerpt: None,
            opt_str_sha256: Some(str_body_sha256.to_owned()),
            opt_e_failure: None,
        }
    }

    /// Returns the F2 parser for a page-level kind.
    fn parser_for(&self, e_kind: SignalKind) -> Box<dyn SignalParser> {
        match e_kind {
            SignalKind::XRobotsTag => Box::new(RobotsTagParser::new(RobotsTagSource::Header)),
            SignalKind::MetaRobots => Box::new(RobotsTagParser::new(RobotsTagSource::Meta)),
            SignalKind::TdmrepHeader => Box::new(TdmrepParser::new(TdmrepSource::Header)),
            _ => Box::new(TdmrepParser::new(TdmrepSource::Meta)),
        }
    }

    /// Returns the status of a source: unreachable when its evidence was not archived.
    fn status(&self, o_source: &PageSource) -> SignalStatus {
        match (o_source.opt_e_failure, o_source.b_present) {
            (Some(_), _) => SignalStatus::Unreachable,
            (None, true) => SignalStatus::Present,
            (None, false) => SignalStatus::Absent,
        }
    }

    /// Builds the observation for one source.
    fn observation(
        &self,
        o_source: PageSource,
        o_response: &FetchResponse,
        dt_observed_at: &Timestamp,
    ) -> SignalObservation {
        let e_status = self.status(&o_source);
        SignalObservation {
            e_kind: o_source.e_kind,
            e_status,
            str_source_url: o_response.str_final_url.clone(),
            dt_observed_at: dt_observed_at.clone(),
            opt_u_http_status: Some(o_response.u_status),
            opt_str_content_sha256: o_source.opt_str_sha256,
            opt_o_excerpt: o_source.opt_o_excerpt,
            opt_str_final_url: Some(o_response.str_final_url.clone()),
            vec_o_redirects: o_response.vec_o_redirects.clone(),
            b_body_truncated: o_response.b_truncated,
            u_body_bytes_seen: o_response.u_body_bytes_seen,
            opt_e_failure: o_source.opt_e_failure,
            opt_e_parsed: (e_status == SignalStatus::Present).then_some(o_source.e_parsed),
        }
    }
}
