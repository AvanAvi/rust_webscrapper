//! Path pattern matching per RFC 9309 sections 2.2.2 and 2.2.3.

use std::time::{Duration, Instant};

use crawler_core::{CrawlDelayParser, PatternMatcher, U_MAX_CRAWL_DELAY_MS};

/// Pattern, path, and whether the pattern must match.
const VEC_CASES: &[(&str, &str, bool)] = &[
    ("/fish", "/fish", true),
    ("/fish", "/fish.html", true),
    ("/fish", "/Fish.asp", false),
    ("/fish*", "/fishheads", true),
    ("/fish/", "/fish", false),
    ("/*.php", "/folder/filename.php", true),
    ("/*.php", "/windows.PHP", false),
    ("/*.php$", "/filename.php?parameters", false),
    ("/*.php$", "/filename.php", true),
    ("/fish*.php", "/fishheads/catfish.php?parameters", true),
    ("/fish*.php", "/Fish.PHP", false),
    ("/a%7Eb", "/a~b", true),
    ("/a~b", "/a%7eb", true),
    ("/a%2fb", "/a/b", false),
    ("/a%2fb", "/a%2Fb", true),
    ("*", "/anything", true),
    ("/$", "/", true),
    ("/$", "/a", false),
    ("/a$b", "/a$bc", true),
    ("/**/x", "/y/x", true),
];

#[test]
fn test_pattern_table() {
    let o_matcher = PatternMatcher::new();
    for (str_pattern, str_path, b_expected) in VEC_CASES.iter().copied() {
        assert_eq!(
            o_matcher.matches(str_pattern, str_path),
            b_expected,
            "{str_pattern} vs {str_path}"
        );
    }
}

#[test]
fn test_pathological_patterns_stay_fast() {
    let str_pattern = "/".to_owned() + &"*a".repeat(2000) + "b$";
    let str_path = "/".to_owned() + &"a".repeat(4000);
    let dt_start = Instant::now();
    assert!(!PatternMatcher::new().matches(&str_pattern, &str_path));
    assert!(
        dt_start.elapsed() < Duration::from_secs(2),
        "took {:?}",
        dt_start.elapsed()
    );
}

#[test]
fn test_crawl_delay_is_integer_milliseconds() {
    let o_parser = CrawlDelayParser::new();
    let vec_o_cases = [
        ("1", Some(1000)),
        ("2.5", Some(2500)),
        ("0.125", Some(125)),
        ("0.1259", Some(125)),
        ("10.", Some(10_000)),
        (" 3 ", Some(3000)),
        ("99999999999", Some(U_MAX_CRAWL_DELAY_MS)),
        (".5", None),
        ("1e3", None),
        ("-1", None),
        ("fast", None),
        ("", None),
        ("1.2.3", None),
    ];
    for (str_value, opt_u_expected) in vec_o_cases {
        assert_eq!(
            o_parser.parse(str_value),
            opt_u_expected,
            "value {str_value:?}"
        );
    }
}
