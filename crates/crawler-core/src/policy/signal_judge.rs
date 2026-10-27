//! Applies policy v1 to one observed signal (docs/POLICY.md, needs legal review).

use crate::policy::policy_rule::PolicyRule;
use crate::policy::reason_writer::ReasonWriter;
use crate::policy::signal_verdict::SignalVerdict;
use crate::purpose::Purpose;
use crate::signal_kind::SignalKind;
use crate::signal_observation::SignalObservation;
use crate::signal_status::SignalStatus;
use crate::signals::ai_txt_finding::AiTxtFinding;
use crate::signals::parsed_signal::ParsedSignal;
use crate::signals::robots_finding::RobotsFinding;
use crate::signals::robots_tag_finding::RobotsTagFinding;
use crate::signals::tdmrep_finding::TdmrepFinding;

/// Decides what one signal means for a purpose. Only present signals can deny, with one
/// exception RFC 9309 requires: an unreachable robots.txt is a full disallow.
#[derive(Debug, Clone, Copy, Default)]
pub struct SignalJudge {
    /// Sentence builder for reasons.
    o_writer: ReasonWriter,
}

impl SignalJudge {
    /// Creates a judge.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the verdict of one observation for the purpose.
    pub fn judge(&self, o_obs: &SignalObservation, e_purpose: Purpose) -> SignalVerdict {
        match o_obs.e_status {
            SignalStatus::Absent => self.verdict(o_obs, None, self.o_writer.absent(o_obs)),
            SignalStatus::Unreachable => self.judge_unreachable(o_obs),
            SignalStatus::Present => self.judge_present(o_obs, e_purpose),
        }
    }

    /// Unreachable robots.txt denies everything; other unreachable sources have no effect.
    fn judge_unreachable(&self, o_obs: &SignalObservation) -> SignalVerdict {
        if o_obs.e_kind == SignalKind::RobotsTxt {
            let str_reason = self.o_writer.robots_unreachable(o_obs);
            return self.verdict(o_obs, Some(PolicyRule::RobotsUnreachable), str_reason);
        }
        self.verdict(o_obs, None, self.o_writer.unreachable(o_obs))
    }

    /// Dispatches a present signal to the rule for its finding.
    fn judge_present(&self, o_obs: &SignalObservation, e_purpose: Purpose) -> SignalVerdict {
        match &o_obs.opt_e_parsed {
            Some(ParsedSignal::RobotsTxt(o_found)) => self.judge_robots(o_obs, o_found, e_purpose),
            Some(ParsedSignal::AiTxt(o_found)) => self.judge_ai_txt(o_obs, o_found, e_purpose),
            Some(ParsedSignal::Tdmrep(o_found)) => self.judge_tdmrep(o_obs, o_found, e_purpose),
            Some(ParsedSignal::RobotsTag(o_found)) => {
                self.judge_robots_tag(o_obs, o_found, e_purpose)
            }
            None => self.verdict(o_obs, None, self.o_writer.unparsed(o_obs)),
        }
    }

    /// robots.txt: a disallow denies every purpose; a Content-Signal `no` denies its purpose.
    fn judge_robots(
        &self,
        o_obs: &SignalObservation,
        o_found: &RobotsFinding,
        e_purpose: Purpose,
    ) -> SignalVerdict {
        if !o_found.b_allowed {
            let str_reason = self.o_writer.robots_disallow(o_obs, o_found);
            return self.verdict(o_obs, Some(PolicyRule::RobotsDisallow), str_reason);
        }
        let opt_b_signal = o_found
            .opt_o_content_signal
            .and_then(|o_s| o_s.for_purpose(e_purpose));
        if opt_b_signal == Some(false) {
            let str_reason = self.o_writer.content_signal_no(o_obs, o_found, e_purpose);
            return self.verdict(o_obs, Some(PolicyRule::ContentSignalNo), str_reason);
        }
        self.verdict(
            o_obs,
            None,
            self.o_writer.robots_allow(o_obs, o_found, e_purpose),
        )
    }

    /// ai.txt: a disallow is a reservation against AI training and AI input.
    fn judge_ai_txt(
        &self,
        o_obs: &SignalObservation,
        o_found: &AiTxtFinding,
        e_purpose: Purpose,
    ) -> SignalVerdict {
        if !o_found.b_reserved {
            return self.verdict(o_obs, None, self.o_writer.no_reservation(o_obs));
        }
        if !e_purpose.is_ai_use() {
            return self.verdict(o_obs, None, self.o_writer.not_for_purpose(o_obs, e_purpose));
        }
        let str_reason = self.o_writer.ai_txt_disallow(o_obs, o_found, e_purpose);
        self.verdict(o_obs, Some(PolicyRule::AiTxtDisallow), str_reason)
    }

    /// TDMRep: `tdm-reservation: 1` reserves against AI training and AI input.
    fn judge_tdmrep(
        &self,
        o_obs: &SignalObservation,
        o_found: &TdmrepFinding,
        e_purpose: Purpose,
    ) -> SignalVerdict {
        if o_found.opt_b_reserved != Some(true) {
            return self.verdict(o_obs, None, self.o_writer.no_reservation(o_obs));
        }
        if !e_purpose.is_ai_use() {
            return self.verdict(o_obs, None, self.o_writer.not_for_purpose(o_obs, e_purpose));
        }
        let str_reason = self.o_writer.tdmrep_reserved(o_obs, o_found, e_purpose);
        self.verdict(o_obs, Some(PolicyRule::TdmrepReserved), str_reason)
    }

    /// X-Robots-Tag and robots meta: `noai` or `noimageai` reserve against AI uses.
    fn judge_robots_tag(
        &self,
        o_obs: &SignalObservation,
        o_found: &RobotsTagFinding,
        e_purpose: Purpose,
    ) -> SignalVerdict {
        let opt_o_hit = match (o_found.b_noai, o_found.b_noimageai) {
            (true, _) => Some((PolicyRule::RobotsTagNoai, "noai")),
            (false, true) => Some((PolicyRule::RobotsTagNoimageai, "noimageai")),
            (false, false) => None,
        };
        let Some((e_rule, str_directive)) = opt_o_hit else {
            return self.verdict(o_obs, None, self.o_writer.no_reservation(o_obs));
        };
        if !e_purpose.is_ai_use() {
            return self.verdict(o_obs, None, self.o_writer.not_for_purpose(o_obs, e_purpose));
        }
        let str_reason = self
            .o_writer
            .robots_tag_deny(o_obs, str_directive, e_purpose);
        self.verdict(o_obs, Some(e_rule), str_reason)
    }

    /// Builds a verdict for the observation's kind.
    fn verdict(
        &self,
        o_obs: &SignalObservation,
        opt_e_rule: Option<PolicyRule>,
        str_reason: String,
    ) -> SignalVerdict {
        SignalVerdict {
            e_kind: o_obs.e_kind,
            opt_e_rule,
            str_reason,
        }
    }
}
