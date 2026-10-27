//! Holds what one observed signal means for the declared purpose.

use crate::policy::policy_rule::PolicyRule;
use crate::signal_kind::SignalKind;

/// Effect of one signal on the decision, with a plain-language sentence explaining it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalVerdict {
    /// Kind of signal judged.
    pub e_kind: SignalKind,
    /// Rule this signal denies under, or `None` when it does not deny.
    pub opt_e_rule: Option<PolicyRule>,
    /// Sentence for a person: which signal, from where, and what it means for the purpose.
    pub str_reason: String,
}

impl SignalVerdict {
    /// Returns the denying rule, if this signal denies.
    pub fn denying_rule(&self) -> Option<PolicyRule> {
        self.opt_e_rule
    }
}
