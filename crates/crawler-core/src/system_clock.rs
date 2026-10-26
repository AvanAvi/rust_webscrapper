//! Reads the real system time.

use time::OffsetDateTime;

use crate::clock::Clock;

/// Clock backed by the operating system.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl SystemClock {
    /// Creates a system clock.
    pub fn new() -> Self {
        Self
    }
}

impl Clock for SystemClock {
    /// Returns the current system time in UTC.
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}
