//! Parses TDMRep headers, meta elements, and `/.well-known/tdmrep.json`.

use serde_json::Value;

use crate::signal_kind::SignalKind;
use crate::signals::header_block::HeaderBlock;
use crate::signals::html_meta_extractor::HtmlMetaExtractor;
use crate::signals::parsed_signal::ParsedSignal;
use crate::signals::pattern_matcher::PatternMatcher;
use crate::signals::signal_context::SignalContext;
use crate::signals::signal_parser::SignalParser;
use crate::signals::tdmrep_finding::TdmrepFinding;
use crate::signals::tdmrep_source::TdmrepSource;

/// Largest tdmrep.json read, in bytes.
const U_MAX_TDMREP_JSON_BYTES: usize = 1024 * 1024;
/// Most tdmrep.json rules examined.
const U_MAX_TDMREP_ENTRIES: usize = 10_000;
/// Longest policy URL kept, in bytes.
const U_MAX_POLICY_URL_BYTES: usize = 2048;

/// TDMRep parser for one publication method. Only `1` and `0` are valid reservation values;
/// anything else is a protocol error and leaves the reservation unset. In tdmrep.json the
/// first rule whose location matches decides (the report: "the most specific match is the
/// first in sequence").
#[derive(Debug, Clone, Copy)]
pub struct TdmrepParser {
    /// Publication method this parser reads.
    e_source: TdmrepSource,
}

impl TdmrepParser {
    /// Creates a parser for one publication method.
    pub fn new(e_source: TdmrepSource) -> Self {
        Self { e_source }
    }

    /// Reads the `tdm-reservation` and `tdm-policy` headers from a header block.
    fn parse_header(&self, by_input: &[u8]) -> TdmrepFinding {
        let o_headers = HeaderBlock::parse(by_input);
        let opt_str_reservation = o_headers.values("tdm-reservation").first().copied();
        self.finding_from(
            opt_str_reservation,
            o_headers.values("tdm-policy").first().copied(),
        )
    }

    /// Reads the `tdm-reservation` and `tdm-policy` meta elements from HTML.
    fn parse_meta(&self, by_input: &[u8]) -> TdmrepFinding {
        let vec_o_metas = HtmlMetaExtractor::new().extract(by_input);
        let cb_first = |str_name: &str| {
            vec_o_metas
                .iter()
                .find(|o_m| o_m.0 == str_name)
                .map(|o_m| o_m.1.as_str())
        };
        self.finding_from(cb_first("tdm-reservation"), cb_first("tdm-policy"))
    }

    /// Returns the finding of the first tdmrep.json rule whose location matches the path.
    fn parse_well_known(&self, by_input: &[u8], str_path: &str) -> TdmrepFinding {
        let by_kept = by_input.get(..U_MAX_TDMREP_JSON_BYTES).unwrap_or(by_input);
        let Ok(Value::Array(vec_e_entries)) = serde_json::from_slice::<Value>(by_kept) else {
            return TdmrepFinding {
                u_protocol_errors: 1,
                ..TdmrepFinding::default()
            };
        };
        let mut u_errors = 0u32;
        for e_entry in vec_e_entries.iter().take(U_MAX_TDMREP_ENTRIES) {
            match e_entry.get("location").and_then(Value::as_str) {
                None => u_errors = u_errors.saturating_add(1),
                Some(str_loc) if PatternMatcher::new().matches(str_loc, str_path) => {
                    return self.finding_for_entry(e_entry, str_loc, u_errors);
                }
                Some(_) => {}
            }
        }
        TdmrepFinding {
            u_protocol_errors: u_errors,
            ..TdmrepFinding::default()
        }
    }

    /// Builds the finding for a matching tdmrep.json rule.
    fn finding_for_entry(
        &self,
        e_entry: &Value,
        str_location: &str,
        u_errors: u32,
    ) -> TdmrepFinding {
        let opt_b_reserved = match e_entry.get("tdm-reservation").and_then(Value::as_u64) {
            Some(1) => Some(true),
            Some(0) => Some(false),
            _ => None,
        };
        let opt_str_policy = e_entry.get("tdm-policy").and_then(Value::as_str);
        TdmrepFinding {
            opt_b_reserved,
            opt_str_policy_url: opt_str_policy.map(|str_p| self.cap(str_p)),
            opt_str_matched_location: Some(self.cap(str_location)),
            u_protocol_errors: u_errors.saturating_add(u32::from(opt_b_reserved.is_none())),
        }
    }

    /// Builds a finding from a header or meta reservation value and policy URL.
    fn finding_from(
        &self,
        opt_str_reservation: Option<&str>,
        opt_str_policy: Option<&str>,
    ) -> TdmrepFinding {
        let opt_b_reserved = match opt_str_reservation.map(str::trim) {
            Some("1") => Some(true),
            Some("0") => Some(false),
            _ => None,
        };
        let b_error = opt_str_reservation.is_some() && opt_b_reserved.is_none();
        TdmrepFinding {
            opt_b_reserved,
            opt_str_policy_url: opt_str_policy.map(|str_p| self.cap(str_p.trim())),
            opt_str_matched_location: None,
            u_protocol_errors: u32::from(b_error),
        }
    }

    /// Returns text cut to the policy URL length cap at a character boundary.
    fn cap(&self, str_text: &str) -> String {
        str_text[..str_text.floor_char_boundary(U_MAX_POLICY_URL_BYTES)].to_owned()
    }
}

impl SignalParser for TdmrepParser {
    /// Returns the signal kind for this parser's publication method.
    fn kind(&self) -> SignalKind {
        match self.e_source {
            TdmrepSource::Header => SignalKind::TdmrepHeader,
            TdmrepSource::Meta => SignalKind::TdmrepMeta,
            TdmrepSource::WellKnown => SignalKind::TdmrepWellKnown,
        }
    }

    /// Returns the TDMRep finding for the context's URL.
    fn parse(&self, by_input: &[u8], o_context: &SignalContext) -> ParsedSignal {
        let o_finding = match self.e_source {
            TdmrepSource::Header => self.parse_header(by_input),
            TdmrepSource::Meta => self.parse_meta(by_input),
            TdmrepSource::WellKnown => self.parse_well_known(by_input, o_context.path()),
        };
        ParsedSignal::Tdmrep(o_finding)
    }
}
