//! Shared setup for the mock-site scenario tests: scripted DNS, short limits, loopback policy.

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Duration;

use crawler_core::{CrawlerIdentity, Purpose};
use crawler_fetch::{
    ClientConfig, FetchError, FetchLimits, FetchRequest, FetchResponse, HostScope, HttpFetcher,
    SafeHttpClient, ScriptedHostLookup,
};
use mock_site::MockSite;
use url::Url;

/// Boxed error for helpers.
pub type BoxError = Box<dyn std::error::Error>;
/// Outcome of one fetch.
pub type FetchResult = Result<FetchResponse, FetchError>;

/// Loopback, where the mock site listens.
pub const E_LOOPBACK: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
/// 0.0.0.0 reaches the 127.0.0.1 listener if ever connected to, so a forbidden connection
/// would show up in the mock site's request log.
pub const E_BLOCKED: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
/// Mock hosts that resolve to loopback.
const VEC_STR_HOSTS: [&str; 12] = [
    "ok.mock",
    "private-redirect.mock",
    "hops.mock",
    "scope.mock",
    "bomb.mock",
    "big.mock",
    "ratelimit.mock",
    "malformed.mock",
    "links.mock",
    "drip.mock",
    "huge.mock",
    "endless.mock",
];

/// Returns the scripted resolver: mock hosts to loopback, internal.mock to 0.0.0.0, and
/// rebind.mock to loopback first and 0.0.0.0 on the second lookup.
pub fn lookup() -> ScriptedHostLookup {
    let o_lookup = VEC_STR_HOSTS
        .iter()
        .fold(ScriptedHostLookup::new(), |o_l, str_h| {
            o_l.with(str_h, vec![E_LOOPBACK])
        });
    o_lookup
        .with("internal.mock", vec![E_BLOCKED])
        .with_sequence("rebind.mock", vec![vec![E_LOOPBACK], vec![E_BLOCKED]])
}

/// Starts the mock site and a client with a 3-second total deadline.
pub async fn start() -> Result<(MockSite, SafeHttpClient), BoxError> {
    start_with_total(Duration::from_secs(3)).await
}

/// Starts the mock site and a client whose total deadline is `dur_total`.
pub async fn start_with_total(dur_total: Duration) -> Result<(MockSite, SafeHttpClient), BoxError> {
    let o_identity = CrawlerIdentity::new(
        "proofcrawl",
        "0.1.0",
        "https://crawler.example/contact",
        Purpose::AiTrain,
    )?;
    let o_limits = FetchLimits {
        dur_connect: Duration::from_secs(1),
        dur_read: Duration::from_secs(5),
        dur_total,
        ..FetchLimits::default()
    };
    let o_config = ClientConfig::new(o_identity)
        .with_test_policy()
        .with_limits(o_limits)
        .with_lookup(Arc::new(lookup()));
    Ok((MockSite::start().await?, SafeHttpClient::new(o_config)?))
}

/// Fetches a URL string with the given redirect scope.
pub async fn fetch(
    o_client: &SafeHttpClient,
    str_url: &str,
    e_scope: HostScope,
) -> Result<FetchResult, BoxError> {
    Ok(o_client
        .fetch(FetchRequest::new(Url::parse(str_url)?).with_scope(e_scope))
        .await)
}
