//! Records the allow or deny decision and why it was made.

use serde::{Deserialize, Serialize};

use crate::outcome::Outcome;
use crate::policy::policy_rule::PolicyRule;
use crate::timestamp::Timestamp;

/// Decision taken before any page fetch, with the rule that decided it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRecord {
    /// Allow or deny.
    #[serde(rename = "outcome")]
    pub e_outcome: Outcome,
    /// Policy rule that decided the outcome; decoding rejects unknown rule ids.
    #[serde(rename = "matched_rule")]
    pub e_matched_rule: PolicyRule,
    /// Plain-language reasons, one per contributing signal.
    #[serde(rename = "reasons")]
    pub vec_str_reasons: Vec<String>,
    /// Version label of the policy that was applied.
    #[serde(rename = "policy_version")]
    pub str_policy_version: String,
    /// Worker time the decision was made; must be earlier than `fetch.started_at`.
    #[serde(rename = "decided_at")]
    pub dt_decided_at: Timestamp,
}
