//! Lists the two robots.txt rule kinds.

use serde::{Deserialize, Serialize};

/// Whether a rule allows or disallows matching paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleKind {
    /// `Allow:` rule.
    Allow,
    /// `Disallow:` rule.
    Disallow,
}
