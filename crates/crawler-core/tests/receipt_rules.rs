//! Receipt rules: the server alone sets `sequence` and `recorded_at`, the decision precedes the
//! fetch, fetch and fetch failure match the outcome, and the crawler version comes from the build.

#[path = "receipt_support/sample.rs"]
mod sample;

use crawler_core::{
    Clock, CoreError, CrawlerRecord, FetchFailure, Outcome, Receipt, ReceiptSubmission,
    STR_CRAWLER_NAME, Timestamp,
};
use sample::{BoxError, sample_clock, sample_submission};
use serde_json::{Value, json};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Returns whether sealing the submission fails with an invalid-receipt error naming `str_part`.
fn rejected(o_submission: ReceiptSubmission, str_part: &str) -> Result<bool, BoxError> {
    let o_clock = sample_clock()?;
    Ok(matches!(
        o_submission.seal(1, &o_clock),
        Err(CoreError::InvalidReceipt { str_reason }) if str_reason.contains(str_part)
    ))
}

#[test]
fn test_worker_cannot_supply_recorded_at_or_sequence() -> TestResult {
    let o_submission = sample_submission(&sample_clock()?)?;
    for (str_field, e_value) in [
        ("recorded_at", json!("2020-01-01T00:00:00.000000Z")),
        ("sequence", json!(1)),
    ] {
        let mut e_json: Value = serde_json::to_value(&o_submission)?;
        e_json[str_field] = e_value;
        let res_o = serde_json::from_value::<ReceiptSubmission>(e_json);
        assert!(
            res_o.is_err(),
            "a submission carrying {str_field} was accepted"
        );
    }
    Ok(())
}

#[test]
fn test_seal_takes_sequence_and_recorded_at_from_the_server() -> TestResult {
    let o_worker_clock = sample_clock()?;
    let o_submission = sample_submission(&o_worker_clock)?;
    let o_server_clock = sample_clock()?;
    o_server_clock.advance(time::Duration::seconds(30));
    let o_receipt = o_submission.clone().seal(42, &o_server_clock)?;
    assert_eq!(o_receipt.sequence(), 42);
    assert_eq!(o_receipt.recorded_at(), &o_server_clock.timestamp()?);
    assert_eq!(
        o_receipt.submission(),
        &o_submission,
        "the submission is kept unchanged"
    );
    Ok(())
}

#[test]
fn test_decision_must_precede_the_fetch() -> TestResult {
    let o_valid = sample_submission(&sample_clock()?)?;
    o_valid.validate()?;
    let dt_started = o_valid
        .opt_o_fetch
        .as_ref()
        .ok_or("sample has a fetch")?
        .dt_started_at
        .clone();
    let dt_later =
        Timestamp::from_datetime(dt_started.as_datetime() + time::Duration::milliseconds(1))?;
    for dt_decided in [dt_started, dt_later] {
        let mut o_bad = o_valid.clone();
        o_bad.o_decision.dt_decided_at = dt_decided;
        assert!(rejected(
            o_bad,
            "decided_at must be earlier than fetch.started_at"
        )?);
    }
    Ok(())
}

#[test]
fn test_receipt_validate_rejects_a_decoded_receipt_decided_after_its_fetch() -> TestResult {
    let o_clock = sample_clock()?;
    let o_receipt = sample_submission(&o_clock)?.seal(1, &o_clock)?;
    o_receipt.validate()?;
    let mut e_json: Value = serde_json::to_value(&o_receipt)?;
    e_json["submission"]["decision"]["decided_at"] = json!("2030-01-01T00:00:00.000000Z");
    let o_decoded: Receipt = serde_json::from_value(e_json)?;
    assert!(matches!(
        o_decoded.validate(),
        Err(CoreError::InvalidReceipt { .. })
    ));
    Ok(())
}

#[test]
fn test_fetch_and_fetch_failure_match_the_outcome() -> TestResult {
    let o_base = sample_submission(&sample_clock()?)?;
    let mut o_denied_with_fetch = o_base.clone();
    o_denied_with_fetch.o_decision.e_outcome = Outcome::Deny;
    assert!(rejected(o_denied_with_fetch, "denied URL")?);
    let mut o_allowed_without = o_base.clone();
    o_allowed_without.opt_o_fetch = None;
    assert!(rejected(o_allowed_without.clone(), "exactly one")?);
    let mut o_allowed_both = o_base.clone();
    o_allowed_both.opt_e_fetch_failure = Some(FetchFailure::Timeout);
    assert!(rejected(o_allowed_both, "exactly one")?);
    o_allowed_without.opt_e_fetch_failure = Some(FetchFailure::Timeout);
    o_allowed_without.validate()?;
    Ok(())
}

#[test]
fn test_signal_set_id_must_be_a_sha256_digest() -> TestResult {
    for str_bad in ["", "abc", &"A".repeat(64), &"g".repeat(64)] {
        let mut o_bad = sample_submission(&sample_clock()?)?;
        o_bad.str_signal_set_id = str_bad.to_owned();
        assert!(rejected(o_bad, "signal_set_id")?, "accepted {str_bad:?}");
    }
    Ok(())
}

/// Returns `[workspace.package] version` from the workspace manifest.
fn workspace_version() -> Result<String, BoxError> {
    let str_manifest =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.toml"))?;
    let str_section = str_manifest
        .split("[workspace.package]")
        .nth(1)
        .ok_or("workspace.package section")?;
    let str_line = str_section
        .lines()
        .find(|str_l| str_l.trim_start().starts_with("version"))
        .ok_or("version line")?;
    Ok(str_line
        .split('"')
        .nth(1)
        .ok_or("quoted version")?
        .to_owned())
}

#[test]
fn test_crawler_version_comes_from_the_build() -> TestResult {
    let o_crawler = CrawlerRecord::from_build();
    assert_eq!(o_crawler.name(), STR_CRAWLER_NAME);
    assert_eq!(o_crawler.version(), env!("CARGO_PKG_VERSION"));
    assert_eq!(
        o_crawler.version(),
        workspace_version()?,
        "workspace manifest version"
    );
    let o_sealed = sample_submission(&sample_clock()?)?;
    assert_eq!(
        o_sealed.o_crawler, o_crawler,
        "submissions carry the build record"
    );
    Ok(())
}
