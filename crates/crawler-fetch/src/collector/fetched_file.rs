//! Holds the outcome of fetching one signal file, ready to be cached.

use crawler_core::{
    Excerpt, RedirectHop, SignalContext, SignalFailure, SignalObservation, SignalStatus, Timestamp,
    U_MAX_EXCERPT_BYTES,
};

use crate::collector::signal_document::SignalDocument;
use crate::collector::signal_file::SignalFile;

/// Cached outcome of one signal-file fetch; evaluated into an observation per URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedFile {
    /// Which file.
    pub e_file: SignalFile,
    /// Present, absent, or unreachable.
    pub e_status: SignalStatus,
    /// URL requested.
    pub str_source_url: String,
    /// URL of the final response after redirects.
    pub opt_str_final_url: Option<String>,
    /// Redirect responses followed.
    pub vec_o_redirects: Vec<RedirectHop>,
    /// Final HTTP status, when there was a response.
    pub opt_u_http_status: Option<u16>,
    /// Why the file was unreachable.
    pub opt_e_failure: Option<SignalFailure>,
    /// SHA-256 hex of the stored body, resolvable through the body archive.
    pub opt_str_sha256: Option<String>,
    /// Bytes of the archived body this file references; zero when nothing was archived.
    pub u_stored_bytes: usize,
    /// Whether the body was cut at the signal-file cap.
    pub b_body_truncated: bool,
    /// Decoded body bytes received before reading stopped.
    pub u_body_bytes_seen: u64,
    /// Excerpt of the body.
    pub opt_o_excerpt: Option<Excerpt>,
    /// Time of the fetch.
    pub dt_observed_at: Timestamp,
    /// Parsed content for present files.
    pub e_document: SignalDocument,
}

impl FetchedFile {
    /// Returns whether the fetch failed (unreachable), which is cached only for the backoff period.
    pub fn is_failure(&self) -> bool {
        self.e_status == SignalStatus::Unreachable
    }

    /// Returns the bytes this cached file accounts for: its archived body, its parsed document,
    /// and its strings, estimated from lengths (allocator slack is not counted).
    pub fn weight(&self) -> usize {
        let u_redirects: usize = self
            .vec_o_redirects
            .iter()
            .map(|o_hop| size_of::<RedirectHop>() + o_hop.str_url.len())
            .sum();
        let u_urls =
            self.str_source_url.len() + self.opt_str_final_url.as_ref().map_or(0, String::len);
        size_of::<Self>()
            + self.u_stored_bytes
            + self.e_document.heap_bytes()
            + U_MAX_EXCERPT_BYTES
            + u_redirects
            + u_urls
    }

    /// Returns the observation of this file for one URL.
    pub fn observe(&self, o_context: &SignalContext) -> SignalObservation {
        SignalObservation {
            e_kind: self.e_file.kind(),
            e_status: self.e_status,
            str_source_url: self.str_source_url.clone(),
            dt_observed_at: self.dt_observed_at.clone(),
            opt_u_http_status: self.opt_u_http_status,
            opt_str_content_sha256: self.opt_str_sha256.clone(),
            opt_o_excerpt: self.opt_o_excerpt.clone(),
            opt_str_final_url: self.opt_str_final_url.clone(),
            vec_o_redirects: self.vec_o_redirects.clone(),
            b_body_truncated: self.b_body_truncated,
            u_body_bytes_seen: self.u_body_bytes_seen,
            opt_e_failure: self.opt_e_failure,
            opt_e_parsed: self.e_document.evaluate(o_context),
        }
    }
}
