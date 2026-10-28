//! Page-level evidence: header signals are parsed from, hashed over, and archived as their
//! canonical header text; meta signals point at the page body hash; refusal fails closed.

use std::sync::Arc;

use crawler_core::{
    BodyArchive, Hasher, MemoryBodyArchive, SignalFailure, SignalKind, SignalObservation,
    SignalStatus, Timestamp, U_DEFAULT_ARCHIVE_CAPACITY,
};
use crawler_fetch::{FetchResponse, PageSignalReader};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;
/// Boxed error for helpers.
type BoxError = Box<dyn std::error::Error>;

/// Returns a page response with the given headers.
fn response(vec_o_headers: &[(&str, &str)]) -> Result<FetchResponse, BoxError> {
    let dt_now = Timestamp::parse("2026-10-28T00:00:00.000000Z")?;
    Ok(FetchResponse {
        u_status: 200,
        str_final_url: String::from("https://a.org/page"),
        vec_o_headers: vec_o_headers
            .iter()
            .map(|(str_n, str_v)| ((*str_n).to_owned(), (*str_v).to_owned()))
            .collect(),
        by_body: b"<meta name=\"robots\" content=\"noai\">".to_vec(),
        u_encoded_bytes: 36,
        vec_o_redirects: Vec::new(),
        vec_e_resolved_ips: Vec::new(),
        b_truncated: false,
        u_body_bytes_seen: 36,
        opt_dur_retry_after: None,
        dt_started_at: dt_now.clone(),
        dt_finished_at: dt_now,
    })
}

/// Returns the page observations of a response read against an archive.
fn observe(
    o_archive: &Arc<MemoryBodyArchive>,
    o_response: &FetchResponse,
) -> Result<Vec<SignalObservation>, BoxError> {
    let dt_now = Timestamp::parse("2026-10-28T00:00:01.000000Z")?;
    Ok(PageSignalReader::new("proofcrawl", o_archive.clone()).observations(o_response, dt_now))
}

/// Returns the hash recorded for a kind.
fn hash_of(vec_o_obs: &[SignalObservation], e_kind: SignalKind) -> Option<String> {
    vec_o_obs
        .iter()
        .find(|o_obs| o_obs.e_kind == e_kind)
        .and_then(|o_obs| o_obs.opt_str_content_sha256.clone())
}

/// Headers in response order A.
const VEC_O_ORDER_A: [(&str, &str); 4] = [
    ("Content-Type", "text/html"),
    ("X-Robots-Tag", "noai"),
    ("TDM-Policy", "https://a.org/p.json"),
    ("tdm-reservation", " 1 "),
];
/// The same headers in another order, with other name case.
const VEC_O_ORDER_B: [(&str, &str); 4] = [
    ("tdm-reservation", "1"),
    ("x-robots-tag", "noai"),
    ("content-type", "text/html"),
    ("tdm-policy", "https://a.org/p.json"),
];

#[test]
fn test_two_header_orderings_give_the_same_hash_and_it_resolves() -> TestResult {
    let o_archive = Arc::new(MemoryBodyArchive::new());
    let vec_o_a = observe(&o_archive, &response(&VEC_O_ORDER_A)?)?;
    let vec_o_b = observe(&o_archive, &response(&VEC_O_ORDER_B)?)?;
    for e_kind in [SignalKind::XRobotsTag, SignalKind::TdmrepHeader] {
        let str_hash = hash_of(&vec_o_a, e_kind).ok_or("header hash")?;
        assert_eq!(
            hash_of(&vec_o_b, e_kind),
            Some(str_hash.clone()),
            "{e_kind:?}"
        );
        let by_stored = o_archive.load(&str_hash).ok_or("archived")?;
        assert_eq!(Hasher::new().sha256_hex(&by_stored), str_hash);
    }
    let str_tdm = hash_of(&vec_o_a, SignalKind::TdmrepHeader).ok_or("tdm hash")?;
    assert_eq!(
        o_archive.load(&str_tdm),
        Some(b"tdm-policy: https://a.org/p.json\ntdm-reservation: 1\n".to_vec()),
        "lowercased names, trimmed values, sorted, one per line"
    );
    Ok(())
}

#[test]
fn test_same_name_header_order_is_kept() -> TestResult {
    let o_archive = Arc::new(MemoryBodyArchive::new());
    let vec_o_a = observe(
        &o_archive,
        &response(&[("tdm-reservation", "1"), ("tdm-reservation", "0")])?,
    )?;
    let vec_o_b = observe(
        &o_archive,
        &response(&[("tdm-reservation", "0"), ("tdm-reservation", "1")])?,
    )?;
    assert_ne!(
        hash_of(&vec_o_a, SignalKind::TdmrepHeader),
        hash_of(&vec_o_b, SignalKind::TdmrepHeader),
        "RFC 9110 5.3: the order of same-name lines is significant"
    );
    Ok(())
}

#[test]
fn test_meta_signals_point_at_the_page_body_hash() -> TestResult {
    let o_response = response(&VEC_O_ORDER_A)?;
    let vec_o_obs = observe(&Arc::new(MemoryBodyArchive::new()), &o_response)?;
    let str_body = Hasher::new().sha256_hex(&o_response.by_body);
    for e_kind in [SignalKind::MetaRobots, SignalKind::TdmrepMeta] {
        assert_eq!(
            hash_of(&vec_o_obs, e_kind),
            Some(str_body.clone()),
            "{e_kind:?}"
        );
    }
    Ok(())
}

#[test]
fn test_refused_header_block_fails_closed() -> TestResult {
    let o_full = Arc::new(MemoryBodyArchive::with_limits(
        U_DEFAULT_ARCHIVE_CAPACITY,
        0,
    ));
    let vec_o_obs = observe(&o_full, &response(&VEC_O_ORDER_A)?)?;
    let o_robots = vec_o_obs
        .iter()
        .find(|o_obs| o_obs.e_kind == SignalKind::XRobotsTag)
        .ok_or("x-robots-tag observation")?;
    assert_eq!(
        (o_robots.e_status, o_robots.opt_e_failure),
        (
            SignalStatus::Unreachable,
            Some(SignalFailure::EvidenceNotArchived)
        )
    );
    assert_eq!(
        (&o_robots.opt_str_content_sha256, &o_robots.opt_e_parsed),
        (&None, &None)
    );
    Ok(())
}
