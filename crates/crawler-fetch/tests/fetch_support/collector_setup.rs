//! Shared setup for the signal-collector tests: mock site, test-policy client, fake clock, archive.

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Duration;

use crawler_core::SequentialIdSource;
use crawler_core::{Clock, CrawlerIdentity, FakeClock, IdSource, MemoryBodyArchive, Purpose};
use crawler_fetch::SignalCollector;
use crawler_fetch::{
    ClientConfig, CollectorConfig, FetchLimits, SafeHttpClient, ScriptedHostLookup,
};
use mock_site::MockSite;
use time::OffsetDateTime;
use url::Url;

/// Boxed error for helpers.
pub type BoxError = Box<dyn std::error::Error>;

/// Mock hosts used by the collector tests, all resolving to loopback.
const VEC_STR_SIG_HOSTS: [&str; 14] = [
    "sig-ok.mock",
    "sig-404.mock",
    "sig-500.mock",
    "sig-redirect.mock",
    "sig-target.mock",
    "sig-big.mock",
    "sig-2mib.mock",
    "sig-ai.mock",
    "sig-tdm-invalid.mock",
    "sig-tdm-float.mock",
    "sig-429.mock",
    "sig-503.mock",
    "sig-timeout.mock",
    "sig-slow.mock",
];
/// Start of fake time: 2026-10-28T00:00:00Z.
const I_START_UNIX: i64 = 1_793_145_600;

/// Everything a collector test needs.
pub struct Harness {
    /// Running mock site.
    pub o_site: MockSite,
    /// Collector under test.
    pub o_collector: Arc<SignalCollector>,
    /// Client the collector uses (also for page fetches).
    pub o_client: Arc<SafeHttpClient>,
    /// Fake clock driving observed_at, TTL, and backoff.
    pub o_clock: Arc<FakeClock>,
    /// Body archive.
    pub o_archive: Arc<MemoryBodyArchive>,
}

impl Harness {
    /// Returns an HTTP-server URL on a mock host.
    pub fn url(&self, str_host: &str, str_path: &str) -> Result<Url, BoxError> {
        Ok(Url::parse(&self.o_site.url(str_host, str_path))?)
    }

    /// Returns a raw-server URL on a mock host.
    pub fn raw_url(&self, str_host: &str, str_path: &str) -> Result<Url, BoxError> {
        Ok(Url::parse(&self.o_site.raw_url(str_host, str_path))?)
    }

    /// Returns how many requests a host received for a path.
    pub fn count(&self, str_host: &str, str_path: &str) -> usize {
        let vec_o_requests = self.o_site.requests();
        vec_o_requests
            .iter()
            .filter(|o_r| o_r.str_host == str_host && o_r.str_path == str_path)
            .count()
    }

    /// Moves fake time forward.
    pub fn advance(&self, dur_step: Duration) {
        self.o_clock
            .advance(time::Duration::try_from(dur_step).unwrap_or(time::Duration::ZERO));
    }
}

/// Returns a test-policy client that resolves every `sig-*.mock` host to loopback.
fn client() -> Result<Arc<SafeHttpClient>, BoxError> {
    let e_loopback = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let o_lookup = VEC_STR_SIG_HOSTS
        .iter()
        .fold(ScriptedHostLookup::new(), |o_l, str_h| {
            o_l.with(str_h, vec![e_loopback])
        });
    let o_identity = CrawlerIdentity::new(
        "proofcrawl",
        "0.1.0",
        "https://crawler.example/contact",
        Purpose::AiTrain,
    )?;
    let o_limits = FetchLimits {
        dur_connect: Duration::from_secs(1),
        dur_read: Duration::from_secs(2),
        dur_total: Duration::from_secs(3),
        ..FetchLimits::default()
    };
    let o_config = ClientConfig::new(o_identity)
        .with_test_policy()
        .with_limits(o_limits);
    Ok(Arc::new(SafeHttpClient::new(
        o_config.with_lookup(Arc::new(o_lookup)),
    )?))
}

/// Starts the mock site and a collector holding at most `u_max_hosts` hosts.
pub async fn harness(u_max_hosts: usize) -> Result<Harness, BoxError> {
    let o_config = CollectorConfig {
        u_max_hosts,
        ..CollectorConfig::new("proofcrawl")
    };
    harness_with(o_config, MemoryBodyArchive::new()).await
}

/// Starts the mock site and a collector with explicit settings and archive.
pub async fn harness_with(
    o_config: CollectorConfig,
    o_archive: MemoryBodyArchive,
) -> Result<Harness, BoxError> {
    let o_client = client()?;
    let o_clock = Arc::new(FakeClock::new(OffsetDateTime::from_unix_timestamp(
        I_START_UNIX,
    )?));
    let o_archive = Arc::new(o_archive);
    let o_ids: Arc<dyn IdSource> = Arc::new(SequentialIdSource::new(1));
    let o_time = (Arc::clone(&o_clock) as Arc<dyn Clock>, o_ids);
    let o_collector =
        SignalCollector::with_ids(o_client.clone(), o_archive.clone(), o_time, o_config);
    let o_site = MockSite::start().await?;
    Ok(Harness {
        o_site,
        o_collector: Arc::new(o_collector),
        o_client,
        o_clock,
        o_archive,
    })
}
