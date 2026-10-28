//! Fetches one signal file and turns the result into a typed, archived, parsed outcome.

use std::sync::Arc;

use crawler_core::{BodyArchive, Clock, Excerpt, SignalFailure, SignalStatus, Timestamp};
use url::Url;

use crate::body_budget::BodyOverflow;
use crate::collector::fetched_file::FetchedFile;
use crate::collector::signal_document::SignalDocument;
use crate::collector::signal_file::SignalFile;
use crate::fetch_error::FetchError;
use crate::fetch_request::FetchRequest;
use crate::fetch_response::FetchResponse;
use crate::http_fetcher::HttpFetcher;

/// Fetches signal files through the guarded client and archives every body by hash.
///
/// Fail closed: when the archive refuses a body, the file is unreachable with the failure
/// `evidence_not_archived`, carries no hash or parsed content, and the policy denies on it.
///
/// Status mapping (RFC 9309 sections 2.3.1.1 to 2.3.1.4): 2xx is present; 429 is unreachable
/// (rate limited); other 4xx is absent ("unavailable"); 5xx and every fetch error are unreachable.
pub struct SignalFileFetcher {
    /// Guarded HTTP client.
    o_fetcher: Arc<dyn HttpFetcher>,
    /// Content-addressed body store.
    o_archive: Arc<dyn BodyArchive>,
    /// Clock for `observed_at`.
    o_clock: Arc<dyn Clock>,
    /// Body cap for signal files (truncated, not rejected).
    u_max_bytes: usize,
}

impl SignalFileFetcher {
    /// Creates a fetcher.
    pub fn new(
        o_fetcher: Arc<dyn HttpFetcher>,
        o_archive: Arc<dyn BodyArchive>,
        o_clock: Arc<dyn Clock>,
        u_max_bytes: usize,
    ) -> Self {
        Self {
            o_fetcher,
            o_archive,
            o_clock,
            u_max_bytes,
        }
    }

    /// Fetches one file of the origin; only an invalid URL or clock is an error.
    pub async fn fetch(
        &self,
        url_origin: &Url,
        e_file: SignalFile,
    ) -> Result<FetchedFile, FetchError> {
        let url_file =
            url_origin
                .join(e_file.path())
                .map_err(|err_url| FetchError::InvalidUrl {
                    str_reason: err_url.to_string(),
                })?;
        let dt_observed_at = self
            .o_clock
            .timestamp()
            .map_err(|err_t| FetchError::Internal {
                str_reason: err_t.to_string(),
            })?;
        let o_request = FetchRequest::new(url_file.clone())
            .with_body_cap(self.u_max_bytes, BodyOverflow::Truncate);
        let o_base = self.base(e_file, url_file.as_str(), dt_observed_at);
        Ok(match self.o_fetcher.fetch(o_request).await {
            Ok(o_response) => self.answered(o_base, &o_response),
            Err(err_fetch) => FetchedFile {
                opt_e_failure: Some(self.failure_of(&err_fetch)),
                ..o_base
            },
        })
    }

    /// Returns an unreachable outcome to be filled in.
    fn base(
        &self,
        e_file: SignalFile,
        str_source_url: &str,
        dt_observed_at: Timestamp,
    ) -> FetchedFile {
        FetchedFile {
            e_file,
            e_status: SignalStatus::Unreachable,
            str_source_url: str_source_url.to_owned(),
            opt_str_final_url: None,
            vec_o_redirects: Vec::new(),
            opt_u_http_status: None,
            opt_e_failure: None,
            opt_str_sha256: None,
            u_stored_bytes: 0,
            b_body_truncated: false,
            u_body_bytes_seen: 0,
            opt_o_excerpt: None,
            dt_observed_at,
            e_document: SignalDocument::Unparsed,
        }
    }

    /// Classifies a response, archives its body, and parses it when present.
    fn answered(&self, o_base: FetchedFile, o_response: &FetchResponse) -> FetchedFile {
        let o_answered = FetchedFile {
            opt_str_final_url: Some(o_response.str_final_url.clone()),
            vec_o_redirects: o_response.vec_o_redirects.clone(),
            opt_u_http_status: Some(o_response.u_status),
            b_body_truncated: o_response.b_truncated,
            u_body_bytes_seen: o_response.u_body_bytes_seen,
            ..o_base
        };
        match self.o_archive.store(&o_response.by_body) {
            Ok(str_sha256) => self.archived(o_answered, &o_response.by_body, str_sha256),
            Err(_) => FetchedFile {
                e_status: SignalStatus::Unreachable,
                opt_e_failure: Some(SignalFailure::EvidenceNotArchived),
                ..o_answered
            },
        }
    }

    /// Fills in status, hash, excerpt, and parsed content for an archived body.
    fn archived(&self, o_answered: FetchedFile, by_body: &[u8], str_sha256: String) -> FetchedFile {
        let (e_status, opt_e_failure) = self.classify(o_answered.opt_u_http_status.unwrap_or(0));
        let e_document = match e_status {
            SignalStatus::Present => SignalDocument::parse(o_answered.e_file, by_body),
            _ => SignalDocument::Unparsed,
        };
        let opt_o_excerpt = Some(Excerpt::new(&String::from_utf8_lossy(by_body)));
        FetchedFile {
            e_status,
            opt_e_failure,
            opt_str_sha256: Some(str_sha256),
            u_stored_bytes: by_body.len(),
            opt_o_excerpt,
            e_document,
            ..o_answered
        }
    }

    /// Maps a final HTTP status to the observation status.
    fn classify(&self, u_status: u16) -> (SignalStatus, Option<SignalFailure>) {
        match u_status {
            200..=299 => (SignalStatus::Present, None),
            429 => (SignalStatus::Unreachable, Some(SignalFailure::RateLimited)),
            400..=499 => (SignalStatus::Absent, None),
            500..=599 => (SignalStatus::Unreachable, Some(SignalFailure::ServerError)),
            _ => (
                SignalStatus::Unreachable,
                Some(SignalFailure::RedirectRefused),
            ),
        }
    }

    /// Maps a fetch error to the failure recorded with an unreachable observation.
    fn failure_of(&self, err_fetch: &FetchError) -> SignalFailure {
        match err_fetch {
            FetchError::Timeout => SignalFailure::Timeout,
            FetchError::BlockedAddress { .. } => SignalFailure::BlockedAddress,
            FetchError::DnsFailure { .. } => SignalFailure::DnsFailure,
            FetchError::TooManyRedirects { .. }
            | FetchError::RedirectDowngrade { .. }
            | FetchError::RedirectOutOfScope { .. } => SignalFailure::RedirectRefused,
            FetchError::BodyTooLarge { .. }
            | FetchError::CompressionRatioExceeded { .. }
            | FetchError::UnsupportedEncoding { .. }
            | FetchError::Decode { .. } => SignalFailure::BadBody,
            _ => SignalFailure::ConnectionFailed,
        }
    }
}
