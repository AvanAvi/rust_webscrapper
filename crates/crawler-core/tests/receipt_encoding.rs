//! One logical receipt has exactly one encoding and one hash.

#[path = "receipt_support/sample.rs"]
mod sample;

use crawler_core::{CanonicalEncoder, Hasher, Receipt, ReceiptSubmission};
use sample::{BoxError, STR_PINNED_CRAWLER_VERSION, sample_clock, sample_submission};
use serde_json::{Value, json};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Canonical encoding of `sample_receipt()`; any format change must update this deliberately.
const STR_EXPECTED_CANONICAL: &str = include_str!("fixtures/sample_receipt.canonical.json");
/// SHA-256 of `STR_EXPECTED_CANONICAL`, cross-checked with `shasum -a 256`.
const STR_EXPECTED_HASH: &str = "393d4cb65ba44687a52f85978671f7048087cb755702fc8da5d4df631c32555b";

/// Returns the sample submission sealed as ledger entry 7, recorded 5 ms after the fetch.
fn sample_receipt() -> Result<Receipt, BoxError> {
    let o_clock = sample_clock()?;
    let o_submission = sample_submission(&o_clock)?;
    o_clock.advance(time::Duration::milliseconds(5));
    Ok(o_submission.seal(7, &o_clock)?)
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
fn test_frozen_vector_survives_version_bump() -> TestResult {
    assert_ne!(
        STR_PINNED_CRAWLER_VERSION,
        env!("CARGO_PKG_VERSION"),
        "the pin must differ from the build version, or this test proves nothing"
    );
    let o_receipt = sample_receipt()?;
    assert_eq!(
        o_receipt.submission().o_crawler.version(),
        STR_PINNED_CRAWLER_VERSION
    );
    assert_eq!(canonical(&o_receipt)?, STR_EXPECTED_CANONICAL.trim_end());
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
    let o_clock = sample_clock()?;
    let mut o_changed = sample_submission(&o_clock)?;
    o_changed.u_lease_epoch += 1;
    o_clock.advance(time::Duration::milliseconds(5));
    let o_changed = o_changed.seal(7, &o_clock)?;
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
fn test_unfetched_submission_writes_explicit_nulls() -> TestResult {
    let mut o_submission = sample_submission(&sample_clock()?)?;
    o_submission.opt_o_fetch = None;
    o_submission.o_target.opt_str_url_final = None;
    let str_text = String::from_utf8(CanonicalEncoder::new().encode(&o_submission)?)?;
    for str_member in [
        "\"fetch\":null",
        "\"fetch_failure\":null",
        "\"url_final\":null",
    ] {
        assert!(
            str_text.contains(str_member),
            "missing {str_member} in {str_text}"
        );
    }
    let o_decoded: ReceiptSubmission = serde_json::from_str(&str_text)?;
    assert_eq!(o_decoded, o_submission);
    Ok(())
}

#[test]
fn test_missing_fields_are_rejected_not_defaulted() -> TestResult {
    let vec_o_cases: [(&[&str], &str); 5] = [
        (&["submission"], "fetch"),
        (&["submission"], "fetch_failure"),
        (&["submission"], "signal_set_id"),
        (&[], "recorded_at"),
        (&["submission", "target"], "url_final"),
    ];
    for (vec_str_parent, str_field) in vec_o_cases {
        let res_o = decode_without(vec_str_parent, str_field)?;
        assert!(res_o.is_err(), "omitted {str_field} was accepted");
    }
    Ok(())
}

/// Decodes the sample receipt with `str_field` removed from the object at `vec_str_parent`.
fn decode_without(
    vec_str_parent: &[&str],
    str_field: &str,
) -> Result<serde_json::Result<Receipt>, BoxError> {
    decode_modified(|e_json| {
        let e_parent = vec_str_parent
            .iter()
            .fold(e_json, |e_node, str_key| &mut e_node[*str_key]);
        e_parent
            .as_object_mut()
            .map(|map_e| map_e.remove(str_field));
    })
}

#[test]
fn test_unknown_fields_and_other_timestamp_forms_are_rejected() -> TestResult {
    let res_extra = decode_modified(|e_json| {
        e_json["submission"]["decision"]["extra"] = json!(1);
    })?;
    assert!(res_extra.is_err(), "unknown field was accepted");
    let res_time = decode_modified(|e_json| {
        e_json["recorded_at"] = json!("2026-10-26T12:34:56Z");
    })?;
    assert!(res_time.is_err(), "non-canonical timestamp was accepted");
    Ok(())
}
