//! Holds the evaluated decision for one URL and converts it to its receipt form.

use crate::decision_record::DecisionRecord;
use crate::outcome::Outcome;
use crate::policy::policy_rule::PolicyRule;
use crate::policy::signal_verdict::SignalVerdict;
use crate::purpose::Purpose;

/// Outcome, deciding rule, and the verdict of every consulted signal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    /// Allow or deny.
    e_outcome: Outcome,
    /// Rule that decided; `NoReservation` when allowed.
    e_rule: PolicyRule,
    /// One verdict per consulted signal, in observation order.
    vec_o_verdicts: Vec<SignalVerdict>,
    /// Purpose the decision is for.
    e_purpose: Purpose,
    /// Policy version label.
    str_policy_version: String,
}

impl Decision {
    /// Builds a decision: deny when any verdict denies, reporting the highest-precedence rule.
    pub fn from_verdicts(
        vec_o_verdicts: Vec<SignalVerdict>,
        e_purpose: Purpose,
        str_policy_version: &str,
    ) -> Self {
        let opt_e_rule = vec_o_verdicts
            .iter()
            .filter_map(SignalVerdict::denying_rule)
            .min();
        Self {
            e_outcome: if opt_e_rule.is_some() {
                Outcome::Deny
            } else {
                Outcome::Allow
            },
            e_rule: opt_e_rule.unwrap_or(PolicyRule::NoReservation),
            vec_o_verdicts,
            e_purpose,
            str_policy_version: str_policy_version.to_owned(),
        }
    }

    /// Returns allow or deny.
    pub fn outcome(&self) -> Outcome {
        self.e_outcome
    }

    /// Returns the deciding rule.
    pub fn rule(&self) -> PolicyRule {
        self.e_rule
    }

    /// Returns the verdict of every consulted signal.
    pub fn verdicts(&self) -> &[SignalVerdict] {
        &self.vec_o_verdicts
    }

    /// Returns one plain sentence per consulted signal, or one sentence when none was observed.
    pub fn reasons(&self) -> Vec<String> {
        if self.vec_o_verdicts.is_empty() {
            return vec![format!(
                "No opt-out signals were observed for this URL, so nothing reserves it against {}.",
                self.e_purpose.label()
            )];
        }
        self.vec_o_verdicts
            .iter()
            .map(|o_v| o_v.str_reason.clone())
            .collect()
    }

    /// Returns the receipt form of the decision.
    pub fn to_record(&self) -> DecisionRecord {
        DecisionRecord {
            e_outcome: self.e_outcome,
            str_matched_rule: self.e_rule.as_str().to_owned(),
            vec_str_reasons: self.reasons(),
            str_policy_version: self.str_policy_version.clone(),
        }
    }
}
