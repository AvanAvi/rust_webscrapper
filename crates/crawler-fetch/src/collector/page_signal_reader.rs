//! Reads page-level signals from a fetched page: X-Robots-Tag, robots meta, TDMRep header and meta.

use crawler_core::signals::html_meta_extractor::HtmlMetaExtractor;
use crawler_core::{
    Excerpt, ParsedSignal, RobotsTagParser, RobotsTagSource, SignalContext, SignalKind,
    SignalObservation, SignalParser, SignalStatus, TdmrepParser, TdmrepSource, Timestamp,
};

use crate::fetch_response::FetchResponse;

/// One page-level source: kind, whether it exists on the page, and what it parsed to.
struct PageSource {
    /// Signal kind.
    e_kind: SignalKind,
    /// Whether the header or meta element exists.
    b_present: bool,
    /// Parser output.
    e_parsed: ParsedSignal,
    /// Matching header lines, for header kinds.
    opt_o_excerpt: Option<Excerpt>,
}

/// Turns one page response into four observations. Headers are read from all response headers
/// (repeated X-Robots-Tag headers are merged); meta elements from the capped body only.
#[derive(Debug, Clone)]
pub struct PageSignalReader {
    /// Product token for agent-scoped directives.
    str_product_token: String,
}

impl PageSignalReader {
    /// Creates a reader for a product token.
    pub fn new(str_product_token: &str) -> Self {
        Self {
            str_product_token: str_product_token.to_owned(),
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

    /// Parses the four page-level sources of a response.
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
        let o_robots = self.header_source(
            o_response,
            SignalKind::XRobotsTag,
            &["x-robots-tag"],
            o_context,
        );
        let vec_str_tdm = ["tdm-reservation", "tdm-policy"];
        let o_tdm = self.header_source(
            o_response,
            SignalKind::TdmrepHeader,
            &vec_str_tdm,
            o_context,
        );
        vec![o_robots, o_tdm]
    }

    /// Returns the robots meta and TDMRep meta sources.
    fn meta_sources(
        &self,
        o_response: &FetchResponse,
        o_context: &SignalContext,
    ) -> Vec<PageSource> {
        let vec_str_names: Vec<String> = HtmlMetaExtractor::new()
            .extract(&o_response.by_body)
            .into_iter()
            .map(|o_m| o_m.0)
            .collect();
        let o_robots = self.meta_source(
            o_response,
            SignalKind::MetaRobots,
            &vec_str_names,
            o_context,
        );
        let o_tdm = self.meta_source(
            o_response,
            SignalKind::TdmrepMeta,
            &vec_str_names,
            o_context,
        );
        vec![o_robots, o_tdm]
    }

    /// Parses a header-based source from the merged header block.
    fn header_source(
        &self,
        o_response: &FetchResponse,
        e_kind: SignalKind,
        vec_str_names: &[&str],
        o_context: &SignalContext,
    ) -> PageSource {
        let vec_str_lines: Vec<String> = o_response
            .vec_o_headers
            .iter()
            .filter(|o_h| vec_str_names.contains(&o_h.0.as_str()))
            .map(|(str_n, str_v)| format!("{str_n}: {str_v}"))
            .collect();
        let by_block = o_response.header_block().into_bytes();
        let e_parsed = self.parser_for(e_kind).parse(&by_block, o_context);
        let opt_o_excerpt =
            (!vec_str_lines.is_empty()).then(|| Excerpt::new(&vec_str_lines.join("\n")));
        PageSource {
            e_kind,
            b_present: !vec_str_lines.is_empty(),
            e_parsed,
            opt_o_excerpt,
        }
    }

    /// Parses a meta-based source from the capped page body.
    fn meta_source(
        &self,
        o_response: &FetchResponse,
        e_kind: SignalKind,
        vec_str_meta_names: &[String],
        o_context: &SignalContext,
    ) -> PageSource {
        let str_token = self.str_product_token.to_ascii_lowercase();
        let b_present = vec_str_meta_names.iter().any(|str_name| match e_kind {
            SignalKind::MetaRobots => str_name == "robots" || *str_name == str_token,
            _ => str_name == "tdm-reservation" || str_name == "tdm-policy",
        });
        let e_parsed = self
            .parser_for(e_kind)
            .parse(&o_response.by_body, o_context);
        PageSource {
            e_kind,
            b_present,
            e_parsed,
            opt_o_excerpt: None,
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

    /// Builds the observation for one source.
    fn observation(
        &self,
        o_source: PageSource,
        o_response: &FetchResponse,
        dt_observed_at: &Timestamp,
    ) -> SignalObservation {
        SignalObservation {
            e_kind: o_source.e_kind,
            e_status: if o_source.b_present {
                SignalStatus::Present
            } else {
                SignalStatus::Absent
            },
            str_source_url: o_response.str_final_url.clone(),
            dt_observed_at: dt_observed_at.clone(),
            opt_u_http_status: Some(o_response.u_status),
            opt_str_content_sha256: None,
            opt_o_excerpt: o_source.opt_o_excerpt,
            opt_str_final_url: Some(o_response.str_final_url.clone()),
            vec_o_redirects: o_response.vec_o_redirects.clone(),
            opt_e_failure: None,
            opt_e_parsed: o_source.b_present.then_some(o_source.e_parsed),
        }
    }
}
