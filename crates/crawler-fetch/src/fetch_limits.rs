//! Holds the time and size limits of a fetch.

use std::time::Duration;

/// Connect deadline.
pub const DUR_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
/// Deadline for each read; resets after every successful read.
pub const DUR_READ_TIMEOUT: Duration = Duration::from_secs(15);
/// Deadline for the whole request, from connecting until the body has finished.
pub const DUR_TOTAL_TIMEOUT: Duration = Duration::from_secs(30);
/// Body cap in bytes, counted after decompression (and also applied to the encoded body).
pub const U_MAX_BODY_BYTES: usize = 5 * 1024 * 1024;
/// Largest allowed ratio of decoded to encoded bytes once the ratio floor is passed.
pub const U_MAX_COMPRESSION_RATIO: usize = 20;
/// Decoded size below which the ratio is not checked (small bodies compress unevenly).
pub const U_RATIO_FLOOR_BYTES: usize = 256 * 1024;
/// Maximum redirects followed.
pub const U_MAX_REDIRECTS: usize = 5;

/// Limits applied to every fetch; tests may shorten the timeouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FetchLimits {
    /// Connect deadline.
    pub dur_connect: Duration,
    /// Per-read deadline.
    pub dur_read: Duration,
    /// Whole-request deadline.
    pub dur_total: Duration,
    /// Body cap after decompression.
    pub u_max_body_bytes: usize,
    /// Maximum decoded-to-encoded ratio.
    pub u_max_ratio: usize,
    /// Decoded size from which the ratio is enforced.
    pub u_ratio_floor_bytes: usize,
    /// Maximum redirects.
    pub u_max_redirects: usize,
}

impl Default for FetchLimits {
    /// Returns the production limits.
    fn default() -> Self {
        Self {
            dur_connect: DUR_CONNECT_TIMEOUT,
            dur_read: DUR_READ_TIMEOUT,
            dur_total: DUR_TOTAL_TIMEOUT,
            u_max_body_bytes: U_MAX_BODY_BYTES,
            u_max_ratio: U_MAX_COMPRESSION_RATIO,
            u_ratio_floor_bytes: U_RATIO_FLOOR_BYTES,
            u_max_redirects: U_MAX_REDIRECTS,
        }
    }
}
