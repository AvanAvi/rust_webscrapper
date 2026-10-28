//! Observation builders shared by the policy rule tests.

use crawler_core::{
    Outcome, ParsedSignal, PolicyEvaluator, PolicyRule, Purpose, RobotsParser, RobotsTagParser,
    RobotsTagSource, SignalContext, SignalKind, SignalObservation, SignalParser, SignalSet,
    SignalStatus, TdmrepParser, TdmrepSource, Timestamp,
};

/// Boxed error for helpers.
pub type BoxError = Box<dyn std::error::Error>;
/// Outcome and rule for one purpose.
pub type Verdict = (Outcome, PolicyRule);

/// All purposes, in result order.
pub const VEC_E_PURPOSES: [Purpose; 4] = [
    Purpose::Search,
    Purpose::AiInput,
    Purpose::AiTrain,
    Purpose::Archive,
];
/// Allowed with no reservation.
pub const O_ALLOW: Verdict = (Outcome::Allow, PolicyRule::NoReservation);

/// Returns the URL a signal of this kind is read from.
pub fn source_url(e_kind: SignalKind) -> &'static str {
    match e_kind {
        SignalKind::RobotsTxt => "https://example.com/robots.txt",
        SignalKind::AiTxt => "https://example.com/ai.txt",
        SignalKind::TdmrepWellKnown => "https://example.com/.well-known/tdmrep.json",
        _ => "https://example.com/page",
    }
}

/// Builds an observation; `opt_e_parsed` is the parser output for present sources.
pub fn obs(
    e_kind: SignalKind,
    e_status: SignalStatus,
    opt_u_http: Option<u16>,
    opt_e_parsed: Option<ParsedSignal>,
) -> Result<SignalObservation, BoxError> {
    Ok(SignalObservation {
        e_kind,
        e_status,
        str_source_url: source_url(e_kind).to_owned(),
        dt_observed_at: Timestamp::parse("2026-10-27T00:00:00.000000Z")?,
        opt_u_http_status: opt_u_http,
        opt_str_content_sha256: None,
        opt_o_excerpt: None,
        opt_str_final_url: None,
        vec_o_redirects: Vec::new(),
        b_body_truncated: false,
        u_body_bytes_seen: 0,
        opt_e_failure: None,
        opt_e_parsed,
    })
}

/// Builds a present observation (HTTP 200) by running a parser on the source text.
pub fn present(
    e_kind: SignalKind,
    o_parser: &dyn SignalParser,
    str_input: &str,
) -> Result<SignalObservation, BoxError> {
    let e_parsed = o_parser.parse(
        str_input.as_bytes(),
        &SignalContext::new("proofcrawl", "/page"),
    );
    obs(e_kind, SignalStatus::Present, Some(200), Some(e_parsed))
}

/// Returns the outcome and rule for each purpose, in `VEC_E_PURPOSES` order.
pub fn outcomes(vec_o_obs: Vec<SignalObservation>) -> Vec<Verdict> {
    let o_set = SignalSet::new(vec_o_obs);
    VEC_E_PURPOSES
        .iter()
        .map(|e_p| PolicyEvaluator::for_purpose(*e_p).evaluate(&o_set))
        .map(|o_d| (o_d.outcome(), o_d.rule()))
        .collect()
}

/// Expected for a signal that denies only the two AI purposes.
pub fn ai_only(e_rule: PolicyRule) -> Vec<Verdict> {
    vec![
        O_ALLOW,
        (Outcome::Deny, e_rule),
        (Outcome::Deny, e_rule),
        O_ALLOW,
    ]
}

/// Builds four disagreeing signals: robots allows and says ai-train=yes, TDMRep reserves,
/// X-Robots-Tag only says noindex, ai.txt is absent.
pub fn conflicting_signals() -> Result<Vec<SignalObservation>, BoxError> {
    let str_robots = "User-agent: *\nContent-Signal: ai-train=yes\nAllow: /\n";
    Ok(vec![
        present(SignalKind::RobotsTxt, &RobotsParser::new(), str_robots)?,
        present(
            SignalKind::TdmrepHeader,
            &TdmrepParser::new(TdmrepSource::Header),
            "tdm-reservation: 1",
        )?,
        present(
            SignalKind::XRobotsTag,
            &RobotsTagParser::new(RobotsTagSource::Header),
            "X-Robots-Tag: noindex",
        )?,
        obs(SignalKind::AiTxt, SignalStatus::Absent, Some(404), None)?,
    ])
}
