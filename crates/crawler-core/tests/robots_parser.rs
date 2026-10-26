//! robots.txt parsing and evaluation against RFC 9309 plus Crawl-delay and Content-Signal.

use std::fs;
use std::path::PathBuf;

use crawler_core::{
    ContentSignal, ParsedSignal, RobotsFinding, RobotsParser, RuleKind, SignalContext,
    SignalParser, U_MAX_ROBOTS_BYTES,
};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// One evaluation case: fixture, token, path, allowed, deciding rule pattern.
type Case = (
    &'static str,
    &'static str,
    &'static str,
    bool,
    Option<&'static str>,
);

/// Expected decisions across the fixtures.
const VEC_CASES: &[Case] = &[
    (
        "robots_standard.txt",
        "proofcrawl",
        "/blocked/x",
        false,
        Some("/blocked"),
    ),
    (
        "robots_standard.txt",
        "PROOFCRAWL",
        "/blocked/ok/1",
        true,
        Some("/blocked/ok"),
    ),
    (
        "robots_standard.txt",
        "proofcrawl",
        "/private/x",
        true,
        None,
    ),
    (
        "robots_standard.txt",
        "somebot",
        "/private/x",
        false,
        Some("/private/"),
    ),
    (
        "robots_standard.txt",
        "somebot",
        "/private/public/x",
        true,
        Some("/private/public/"),
    ),
    (
        "robots_standard.txt",
        "somebot",
        "/doc.pdf",
        false,
        Some("/*.pdf$"),
    ),
    ("robots_standard.txt", "somebot", "/doc.pdf?x=1", true, None),
    (
        "robots_conflict.txt",
        "ProofCrawl",
        "/page",
        true,
        Some("/page"),
    ),
    ("robots_conflict.txt", "proofcrawl", "/other", true, None),
    ("robots_conflict.txt", "zzz", "/page", false, Some("/")),
    ("robots_conflict.txt", "zzz", "/robots.txt", true, None),
    (
        "robots_malformed.txt",
        "proofcrawl",
        "/Secret/x",
        false,
        Some("/Secret"),
    ),
    (
        "robots_malformed.txt",
        "proofcrawl",
        "/Secret/open/1",
        true,
        Some("/Secret/open"),
    ),
    (
        "robots_malformed.txt",
        "proofcrawl",
        "/secret/x",
        true,
        None,
    ),
    (
        "robots_malformed.txt",
        "proofcrawl",
        "/before-any-agent",
        true,
        None,
    ),
    ("robots_empty.txt", "proofcrawl", "/anything", true, None),
];

/// Reads a signal fixture.
fn fixture(str_name: &str) -> Result<Vec<u8>, std::io::Error> {
    fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/signals")
            .join(str_name),
    )
}

/// Parses bytes and evaluates them for one token and path.
fn evaluate(by_input: &[u8], str_token: &str, str_path: &str) -> RobotsFinding {
    RobotsParser::new()
        .parse_document(by_input)
        .evaluate(&SignalContext::new(str_token, str_path))
}

#[test]
fn test_fixture_decisions() -> TestResult {
    for (str_file, str_token, str_path, b_allowed, opt_str_pattern) in VEC_CASES.iter().copied() {
        let o_finding = evaluate(&fixture(str_file)?, str_token, str_path);
        let opt_str_rule = o_finding
            .opt_o_matched_rule
            .as_ref()
            .map(|o_r| o_r.str_pattern.as_str());
        assert_eq!(
            (o_finding.b_allowed, opt_str_rule),
            (b_allowed, opt_str_pattern),
            "{str_file} {str_token} {str_path}"
        );
    }
    Ok(())
}

#[test]
fn test_group_directives_and_counts() -> TestResult {
    let o_star = evaluate(&fixture("robots_standard.txt")?, "somebot", "/");
    assert_eq!(o_star.opt_u_crawl_delay_ms, Some(2500));
    let o_expected = ContentSignal {
        opt_b_search: Some(true),
        opt_b_ai_input: None,
        opt_b_ai_train: Some(false),
    };
    assert_eq!(o_star.opt_o_content_signal, Some(o_expected));
    assert_eq!(
        (
            o_star.opt_str_matched_agent.as_deref(),
            o_star.u_groups,
            o_star.u_invalid_lines
        ),
        (Some("*"), 2, 0)
    );
    let o_named = evaluate(&fixture("robots_standard.txt")?, "ProofCrawl", "/");
    assert_eq!(
        (
            o_named.opt_str_matched_agent.as_deref(),
            o_named.opt_u_crawl_delay_ms
        ),
        (Some("proofcrawl"), None)
    );
    Ok(())
}

#[test]
fn test_matching_groups_are_combined() -> TestResult {
    let o_finding = evaluate(&fixture("robots_conflict.txt")?, "proofcrawl", "/page");
    assert_eq!(o_finding.opt_u_crawl_delay_ms, Some(3000));
    let o_expected = ContentSignal {
        opt_b_search: None,
        opt_b_ai_input: Some(true),
        opt_b_ai_train: Some(false),
    };
    assert_eq!(o_finding.opt_o_content_signal, Some(o_expected));
    assert_eq!(
        o_finding.opt_o_matched_rule.map(|o_r| o_r.e_kind),
        Some(RuleKind::Allow)
    );
    Ok(())
}

#[test]
fn test_malformed_lines_are_counted_not_fatal() -> TestResult {
    let o_finding = evaluate(&fixture("robots_malformed.txt")?, "proofcrawl", "/");
    assert_eq!(
        o_finding.u_invalid_lines, 6,
        "orphan rule, no colon, bad pattern, bad delay, 2 bad pairs"
    );
    assert_eq!(o_finding.opt_u_crawl_delay_ms, None);
    let o_signal = o_finding.opt_o_content_signal.unwrap_or_default();
    assert_eq!(
        (o_signal.opt_b_search, o_signal.opt_b_ai_train),
        (None, Some(false))
    );
    Ok(())
}

#[test]
fn test_no_groups_means_everything_allowed() {
    let o_finding = evaluate(b"Sitemap: https://example.com/s.xml\n", "proofcrawl", "/x");
    assert!(o_finding.b_allowed);
    assert_eq!(
        (o_finding.opt_str_matched_agent, o_finding.u_groups),
        (None, 0)
    );
}

#[test]
fn test_huge_input_is_cut_at_the_parsing_limit() {
    let mut by_input = b"User-agent: *\nDisallow: /early\nDisallow: /".to_vec();
    by_input.extend(std::iter::repeat_n(b'x', U_MAX_ROBOTS_BYTES + 10));
    by_input.extend(b"\nDisallow: /late\n");
    let o_finding = evaluate(&by_input, "proofcrawl", "/late");
    assert!(o_finding.b_truncated);
    assert!(o_finding.b_allowed, "rules after the limit must be ignored");
    assert!(!evaluate(&by_input, "proofcrawl", "/early").b_allowed);
}

#[test]
fn test_parser_trait_reports_kind_and_finding() -> TestResult {
    let o_parser = RobotsParser::new();
    let e_parsed = o_parser.parse(
        &fixture("robots_standard.txt")?,
        &SignalContext::new("somebot", "/private/x"),
    );
    assert!(matches!(
        e_parsed,
        ParsedSignal::RobotsTxt(RobotsFinding {
            b_allowed: false,
            ..
        })
    ));
    assert_eq!(o_parser.kind(), crawler_core::SignalKind::RobotsTxt);
    Ok(())
}
