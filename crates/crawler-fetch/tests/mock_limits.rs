//! Mock-site scenarios for size, ratio, time, and rate limits (needs `--features test-policy`).
#![cfg(feature = "test-policy")]

#[path = "fetch_support/mock_client.rs"]
mod mock_client;

use std::time::{Duration, Instant};

use crawler_core::{ParsedSignal, RobotsTagParser, RobotsTagSource, SignalContext, SignalParser};
use crawler_fetch::{FetchError, HostScope, U_MAX_BODY_BYTES};
use mock_client::{fetch, start, start_with_total};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;
/// Deadline for scenarios where the mock compresses tens of megabytes first.
const DUR_SLOW_SERVER: Duration = Duration::from_secs(20);

#[tokio::test]
async fn test_gzip_bomb_hits_the_ratio_limit() -> TestResult {
    let (o_site, o_client) = start_with_total(DUR_SLOW_SERVER).await?;
    let res_o_resp = fetch(&o_client, &o_site.url("bomb.mock", "/"), HostScope::AnyHost).await?;
    assert_eq!(
        res_o_resp,
        Err(FetchError::CompressionRatioExceeded { u_limit: 20 })
    );
    Ok(())
}

#[tokio::test]
async fn test_decompressed_size_is_capped() -> TestResult {
    let (o_site, o_client) = start_with_total(DUR_SLOW_SERVER).await?;
    let res_o_resp = fetch(&o_client, &o_site.url("big.mock", "/"), HostScope::AnyHost).await?;
    assert!(
        matches!(
            res_o_resp,
            Err(FetchError::BodyTooLarge {
                u_limit: U_MAX_BODY_BYTES
            })
        ),
        "got {:?}",
        res_o_resp.as_ref().map(|o_r| o_r.by_body.len())
    );
    Ok(())
}

#[tokio::test]
async fn test_oversized_raw_bodies_are_capped() -> TestResult {
    let (o_site, o_client) = start().await?;
    for str_host in ["huge.mock", "endless.mock"] {
        let res_o_resp = fetch(
            &o_client,
            &o_site.raw_url(str_host, "/"),
            HostScope::AnyHost,
        )
        .await?;
        assert_eq!(
            res_o_resp,
            Err(FetchError::BodyTooLarge {
                u_limit: U_MAX_BODY_BYTES
            }),
            "{str_host}"
        );
    }
    assert_eq!(
        o_site.raw_connections(),
        2,
        "the counter sees connections the policy allows"
    );
    Ok(())
}

#[tokio::test]
async fn test_slow_drip_times_out() -> TestResult {
    let (o_site, o_client) = start().await?;
    let dt_start = Instant::now();
    let res_o_resp = fetch(
        &o_client,
        &o_site.raw_url("drip.mock", "/"),
        HostScope::AnyHost,
    )
    .await?;
    assert_eq!(res_o_resp, Err(FetchError::Timeout));
    assert!(
        dt_start.elapsed() < Duration::from_secs(5),
        "took {:?}",
        dt_start.elapsed()
    );
    Ok(())
}

#[tokio::test]
async fn test_rate_limit_is_reported_without_retrying() -> TestResult {
    let (o_site, o_client) = start().await?;
    let o_resp = fetch(
        &o_client,
        &o_site.url("ratelimit.mock", "/"),
        HostScope::AnyHost,
    )
    .await??;
    assert_eq!(
        (o_resp.u_status, o_resp.opt_u_retry_after_secs),
        (429, Some(120))
    );
    assert_eq!(o_site.requests().len(), 1, "the client must not retry");
    Ok(())
}

#[tokio::test]
async fn test_malformed_html_is_returned_and_parses_safely() -> TestResult {
    let (o_site, o_client) = start().await?;
    let o_bad = fetch(
        &o_client,
        &o_site.url("malformed.mock", "/"),
        HostScope::AnyHost,
    )
    .await??;
    assert!(o_bad.by_body.contains(&0xff));
    let e_parsed = RobotsTagParser::new(RobotsTagSource::Meta)
        .parse(&o_bad.by_body, &SignalContext::new("proofcrawl", "/"));
    assert!(matches!(e_parsed, ParsedSignal::RobotsTag(_)));
    Ok(())
}

#[tokio::test]
async fn test_link_explosion_stays_within_the_cap() -> TestResult {
    let (o_site, o_client) = start().await?;
    let o_links = fetch(
        &o_client,
        &o_site.url("links.mock", "/"),
        HostScope::AnyHost,
    )
    .await??;
    assert!(o_links.by_body.len() <= U_MAX_BODY_BYTES);
    assert_eq!(
        String::from_utf8_lossy(&o_links.by_body)
            .matches("<a href=")
            .count(),
        20_000
    );
    Ok(())
}
