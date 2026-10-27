//! Reads response bodies within the size cap and decodes them with a ratio check.

use axum::http::header::CONTENT_ENCODING;
use axum::http::{Request, Response};
use tower::{ServiceExt, service_fn};
use tower_http::decompression::Decompression;

use crate::fetch_error::FetchError;
use crate::fetch_limits::FetchLimits;

/// Content encodings the client decodes; anything else is refused rather than stored opaque.
const VEC_STR_SUPPORTED_ENCODINGS: [&str; 3] = ["gzip", "deflate", "br"];

/// Body reading: the encoded body is capped while it arrives (Content-Length is checked first
/// but never trusted), then decoded with the cap and a ratio limit applied after decompression.
#[derive(Debug, Clone, Copy)]
pub struct BodyReader {
    /// Size and ratio limits.
    o_limits: FetchLimits,
}

impl BodyReader {
    /// Creates a reader with limits.
    pub fn new(o_limits: FetchLimits) -> Self {
        Self { o_limits }
    }

    /// Reads the encoded body, refusing it as soon as it exceeds the cap.
    pub async fn read_encoded(
        &self,
        o_response: &mut reqwest::Response,
    ) -> Result<Vec<u8>, FetchError> {
        let u_cap = self.o_limits.u_max_body_bytes;
        if o_response
            .content_length()
            .is_some_and(|u_len| u_len > u_cap as u64)
        {
            return Err(FetchError::BodyTooLarge { u_limit: u_cap });
        }
        let mut by_body = Vec::new();
        while let Some(by_chunk) = o_response
            .chunk()
            .await
            .map_err(|err_body| self.body_error(&err_body))?
        {
            if by_body.len() + by_chunk.len() > u_cap {
                return Err(FetchError::BodyTooLarge { u_limit: u_cap });
            }
            by_body.extend_from_slice(&by_chunk);
        }
        Ok(by_body)
    }

    /// Returns the decoded body; identity or absent encoding returns the bytes unchanged.
    pub async fn decode(
        &self,
        opt_str_encoding: Option<&str>,
        by_encoded: Vec<u8>,
    ) -> Result<Vec<u8>, FetchError> {
        let str_encoding = opt_str_encoding
            .map(|str_e| str_e.trim().to_ascii_lowercase())
            .unwrap_or_default();
        if str_encoding.is_empty() || str_encoding == "identity" {
            return Ok(by_encoded);
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
        let u_encoded = by_encoded.len();
        let o_decoded = self.decoder(&str_canonical, by_encoded).await?;
        self.read_decoded(o_decoded, u_encoded).await
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

    /// Reads decoded chunks, enforcing the cap and the ratio after every chunk.
    async fn read_decoded(
        &self,
        mut o_decoded: reqwest::Response,
        u_encoded: usize,
    ) -> Result<Vec<u8>, FetchError> {
        let mut by_body = Vec::new();
        while let Some(by_chunk) = o_decoded
            .chunk()
            .await
            .map_err(|err_body| self.decode_error(&err_body))?
        {
            by_body.extend_from_slice(&by_chunk);
            self.check_decoded(by_body.len(), u_encoded)?;
        }
        Ok(by_body)
    }

    /// Refuses decoded output over the cap or growing faster than the allowed ratio.
    fn check_decoded(&self, u_decoded: usize, u_encoded: usize) -> Result<(), FetchError> {
        if u_decoded > self.o_limits.u_max_body_bytes {
            return Err(FetchError::BodyTooLarge {
                u_limit: self.o_limits.u_max_body_bytes,
            });
        }
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

    /// Maps an error while decoding.
    fn decode_error(&self, err_body: &reqwest::Error) -> FetchError {
        FetchError::Decode {
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
