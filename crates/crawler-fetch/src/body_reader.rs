//! Reads response bodies within a byte budget and decodes them with a ratio check.

use http::header::CONTENT_ENCODING;
use http::{Request, Response};
use tower::{ServiceExt, service_fn};
use tower_http::decompression::Decompression;

use crate::body_budget::{BodyBudget, BodyOverflow};
use crate::fetch_error::FetchError;
use crate::fetch_limits::FetchLimits;

/// Content encodings the client decodes; anything else is refused rather than stored opaque.
const VEC_STR_SUPPORTED_ENCODINGS: [&str; 3] = ["gzip", "deflate", "br"];

/// Bytes read so far, how many were seen, and whether reading stopped at the cap.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReadBody {
    /// Bytes kept.
    pub by_body: Vec<u8>,
    /// Whether more bytes existed than the budget allowed.
    pub b_truncated: bool,
    /// Bytes received, including the part of the last chunk past the cap; reading stops after
    /// the first chunk that crosses the cap, so this is a lower bound of the full body size.
    pub u_bytes_seen: u64,
}

/// Body reading: the encoded body is capped while it arrives (Content-Length is checked first
/// in reject mode, never trusted), then decoded with the cap and a ratio limit applied after
/// decompression. In truncate mode the cap cuts the body instead of failing.
#[derive(Debug, Clone, Copy)]
pub struct BodyReader {
    /// Ratio limits.
    o_limits: FetchLimits,
}

impl BodyReader {
    /// Creates a reader with limits.
    pub fn new(o_limits: FetchLimits) -> Self {
        Self { o_limits }
    }

    /// Reads the encoded body within the budget.
    pub async fn read_encoded(
        &self,
        o_response: &mut reqwest::Response,
        o_budget: BodyBudget,
    ) -> Result<ReadBody, FetchError> {
        let b_reject = o_budget.e_overflow == BodyOverflow::Reject;
        if b_reject
            && o_response
                .content_length()
                .is_some_and(|u_len| u_len > o_budget.u_cap as u64)
        {
            return Err(FetchError::BodyTooLarge {
                u_limit: o_budget.u_cap,
            });
        }
        let mut o_read = ReadBody::default();
        while let Some(by_chunk) = o_response
            .chunk()
            .await
            .map_err(|err_body| self.body_error(&err_body))?
        {
            if self.append(&mut o_read, &by_chunk, o_budget)? {
                break;
            }
        }
        Ok(o_read)
    }

    /// Returns the decoded body; identity or absent encoding returns the bytes unchanged.
    pub async fn decode(
        &self,
        opt_str_encoding: Option<&str>,
        o_encoded: ReadBody,
        o_budget: BodyBudget,
    ) -> Result<ReadBody, FetchError> {
        let str_encoding = opt_str_encoding
            .map(|str_e| str_e.trim().to_ascii_lowercase())
            .unwrap_or_default();
        if str_encoding.is_empty() || str_encoding == "identity" {
            return Ok(o_encoded);
        }
        let str_canonical = if str_encoding == "x-gzip" {
            String::from("gzip")
        } else {
            str_encoding
        };
        if !VEC_STR_SUPPORTED_ENCODINGS.contains(&str_canonical.as_str()) {
            return Err(FetchError::UnsupportedEncoding {
                str_encoding: str_canonical,
            });
        }
        let (u_encoded, b_partial_input) = (o_encoded.by_body.len(), o_encoded.b_truncated);
        let o_decoded = self.decoder(&str_canonical, o_encoded.by_body).await?;
        self.read_decoded(o_decoded, u_encoded, o_budget, b_partial_input)
            .await
    }

    /// Appends a chunk within the budget; returns true when reading should stop (truncated).
    fn append(
        &self,
        o_read: &mut ReadBody,
        by_chunk: &[u8],
        o_budget: BodyBudget,
    ) -> Result<bool, FetchError> {
        o_read.u_bytes_seen = o_read.u_bytes_seen.saturating_add(by_chunk.len() as u64);
        if o_read.by_body.len() + by_chunk.len() <= o_budget.u_cap {
            o_read.by_body.extend_from_slice(by_chunk);
            return Ok(false);
        }
        if o_budget.e_overflow == BodyOverflow::Reject {
            return Err(FetchError::BodyTooLarge {
                u_limit: o_budget.u_cap,
            });
        }
        let u_room = o_budget.u_cap - o_read.by_body.len();
        o_read
            .by_body
            .extend_from_slice(by_chunk.get(..u_room).unwrap_or_default());
        o_read.b_truncated = true;
        Ok(true)
    }

    /// Wraps the encoded bytes in tower-http's decompression service and returns the decoded stream.
    async fn decoder(
        &self,
        str_encoding: &str,
        by_encoded: Vec<u8>,
    ) -> Result<reqwest::Response, FetchError> {
        let o_inner = Response::builder()
            .header(CONTENT_ENCODING, str_encoding)
            .body(reqwest::Body::from(by_encoded))
            .map_err(|err_http| FetchError::Internal {
                str_reason: err_http.to_string(),
            })?;
        let mut opt_o_slot = Some(o_inner);
        let o_service = Decompression::new(service_fn(move |_o_request: Request<()>| {
            let res_o_response = opt_o_slot.take().ok_or_else(|| FetchError::Internal {
                str_reason: String::from("decoder reused"),
            });
            async move { res_o_response }
        }));
        let o_decoded = o_service.oneshot(Request::new(())).await?;
        Ok(reqwest::Response::from(o_decoded.map(reqwest::Body::wrap)))
    }

    /// Reads decoded chunks within the budget, checking the ratio after every chunk. When the
    /// encoded input was itself cut, a decode error at its end just ends the (truncated) output.
    async fn read_decoded(
        &self,
        mut o_decoded: reqwest::Response,
        u_encoded: usize,
        o_budget: BodyBudget,
        b_partial_input: bool,
    ) -> Result<ReadBody, FetchError> {
        let mut o_read = ReadBody {
            by_body: Vec::new(),
            b_truncated: b_partial_input,
            u_bytes_seen: 0,
        };
        loop {
            let by_chunk = match o_decoded.chunk().await {
                Ok(Some(by_chunk)) => by_chunk,
                Ok(None) => return Ok(o_read),
                Err(err_body) => return self.end_of_partial(o_read, &err_body, b_partial_input),
            };
            self.check_ratio(o_read.by_body.len() + by_chunk.len(), u_encoded)?;
            if self.append(&mut o_read, &by_chunk, o_budget)? {
                return Ok(o_read);
            }
        }
    }

    /// Accepts a decode error caused by cut input; any other decode error fails the fetch.
    fn end_of_partial(
        &self,
        o_read: ReadBody,
        err_body: &reqwest::Error,
        b_partial_input: bool,
    ) -> Result<ReadBody, FetchError> {
        if b_partial_input {
            return Ok(o_read);
        }
        Err(FetchError::Decode {
            str_reason: self.describe(err_body),
        })
    }

    /// Refuses decoded output growing faster than the allowed ratio once past the floor.
    fn check_ratio(&self, u_decoded: usize, u_encoded: usize) -> Result<(), FetchError> {
        let b_past_floor = u_decoded > self.o_limits.u_ratio_floor_bytes;
        if b_past_floor && u_decoded > u_encoded.max(1).saturating_mul(self.o_limits.u_max_ratio) {
            return Err(FetchError::CompressionRatioExceeded {
                u_limit: self.o_limits.u_max_ratio,
            });
        }
        Ok(())
    }

    /// Maps an error while reading the encoded body.
    fn body_error(&self, err_body: &reqwest::Error) -> FetchError {
        if err_body.is_timeout() {
            return FetchError::Timeout;
        }
        FetchError::Transport {
            str_reason: self.describe(err_body),
        }
    }

    /// Describes an error with its source chain but without any URL.
    fn describe(&self, err_body: &reqwest::Error) -> String {
        let mut str_text = err_body.to_string();
        let mut opt_err_source = std::error::Error::source(err_body);
        while let Some(err_source) = opt_err_source {
            str_text.push_str(&format!(": {err_source}"));
            opt_err_source = err_source.source();
        }
        str_text
    }
}
