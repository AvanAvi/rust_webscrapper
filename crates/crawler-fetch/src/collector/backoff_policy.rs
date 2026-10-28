//! Computes how long a failed signal fetch is cached before it is retried.

use std::time::Duration;

/// First retry delay.
pub const DUR_BACKOFF_BASE: Duration = Duration::from_secs(60);
/// Growth factor per consecutive failure.
pub const U_BACKOFF_FACTOR: u32 = 2;
/// Longest delay; equals the default signal TTL so a failure is never cached longer than a success.
pub const DUR_BACKOFF_CAP: Duration = Duration::from_secs(3600);
/// Jitter in percent applied around the delay (±).
pub const U_BACKOFF_JITTER_PERCENT: u32 = 20;

/// Exponential backoff with symmetric jitter: delay(n) = min(base * factor^(n-1), cap) * (1 ± jitter),
/// capped again after jitter. Randomness comes from the injected `IdSource`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackoffPolicy {
    /// First retry delay.
    pub dur_base: Duration,
    /// Growth factor.
    pub u_factor: u32,
    /// Longest delay.
    pub dur_cap: Duration,
    /// Jitter in percent.
    pub u_jitter_percent: u32,
}

impl Default for BackoffPolicy {
    /// Returns base 60 s, factor 2, cap 1 h, jitter ±20 %.
    fn default() -> Self {
        Self {
            dur_base: DUR_BACKOFF_BASE,
            u_factor: U_BACKOFF_FACTOR,
            dur_cap: DUR_BACKOFF_CAP,
            u_jitter_percent: U_BACKOFF_JITTER_PERCENT,
        }
    }
}

impl BackoffPolicy {
    /// Returns the delay after `u_failures` consecutive failures (1 for the first), jittered by `u_random`.
    pub fn delay(&self, u_failures: u32, u_random: u128) -> Duration {
        let u_exponent = u_failures.saturating_sub(1).min(31);
        let dur_raw = self
            .dur_base
            .saturating_mul(self.u_factor.saturating_pow(u_exponent))
            .min(self.dur_cap);
        let u_jitter = self.u_jitter_percent.min(100);
        let u_pick = u32::try_from(u_random % u128::from(2 * u_jitter + 1)).unwrap_or(0);
        let u_percent = 100 - u_jitter + u_pick;
        (dur_raw.saturating_mul(u_percent) / 100).min(self.dur_cap)
    }
}
