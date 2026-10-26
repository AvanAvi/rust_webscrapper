//! Records the page-level robots directives that apply to this crawler.

use serde::{Deserialize, Serialize};

/// Directives that apply to this crawler, lowercased, sorted, and unique.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RobotsTagFinding {
    /// All applicable directives, such as `noindex` or `max-snippet: 20`.
    #[serde(rename = "directives")]
    pub vec_str_directives: Vec<String>,
    /// Whether `noai` applies.
    #[serde(rename = "noai")]
    pub b_noai: bool,
    /// Whether `noimageai` applies.
    #[serde(rename = "noimageai")]
    pub b_noimageai: bool,
}
