//! Retry-After parsing for 429 and 503: delta-seconds, HTTP-date, cap, and other statuses.

use std::time::Duration;

use crawler_fetch::{DUR_MAX_RETRY_AFTER, RetryAfterParser};
use time::OffsetDateTime;

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Status, header value, and expected wait; "now" is 2015-10-21T07:28:00Z.
const VEC_O_CASES: &[(u16, Option<&str>, Option<Duration>)] = &[
    (429, Some("120"), Some(Duration::from_secs(120))),
    (
        503,
        Some("Wed, 21 Oct 2015 07:30:00 GMT"),
        Some(Duration::from_secs(120)),
    ),
    (
        503,
        Some("Wed, 21 Oct 2015 07:00:00 GMT"),
        Some(Duration::ZERO),
    ),
    (429, Some("999999"), Some(DUR_MAX_RETRY_AFTER)),
    (500, Some("120"), None),
    (429, Some("soon"), None),
    (429, None, None),
];

#[test]
fn test_retry_after_forms_and_limits() -> TestResult {
    let o_parser = RetryAfterParser::new();
    let dt_now = OffsetDateTime::from_unix_timestamp(1_445_412_480)?;
    for (u_status, opt_str_value, opt_dur_expected) in VEC_O_CASES.iter().copied() {
        let opt_dur_actual = o_parser.parse(u_status, opt_str_value, dt_now);
        assert_eq!(
            opt_dur_actual, opt_dur_expected,
            "{u_status} {opt_str_value:?}"
        );
    }
    Ok(())
}
