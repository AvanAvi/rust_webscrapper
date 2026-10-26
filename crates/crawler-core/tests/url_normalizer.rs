//! Example-based tests: equivalent spellings collapse, distinct URLs stay distinct.

use crawler_core::{CoreError, UrlNormalizer};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Spellings and the key each must normalize to.
const VEC_EQUIVALENT: &[(&str, &str)] = &[
    (
        "HTTP://Example.COM:80/a/./b/../c?x=%7e#frag",
        "http://example.com/a/c?x=~",
    ),
    ("https://example.com:443", "https://example.com/"),
    ("http://example.com/%7a%2f%3a", "http://example.com/z%2F%3A"),
    ("http://example.com/a/%2E%2E/b", "http://example.com/b"),
    ("https://\u{4f8b}\u{3048}.jp/", "https://xn--r8jz45g.jp/"),
    (
        "http://example.com/%zz?q=%zz",
        "http://example.com/%zz?q=%zz",
    ),
    (
        "http://example.com/path/?b=2&a=1",
        "http://example.com/path/?b=2&a=1",
    ),
];

/// Pairs that look alike but must stay distinct keys.
const VEC_DISTINCT: &[(&str, &str)] = &[
    ("http://example.com/a%2Fb", "http://example.com/a/b"),
    ("http://example.com/a", "http://example.com/a/"),
    ("http://example.com/A", "http://example.com/a"),
    ("http://example.com/?a=1&b=2", "http://example.com/?b=2&a=1"),
    ("http://example.com/?", "http://example.com/"),
    ("http://example.com/", "https://example.com/"),
    ("http://example.com:8080/", "http://example.com/"),
    ("http://example.com/%zz", "http://example.com/%25zz"),
];

#[test]
fn test_equivalent_spellings_collapse() -> TestResult {
    let o_normalizer = UrlNormalizer::new();
    for (str_input, str_expected) in VEC_EQUIVALENT.iter().copied() {
        assert_eq!(
            o_normalizer.normalize_key(str_input)?,
            str_expected,
            "input {str_input}"
        );
    }
    Ok(())
}

#[test]
fn test_distinct_urls_never_merge() -> TestResult {
    let o_normalizer = UrlNormalizer::new();
    for (str_left, str_right) in VEC_DISTINCT.iter().copied() {
        let str_key_left = o_normalizer.normalize_key(str_left)?;
        assert_ne!(
            str_key_left,
            o_normalizer.normalize_key(str_right)?,
            "{str_left} vs {str_right}"
        );
    }
    Ok(())
}

#[test]
fn test_rejects_other_schemes_and_garbage() {
    let o_normalizer = UrlNormalizer::new();
    let res_ftp = o_normalizer.normalize("ftp://example.com/file");
    assert!(matches!(res_ftp, Err(CoreError::UnsupportedScheme { .. })));
    assert!(matches!(
        o_normalizer.normalize("not a url"),
        Err(CoreError::InvalidUrl { .. })
    ));
    let res_file = o_normalizer.normalize("file:///etc/passwd");
    assert!(matches!(res_file, Err(CoreError::UnsupportedScheme { .. })));
}

#[test]
fn test_credentials_are_rejected_and_redacted() {
    let str_input = "https://alice:hunter2@example.com/private?token=abc123#frag";
    let res_url = UrlNormalizer::new().normalize(str_input);
    let Err(CoreError::CredentialsInUrl { str_redacted_url }) = &res_url else {
        panic!("expected CredentialsInUrl, got {res_url:?}");
    };
    assert_eq!(str_redacted_url, "https://example.com/private");
    let str_shown = format!(
        "{} {:?}",
        res_url.as_ref().map(|_| ()).unwrap_err(),
        res_url
    );
    for str_secret in ["alice", "hunter2", "abc123", "token", "frag"] {
        assert!(
            !str_shown.contains(str_secret),
            "leaked {str_secret}: {str_shown}"
        );
    }
}
