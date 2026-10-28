//! Holds the result of one completed fetch.

use std::net::IpAddr;
use std::time::Duration;

use crawler_core::{FetchRecord, Hasher, RedirectHop, Timestamp};

/// Final response with decoded body and everything a receipt needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchResponse {
    /// Final HTTP status.
    pub u_status: u16,
    /// URL of the final response.
    pub str_final_url: String,
    /// Response headers, names lowercased, in order.
    pub vec_o_headers: Vec<(String, String)>,
    /// Decoded body, never larger than the cap.
    pub by_body: Vec<u8>,
    /// Bytes received on the wire before decoding.
    pub u_encoded_bytes: u64,
    /// Redirect responses followed, in order.
    pub vec_o_redirects: Vec<RedirectHop>,
    /// Vetted addresses the connector was given.
    pub vec_e_resolved_ips: Vec<IpAddr>,
    /// Whether the body was cut at the request's cap (truncate mode only).
    pub b_truncated: bool,
    /// Decoded body bytes received before reading stopped; above the body length only when
    /// truncated.
    pub u_body_bytes_seen: u64,
    /// Retry-After of a 429 or 503 response, capped at `DUR_MAX_RETRY_AFTER`.
    pub opt_dur_retry_after: Option<Duration>,
    /// Time the request started.
    pub dt_started_at: Timestamp,
    /// Time the body was complete.
    pub dt_finished_at: Timestamp,
}

impl FetchResponse {
    /// Returns the first value of a header (name in lowercase).
    pub fn header(&self, str_name: &str) -> Option<&str> {
        self.vec_o_headers
            .iter()
            .find(|o_h| o_h.0 == str_name)
            .map(|o_h| o_h.1.as_str())
    }

    /// Returns the headers as a `name: value` block for the signal parsers.
    pub fn header_block(&self) -> String {
        self.vec_o_headers
            .iter()
            .map(|(str_n, str_v)| format!("{str_n}: {str_v}\n"))
            .collect()
    }

    /// Returns the receipt form of this fetch.
    pub fn to_record(&self, o_hasher: &Hasher) -> FetchRecord {
        FetchRecord {
            dt_started_at: self.dt_started_at.clone(),
            dt_finished_at: self.dt_finished_at.clone(),
            u_http_status: self.u_status,
            opt_str_content_type: self.header("content-type").map(str::to_owned),
            u_bytes: self.by_body.len() as u64,
            str_body_sha256: o_hasher.sha256_hex(&self.by_body),
            vec_o_redirects: self.vec_o_redirects.clone(),
            vec_e_resolved_ips: self.vec_e_resolved_ips.clone(),
        }
    }
}
