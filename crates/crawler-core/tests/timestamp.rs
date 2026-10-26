//! Timestamp has one textual form, truncates to microseconds, and rejects everything else.

use crawler_core::{Clock, CoreError, FakeClock, Timestamp};
use time::{OffsetDateTime, UtcOffset};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// 2026-10-26T12:34:56Z as a Unix timestamp.
const I_SAMPLE_UNIX: i64 = 1_793_018_096;

#[test]
fn test_fixed_format_with_six_fraction_digits() -> TestResult {
    let dt_value =
        OffsetDateTime::from_unix_timestamp(I_SAMPLE_UNIX)?.replace_nanosecond(123_456_789)?;
    let dt_stamp = Timestamp::from_datetime(dt_value)?;
    assert_eq!(dt_stamp.as_str(), "2026-10-26T12:34:56.123456Z");
    let dt_whole = Timestamp::from_datetime(OffsetDateTime::from_unix_timestamp(I_SAMPLE_UNIX)?)?;
    assert_eq!(dt_whole.as_str(), "2026-10-26T12:34:56.000000Z");
    Ok(())
}

#[test]
fn test_truncation_gives_one_value_per_microsecond() -> TestResult {
    let dt_base = OffsetDateTime::from_unix_timestamp(I_SAMPLE_UNIX)?;
    let dt_a = Timestamp::from_datetime(dt_base.replace_nanosecond(5_000)?)?;
    let dt_b = Timestamp::from_datetime(dt_base.replace_nanosecond(5_999)?)?;
    assert_eq!(dt_a, dt_b);
    assert_eq!(Timestamp::parse(dt_a.as_str())?, dt_a);
    Ok(())
}

#[test]
fn test_non_utc_offset_is_rejected() -> TestResult {
    let dt_local = OffsetDateTime::from_unix_timestamp(I_SAMPLE_UNIX)?
        .to_offset(UtcOffset::from_hms(1, 0, 0)?);
    assert!(matches!(
        Timestamp::from_datetime(dt_local),
        Err(CoreError::InvalidTimestamp { .. })
    ));
    Ok(())
}

#[test]
fn test_other_text_forms_are_rejected() {
    for str_bad in [
        "2026-10-26T12:34:56Z",
        "2026-10-26T12:34:56.123Z",
        "2026-10-26T12:34:56.123456789Z",
        "2026-10-26T12:34:56.123456+00:00",
        "2026-10-26T13:34:56.123456+01:00",
        "2026-10-26t12:34:56.123456z",
        "2026-10-26 12:34:56.123456Z",
        " 2026-10-26T12:34:56.123456Z",
    ] {
        assert!(Timestamp::parse(str_bad).is_err(), "accepted {str_bad}");
    }
}

#[test]
fn test_serde_uses_the_same_single_form() -> TestResult {
    let dt_stamp = Timestamp::parse("2026-10-26T12:34:56.000001Z")?;
    assert_eq!(
        serde_json::to_string(&dt_stamp)?,
        "\"2026-10-26T12:34:56.000001Z\""
    );
    assert!(serde_json::from_str::<Timestamp>("\"2026-10-26T12:34:56Z\"").is_err());
    Ok(())
}

#[test]
fn test_clock_produces_timestamps() -> TestResult {
    let o_clock = FakeClock::new(OffsetDateTime::from_unix_timestamp(I_SAMPLE_UNIX)?);
    o_clock.advance(time::Duration::milliseconds(1500));
    assert_eq!(o_clock.timestamp()?.as_str(), "2026-10-26T12:34:57.500000Z");
    Ok(())
}
