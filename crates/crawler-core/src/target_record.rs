//! Records which URL a receipt is about.

use serde::{Deserialize, Serialize};

/// URL forms of the receipt's target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetRecord {
    /// URL exactly as it was requested or discovered.
    #[serde(rename = "url_requested")]
    pub str_url_requested: String,
    /// Normalized URL used as the deduplication key.
    #[serde(rename = "url_normalized")]
    pub str_url_normalized: String,
    /// URL after redirects; `None` when nothing was fetched.
    #[serde(rename = "url_final", deserialize_with = "Option::deserialize")]
    pub opt_str_url_final: Option<String>,
    /// Host the decision applies to.
    #[serde(rename = "host")]
    pub str_host: String,
    /// Link distance from the job's seeds (seeds are 0); checks depth and scope limits.
    #[serde(rename = "depth")]
    pub u_depth: u32,
}
