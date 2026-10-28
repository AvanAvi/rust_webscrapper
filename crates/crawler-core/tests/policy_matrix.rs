//! Decision matrix: every signal scenario against every purpose, with no blank cells.

#[path = "policy_support/matrix_table.rs"]
mod matrix_table;

use std::collections::BTreeSet;

use matrix_table::{Cell, Scenario, VEC_E_PURPOSES, VEC_EXPECTED, VEC_SCENARIOS};

use crawler_core::{
    AiTxtParser, DecisionRecord, Outcome, PolicyEvaluator, PolicyRule, Purpose, RobotsParser,
    RobotsTagParser, RobotsTagSource, SignalContext, SignalFailure, SignalKind, SignalObservation,
    SignalParser, SignalSet, SignalStatus, TdmrepParser, TdmrepSource, Timestamp,
};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;
/// Returns the parser for a signal kind.
fn parser_for(e_kind: SignalKind) -> Box<dyn SignalParser> {
    match e_kind {
        SignalKind::RobotsTxt => Box::new(RobotsParser::new()),
        SignalKind::AiTxt => Box::new(AiTxtParser::new()),
        SignalKind::TdmrepWellKnown => Box::new(TdmrepParser::new(TdmrepSource::WellKnown)),
        SignalKind::TdmrepHeader => Box::new(TdmrepParser::new(TdmrepSource::Header)),
        SignalKind::TdmrepMeta => Box::new(TdmrepParser::new(TdmrepSource::Meta)),
        SignalKind::XRobotsTag => Box::new(RobotsTagParser::new(RobotsTagSource::Header)),
        SignalKind::MetaRobots => Box::new(RobotsTagParser::new(RobotsTagSource::Meta)),
    }
}

/// Returns the URL a signal of this kind is read from.
fn source_url(e_kind: SignalKind) -> &'static str {
    match e_kind {
        SignalKind::RobotsTxt => "https://example.com/robots.txt",
        SignalKind::AiTxt => "https://example.com/ai.txt",
        SignalKind::TdmrepWellKnown => "https://example.com/.well-known/tdmrep.json",
        _ => "https://example.com/page",
    }
}

/// Builds the observation of a scenario, parsing the source with the real parser when present.
fn observation(o_scenario: &Scenario) -> Result<SignalObservation, Box<dyn std::error::Error>> {
    let (_, e_kind, e_status, opt_u_http, str_input) = *o_scenario;
    let o_context = SignalContext::new("proofcrawl", "/page");
    let opt_e_parsed = (e_status == SignalStatus::Present)
        .then(|| parser_for(e_kind).parse(str_input.as_bytes(), &o_context));
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

/// Checks one cell: outcome, matched rule, and a single readable reason naming the source.
fn check_cell(o_scenario: &Scenario, e_purpose: Purpose, o_cell: Cell) -> TestResult {
    let o_set = SignalSet::new(vec![observation(o_scenario)?]);
    let o_record = PolicyEvaluator::for_purpose(e_purpose)
        .evaluate(&o_set)
        .to_record(Timestamp::parse("2026-10-27T00:00:01.000000Z")?);
    let str_where = format!("{} / {:?}", o_scenario.0, e_purpose);
    let o_actual = (o_record.e_outcome, o_record.e_matched_rule.as_str());
    assert_eq!(o_actual, o_cell, "{str_where}");
    assert_eq!(
        o_record.vec_str_reasons.len(),
        1,
        "{str_where}: one signal consulted"
    );
    check_reason(
        &str_where,
        &o_record.vec_str_reasons[0],
        o_scenario.1,
        o_cell.0,
    );
    Ok(())
}

/// Checks that a reason is a plain sentence naming the source and saying deny only when denied.
fn check_reason(str_where: &str, str_reason: &str, e_kind: SignalKind, e_outcome: Outcome) {
    assert!(
        str_reason.contains(source_url(e_kind)),
        "{str_where}: {str_reason}"
    );
    let b_plain =
        str_reason.ends_with('.') && !str_reason.contains("Some(") && !str_reason.contains("None");
    assert!(b_plain, "{str_where}: {str_reason}");
    let b_says_deny = ["denies", "may not be crawled", "treated as disallowed"]
        .iter()
        .any(|str_w| str_reason.contains(str_w));
    assert_eq!(
        b_says_deny,
        e_outcome == Outcome::Deny,
        "{str_where}: {str_reason}"
    );
}

#[test]
fn test_matrix_has_no_blank_cells() {
    let vec_str_rows: Vec<&str> = VEC_SCENARIOS.iter().map(|o_s| o_s.0).collect();
    let vec_str_expected: Vec<&str> = VEC_EXPECTED.iter().map(|o_e| o_e.0).collect();
    assert_eq!(vec_str_rows, vec_str_expected);
    let set_e_kinds: BTreeSet<String> = VEC_SCENARIOS
        .iter()
        .map(|o_s| format!("{:?}", o_s.1))
        .collect();
    assert_eq!(set_e_kinds.len(), 7, "every SignalKind appears");
}

/// Checks every purpose column of one scenario row.
fn check_row(o_scenario: &Scenario, vec_o_cells: &[Cell; 4]) -> TestResult {
    for (e_purpose, o_cell) in VEC_E_PURPOSES.iter().zip(vec_o_cells) {
        check_cell(o_scenario, *e_purpose, *o_cell)?;
    }
    Ok(())
}

#[test]
fn test_decision_matrix() -> TestResult {
    for (o_scenario, (_, vec_o_cells)) in VEC_SCENARIOS.iter().zip(VEC_EXPECTED) {
        check_row(o_scenario, vec_o_cells)?;
    }
    Ok(())
}

/// Returns every rule the evaluator emits over the matrix plus an unarchived-evidence signal.
fn emitted_rules() -> Result<BTreeSet<PolicyRule>, Box<dyn std::error::Error>> {
    let mut o_unarchived = observation(&VEC_SCENARIOS[0])?;
    o_unarchived.e_status = SignalStatus::Unreachable;
    o_unarchived.opt_e_failure = Some(SignalFailure::EvidenceNotArchived);
    o_unarchived.opt_e_parsed = None;
    let mut vec_vec_o_sets = vec![vec![o_unarchived]];
    for o_scenario in VEC_SCENARIOS {
        vec_vec_o_sets.push(vec![observation(o_scenario)?]);
    }
    let set_e_rules = vec_vec_o_sets
        .iter()
        .flat_map(|vec_o_obs| {
            VEC_E_PURPOSES.iter().map(|e_purpose| {
                PolicyEvaluator::for_purpose(*e_purpose)
                    .evaluate(&SignalSet::new(vec_o_obs.clone()))
                    .rule()
            })
        })
        .collect();
    Ok(set_e_rules)
}

#[test]
fn test_every_emitted_rule_round_trips_through_the_receipt() -> TestResult {
    let set_e_emitted = emitted_rules()?;
    let set_e_listed: BTreeSet<PolicyRule> = PolicyRule::VEC_E_ALL.into_iter().collect();
    assert_eq!(
        set_e_emitted, set_e_listed,
        "the evaluator emits exactly the listed rules"
    );
    let dt_at = Timestamp::parse("2026-10-27T00:00:01.000000Z")?;
    for e_rule in set_e_emitted {
        let o_record = DecisionRecord {
            e_outcome: Outcome::Deny,
            e_matched_rule: e_rule,
            vec_str_reasons: Vec::new(),
            str_policy_version: String::from("v1"),
            dt_decided_at: dt_at.clone(),
        };
        let str_json = serde_json::to_string(&o_record)?;
        assert!(str_json.contains(&format!("\"matched_rule\":\"{}\"", e_rule.as_str())));
        assert_eq!(serde_json::from_str::<DecisionRecord>(&str_json)?, o_record);
    }
    Ok(())
}

#[test]
fn test_unknown_rule_ids_are_rejected_on_decode() {
    for str_rule in [
        "robots.allow",
        "ROBOTS_TXT.DISALLOW",
        "",
        "robots_txt.disallow ",
    ] {
        let str_json = format!(
            "{{\"decided_at\":\"2026-10-27T00:00:01.000000Z\",\"matched_rule\":\"{str_rule}\",\"outcome\":\"deny\",\"policy_version\":\"v1\",\"reasons\":[]}}"
        );
        assert!(
            serde_json::from_str::<DecisionRecord>(&str_json).is_err(),
            "accepted {str_rule:?}"
        );
    }
}
