//! Records the allow or deny decision and why it was made.

use serde::{Deserialize, Serialize};

use crate::outcome::Outcome;

/// Decision taken before any page fetch, with the rule that decided it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRecord {
    /// Allow or deny.
    #[serde(rename = "outcome")]
    pub e_outcome: Outcome,
    /// Identifier of the policy rule that decided the outcome.
    #[serde(rename = "matched_rule")]
    pub str_matched_rule: String,
    /// Plain-language reasons, one per contributing signal.
    #[serde(rename = "reasons")]
    pub vec_str_reasons: Vec<String>,
    /// Version label of the policy that was applied.
    #[serde(rename = "policy_version")]
    pub str_policy_version: String,
}
