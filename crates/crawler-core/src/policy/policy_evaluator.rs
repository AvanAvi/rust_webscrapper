//! Evaluates observed signals into a crawl decision for one purpose.

use crate::policy::decision::Decision;
use crate::policy::signal_judge::SignalJudge;
use crate::policy::signal_verdict::SignalVerdict;
use crate::purpose::Purpose;
use crate::signals::signal_set::SignalSet;

/// Version label of the signal semantics in docs/POLICY.md.
pub const STR_POLICY_VERSION: &str = "v1";

/// Evaluates observed signals into a crawl decision for one purpose. Deny wins: any denying
/// signal denies, and every consulted signal is listed in the decision.
#[derive(Debug, Clone)]
pub struct PolicyEvaluator {
    /// Purpose this crawler declares for the whole job.
    e_purpose: Purpose,
    /// Version label stored in every decision.
    str_policy_version: String,
    /// Per-signal rule application.
    o_judge: SignalJudge,
}

impl PolicyEvaluator {
    /// Creates an evaluator for a purpose and policy version.
    pub fn new(e_purpose: Purpose, str_policy_version: String) -> Self {
        Self {
            e_purpose,
            str_policy_version,
            o_judge: SignalJudge::new(),
        }
    }

    /// Creates an evaluator for a purpose using the current policy version.
    pub fn for_purpose(e_purpose: Purpose) -> Self {
        Self::new(e_purpose, STR_POLICY_VERSION.to_owned())
    }

    /// Returns the decision for one URL given its observed signals.
    pub fn evaluate(&self, o_signals: &SignalSet) -> Decision {
        let vec_o_verdicts: Vec<SignalVerdict> = o_signals
            .observations()
            .iter()
            .map(|o_obs| self.o_judge.judge(o_obs, self.e_purpose))
            .collect();
        Decision::from_verdicts(vec_o_verdicts, self.e_purpose, &self.str_policy_version)
    }
}
