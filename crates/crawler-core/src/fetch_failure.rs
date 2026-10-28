//! Lists why an allowed page fetch produced no response.

use serde::{Deserialize, Serialize};

/// Reason an allowed URL has no fetch record. A response of any status, 4xx and 5xx included,
/// is a fetch record instead; these are the cases where no usable response exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FetchFailure {
    /// A connect, read, or total time limit expired.
    Timeout,
    /// Every resolved address was refused by the address policy.
    BlockedAddress,
    /// The host name did not resolve.
    DnsFailure,
    /// The connection failed or broke.
    ConnectionFailed,
    /// A redirect was refused (too many hops, downgrade, or out of scope).
    RedirectRefused,
    /// The body exceeded the page cap.
    BodyTooLarge,
    /// The body could not be decoded, or its compression ratio was refused.
    BadBody,
    /// The URL could not be requested at all (invalid URL or scheme, credentials).
    InvalidRequest,
    /// The crawler failed internally; the site is not at fault.
    Internal,
}
