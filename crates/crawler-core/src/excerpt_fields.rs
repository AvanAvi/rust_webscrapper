//! Carries an excerpt's raw decoded fields before they are validated.

use serde::Deserialize;

/// Unvalidated excerpt fields as they appear in JSON.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExcerptFields {
    /// Excerpt text.
    #[serde(rename = "text")]
    pub(crate) str_text: String,
    /// Whether the text was cut.
    #[serde(rename = "truncated")]
    pub(crate) b_truncated: bool,
    /// UTF-8 length of the decoded text the excerpt was cut from.
    #[serde(rename = "text_bytes")]
    pub(crate) u_text_bytes: u64,
}
