//! Collects the per-host signal observations for a page URL, with caching and single flight.

use std::sync::Arc;

use crawler_core::{BodyArchive, Clock, IdSource, SignalContext, SignalObservation};
use time::OffsetDateTime;
use url::Url;

use crate::collector::collector_config::CollectorConfig;
use crate::collector::fetched_file::FetchedFile;
use crate::collector::host_cache::{Acquire, HostCache, LeaderTicket};
use crate::collector::leader_guard::LeaderGuard;
use crate::collector::page_signal_reader::PageSignalReader;
use crate::collector::signal_file::SignalFile;
use crate::collector::signal_file_fetcher::SignalFileFetcher;
use crate::fetch_error::FetchError;
use crate::fetch_response::FetchResponse;
use crate::http_fetcher::HttpFetcher;

/// Fetches robots.txt, ai.txt, and tdmrep.json once per host, caches present and absent results
/// for the TTL and failures for the backoff delay only, and lets concurrent callers share one
/// in-flight fetch per file. Cached results apply in the context of the initial origin
/// (RFC 9309 section 2.3.1.2), even after a cross-host redirect.
pub struct SignalCollector {
    /// Per-host cache with single flight.
    o_cache: Arc<HostCache>,
    /// Signal-file fetcher.
    o_files: SignalFileFetcher,
    /// Clock for expiry.
    o_clock: Arc<dyn Clock>,
    /// Randomness for backoff jitter.
    o_ids: Arc<dyn IdSource>,
    /// Settings.
    o_config: CollectorConfig,
    /// Archive for page-level header blocks.
    o_archive: Arc<dyn BodyArchive>,
}

impl SignalCollector {
    /// Creates a collector.
    pub fn new(
        o_fetcher: Arc<dyn HttpFetcher>,
        o_archive: Arc<dyn BodyArchive>,
        o_clock: Arc<dyn Clock>,
        o_config: CollectorConfig,
    ) -> Self {
        Self::with_ids(
            o_fetcher,
            o_archive,
            (o_clock, Arc::new(crawler_core::RandomIdSource::new())),
            o_config,
        )
    }

    /// Creates a collector with an explicit randomness source for jitter.
    pub fn with_ids(
        o_fetcher: Arc<dyn HttpFetcher>,
        o_archive: Arc<dyn BodyArchive>,
        o_time: (Arc<dyn Clock>, Arc<dyn IdSource>),
        o_config: CollectorConfig,
    ) -> Self {
        let (o_clock, o_ids) = o_time;
        Self {
            o_cache: Arc::new(HostCache::new(
                o_config.u_max_hosts,
                o_config.u_max_cached_bytes,
                Arc::clone(&o_archive),
            )),
            o_files: SignalFileFetcher::new(
                o_fetcher,
                Arc::clone(&o_archive),
                Arc::clone(&o_clock),
                o_config.u_max_signal_bytes,
            ),
            o_clock,
            o_ids,
            o_config,
            o_archive,
        }
    }

    /// Returns the robots.txt, ai.txt, and tdmrep.json observations for a page URL.
    pub async fn collect(&self, url_page: &Url) -> Result<Vec<SignalObservation>, FetchError> {
        let url_origin = url_page
            .join("/")
            .map_err(|err_url| FetchError::InvalidUrl {
                str_reason: err_url.to_string(),
            })?;
        let o_context =
            SignalContext::new(&self.o_config.str_product_token, &self.path_of(url_page));
        let mut vec_o_observations = Vec::with_capacity(SignalFile::VEC_E_ALL.len());
        for e_file in SignalFile::VEC_E_ALL {
            vec_o_observations.push(self.file(&url_origin, e_file).await?.observe(&o_context));
        }
        Ok(vec_o_observations)
    }

    /// Returns the cached or freshly fetched outcome of one file; concurrent callers share one fetch.
    pub async fn file(
        &self,
        url_origin: &Url,
        e_file: SignalFile,
    ) -> Result<Arc<FetchedFile>, FetchError> {
        loop {
            match self
                .o_cache
                .acquire(url_origin.as_str(), e_file, self.o_clock.now())
            {
                Acquire::Hit(o_file) => return Ok(o_file),
                Acquire::Wait(o_notified) => o_notified.await,
                Acquire::Lead(o_ticket) => return self.lead(o_ticket, url_origin, e_file).await,
                Acquire::Bypass => return self.bypass(url_origin, e_file).await,
            }
        }
    }

    /// Returns the summed weight of the cached files, in bytes.
    pub fn cached_bytes(&self) -> usize {
        self.o_cache.cached_bytes()
    }

    /// Returns the page-level observations (X-Robots-Tag, robots meta, TDMRep header and meta).
    pub fn page_observations(
        &self,
        o_response: &FetchResponse,
    ) -> Result<Vec<SignalObservation>, FetchError> {
        let dt_observed_at = self
            .o_clock
            .timestamp()
            .map_err(|err_t| FetchError::Internal {
                str_reason: err_t.to_string(),
            })?;
        Ok(PageSignalReader::new(
            &self.o_config.str_product_token,
            Arc::clone(&self.o_archive),
        )
        .observations(o_response, dt_observed_at))
    }

    /// Fetches as the single-flight leader and stores the result; the guard covers cancellation.
    async fn lead(
        &self,
        o_ticket: LeaderTicket,
        url_origin: &Url,
        e_file: SignalFile,
    ) -> Result<Arc<FetchedFile>, FetchError> {
        let u_failures_before = o_ticket.u_failures;
        let o_guard = LeaderGuard::new(Arc::clone(&self.o_cache), o_ticket);
        let o_file = Arc::new(self.o_files.fetch(url_origin, e_file).await?);
        o_guard.finish(Arc::clone(&o_file), self.expiry(&o_file, u_failures_before));
        Ok(o_file)
    }

    /// Fetches without caching; the archive reference is not kept, so the body stays evictable.
    async fn bypass(
        &self,
        url_origin: &Url,
        e_file: SignalFile,
    ) -> Result<Arc<FetchedFile>, FetchError> {
        let o_file = self.o_files.fetch(url_origin, e_file).await?;
        self.o_cache.release_file(&o_file);
        Ok(Arc::new(o_file))
    }

    /// Returns the expiry and failure streak: TTL after a response, backoff after a failure.
    fn expiry(&self, o_file: &FetchedFile, u_failures_before: u32) -> (OffsetDateTime, u32) {
        let dt_now = self.o_clock.now();
        if !o_file.is_failure() {
            return (self.after(dt_now, self.o_config.effective_ttl()), 0);
        }
        let u_failures = u_failures_before.saturating_add(1);
        let dur_wait = self
            .o_config
            .o_backoff
            .delay(u_failures, self.o_ids.next_id().as_u128());
        (self.after(dt_now, dur_wait), u_failures)
    }

    /// Adds a duration to a time without overflowing.
    fn after(&self, dt_now: OffsetDateTime, dur_wait: std::time::Duration) -> OffsetDateTime {
        let dur_span = time::Duration::try_from(dur_wait).unwrap_or(time::Duration::MAX);
        dt_now.saturating_add(dur_span)
    }

    /// Returns the path plus `?query` that robots rules compare.
    fn path_of(&self, url_page: &Url) -> String {
        match url_page.query() {
            Some(str_query) => format!("{}?{str_query}", url_page.path()),
            None => url_page.path().to_owned(),
        }
    }
}
