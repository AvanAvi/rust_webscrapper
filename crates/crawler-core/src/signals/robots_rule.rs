//! Holds one allow or disallow rule.

use serde::{Deserialize, Serialize};

use crate::signals::rule_kind::RuleKind;

/// One rule exactly as written in the file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RobotsRule {
    /// Allow or disallow.
    #[serde(rename = "kind")]
    pub e_kind: RuleKind,
    /// Path pattern as written, with `*` and `$` wildcards.
    #[serde(rename = "pattern")]
    pub str_pattern: String,
}
