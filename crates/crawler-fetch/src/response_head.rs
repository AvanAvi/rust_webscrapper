//! Captures the status line and headers of a response before its body is read.

use std::sync::Arc;
use std::time::Duration;

use crawler_core::Timestamp;
use time::OffsetDateTime;

use crate::body_reader::ReadBody;
use crate::fetch_log::FetchLog;
use crate::fetch_response::FetchResponse;
use crate::retry_after_parser::RetryAfterParser;

/// Status, final URL, headers, encoding, and Retry-After of a response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseHead {
    /// Final status.
    pub u_status: u16,
    /// Final URL.
    pub str_final_url: String,
    /// Headers, names lowercased; non-UTF-8 values are decoded lossily.
    pub vec_o_headers: Vec<(String, String)>,
    /// Content-Encoding, if any.
    pub opt_str_encoding: Option<String>,
    /// Retry-After of a 429 or 503 response, capped.
    pub opt_dur_retry_after: Option<Duration>,
}

impl ResponseHead {
    /// Captures the head of a reqwest response; `dt_now` resolves date-form Retry-After values.
    pub fn from_response(o_response: &reqwest::Response, dt_now: OffsetDateTime) -> Self {
        let vec_o_headers = Self::headers_of(o_response);
        let cb_find = |str_name: &str| {
            vec_o_headers
                .iter()
                .find(|o_h| o_h.0 == str_name)
                .map(|o_h| o_h.1.clone())
        };
        let opt_str_encoding = cb_find("content-encoding");
        let u_status = o_response.status().as_u16();
        let opt_dur_retry_after =
            RetryAfterParser::new().parse(u_status, cb_find("retry-after").as_deref(), dt_now);
        Self {
            u_status,
            str_final_url: o_response.url().to_string(),
            vec_o_headers,
            opt_str_encoding,
            opt_dur_retry_after,
        }
    }

    /// Returns the headers with lowercase names and lossily decoded values.
    fn headers_of(o_response: &reqwest::Response) -> Vec<(String, String)> {
        o_response
            .headers()
            .iter()
            .map(|(o_name, o_value)| {
                (
                    o_name.as_str().to_owned(),
                    String::from_utf8_lossy(o_value.as_bytes()).into_owned(),
                )
            })
            .collect()
    }

    /// Combines the head with the decoded body and the fetch log into the final response.
    pub fn into_response(
        self,
        o_body: ReadBody,
        u_encoded_bytes: u64,
        o_log: &Arc<FetchLog>,
        o_times: (Timestamp, Timestamp),
    ) -> FetchResponse {
        FetchResponse {
            u_status: self.u_status,
            str_final_url: self.str_final_url,
            vec_o_headers: self.vec_o_headers,
            by_body: o_body.by_body,
            u_encoded_bytes,
            vec_o_redirects: o_log.redirects(),
            vec_e_resolved_ips: o_log.ips(),
            b_truncated: o_body.b_truncated,
            opt_dur_retry_after: self.opt_dur_retry_after,
            dt_started_at: o_times.0,
            dt_finished_at: o_times.1,
        }
    }
}
