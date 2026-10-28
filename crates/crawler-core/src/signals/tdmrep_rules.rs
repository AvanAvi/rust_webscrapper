//! Holds a parsed `/.well-known/tdmrep.json` so it can be cached and evaluated per path.

use serde_json::Value;

use crate::signals::pattern_matcher::PatternMatcher;
use crate::signals::tdmrep_finding::TdmrepFinding;

/// Largest tdmrep.json read, in bytes.
const U_MAX_TDMREP_JSON_BYTES: usize = 1024 * 1024;
/// Most tdmrep.json rules kept.
const U_MAX_TDMREP_ENTRIES: usize = 10_000;
/// Longest location or policy URL kept, in bytes.
const U_MAX_FIELD_BYTES: usize = 2048;

/// One tdmrep.json entry; a missing location makes the entry a protocol error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TdmrepEntry {
    /// Location pattern, if the entry has one.
    pub opt_str_location: Option<String>,
    /// `Some(true)` for 1, `Some(false)` for 0, `None` for any other value.
    pub opt_b_reserved: Option<bool>,
    /// Policy URL, if given.
    pub opt_str_policy_url: Option<String>,
}

/// Entries of one tdmrep.json in file order, or a marker that the file was not a JSON array.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TdmrepRules {
    /// Entries in order.
    vec_o_entries: Vec<TdmrepEntry>,
    /// Whether the file was not a JSON array at all.
    b_unparseable: bool,
}

impl TdmrepRules {
    /// Parses a tdmrep.json body; never fails, malformed input is recorded.
    pub fn parse(by_input: &[u8]) -> Self {
        let by_kept = by_input.get(..U_MAX_TDMREP_JSON_BYTES).unwrap_or(by_input);
        let Ok(Value::Array(vec_e_values)) = serde_json::from_slice::<Value>(by_kept) else {
            return Self {
                vec_o_entries: Vec::new(),
                b_unparseable: true,
            };
        };
        let vec_o_entries = vec_e_values
            .iter()
            .take(U_MAX_TDMREP_ENTRIES)
            .map(Self::entry_of)
            .collect();
        Self {
            vec_o_entries,
            b_unparseable: false,
        }
    }

    /// Returns an estimate of the heap bytes the entries hold.
    pub fn heap_bytes(&self) -> usize {
        self.vec_o_entries
            .iter()
            .map(|o_entry| {
                let u_location = o_entry.opt_str_location.as_ref().map_or(0, String::len);
                let u_policy = o_entry.opt_str_policy_url.as_ref().map_or(0, String::len);
                size_of::<TdmrepEntry>() + u_location + u_policy
            })
            .sum()
    }

    /// Returns the finding of the first entry whose location matches the path.
    pub fn evaluate(&self, str_path: &str) -> TdmrepFinding {
        if self.b_unparseable {
            return TdmrepFinding {
                u_protocol_errors: 1,
                ..TdmrepFinding::default()
            };
        }
        let o_matcher = PatternMatcher::new();
        let mut u_errors = 0u32;
        for o_entry in &self.vec_o_entries {
            let Some(str_location) = &o_entry.opt_str_location else {
                u_errors = u_errors.saturating_add(1);
                continue;
            };
            if o_matcher.matches(str_location, str_path) {
                return Self::finding(o_entry, str_location, u_errors);
            }
        }
        TdmrepFinding {
            u_protocol_errors: u_errors,
            ..TdmrepFinding::default()
        }
    }

    /// Reads one JSON value as an entry; only integer 0 and 1 are valid reservations.
    fn entry_of(e_value: &Value) -> TdmrepEntry {
        let opt_b_reserved = match e_value.get("tdm-reservation").and_then(Value::as_u64) {
            Some(1) => Some(true),
            Some(0) => Some(false),
            _ => None,
        };
        TdmrepEntry {
            opt_str_location: e_value
                .get("location")
                .and_then(Value::as_str)
                .map(Self::cap),
            opt_b_reserved,
            opt_str_policy_url: e_value
                .get("tdm-policy")
                .and_then(Value::as_str)
                .map(Self::cap),
        }
    }

    /// Builds the finding for a matching entry; an invalid reservation counts as an error.
    fn finding(o_entry: &TdmrepEntry, str_location: &str, u_errors: u32) -> TdmrepFinding {
        TdmrepFinding {
            opt_b_reserved: o_entry.opt_b_reserved,
            opt_str_policy_url: o_entry.opt_str_policy_url.clone(),
            opt_str_matched_location: Some(str_location.to_owned()),
            u_protocol_errors: u_errors.saturating_add(u32::from(o_entry.opt_b_reserved.is_none())),
        }
    }

    /// Returns text cut to the field cap at a character boundary.
    fn cap(str_text: &str) -> String {
        str_text[..str_text.floor_char_boundary(U_MAX_FIELD_BYTES)].to_owned()
    }
}
