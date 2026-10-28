//! Defines the single error type of the core crate.

/// Errors raised by URL normalization and canonical encoding.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// The text is not a valid absolute URL.
    #[error("invalid URL: {str_reason}")]
    InvalidUrl {
        /// Parser explanation; never contains the URL itself.
        str_reason: String,
    },
    /// The URL uses a scheme other than http or https.
    #[error("unsupported URL scheme `{str_scheme}`: only http and https are allowed")]
    UnsupportedScheme {
        /// Scheme that was rejected.
        str_scheme: String,
    },
    /// The URL carries a user name or password.
    #[error("URL contains credentials, which are never crawled or recorded: {str_redacted_url}")]
    CredentialsInUrl {
        /// The URL with user info, query, and fragment removed; the original is never kept.
        str_redacted_url: String,
    },
    /// The archive has no room and every stored body is still referenced.
    #[error("body archive is full ({u_capacity} bytes) and every body is referenced")]
    ArchiveFull {
        /// Capacity in bytes.
        u_capacity: usize,
    },
    /// A body is larger than the archive accepts.
    #[error("body exceeds the archive limit of {u_limit} bytes")]
    ArchiveBodyTooLarge {
        /// Size limit in bytes.
        u_limit: usize,
    },
    /// No body is stored under a hash.
    #[error("no archived body with SHA-256 {str_sha256}")]
    UnknownBody {
        /// The hash looked up.
        str_sha256: String,
    },
    /// A crawler identity is malformed or impersonates another crawler.
    #[error("invalid crawler identity: {str_reason}")]
    InvalidIdentity {
        /// What was wrong.
        str_reason: String,
    },
    /// Decoded excerpt fields are inconsistent with each other.
    #[error("invalid excerpt: {str_reason}")]
    InvalidExcerpt {
        /// What was inconsistent.
        str_reason: String,
    },
    /// A time value is not a UTC instant in the one accepted format.
    #[error("invalid timestamp: {str_reason}")]
    InvalidTimestamp {
        /// What was wrong with the value.
        str_reason: String,
    },
    /// A number cannot be represented exactly in canonical JSON.
    #[error(
        "number {str_number} is not canonical: only integers within ±(2^53-1) are allowed (RFC 7493 section 2.2)"
    )]
    NonCanonicalNumber {
        /// Textual form of the rejected number.
        str_number: String,
    },
    /// A value could not be converted to JSON.
    #[error("serialization failed: {err_source}")]
    Serialization {
        /// Underlying serde_json error.
        #[from]
        err_source: serde_json::Error,
    },
}
