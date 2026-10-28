//! Records what was fetched for an allowed URL.

use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::redirect_hop::RedirectHop;
use crate::timestamp::Timestamp;

/// Facts about one page fetch; absent from denied receipts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FetchRecord {
    /// Time the request started.
    #[serde(rename = "started_at")]
    pub dt_started_at: Timestamp,
    /// Time the response body finished or the fetch was aborted.
    #[serde(rename = "finished_at")]
    pub dt_finished_at: Timestamp,
    /// Final HTTP status.
    #[serde(rename = "http_status")]
    pub u_http_status: u16,
    /// Content-Type of the final response, if sent.
    #[serde(rename = "content_type", deserialize_with = "Option::deserialize")]
    pub opt_str_content_type: Option<String>,
    /// Body size in bytes after decompression.
    #[serde(rename = "body_bytes")]
    pub u_body_bytes: u64,
    /// SHA-256 hex of the decompressed body.
    #[serde(rename = "body_sha256")]
    pub str_body_sha256: String,
    /// Redirects followed, in order.
    #[serde(rename = "redirects")]
    pub vec_o_redirects: Vec<RedirectHop>,
    /// Addresses the connections were made to, after SSRF vetting.
    #[serde(rename = "resolved_ips")]
    pub vec_e_resolved_ips: Vec<IpAddr>,
    /// Retry-After of a 429 or 503 response in milliseconds, capped at one hour.
    #[serde(rename = "retry_after_ms", deserialize_with = "Option::deserialize")]
    pub opt_u_retry_after_ms: Option<u64>,
}
