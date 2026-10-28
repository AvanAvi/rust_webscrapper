//! Evidence guarantees of the signal collector: truncated bodies are archived exactly as parsed,
//! a body the archive refuses fails closed through the policy evaluator, and the cache stays
//! within its byte budget (needs `--features test-policy`).
#![cfg(feature = "test-policy")]

#[path = "fetch_support/collector_setup.rs"]
mod collector_setup;

use collector_setup::{BoxError, Harness, harness, harness_with};
use crawler_core::{
    BodyArchive, Hasher, MemoryBodyArchive, Outcome, PolicyEvaluator, PolicyRule, Purpose,
    RobotsParser, SignalContext, SignalFailure, SignalKind, SignalObservation, SignalParser,
    SignalSet, SignalStatus, U_DEFAULT_ARCHIVE_CAPACITY, U_MAX_ARCHIVED_BODY_BYTES,
};
use crawler_fetch::{CollectorConfig, FetchRequest, HttpFetcher};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;
/// Every purpose a decision can be made for.
const VEC_E_PURPOSES: [Purpose; 4] = [
    Purpose::Search,
    Purpose::AiInput,
    Purpose::AiTrain,
    Purpose::Archive,
];
/// Size of sig-2mib.mock's robots.txt.
const U_TWO_MIB: u64 = 2 * 1024 * 1024;

/// Collects the signals of a page.
async fn collect(
    o_h: &Harness,
    str_host: &str,
    str_path: &str,
) -> Result<Vec<SignalObservation>, BoxError> {
    Ok(o_h
        .o_collector
        .collect(&o_h.url(str_host, str_path)?)
        .await?)
}

#[tokio::test]
async fn test_two_mib_robots_archives_one_mib_that_reparses_identically() -> TestResult {
    let o_h = harness(100).await?;
    let vec_o_obs = collect(&o_h, "sig-2mib.mock", "/early/x").await?;
    let o_robots = &vec_o_obs[0];
    assert!(o_robots.b_body_truncated, "cut at the signal cap");
    assert!(o_robots.u_body_bytes_seen > U_MAX_ARCHIVED_BODY_BYTES as u64);
    assert!(o_robots.u_body_bytes_seen <= U_TWO_MIB);
    let str_sha = o_robots
        .opt_str_content_sha256
        .clone()
        .ok_or("archived body has a hash")?;
    let by_body = o_h
        .o_archive
        .load(&str_sha)
        .ok_or("hash resolves in the archive")?;
    assert_eq!(by_body.len(), U_MAX_ARCHIVED_BODY_BYTES, "stored prefix");
    assert_eq!(Hasher::new().sha256_hex(&by_body), str_sha, "stored hash");
    assert_reparse_matches_live(&by_body, &vec_o_obs);
    Ok(())
}

/// Re-parses an archived robots.txt and checks the finding and decision equal the live ones.
fn assert_reparse_matches_live(by_body: &[u8], vec_o_obs: &[SignalObservation]) {
    let o_context = SignalContext::new("proofcrawl", "/early/x");
    let e_reparsed = RobotsParser::new().parse(by_body, &o_context);
    assert_eq!(
        Some(e_reparsed),
        vec_o_obs[0].opt_e_parsed,
        "same finding as the live decision"
    );
    let o_set = SignalSet::new(vec_o_obs.to_vec());
    let o_live = PolicyEvaluator::for_purpose(Purpose::Search).evaluate(&o_set);
    assert_eq!(
        (o_live.outcome(), o_live.rule()),
        (Outcome::Deny, PolicyRule::RobotsDisallow)
    );
}

/// Checks that every purpose is denied under `evidence.not_archived` with a plain reason.
fn assert_denied_for_every_purpose(vec_o_obs: &[SignalObservation]) {
    let o_set = SignalSet::new(vec_o_obs.to_vec());
    for e_purpose in VEC_E_PURPOSES {
        let o_decision = PolicyEvaluator::for_purpose(e_purpose).evaluate(&o_set);
        let vec_str_reasons = o_decision.reasons();
        assert_eq!(
            (o_decision.outcome(), o_decision.rule()),
            (Outcome::Deny, PolicyRule::EvidenceNotArchived),
            "{e_purpose:?}: {vec_str_reasons:?}"
        );
        assert!(
            vec_str_reasons
                .iter()
                .any(|str_r| str_r.contains("could not be archived as evidence")),
            "{vec_str_reasons:?}"
        );
    }
}

#[tokio::test]
async fn test_archive_byte_cap_hit_fails_closed_for_every_purpose() -> TestResult {
    let o_archive = MemoryBodyArchive::with_capacity(80);
    let o_h = harness_with(CollectorConfig::new("proofcrawl"), o_archive).await?;
    let vec_o_obs = collect(&o_h, "sig-ok.mock", "/page").await?;
    assert_eq!(
        vec_o_obs[0].e_status,
        SignalStatus::Present,
        "74-byte robots.txt fits"
    );
    let o_ai = &vec_o_obs[1];
    assert_eq!(o_ai.e_kind, SignalKind::AiTxt);
    assert_eq!(
        (o_ai.e_status, o_ai.opt_e_failure),
        (
            SignalStatus::Unreachable,
            Some(SignalFailure::EvidenceNotArchived)
        )
    );
    assert_eq!(
        (&o_ai.opt_str_content_sha256, &o_ai.opt_e_parsed),
        (&None, &None)
    );
    assert_denied_for_every_purpose(&vec_o_obs);
    Ok(())
}

#[tokio::test]
async fn test_absent_robots_that_cannot_be_archived_does_not_allow() -> TestResult {
    let o_archive = MemoryBodyArchive::with_limits(U_DEFAULT_ARCHIVE_CAPACITY, 0);
    let o_h = harness_with(CollectorConfig::new("proofcrawl"), o_archive).await?;
    let vec_o_obs = collect(&o_h, "sig-404.mock", "/x").await?;
    assert_eq!(vec_o_obs[0].opt_u_http_status, Some(404));
    assert_eq!(
        vec_o_obs[0].opt_e_failure,
        Some(SignalFailure::EvidenceNotArchived)
    );
    assert_denied_for_every_purpose(&vec_o_obs);
    Ok(())
}

#[tokio::test]
async fn test_cache_byte_budget_drops_least_recently_used_hosts() -> TestResult {
    let o_config = CollectorConfig {
        u_max_cached_bytes: 1,
        ..CollectorConfig::new("proofcrawl")
    };
    let o_h = harness_with(o_config, MemoryBodyArchive::new()).await?;
    for str_host in ["sig-ok.mock", "sig-ai.mock", "sig-ok.mock"] {
        o_h.advance(std::time::Duration::from_secs(1));
        collect(&o_h, str_host, "/page").await?;
    }
    assert_eq!(
        o_h.count("sig-ok.mock", "/robots.txt"),
        2,
        "dropped for bytes, refetched"
    );
    assert!(
        o_h.o_collector.cached_bytes() < 64 * 1024,
        "only the most recent host remains: {} bytes",
        o_h.o_collector.cached_bytes()
    );
    Ok(())
}

#[tokio::test]
async fn test_archived_prefix_matches_the_full_body_the_server_sent() -> TestResult {
    let o_h = harness(100).await?;
    let vec_o_obs = collect(&o_h, "sig-2mib.mock", "/x").await?;
    let str_sha = vec_o_obs[0].opt_str_content_sha256.clone().ok_or("hash")?;
    let by_stored = o_h.o_archive.load(&str_sha).ok_or("stored")?;
    let o_full = o_h
        .o_client
        .fetch(FetchRequest::new(o_h.url("sig-2mib.mock", "/robots.txt")?))
        .await?;
    assert_eq!(
        (o_full.b_truncated, o_full.u_body_bytes_seen),
        (false, U_TWO_MIB),
        "page cap (5 MB) reads it all"
    );
    assert!(
        o_full.by_body.starts_with(&by_stored),
        "archive keeps the exact prefix"
    );
    Ok(())
}

#[tokio::test]
async fn test_untruncated_body_reports_its_full_size() -> TestResult {
    let o_h = harness(100).await?;
    let url_page = o_h.raw_url("sig-slow.mock", "/page")?;
    let o_robots = o_h.o_collector.collect(&url_page).await?.remove(0);
    let str_sha = o_robots.opt_str_content_sha256.clone().ok_or("hash")?;
    let by_stored = o_h.o_archive.load(&str_sha).ok_or("stored")?;
    assert!(!o_robots.b_body_truncated);
    assert_eq!(o_robots.u_body_bytes_seen, by_stored.len() as u64);
    Ok(())
}
