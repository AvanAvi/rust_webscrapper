//! Lists the possible crawl decisions.

use serde::{Deserialize, Serialize};

/// Whether a URL may be fetched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// The page may be fetched.
    Allow,
    /// The page must not be fetched.
    Deny,
}
