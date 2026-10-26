//! Parses Crawl-delay values into whole milliseconds without floating point.

/// Largest accepted delay; longer values are capped to one day.
pub const U_MAX_CRAWL_DELAY_MS: u64 = 86_400_000;

/// Reads `N` or `N.fff` seconds as milliseconds; anything else is invalid.
///
/// Crawl-delay is not part of RFC 9309 (section 2.2.4 lets crawlers interpret other records).
/// Integer milliseconds keep receipts free of floats; digits beyond the third decimal are dropped.
#[derive(Debug, Clone, Copy, Default)]
pub struct CrawlDelayParser;

impl CrawlDelayParser {
    /// Creates a parser.
    pub fn new() -> Self {
        Self
    }

    /// Returns the delay in milliseconds, or `None` when the value is not a plain decimal.
    pub fn parse(&self, str_value: &str) -> Option<u64> {
        let (str_whole, str_fraction) = str_value
            .trim()
            .split_once('.')
            .unwrap_or((str_value.trim(), ""));
        if str_whole.is_empty() || !self.is_digits(str_whole) || !self.is_digits(str_fraction) {
            return None;
        }
        let u_whole_ms = self.digits_value(str_whole).saturating_mul(1000);
        let str_millis: String = str_fraction.chars().chain("000".chars()).take(3).collect();
        Some(
            u_whole_ms
                .saturating_add(self.digits_value(&str_millis))
                .min(U_MAX_CRAWL_DELAY_MS),
        )
    }

    /// Returns whether the text is only ASCII digits (empty counts as digits).
    fn is_digits(&self, str_text: &str) -> bool {
        str_text.bytes().all(|u_byte| u_byte.is_ascii_digit())
    }

    /// Returns the value of an all-digit string, saturating instead of overflowing.
    fn digits_value(&self, str_digits: &str) -> u64 {
        str_digits.bytes().fold(0u64, |u_acc, u_byte| {
            u_acc
                .saturating_mul(10)
                .saturating_add(u64::from(u_byte - b'0'))
        })
    }
}
