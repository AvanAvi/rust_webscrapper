//! Parses Retry-After on 429 and 503 responses into a capped duration.

use std::time::Duration;

use time::macros::format_description;
use time::{OffsetDateTime, PrimitiveDateTime};

/// Longest Retry-After honoured; longer values are capped (the crawl window should move on).
pub const DUR_MAX_RETRY_AFTER: Duration = Duration::from_secs(3600);

/// Reads `Retry-After` as delta-seconds or as an IMF-fixdate (RFC 9110 section 10.2.3), only for
/// status 429 and 503. Past dates give zero; values over the cap are capped.
#[derive(Debug, Clone, Copy, Default)]
pub struct RetryAfterParser;

impl RetryAfterParser {
    /// Creates a parser.
    pub fn new() -> Self {
        Self
    }

    /// Returns the wait, or `None` for other statuses and unparseable values.
    pub fn parse(
        &self,
        u_status: u16,
        opt_str_value: Option<&str>,
        dt_now: OffsetDateTime,
    ) -> Option<Duration> {
        if u_status != 429 && u_status != 503 {
            return None;
        }
        let str_value = opt_str_value?.trim();
        let opt_dur_wait = match str_value.bytes().all(|u_b| u_b.is_ascii_digit()) {
            true => str_value.parse::<u64>().ok().map(Duration::from_secs),
            false => self.until_date(str_value, dt_now),
        };
        opt_dur_wait.map(|dur_wait| dur_wait.min(DUR_MAX_RETRY_AFTER))
    }

    /// Returns the time from now until an IMF-fixdate, zero if it has passed.
    fn until_date(&self, str_value: &str, dt_now: OffsetDateTime) -> Option<Duration> {
        let o_format = format_description!(
            "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second] GMT"
        );
        let dt_then = PrimitiveDateTime::parse(str_value, o_format)
            .ok()?
            .assume_utc();
        Some(Duration::try_from(dt_then - dt_now).unwrap_or(Duration::ZERO))
    }
}
