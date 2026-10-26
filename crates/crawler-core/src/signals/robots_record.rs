//! Holds one `key: value` line of a robots-style file.

/// One record with its key lowercased and both parts trimmed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobotsRecord {
    /// Lowercased record key, such as `user-agent` or `disallow`.
    pub str_key: String,
    /// Trimmed value with any comment removed.
    pub str_value: String,
}
