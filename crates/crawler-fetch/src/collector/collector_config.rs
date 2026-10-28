//! Holds the settings of the signal collector.

use std::time::Duration;

use crate::collector::backoff_policy::BackoffPolicy;

/// How long a successful signal fetch is reused.
pub const DUR_SIGNAL_TTL: Duration = Duration::from_secs(3600);
/// Upper bound on the TTL: RFC 9309 section 2.4 says a cached robots.txt SHOULD NOT be used
/// for more than 24 hours.
pub const DUR_MAX_SIGNAL_TTL: Duration = Duration::from_secs(24 * 3600);
/// Most hosts kept in the cache; at the limit the least recently used idle host is dropped.
pub const U_MAX_CACHED_HOSTS: usize = 10_000;
/// Most bytes the cache accounts for (archived bodies plus parsed forms). Without it a full
/// 10,000-host cache could pin 10,000 x 3 x 1 MiB = about 30 GiB; with it the cache stays near
/// 128 MiB, half the default archive, so bodies it pins never fill the archive.
pub const U_MAX_CACHED_BYTES: usize = 128 * 1024 * 1024;
/// Fetch cap for signal files; longer bodies are truncated (pages keep the 5 MB cap).
pub const U_MAX_SIGNAL_BYTES: usize = 1024 * 1024;

/// Collector settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectorConfig {
    /// Crawler product token used to pick robots.txt groups.
    pub str_product_token: String,
    /// Time a successful (present or absent) fetch is reused; never above 24 hours.
    pub dur_ttl: Duration,
    /// Most cached hosts.
    pub u_max_hosts: usize,
    /// Most bytes the cached files account for.
    pub u_max_cached_bytes: usize,
    /// Retry schedule for failed fetches.
    pub o_backoff: BackoffPolicy,
    /// Signal-file body cap.
    pub u_max_signal_bytes: usize,
}

impl CollectorConfig {
    /// Creates the default configuration for a product token.
    pub fn new(str_product_token: &str) -> Self {
        Self {
            str_product_token: str_product_token.to_owned(),
            dur_ttl: DUR_SIGNAL_TTL,
            u_max_hosts: U_MAX_CACHED_HOSTS,
            u_max_cached_bytes: U_MAX_CACHED_BYTES,
            o_backoff: BackoffPolicy::default(),
            u_max_signal_bytes: U_MAX_SIGNAL_BYTES,
        }
    }

    /// Returns the TTL, clamped to the RFC 9309 maximum.
    pub fn effective_ttl(&self) -> Duration {
        self.dur_ttl.min(DUR_MAX_SIGNAL_TTL)
    }
}
