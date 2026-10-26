//! Records what ai.txt says about one URL.

use serde::{Deserialize, Serialize};

use crate::signals::robots_rule::RobotsRule;

/// The deciding ai.txt rule; a disallow rule is a reservation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiTxtFinding {
    /// Whether a disallow rule covers the page.
    #[serde(rename = "reserved")]
    pub b_reserved: bool,
    /// User-agent group that applied: the product token, `*`, or `None`.
    #[serde(rename = "matched_agent", deserialize_with = "Option::deserialize")]
    pub opt_str_matched_agent: Option<String>,
    /// Rule that decided, or `None` when no rule matched.
    #[serde(rename = "matched_rule", deserialize_with = "Option::deserialize")]
    pub opt_o_matched_rule: Option<RobotsRule>,
    /// Page extension assumed to match a file-type rule, when the path alone matched nothing.
    #[serde(rename = "assumed_extension", deserialize_with = "Option::deserialize")]
    pub opt_str_assumed_extension: Option<String>,
    /// Lines and records that could not be used.
    #[serde(rename = "invalid_lines")]
    pub u_invalid_lines: u32,
    /// Whether the file exceeded the parsing limit.
    #[serde(rename = "truncated")]
    pub b_truncated: bool,
}
