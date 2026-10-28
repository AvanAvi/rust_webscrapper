//! Every value of every enum the receipt or a stored observation carries. Each list is checked
//! against an exhaustive `match`, so adding a variant breaks the build until the list grows, and
//! the encoded names enter the schema fingerprint.

use crawler_core::{
    FetchFailure, Outcome, PolicyRule, Purpose, RuleKind, SignalFailure, SignalKind, SignalStatus,
};
use serde::Serialize;

/// Boxed error for helpers.
type BoxError = Box<dyn std::error::Error>;

/// Every purpose.
const VEC_E_PURPOSES: [Purpose; 4] = [
    Purpose::Search,
    Purpose::AiInput,
    Purpose::AiTrain,
    Purpose::Archive,
];
/// Every signal kind.
const VEC_E_KINDS: [SignalKind; 7] = [
    SignalKind::RobotsTxt,
    SignalKind::AiTxt,
    SignalKind::TdmrepWellKnown,
    SignalKind::TdmrepHeader,
    SignalKind::TdmrepMeta,
    SignalKind::XRobotsTag,
    SignalKind::MetaRobots,
];
/// Every signal failure.
const VEC_E_SIGNAL_FAILURES: [SignalFailure; 9] = [
    SignalFailure::ServerError,
    SignalFailure::RateLimited,
    SignalFailure::Timeout,
    SignalFailure::BlockedAddress,
    SignalFailure::DnsFailure,
    SignalFailure::ConnectionFailed,
    SignalFailure::RedirectRefused,
    SignalFailure::BadBody,
    SignalFailure::EvidenceNotArchived,
];
/// Every fetch failure.
const VEC_E_FETCH_FAILURES: [FetchFailure; 9] = [
    FetchFailure::Timeout,
    FetchFailure::BlockedAddress,
    FetchFailure::DnsFailure,
    FetchFailure::ConnectionFailed,
    FetchFailure::RedirectRefused,
    FetchFailure::BodyTooLarge,
    FetchFailure::BadBody,
    FetchFailure::InvalidRequest,
    FetchFailure::Internal,
];

/// Returns `enum <name>: a|b|c` with the encoded names, after checking that `cb_index` (an
/// exhaustive match) numbers the listed values 0, 1, 2, ... in order.
fn line<T: Serialize + Copy>(
    str_name: &str,
    vec_e_all: &[T],
    cb_index: impl Fn(T) -> usize,
) -> Result<String, BoxError> {
    assert!(
        vec_e_all
            .iter()
            .map(|e_v| cb_index(*e_v))
            .eq(0..vec_e_all.len()),
        "enum {str_name}: list and match disagree"
    );
    let vec_str_names = vec_e_all
        .iter()
        .map(|e_v| {
            Ok(serde_json::to_value(e_v)?
                .as_str()
                .unwrap_or("?")
                .to_owned())
        })
        .collect::<Result<Vec<String>, BoxError>>()?;
    Ok(format!("enum {str_name}: {}\n", vec_str_names.join("|")))
}

/// Returns the purpose line.
fn purposes() -> Result<String, BoxError> {
    line("purpose", &VEC_E_PURPOSES, |e_v| match e_v {
        Purpose::Search => 0,
        Purpose::AiInput => 1,
        Purpose::AiTrain => 2,
        Purpose::Archive => 3,
    })
}

/// Returns the outcome and rule-kind lines.
fn small_enums() -> Result<String, BoxError> {
    let str_outcome = line(
        "outcome",
        &[Outcome::Allow, Outcome::Deny],
        |e_v| match e_v {
            Outcome::Allow => 0,
            Outcome::Deny => 1,
        },
    )?;
    let vec_e_rule_kinds = [RuleKind::Allow, RuleKind::Disallow];
    let str_rule_kind = line("rule_kind", &vec_e_rule_kinds, |e_v| match e_v {
        RuleKind::Allow => 0,
        RuleKind::Disallow => 1,
    })?;
    Ok(format!("{str_outcome}{str_rule_kind}"))
}

/// Returns the signal kind line.
fn signal_kinds() -> Result<String, BoxError> {
    line("signal_kind", &VEC_E_KINDS, |e_v| match e_v {
        SignalKind::RobotsTxt => 0,
        SignalKind::AiTxt => 1,
        SignalKind::TdmrepWellKnown => 2,
        SignalKind::TdmrepHeader => 3,
        SignalKind::TdmrepMeta => 4,
        SignalKind::XRobotsTag => 5,
        SignalKind::MetaRobots => 6,
    })
}

/// Returns the signal status line.
fn signal_statuses() -> Result<String, BoxError> {
    use SignalStatus::{Absent, Present, Unreachable};
    line(
        "signal_status",
        &[Present, Absent, Unreachable],
        |e_v| match e_v {
            Present => 0,
            Absent => 1,
            Unreachable => 2,
        },
    )
}

/// Returns the signal failure line.
fn signal_failures() -> Result<String, BoxError> {
    line("signal_failure", &VEC_E_SIGNAL_FAILURES, |e_v| match e_v {
        SignalFailure::ServerError => 0,
        SignalFailure::RateLimited => 1,
        SignalFailure::Timeout => 2,
        SignalFailure::BlockedAddress => 3,
        SignalFailure::DnsFailure => 4,
        SignalFailure::ConnectionFailed => 5,
        SignalFailure::RedirectRefused => 6,
        SignalFailure::BadBody => 7,
        SignalFailure::EvidenceNotArchived => 8,
    })
}

/// Returns the fetch failure line.
fn fetch_failures() -> Result<String, BoxError> {
    line("fetch_failure", &VEC_E_FETCH_FAILURES, |e_v| match e_v {
        FetchFailure::Timeout => 0,
        FetchFailure::BlockedAddress => 1,
        FetchFailure::DnsFailure => 2,
        FetchFailure::ConnectionFailed => 3,
        FetchFailure::RedirectRefused => 4,
        FetchFailure::BodyTooLarge => 5,
        FetchFailure::BadBody => 6,
        FetchFailure::InvalidRequest => 7,
        FetchFailure::Internal => 8,
    })
}

/// Returns the matched-rule line, in precedence order.
fn policy_rules() -> Result<String, BoxError> {
    line("matched_rule", &PolicyRule::VEC_E_ALL, |e_v| match e_v {
        PolicyRule::EvidenceNotArchived => 0,
        PolicyRule::RobotsUnreachable => 1,
        PolicyRule::RobotsDisallow => 2,
        PolicyRule::ContentSignalNo => 3,
        PolicyRule::TdmrepReserved => 4,
        PolicyRule::AiTxtDisallow => 5,
        PolicyRule::RobotsTagNoai => 6,
        PolicyRule::RobotsTagNoimageai => 7,
        PolicyRule::NoReservation => 8,
    })
}

/// Returns every enum line.
pub fn enum_lines() -> Result<String, BoxError> {
    Ok([
        purposes()?,
        small_enums()?,
        signal_kinds()?,
        signal_statuses()?,
        signal_failures()?,
        fetch_failures()?,
        policy_rules()?,
    ]
    .concat())
}
