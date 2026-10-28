//! One logical receipt has exactly one encoding and one hash.

use std::net::{IpAddr, Ipv4Addr};

use crawler_core::{
    CanonicalEncoder, Clock, DecisionRecord, Excerpt, FakeClock, FetchRecord, Hasher, IdSource,
    IdentityRecord, Outcome, ParsedSignal, Purpose, Receipt, RedirectHop, RobotsFinding,
    RobotsRule, RuleKind, SequentialIdSource, SignalKind, SignalObservation, SignalStatus,
    TargetRecord, U_RECEIPT_SCHEMA_VERSION,
};
use serde_json::{Value, json};
use time::OffsetDateTime;

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;
/// Boxed error for helpers.
type BoxError = Box<dyn std::error::Error>;

/// Canonical encoding of `sample_receipt()`; any format change must update this deliberately.
const STR_EXPECTED_CANONICAL: &str = include_str!("fixtures/sample_receipt.canonical.json");
/// SHA-256 of `STR_EXPECTED_CANONICAL`, cross-checked with `shasum -a 256`.
const STR_EXPECTED_HASH: &str = "1d581f93f5a064e7457609e3e466add2a624f31f6ea68e69f0ad6251b96960a5";

/// Returns the fake clock all samples use (2026-10-26T12:34:56Z).
fn sample_clock() -> Result<FakeClock, BoxError> {
    Ok(FakeClock::new(OffsetDateTime::from_unix_timestamp(
        1_793_018_096,
    )?))
}

/// Returns an observed robots.txt signal.
fn sample_signal(o_clock: &FakeClock) -> Result<SignalObservation, BoxError> {
    Ok(SignalObservation {
        e_kind: SignalKind::RobotsTxt,
        e_status: SignalStatus::Present,
        str_source_url: String::from("https://example.com/robots.txt"),
        dt_observed_at: o_clock.timestamp()?,
        opt_u_http_status: Some(200),
        opt_str_content_sha256: Some(Hasher::new().sha256_hex(b"User-agent: *\nAllow: /\n")),
        opt_o_excerpt: Some(Excerpt::new("User-agent: *\nAllow: /")),
        opt_str_final_url: Some(String::from("https://example.com/robots.txt")),
        vec_o_redirects: Vec::new(),
        opt_e_failure: None,
        opt_e_parsed: Some(ParsedSignal::RobotsTxt(RobotsFinding {
            b_allowed: true,
            opt_str_matched_agent: Some(String::from("*")),
            opt_o_matched_rule: Some(RobotsRule {
                e_kind: RuleKind::Allow,
                str_pattern: String::from("/"),
            }),
            opt_u_crawl_delay_ms: None,
            opt_o_content_signal: None,
            u_groups: 1,
            u_invalid_lines: 0,
            b_truncated: false,
        })),
    })
}

/// Returns a fetch record with one redirect.
fn sample_fetch(o_clock: &FakeClock) -> Result<FetchRecord, BoxError> {
    let dt_started_at = o_clock.timestamp()?;
    o_clock.advance(time::Duration::milliseconds(250));
    Ok(FetchRecord {
        dt_started_at,
        dt_finished_at: o_clock.timestamp()?,
        u_http_status: 200,
        opt_str_content_type: Some(String::from("text/html; charset=utf-8")),
        u_bytes: 1234,
        str_body_sha256: Hasher::new().sha256_hex(b"<html></html>"),
        vec_o_redirects: vec![RedirectHop {
            str_url: String::from("http://example.com/a"),
            u_status: 301,
        }],
        vec_e_resolved_ips: vec![IpAddr::V4(Ipv4Addr::new(93, 184, 215, 14))],
    })
}

/// Returns the declared crawler identity.
fn sample_identity() -> IdentityRecord {
    IdentityRecord {
        str_user_agent: String::from("proofcrawl/0.1 (+https://crawler.example/contact)"),
        e_purpose: Purpose::AiTrain,
        str_contact_url: String::from("https://crawler.example/contact"),
    }
}

/// Returns the target URL forms.
fn sample_target() -> TargetRecord {
    TargetRecord {
        str_url_requested: String::from("HTTP://Example.com:80/a"),
        str_url_normalized: String::from("http://example.com/a"),
        opt_str_url_final: Some(String::from("https://example.com/a")),
        str_host: String::from("example.com"),
    }
}

/// Returns an allow decision.
fn sample_decision() -> DecisionRecord {
    DecisionRecord {
        e_outcome: Outcome::Allow,
        str_matched_rule: String::from("robots.allow"),
        vec_str_reasons: vec![String::from("robots.txt allows /a for *")],
        str_policy_version: String::from("v1"),
    }
}

/// Returns a fully populated allowed receipt built from fake time and IDs.
fn sample_receipt() -> Result<Receipt, BoxError> {
    let o_clock = sample_clock()?;
    let o_ids = SequentialIdSource::new(1);
    Ok(Receipt {
        u_schema_version: U_RECEIPT_SCHEMA_VERSION,
        uuid_receipt_id: o_ids.next_id(),
        u_sequence: 7,
        uuid_job_id: o_ids.next_id(),
        uuid_task_id: o_ids.next_id(),
        u_lease_epoch: 3,
        uuid_worker_id: o_ids.next_id(),
        o_identity: sample_identity(),
        o_target: sample_target(),
        vec_o_signals: vec![sample_signal(&o_clock)?],
        o_decision: sample_decision(),
        opt_o_fetch: Some(sample_fetch(&o_clock)?),
    })
}

/// Returns the canonical text of a receipt.
fn canonical(o_receipt: &Receipt) -> Result<String, BoxError> {
    Ok(String::from_utf8(
        o_receipt.canonical_bytes(&CanonicalEncoder::new())?,
    )?)
}

/// Parses JSON into a value, removes or inserts a field via the closure, and tries to decode it.
fn decode_modified<F: Fn(&mut Value)>(cb_edit: F) -> Result<serde_json::Result<Receipt>, BoxError> {
    let mut e_json: Value = serde_json::from_str(&canonical(&sample_receipt()?)?)?;
    cb_edit(&mut e_json);
    Ok(serde_json::from_value::<Receipt>(e_json))
}

#[test]
fn test_fixed_receipt_vector() -> TestResult {
    let o_receipt = sample_receipt()?;
    assert_eq!(canonical(&o_receipt)?, STR_EXPECTED_CANONICAL.trim_end());
    let by_hash = o_receipt.content_hash(&CanonicalEncoder::new(), &Hasher::new())?;
    assert_eq!(Hasher::new().to_hex(&by_hash), STR_EXPECTED_HASH);
    Ok(())
}

#[test]
fn test_same_receipt_same_hash_and_change_changes_it() -> TestResult {
    let (o_encoder, o_hasher) = (CanonicalEncoder::new(), Hasher::new());
    let by_first = sample_receipt()?.content_hash(&o_encoder, &o_hasher)?;
    assert_eq!(
        by_first,
        sample_receipt()?.content_hash(&o_encoder, &o_hasher)?
    );
    let mut o_changed = sample_receipt()?;
    o_changed.u_lease_epoch += 1;
    assert_ne!(by_first, o_changed.content_hash(&o_encoder, &o_hasher)?);
    Ok(())
}

#[test]
fn test_other_json_spellings_decode_to_one_encoding() -> TestResult {
    let o_receipt = sample_receipt()?;
    let str_canonical = canonical(&o_receipt)?;
    let str_declared_order = serde_json::to_string(&o_receipt)?;
    let str_pretty_escaped =
        serde_json::to_string_pretty(&o_receipt)?.replace("\"user_agent\"", "\"\\u0075ser_agent\"");
    assert_ne!(str_declared_order, str_canonical);
    for str_spelling in [
        str_declared_order,
        str_pretty_escaped,
        str_canonical.clone(),
    ] {
        let o_decoded: Receipt = serde_json::from_str(&str_spelling)?;
        assert_eq!(o_decoded, o_receipt);
        assert_eq!(canonical(&o_decoded)?, str_canonical);
    }
    Ok(())
}

#[test]
fn test_denied_receipt_writes_explicit_nulls() -> TestResult {
    let mut o_receipt = sample_receipt()?;
    o_receipt.opt_o_fetch = None;
    o_receipt.o_target.opt_str_url_final = None;
    o_receipt.vec_o_signals[0].opt_o_excerpt = None;
    let str_text = canonical(&o_receipt)?;
    for str_member in ["\"fetch\":null", "\"url_final\":null", "\"excerpt\":null"] {
        assert!(
            str_text.contains(str_member),
            "missing {str_member} in {str_text}"
        );
    }
    Ok(())
}

#[test]
fn test_missing_fields_are_rejected_not_defaulted() -> TestResult {
    let res_fetch = decode_modified(|e_json| {
        e_json.as_object_mut().map(|map_e| map_e.remove("fetch"));
    })?;
    assert!(res_fetch.is_err(), "omitted fetch was accepted");
    let res_final = decode_modified(|e_json| {
        e_json["target"]
            .as_object_mut()
            .map(|map_e| map_e.remove("url_final"));
    })?;
    assert!(res_final.is_err(), "omitted url_final was accepted");
    let res_parsed = decode_modified(|e_json| {
        e_json["signals"][0]
            .as_object_mut()
            .map(|map_e| map_e.remove("parsed"));
    })?;
    assert!(res_parsed.is_err(), "omitted parsed was accepted");
    Ok(())
}

#[test]
fn test_unknown_fields_and_other_timestamp_forms_are_rejected() -> TestResult {
    let res_extra = decode_modified(|e_json| {
        e_json["decision"]["extra"] = json!(1);
    })?;
    assert!(res_extra.is_err(), "unknown field was accepted");
    let res_time = decode_modified(|e_json| {
        e_json["signals"][0]["observed_at"] = json!("2026-10-26T12:34:56Z");
    })?;
    assert!(res_time.is_err(), "non-canonical timestamp was accepted");
    Ok(())
}
