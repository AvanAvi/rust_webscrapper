//! Lists why a signal source could not be retrieved.

use serde::{Deserialize, Serialize};

/// Typed reason a signal fetch produced no usable response (status `unreachable`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalFailure {
    /// The server answered with a 5xx status.
    ServerError,
    /// The server answered 429 Too Many Requests.
    RateLimited,
    /// A connect, read, or total deadline passed.
    Timeout,
    /// The host or a redirect target resolved to a blocked address.
    BlockedAddress,
    /// The name could not be resolved.
    DnsFailure,
    /// The connection could not be established or broke.
    ConnectionFailed,
    /// More redirects than allowed, or a refused redirect.
    RedirectRefused,
    /// The body could not be decoded or exceeded a decoding limit.
    BadBody,
    /// The fetched body could not be stored in the archive, so the observation is not evidence;
    /// the policy fails closed and denies.
    EvidenceNotArchived,
}

impl SignalFailure {
    /// Returns a short plain-language description.
    pub fn label(&self) -> &'static str {
        match self {
            Self::ServerError => "server error",
            Self::RateLimited => "rate limited",
            Self::Timeout => "timed out",
            Self::BlockedAddress => "address refused by the address policy",
            Self::DnsFailure => "name could not be resolved",
            Self::ConnectionFailed => "connection failed",
            Self::RedirectRefused => "redirect refused",
            Self::BadBody => "body could not be read safely",
            Self::EvidenceNotArchived => "body could not be archived as evidence",
        }
    }
}
