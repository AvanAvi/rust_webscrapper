//! Hasher vectors, fake clock and IDs, excerpt cap, and purpose naming.

use crawler_core::{
    Clock, Excerpt, FakeClock, Hasher, IdSource, Purpose, RandomIdSource, SequentialIdSource,
    SystemClock, U_MAX_EXCERPT_BYTES,
};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_sha256_matches_fips_vectors() {
    let o_hasher = Hasher::new();
    assert_eq!(
        o_hasher.sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        o_hasher.sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn test_fake_clock_moves_only_when_told() -> TestResult {
    let dt_start = OffsetDateTime::from_unix_timestamp(0)?;
    let o_clock = FakeClock::new(dt_start);
    assert_eq!(o_clock.now(), dt_start);
    o_clock.advance(Duration::seconds(90));
    assert_eq!(o_clock.now(), dt_start + Duration::seconds(90));
    o_clock.set(dt_start);
    assert_eq!(o_clock.now(), dt_start);
    Ok(())
}

#[test]
fn test_system_clock_is_utc() -> TestResult {
    assert_eq!(SystemClock::new().now().offset(), time::UtcOffset::UTC);
    assert!(SystemClock::new().timestamp()?.as_str().ends_with('Z'));
    Ok(())
}

#[test]
fn test_sequential_ids_are_predictable() {
    let o_ids = SequentialIdSource::new(41);
    assert_eq!(o_ids.next_id(), Uuid::from_u128(41));
    assert_eq!(o_ids.next_id(), Uuid::from_u128(42));
}

#[test]
fn test_random_ids_are_version_4_and_distinct() {
    let o_ids = RandomIdSource::new();
    let (uuid_a, uuid_b) = (o_ids.next_id(), o_ids.next_id());
    assert_ne!(uuid_a, uuid_b);
    assert_eq!(uuid_a.get_version_num(), 4);
}

#[test]
fn test_excerpt_is_capped_at_a_char_boundary() -> TestResult {
    let str_long = "a".repeat(U_MAX_EXCERPT_BYTES - 1) + "\u{e9}tail";
    let o_excerpt = Excerpt::new(&str_long);
    assert_eq!(o_excerpt.as_str().len(), U_MAX_EXCERPT_BYTES - 1);
    let o_decoded: Excerpt = serde_json::from_str(&serde_json::to_string(&str_long)?)?;
    assert_eq!(o_decoded, o_excerpt);
    Ok(())
}

#[test]
fn test_purpose_defaults_to_strictest_and_uses_snake_case() -> TestResult {
    assert_eq!(Purpose::default(), Purpose::AiTrain);
    assert_eq!(serde_json::to_string(&Purpose::AiInput)?, "\"ai_input\"");
    assert_eq!(serde_json::to_string(&Purpose::AiTrain)?, "\"ai_train\"");
    Ok(())
}
