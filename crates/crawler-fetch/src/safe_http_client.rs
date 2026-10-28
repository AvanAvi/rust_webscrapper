//! The real fetcher: reqwest over rustls with vetted resolution, guarded redirects, and limits.

use std::sync::Arc;

use crawler_core::{Clock, CrawlerIdentity, Timestamp};
use reqwest::header::ACCEPT_ENCODING;
use reqwest::redirect::Policy;
use url::Url;

use crate::address_policy::AddressPolicy;
use crate::body_budget::{BodyBudget, BodyOverflow};
use crate::body_reader::BodyReader;
use crate::client_config::ClientConfig;
use crate::fetch_error::FetchError;
use crate::fetch_limits::FetchLimits;
use crate::fetch_log::FetchLog;
use crate::fetch_request::FetchRequest;
use crate::fetch_response::FetchResponse;
use crate::host_lookup::HostLookup;
use crate::http_fetcher::{FetchFuture, HttpFetcher};
use crate::redirect_guard::RedirectGuard;
use crate::response_head::ResponseHead;
use crate::url_safety::UrlSafety;
use crate::vetting_resolver::VettingResolver;

/// Reserved example domains (RFC 2606/6761); a contact URL under them is a placeholder.
const VEC_STR_PLACEHOLDER_DOMAINS: [&str; 4] =
    ["example", "example.com", "example.net", "example.org"];
/// Encodings the client asks for; all are decoded by `BodyReader`.
const STR_ACCEPT_ENCODING: &str = "gzip, deflate, br";

/// SSRF-guarded HTTP client. Each fetch gets a fresh reqwest client with no connection pool,
/// no proxies, no retries, and no referer, so nothing carries over between fetches.
pub struct SafeHttpClient {
    /// Additional root available only to the test-policy build.
    #[cfg(feature = "test-policy")]
    opt_o_test_root: Option<reqwest::Certificate>,
    /// Last URL-stripped transport error for typed test assertions.
    #[cfg(feature = "test-policy")]
    mtx_opt_err_test_transport: std::sync::Mutex<Option<reqwest::Error>>,
    /// Declared identity.
    o_identity: CrawlerIdentity,
    /// Address policy.
    o_policy: Arc<dyn AddressPolicy>,
    /// URL validator.
    o_safety: UrlSafety,
    /// Limits.
    o_limits: FetchLimits,
    /// Name lookup.
    o_lookup: Arc<dyn HostLookup>,
    /// Clock for timestamps.
    o_clock: Arc<dyn Clock>,
}

impl SafeHttpClient {
    /// Builds a client; refuses the test policy without `test-policy` and placeholder contacts.
    pub fn new(o_config: ClientConfig) -> Result<Self, FetchError> {
        let o_policy = o_config.e_policy.build()?;
        Self::check_contact(&o_config.o_identity)?;
        Ok(Self {
            #[cfg(feature = "test-policy")]
            opt_o_test_root: o_config.opt_o_test_root,
            #[cfg(feature = "test-policy")]
            mtx_opt_err_test_transport: std::sync::Mutex::new(None),
            o_safety: UrlSafety::new(Arc::clone(&o_policy)),
            o_policy,
            o_identity: o_config.o_identity,
            o_limits: o_config.o_limits,
            o_lookup: o_config.o_lookup,
            o_clock: o_config.o_clock,
        })
    }

    /// Returns `Ok` when a URL may be requested (scheme, credentials, IP-literal address).
    pub fn check_url(&self, url_target: &Url) -> Result<(), FetchError> {
        self.o_safety.check(url_target)
    }

    /// Refuses contact URLs under reserved example domains unless `test-policy` is compiled in.
    fn check_contact(o_identity: &CrawlerIdentity) -> Result<(), FetchError> {
        let str_contact = o_identity.to_record().str_contact_url;
        let str_host = Url::parse(&str_contact)
            .ok()
            .and_then(|url_c| url_c.host_str().map(str::to_owned))
            .unwrap_or_default();
        let b_placeholder = VEC_STR_PLACEHOLDER_DOMAINS
            .iter()
            .any(|str_d| str_host == *str_d || str_host.ends_with(&format!(".{str_d}")));
        if b_placeholder && !cfg!(feature = "test-policy") {
            return Err(FetchError::PlaceholderContact { str_host });
        }
        Ok(())
    }

    /// Performs one fetch end to end.
    async fn run(&self, o_request: FetchRequest) -> Result<FetchResponse, FetchError> {
        self.o_safety.check(&o_request.url_target)?;
        let o_log = Arc::new(FetchLog::new());
        let o_client = self.build_client(&o_request, &o_log)?;
        let dt_started_at = self.now()?;
        let res_o_response = o_client
            .get(o_request.url_target.clone())
            .header(ACCEPT_ENCODING, STR_ACCEPT_ENCODING)
            .send()
            .await;
        let o_response =
            res_o_response.map_err(|err_send| self.handle_send_error(err_send, &o_log))?;
        self.finish(
            o_response,
            &o_log,
            (dt_started_at, self.budget_for(&o_request)),
        )
        .await
    }

    /// Reads and decodes the body within limits and assembles the response.
    async fn finish(
        &self,
        mut o_response: reqwest::Response,
        o_log: &Arc<FetchLog>,
        o_start: (Timestamp, BodyBudget),
    ) -> Result<FetchResponse, FetchError> {
        let (dt_started_at, o_budget) = o_start;
        let o_head = ResponseHead::from_response(&o_response, self.o_clock.now());
        let o_reader = BodyReader::new(self.o_limits);
        let o_encoded = o_reader.read_encoded(&mut o_response, o_budget).await?;
        let u_encoded_bytes = o_encoded.by_body.len() as u64;
        let o_body = o_reader
            .decode(o_head.opt_str_encoding.as_deref(), o_encoded, o_budget)
            .await?;
        Ok(o_head.into_response(o_body, u_encoded_bytes, o_log, (dt_started_at, self.now()?)))
    }

    /// Returns the request's budget, never above the client cap; pages default to rejecting.
    fn budget_for(&self, o_request: &FetchRequest) -> BodyBudget {
        let o_default = BodyBudget {
            u_cap: self.o_limits.u_max_body_bytes,
            e_overflow: BodyOverflow::Reject,
        };
        let o_budget = o_request.opt_o_budget.unwrap_or(o_default);
        BodyBudget {
            u_cap: o_budget.u_cap.min(self.o_limits.u_max_body_bytes),
            e_overflow: o_budget.e_overflow,
        }
    }

    /// Builds the per-fetch reqwest client.
    fn build_client(
        &self,
        o_request: &FetchRequest,
        o_log: &Arc<FetchLog>,
    ) -> Result<reqwest::Client, FetchError> {
        let o_guard = RedirectGuard::new(
            self.o_safety.clone(),
            o_request.e_scope,
            self.o_limits.u_max_redirects,
            Arc::clone(o_log),
        );
        let o_resolver = VettingResolver::new(
            Arc::clone(&self.o_policy),
            Arc::clone(&self.o_lookup),
            Arc::clone(o_log),
        );
        self.base_builder()
            .redirect(Policy::custom(move |o_attempt| o_guard.decide(o_attempt)))
            .dns_resolver(Arc::new(o_resolver))
            .build()
            .map_err(|err_build| FetchError::Internal {
                str_reason: err_build.without_url().to_string(),
            })
    }

    /// Returns a builder with identity, timeouts, and everything that could leak state disabled.
    fn base_builder(&self) -> reqwest::ClientBuilder {
        let o_builder = reqwest::Client::builder()
            .user_agent(self.o_identity.user_agent())
            .no_proxy()
            .connect_timeout(self.o_limits.dur_connect)
            .read_timeout(self.o_limits.dur_read)
            .timeout(self.o_limits.dur_total)
            .pool_max_idle_per_host(0)
            .referer(false)
            .retry(reqwest::retry::never());
        #[cfg(feature = "test-policy")]
        let o_builder = o_builder.tls_certs_merge(self.opt_o_test_root.clone());
        o_builder
    }

    /// Takes the last transport error for typed certificate assertions in tests.
    #[cfg(feature = "test-policy")]
    pub fn take_test_transport_error(&self) -> Option<reqwest::Error> {
        self.mtx_opt_err_test_transport.lock().ok()?.take()
    }

    /// Maps an owned error and retains URL-stripped diagnostics only in test-policy builds.
    fn handle_send_error(&self, err_send: reqwest::Error, o_log: &FetchLog) -> FetchError {
        let err_send = err_send.without_url();
        let err_fetch = self.send_error(&err_send, o_log);
        #[cfg(feature = "test-policy")]
        if let Ok(mut o_slot) = self.mtx_opt_err_test_transport.lock() {
            *o_slot = Some(err_send);
        }
        err_fetch
    }

    /// Maps a send error, preferring a refusal recorded by the resolver or redirect guard.
    fn send_error(&self, err_send: &reqwest::Error, o_log: &FetchLog) -> FetchError {
        if let Some(err_refusal) = o_log.refusal() {
            return err_refusal;
        }
        if err_send.is_timeout() {
            return FetchError::Timeout;
        }
        let str_reason = self.chain(err_send);
        if err_send.is_connect() {
            return FetchError::Connect { str_reason };
        }
        FetchError::Transport { str_reason }
    }

    /// Describes an error and its sources; reqwest's own text has its URL removed.
    fn chain(&self, err_send: &reqwest::Error) -> String {
        let mut str_text = String::from(if err_send.is_connect() {
            "connect error"
        } else {
            "request error"
        });
        let mut opt_err_source = std::error::Error::source(err_send);
        while let Some(err_source) = opt_err_source {
            str_text.push_str(&format!(": {err_source}"));
            opt_err_source = err_source.source();
        }
        str_text
    }

    /// Returns the current time as a timestamp.
    fn now(&self) -> Result<Timestamp, FetchError> {
        self.o_clock
            .timestamp()
            .map_err(|err_time| FetchError::Internal {
                str_reason: err_time.to_string(),
            })
    }
}

impl HttpFetcher for SafeHttpClient {
    /// Fetches one URL within the safety limits.
    fn fetch(&self, o_request: FetchRequest) -> FetchFuture<'_> {
        Box::pin(self.run(o_request))
    }
}
