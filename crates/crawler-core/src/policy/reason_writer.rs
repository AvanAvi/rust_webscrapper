//! Writes the plain-language sentences recorded as decision reasons.

use crate::purpose::Purpose;
use crate::signal_kind::SignalKind;
use crate::signal_observation::SignalObservation;
use crate::signals::ai_txt_finding::AiTxtFinding;
use crate::signals::content_signal::ContentSignal;
use crate::signals::robots_finding::RobotsFinding;
use crate::signals::tdmrep_finding::TdmrepFinding;

/// Builds one sentence per consulted signal; sentences name the signal, where it was read,
/// and what it means for the declared purpose.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReasonWriter;

impl ReasonWriter {
    /// Creates a writer.
    pub fn new() -> Self {
        Self
    }

    /// Describes a robots.txt rule that disallows the URL.
    pub fn robots_disallow(&self, o_obs: &SignalObservation, o_found: &RobotsFinding) -> String {
        let str_rule = o_found
            .opt_o_matched_rule
            .as_ref()
            .map_or_else(String::new, |o_r| {
                format!(" (rule \"Disallow: {}\")", o_r.str_pattern)
            });
        format!(
            "{} disallows this URL for {}{str_rule}, so it may not be crawled for any purpose.",
            self.source(o_obs),
            self.group(o_found)
        )
    }

    /// Describes a robots.txt that could not be retrieved.
    pub fn robots_unreachable(&self, o_obs: &SignalObservation) -> String {
        format!(
            "{} could not be retrieved ({}), so the whole site is treated as disallowed for this crawl, as RFC 9309 requires.",
            self.source(o_obs),
            self.status(o_obs)
        )
    }

    /// Describes a Content-Signal `no` for the purpose.
    pub fn content_signal_no(
        &self,
        o_obs: &SignalObservation,
        o_found: &RobotsFinding,
        e_purpose: Purpose,
    ) -> String {
        let str_key = ContentSignal::key_for(e_purpose).unwrap_or("none");
        format!(
            "{} sets Content-Signal {str_key}=no for {}, which denies {}.",
            self.source(o_obs),
            self.group(o_found),
            e_purpose.label()
        )
    }

    /// Describes a robots.txt that allows the URL and raises no Content-Signal objection.
    pub fn robots_allow(
        &self,
        o_obs: &SignalObservation,
        o_found: &RobotsFinding,
        e_purpose: Purpose,
    ) -> String {
        format!(
            "{} allows this URL for {} and states no Content-Signal objection to {}.",
            self.source(o_obs),
            self.group(o_found),
            e_purpose.label()
        )
    }

    /// Describes an ai.txt rule that reserves the URL against an AI purpose.
    pub fn ai_txt_disallow(
        &self,
        o_obs: &SignalObservation,
        o_found: &AiTxtFinding,
        e_purpose: Purpose,
    ) -> String {
        let str_rule = o_found
            .opt_o_matched_rule
            .as_ref()
            .map_or_else(String::new, |o_r| {
                format!(" (rule \"Disallow: {}\")", o_r.str_pattern)
            });
        let str_assumed = o_found
            .opt_str_assumed_extension
            .as_ref()
            .map_or_else(String::new, |str_e| {
                format!(", assuming the page is .{str_e}")
            });
        format!(
            "{} disallows this URL{str_rule}{str_assumed}, which denies {}.",
            self.source(o_obs),
            e_purpose.label()
        )
    }

    /// Describes a TDMRep reservation that denies an AI purpose.
    pub fn tdmrep_reserved(
        &self,
        o_obs: &SignalObservation,
        o_found: &TdmrepFinding,
        e_purpose: Purpose,
    ) -> String {
        let str_where = o_found
            .opt_str_matched_location
            .as_ref()
            .map_or_else(String::new, |str_l| format!(" for location \"{str_l}\""));
        format!(
            "{} reserves text-and-data-mining rights (tdm-reservation: 1){str_where}, which denies {}.",
            self.source(o_obs),
            e_purpose.label()
        )
    }

    /// Describes a page-level robots directive that denies an AI purpose.
    pub fn robots_tag_deny(
        &self,
        o_obs: &SignalObservation,
        str_directive: &str,
        e_purpose: Purpose,
    ) -> String {
        format!(
            "{} contains \"{str_directive}\", which denies {}.",
            self.source(o_obs),
            e_purpose.label()
        )
    }

    /// Describes an AI-only reservation that does not cover a non-AI purpose.
    pub fn not_for_purpose(&self, o_obs: &SignalObservation, e_purpose: Purpose) -> String {
        format!(
            "{} reserves this URL against AI use only, which does not cover {}.",
            self.source(o_obs),
            e_purpose.label()
        )
    }

    /// Describes a present signal that reserves nothing.
    pub fn no_reservation(&self, o_obs: &SignalObservation) -> String {
        format!("{} does not reserve this URL.", self.source(o_obs))
    }

    /// Describes a signal source that does not exist.
    pub fn absent(&self, o_obs: &SignalObservation) -> String {
        if o_obs.e_kind == SignalKind::RobotsTxt {
            return format!(
                "{} was not found ({}), so no robots.txt rules apply.",
                self.source(o_obs),
                self.status(o_obs)
            );
        }
        format!(
            "{} was not found ({}); a missing signal is never treated as a reservation.",
            self.source(o_obs),
            self.status(o_obs)
        )
    }

    /// Describes a non-robots signal source that could not be retrieved.
    pub fn unreachable(&self, o_obs: &SignalObservation) -> String {
        format!(
            "{} could not be retrieved ({}); only signals that were actually observed can deny, so it has no effect.",
            self.source(o_obs),
            self.status(o_obs)
        )
    }

    /// Describes a present source that produced no parsed finding.
    pub fn unparsed(&self, o_obs: &SignalObservation) -> String {
        format!(
            "{} was present but not parsed, so it has no effect.",
            self.source(o_obs)
        )
    }

    /// Names the signal and where it was read, as the start of a sentence.
    fn source(&self, o_obs: &SignalObservation) -> String {
        let str_url = &o_obs.str_source_url;
        match o_obs.e_kind {
            SignalKind::RobotsTxt => format!("robots.txt at {str_url}"),
            SignalKind::AiTxt => format!("ai.txt at {str_url}"),
            SignalKind::TdmrepWellKnown => format!("The TDMRep file at {str_url}"),
            SignalKind::TdmrepHeader => format!("The tdm-reservation header of {str_url}"),
            SignalKind::TdmrepMeta => format!("The tdm-reservation meta tag of {str_url}"),
            SignalKind::XRobotsTag => format!("The X-Robots-Tag header of {str_url}"),
            SignalKind::MetaRobots => format!("The robots meta tag of {str_url}"),
        }
    }

    /// Names the robots.txt group that applied.
    fn group(&self, o_found: &RobotsFinding) -> String {
        o_found.opt_str_matched_agent.as_ref().map_or_else(
            || String::from("every crawler (no group applies)"),
            |str_a| format!("user-agent group \"{str_a}\""),
        )
    }

    /// Describes how retrieval ended.
    fn status(&self, o_obs: &SignalObservation) -> String {
        o_obs.opt_u_http_status.map_or_else(
            || String::from("no HTTP response"),
            |u_s| format!("HTTP {u_s}"),
        )
    }
}
