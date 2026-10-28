//! Policy v1 rules: noai scoping, absent sources, robots 4xx/5xx, Content-Signal, and conflicts.

#[path = "policy_support/observations.rs"]
mod observations;

use crawler_core::{
    Outcome, PolicyEvaluator, PolicyRule, Purpose, RobotsParser, RobotsTagParser, RobotsTagSource,
    SignalKind, SignalSet, SignalStatus, Timestamp,
};
use observations::{O_ALLOW, ai_only, conflicting_signals, obs, outcomes, present};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// X-Robots-Tag values whose `noai` applies to `proofcrawl`.
const VEC_STR_APPLYING_HEADERS: [&str; 4] = [
    "X-Robots-Tag: noai",
    "X-Robots-Tag: *: noai",
    "X-Robots-Tag: ProofCrawl: noai",
    "X-Robots-Tag: googlebot: noindex, proofcrawl: noai",
];

#[test]
fn test_noai_header_in_every_applicable_scope_denies_ai_uses_only() -> TestResult {
    let o_parser = RobotsTagParser::new(RobotsTagSource::Header);
    for str_header in VEC_STR_APPLYING_HEADERS {
        let o_obs = present(SignalKind::XRobotsTag, &o_parser, str_header)?;
        assert_eq!(
            outcomes(vec![o_obs]),
            ai_only(PolicyRule::RobotsTagNoai),
            "{str_header}"
        );
    }
    Ok(())
}

#[test]
fn test_noai_meta_for_robots_and_own_token_denies_ai_uses_only() -> TestResult {
    let o_parser = RobotsTagParser::new(RobotsTagSource::Meta);
    for str_html in [
        r#"<meta name="robots" content="noai">"#,
        r#"<meta name="proofcrawl" content="noai">"#,
    ] {
        let o_obs = present(SignalKind::MetaRobots, &o_parser, str_html)?;
        assert_eq!(
            outcomes(vec![o_obs]),
            ai_only(PolicyRule::RobotsTagNoai),
            "{str_html}"
        );
    }
    Ok(())
}

#[test]
fn test_noai_for_another_bot_does_not_apply() -> TestResult {
    let o_parser = RobotsTagParser::new(RobotsTagSource::Header);
    let o_obs = present(
        SignalKind::XRobotsTag,
        &o_parser,
        "X-Robots-Tag: googlebot: noai",
    )?;
    assert_eq!(outcomes(vec![o_obs]), vec![O_ALLOW; 4]);
    Ok(())
}

#[test]
fn test_noai_denies_search_only_when_another_signal_does() -> TestResult {
    let o_noai = present(
        SignalKind::XRobotsTag,
        &RobotsTagParser::new(RobotsTagSource::Header),
        "X-Robots-Tag: noai",
    )?;
    let o_robots = present(
        SignalKind::RobotsTxt,
        &RobotsParser::new(),
        "User-agent: *\nDisallow: /page\n",
    )?;
    assert_eq!(
        outcomes(vec![o_noai, o_robots]),
        vec![(Outcome::Deny, PolicyRule::RobotsDisallow); 4]
    );
    Ok(())
}

#[test]
fn test_absent_or_unreachable_ai_txt_and_tdmrep_never_deny() -> TestResult {
    for (e_status, opt_u_http) in [
        (SignalStatus::Absent, Some(404)),
        (SignalStatus::Unreachable, None),
    ] {
        let vec_o_obs = vec![
            obs(SignalKind::AiTxt, e_status, opt_u_http, None)?,
            obs(SignalKind::TdmrepWellKnown, e_status, opt_u_http, None)?,
        ];
        assert_eq!(outcomes(vec_o_obs), vec![O_ALLOW; 4], "{e_status:?}");
    }
    Ok(())
}

#[test]
fn test_robots_4xx_means_no_rules() -> TestResult {
    let o_obs = obs(SignalKind::RobotsTxt, SignalStatus::Absent, Some(404), None)?;
    let o_decision = PolicyEvaluator::for_purpose(Purpose::AiTrain)
        .evaluate(&SignalSet::new(vec![o_obs.clone()]));
    assert!(
        o_decision.reasons()[0].contains("was not found (HTTP 404), so no robots.txt rules apply")
    );
    assert_eq!(outcomes(vec![o_obs]), vec![O_ALLOW; 4]);
    Ok(())
}

#[test]
fn test_robots_5xx_is_full_disallow() -> TestResult {
    let o_obs = obs(
        SignalKind::RobotsTxt,
        SignalStatus::Unreachable,
        Some(503),
        None,
    )?;
    assert_eq!(
        outcomes(vec![o_obs]),
        vec![(Outcome::Deny, PolicyRule::RobotsUnreachable); 4]
    );
    Ok(())
}

#[test]
fn test_robots_unreachable_without_response_is_full_disallow() -> TestResult {
    let o_obs = obs(SignalKind::RobotsTxt, SignalStatus::Unreachable, None, None)?;
    let o_decision = PolicyEvaluator::for_purpose(Purpose::Search)
        .evaluate(&SignalSet::new(vec![o_obs.clone()]));
    assert!(o_decision.reasons()[0].contains("(no HTTP response)"));
    assert_eq!(
        outcomes(vec![o_obs]),
        vec![(Outcome::Deny, PolicyRule::RobotsUnreachable); 4]
    );
    Ok(())
}

#[test]
fn test_content_signal_no_denies_and_missing_line_does_not() -> TestResult {
    let o_train_no = vec![
        O_ALLOW,
        O_ALLOW,
        (Outcome::Deny, PolicyRule::ContentSignalNo),
        O_ALLOW,
    ];
    let vec_o_cases = [
        (
            "User-agent: *\nContent-Signal: ai-train=no\nAllow: /\n",
            o_train_no,
        ),
        ("User-agent: *\nAllow: /\n", vec![O_ALLOW; 4]),
        (
            "User-agent: *\nContent-Signal: search=yes\n",
            vec![O_ALLOW; 4],
        ),
    ];
    for (str_text, vec_o_expected) in vec_o_cases {
        let o_obs = present(SignalKind::RobotsTxt, &RobotsParser::new(), str_text)?;
        assert_eq!(outcomes(vec![o_obs]), vec_o_expected, "{str_text:?}");
    }
    Ok(())
}

/// Sources of `conflicting_signals()`, in order.
const VEC_STR_CONFLICT_SOURCES: [&str; 4] = [
    "robots.txt at",
    "tdm-reservation header",
    "X-Robots-Tag header",
    "ai.txt at",
];

/// Checks that there is exactly one reason per consulted signal, in observation order.
fn assert_lists_every_source(vec_str_reasons: &[String]) {
    assert_eq!(
        vec_str_reasons.len(),
        VEC_STR_CONFLICT_SOURCES.len(),
        "every consulted signal is listed"
    );
    for (str_reason, str_source) in vec_str_reasons.iter().zip(VEC_STR_CONFLICT_SOURCES) {
        assert!(str_reason.contains(str_source), "{str_reason}");
    }
}

#[test]
fn test_conflicting_signals_deny_wins_and_every_signal_is_listed() -> TestResult {
    let o_decision = PolicyEvaluator::for_purpose(Purpose::AiTrain)
        .evaluate(&SignalSet::new(conflicting_signals()?));
    assert_eq!(
        (o_decision.outcome(), o_decision.rule()),
        (Outcome::Deny, PolicyRule::TdmrepReserved)
    );
    assert_lists_every_source(&o_decision.reasons());
    assert_eq!(
        o_decision
            .to_record(Timestamp::parse("2026-10-27T00:00:01.000000Z")?)
            .e_matched_rule
            .as_str(),
        "tdmrep.reservation"
    );
    Ok(())
}

#[test]
fn test_several_denials_report_the_highest_precedence_rule() -> TestResult {
    let mut vec_o_obs = conflicting_signals()?;
    vec_o_obs.push(present(
        SignalKind::RobotsTxt,
        &RobotsParser::new(),
        "User-agent: *\nDisallow: /\n",
    )?);
    let o_decision =
        PolicyEvaluator::for_purpose(Purpose::AiTrain).evaluate(&SignalSet::new(vec_o_obs));
    assert_eq!(o_decision.rule(), PolicyRule::RobotsDisallow);
    let u_denying = o_decision
        .verdicts()
        .iter()
        .filter(|o_v| o_v.denying_rule().is_some())
        .count();
    assert_eq!(u_denying, 2);
    Ok(())
}

#[test]
fn test_no_signals_allows_with_one_reason() {
    let o_decision =
        PolicyEvaluator::for_purpose(Purpose::Archive).evaluate(&SignalSet::new(Vec::new()));
    assert_eq!(
        (o_decision.outcome(), o_decision.reasons().len()),
        (Outcome::Allow, 1)
    );
}
