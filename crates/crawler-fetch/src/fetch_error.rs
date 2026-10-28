//! Defines the single error type of the fetch crate.

/// Why a fetch was refused or failed. Text never contains credentials or query strings.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FetchError {
    /// The URL could not be parsed or has no host.
    #[error("invalid URL: {str_reason}")]
    InvalidUrl {
        /// Parser explanation.
        str_reason: String,
    },
    /// The URL scheme is not http or https.
    #[error("unsupported URL scheme `{str_scheme}`")]
    UnsupportedScheme {
        /// Rejected scheme.
        str_scheme: String,
    },
    /// The URL carries a user name or password.
    #[error("URL contains credentials")]
    CredentialsInUrl,
    /// The target address is in a forbidden range.
    #[error("address {str_ip} of host `{str_host}` is blocked ({str_range})")]
    BlockedAddress {
        /// Host that resolved to, or was, the address.
        str_host: String,
        /// The blocked address.
        str_ip: String,
        /// Name of the forbidden range.
        str_range: String,
    },
    /// Name resolution failed.
    #[error("could not resolve `{str_host}`: {str_reason}")]
    DnsFailure {
        /// Host that failed.
        str_host: String,
        /// Resolver explanation.
        str_reason: String,
    },
    /// More redirects than allowed.
    #[error("more than {u_max} redirects")]
    TooManyRedirects {
        /// Redirect limit.
        u_max: usize,
    },
    /// A redirect from https to http.
    #[error("redirect from https to http refused: {str_to}")]
    RedirectDowngrade {
        /// Redacted redirect target.
        str_to: String,
    },
    /// A redirect left the allowed host scope.
    #[error("redirect outside the allowed host scope refused: {str_to}")]
    RedirectOutOfScope {
        /// Redacted redirect target.
        str_to: String,
    },
    /// The body exceeds the size cap (counted after decompression).
    #[error("response body exceeds {u_limit} bytes")]
    BodyTooLarge {
        /// Byte cap.
        u_limit: usize,
    },
    /// Decompressed output grew too fast relative to the compressed input.
    #[error("compression ratio exceeds {u_limit}:1")]
    CompressionRatioExceeded {
        /// Maximum ratio.
        u_limit: usize,
    },
    /// The Content-Encoding is not one the client decodes.
    #[error("unsupported content encoding `{str_encoding}`")]
    UnsupportedEncoding {
        /// Encoding as sent.
        str_encoding: String,
    },
    /// The compressed body could not be decoded.
    #[error("could not decode body: {str_reason}")]
    Decode {
        /// Decoder explanation.
        str_reason: String,
    },
    /// A connect, read, or total deadline passed.
    #[error("timed out")]
    Timeout,
    /// The connection could not be established.
    #[error("connection failed: {str_reason}")]
    Connect {
        /// Transport explanation without the URL.
        str_reason: String,
    },
    /// Any other transport failure.
    #[error("transport error: {str_reason}")]
    Transport {
        /// Transport explanation without the URL.
        str_reason: String,
    },
    /// A test-only address policy was requested in a build without the `test-policy` feature.
    #[error("the loopback test policy is not compiled into this build")]
    TestPolicyUnavailable,
    /// The crawler identity still uses a placeholder contact URL.
    #[error("the contact URL `{str_host}` is a placeholder; set a real contact before crawling")]
    PlaceholderContact {
        /// Placeholder host.
        str_host: String,
    },
    /// The HTTP client could not be built or an internal step failed.
    #[error("internal fetch error: {str_reason}")]
    Internal {
        /// Explanation.
        str_reason: String,
    },
}

impl FetchError {
    /// Returns the receipt's `fetch_failure` for an allowed page fetch that ended in this error.
    /// The match is exhaustive, so a new error variant must be mapped here.
    pub fn fetch_failure(&self) -> crawler_core::FetchFailure {
        use crawler_core::FetchFailure as Failure;
        match self {
            Self::Timeout => Failure::Timeout,
            Self::BlockedAddress { .. } => Failure::BlockedAddress,
            Self::DnsFailure { .. } => Failure::DnsFailure,
            Self::Connect { .. } | Self::Transport { .. } => Failure::ConnectionFailed,
            Self::TooManyRedirects { .. }
            | Self::RedirectDowngrade { .. }
            | Self::RedirectOutOfScope { .. } => Failure::RedirectRefused,
            Self::BodyTooLarge { .. } => Failure::BodyTooLarge,
            Self::CompressionRatioExceeded { .. }
            | Self::UnsupportedEncoding { .. }
            | Self::Decode { .. } => Failure::BadBody,
            Self::InvalidUrl { .. } | Self::UnsupportedScheme { .. } | Self::CredentialsInUrl => {
                Failure::InvalidRequest
            }
            Self::TestPolicyUnavailable
            | Self::PlaceholderContact { .. }
            | Self::Internal { .. } => Failure::Internal,
        }
    }
}
