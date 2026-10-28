//! Signal collector caching: single flight, cancellation takeover, shared failures, backoff, TTL,
//! host bound, and archive round trips (needs `--features test-policy`).
#![cfg(feature = "test-policy")]

#[path = "fetch_support/collector_setup.rs"]
mod collector_setup;

use std::sync::Arc;
use std::time::Duration;

use collector_setup::{BoxError, Harness, harness};
use crawler_core::{
    AiTxtParser, BodyArchive, ParsedSignal, RobotsParser, SignalContext, SignalFailure, SignalKind,
    SignalObservation, SignalParser, SignalStatus, TdmrepParser, TdmrepSource,
};
use crawler_fetch::SignalFile;
use tokio::task::JoinSet;
use url::Url;

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;
/// Outcome of one `file` call.
type FileResult = Result<Arc<crawler_fetch::FetchedFile>, crawler_fetch::FetchError>;
/// Signal-file paths.
const VEC_STR_FILES: [&str; 3] = ["/robots.txt", "/ai.txt", "/.well-known/tdmrep.json"];

/// Runs `u_tasks` concurrent collections of one URL and returns every result.
async fn collect_concurrently(
    o_h: &Harness,
    url_page: &Url,
    u_tasks: usize,
) -> Result<Vec<Vec<SignalObservation>>, BoxError> {
    let mut o_set = JoinSet::new();
    for _ in 0..u_tasks {
        let (o_collector, url_task) = (Arc::clone(&o_h.o_collector), url_page.clone());
        o_set.spawn(async move { o_collector.collect(&url_task).await });
    }
    let mut vec_vec_results = Vec::new();
    while let Some(res_o_joined) = o_set.join_next().await {
        vec_vec_results.push(res_o_joined??);
    }
    Ok(vec_vec_results)
}

#[tokio::test]
async fn test_fifty_concurrent_evaluations_fetch_each_file_once() -> TestResult {
    let o_h = harness(100).await?;
    let vec_vec_results = collect_concurrently(&o_h, &o_h.url("sig-ok.mock", "/page")?, 50).await?;
    assert_eq!(vec_vec_results.len(), 50);
    for str_path in VEC_STR_FILES {
        assert_eq!(o_h.count("sig-ok.mock", str_path), 1, "{str_path}");
    }
    assert!(
        vec_vec_results
            .windows(2)
            .all(|vec_pair| vec_pair[0] == vec_pair[1]),
        "all callers see the same observations"
    );
    Ok(())
}

/// Spawns tasks that each ask for robots.txt of an origin.
fn spawn_file_tasks(o_h: &Harness, url_origin: &Url, u_tasks: usize) -> JoinSet<FileResult> {
    let mut o_set = JoinSet::new();
    for _ in 0..u_tasks {
        let (o_collector, url_task) = (Arc::clone(&o_h.o_collector), url_origin.clone());
        o_set.spawn(async move { o_collector.file(&url_task, SignalFile::RobotsTxt).await });
    }
    o_set
}

#[tokio::test]
async fn test_cancelled_leader_hands_over_to_a_waiter() -> TestResult {
    let o_h = harness(100).await?;
    let url_origin = o_h.raw_url("sig-slow.mock", "/")?;
    let mut o_leader = spawn_file_tasks(&o_h, &url_origin, 1);
    tokio::time::sleep(Duration::from_millis(150)).await;
    let o_waiters = spawn_file_tasks(&o_h, &url_origin, 5);
    tokio::time::sleep(Duration::from_millis(150)).await;
    o_leader.abort_all();
    let vec_res_files = tokio::time::timeout(Duration::from_secs(5), o_waiters.join_all()).await?;
    for res_o_file in vec_res_files {
        assert_eq!(
            res_o_file?.e_status,
            SignalStatus::Present,
            "a waiter takes over"
        );
    }
    assert_eq!(
        o_h.count("sig-slow.mock", "/robots.txt"),
        2,
        "cancelled leader plus takeover"
    );
    Ok(())
}

#[tokio::test]
async fn test_shared_failure_reaches_every_waiter_without_retry_storm() -> TestResult {
    let o_h = harness(100).await?;
    let url_page = o_h.url("sig-500.mock", "/page")?;
    let vec_vec_results = collect_concurrently(&o_h, &url_page, 50).await?;
    for vec_o_obs in &vec_vec_results {
        assert_eq!(
            (vec_o_obs[0].e_status, vec_o_obs[0].opt_e_failure),
            (SignalStatus::Unreachable, Some(SignalFailure::ServerError))
        );
    }
    o_h.o_collector.collect(&url_page).await?;
    assert_eq!(
        o_h.count("sig-500.mock", "/robots.txt"),
        1,
        "50 callers and a later one share one request"
    );
    Ok(())
}

#[tokio::test]
async fn test_failures_are_retried_with_exponential_backoff_not_after_the_ttl() -> TestResult {
    let o_h = harness(100).await?;
    let url_page = o_h.url("sig-500.mock", "/page")?;
    let vec_o_steps: [(u64, usize); 5] = [(0, 1), (40, 1), (40, 2), (90, 2), (60, 3)];
    for (u_advance_secs, u_expected) in vec_o_steps {
        o_h.advance(Duration::from_secs(u_advance_secs));
        o_h.o_collector.collect(&url_page).await?;
        assert_eq!(
            o_h.count("sig-500.mock", "/robots.txt"),
            u_expected,
            "after +{u_advance_secs}s"
        );
    }
    Ok(())
}

#[tokio::test]
async fn test_successful_fetches_are_reused_until_the_ttl() -> TestResult {
    let o_h = harness(100).await?;
    let url_page = o_h.url("sig-ok.mock", "/page")?;
    for (u_advance_secs, u_expected) in [(0u64, 1usize), (59 * 60, 1), (2 * 60, 2)] {
        o_h.advance(Duration::from_secs(u_advance_secs));
        o_h.o_collector.collect(&url_page).await?;
        assert_eq!(
            o_h.count("sig-ok.mock", "/robots.txt"),
            u_expected,
            "after +{u_advance_secs}s"
        );
    }
    Ok(())
}

#[tokio::test]
async fn test_host_bound_drops_the_least_recently_used_host() -> TestResult {
    let o_h = harness(2).await?;
    for str_host in [
        "sig-ok.mock",
        "sig-ai.mock",
        "sig-tdm-float.mock",
        "sig-ok.mock",
    ] {
        o_h.advance(Duration::from_secs(1));
        o_h.o_collector
            .collect(&o_h.url(str_host, "/page")?)
            .await?;
    }
    assert_eq!(
        o_h.count("sig-ok.mock", "/robots.txt"),
        2,
        "sig-ok was evicted at the 2-host bound and refetched"
    );
    assert_eq!(o_h.count("sig-ai.mock", "/robots.txt"), 1);
    Ok(())
}

/// Returns the parser that re-parses an archived body of a signal kind.
fn parser_for(e_kind: SignalKind) -> Box<dyn SignalParser> {
    match e_kind {
        SignalKind::RobotsTxt => Box::new(RobotsParser::new()),
        SignalKind::AiTxt => Box::new(AiTxtParser::new()),
        _ => Box::new(TdmrepParser::new(TdmrepSource::WellKnown)),
    }
}

/// Checks that an observation's hash resolves and its archived body re-parses to the same finding.
fn check_round_trip(
    o_h: &Harness,
    o_obs: &SignalObservation,
    o_context: &SignalContext,
) -> TestResult {
    let str_sha = o_obs
        .opt_str_content_sha256
        .clone()
        .ok_or("every answered fetch has a hash")?;
    let by_body = o_h.o_archive.load(&str_sha).ok_or("hash must resolve")?;
    let b_present = o_obs.e_status == SignalStatus::Present;
    let opt_e_reparsed: Option<ParsedSignal> =
        b_present.then(|| parser_for(o_obs.e_kind).parse(&by_body, o_context));
    assert_eq!(opt_e_reparsed, o_obs.opt_e_parsed, "{:?}", o_obs.e_kind);
    Ok(())
}

#[tokio::test]
async fn test_every_hash_resolves_and_reparses_identically() -> TestResult {
    let o_h = harness(100).await?;
    for str_host in ["sig-ok.mock", "sig-404.mock", "sig-tdm-float.mock"] {
        check_host_round_trips(&o_h, str_host).await?;
    }
    Ok(())
}

/// Collects one host's signals and checks every observation's archive round trip.
async fn check_host_round_trips(o_h: &Harness, str_host: &str) -> TestResult {
    let o_context = SignalContext::new("proofcrawl", "/reserved/x");
    let vec_o_obs = o_h
        .o_collector
        .collect(&o_h.url(str_host, "/reserved/x")?)
        .await?;
    for o_obs in &vec_o_obs {
        check_round_trip(o_h, o_obs, &o_context)?;
    }
    Ok(())
}

#[tokio::test]
async fn test_rate_limit_and_unavailable_carry_a_capped_retry_after() -> TestResult {
    use crawler_fetch::{DUR_MAX_RETRY_AFTER, FetchRequest, HttpFetcher};
    let o_h = harness(100).await?;
    let o_429 = o_h
        .o_client
        .fetch(FetchRequest::new(o_h.url("sig-429.mock", "/page")?))
        .await?;
    assert_eq!(o_429.opt_dur_retry_after, Some(Duration::from_secs(120)));
    let o_record = o_429.to_record(&crawler_core::Hasher::new());
    assert_eq!(
        o_record.opt_u_retry_after_ms,
        Some(120_000),
        "receipt keeps it in ms"
    );
    let o_503 = o_h
        .o_client
        .fetch(FetchRequest::new(o_h.url("sig-503.mock", "/page")?))
        .await?;
    assert_eq!(
        o_503.opt_dur_retry_after,
        Some(DUR_MAX_RETRY_AFTER),
        "a 2099 date is capped"
    );
    Ok(())
}

#[tokio::test]
async fn test_rate_limited_robots_is_unreachable_not_absent() -> TestResult {
    let o_h = harness(100).await?;
    let vec_o_obs = o_h
        .o_collector
        .collect(&o_h.url("sig-429.mock", "/page")?)
        .await?;
    let o_expected = (SignalStatus::Unreachable, Some(SignalFailure::RateLimited));
    assert_eq!(
        (vec_o_obs[0].e_status, vec_o_obs[0].opt_e_failure),
        o_expected
    );
    Ok(())
}
