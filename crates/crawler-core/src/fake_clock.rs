//! Provides a manually controlled clock for tests.

use std::sync::{Mutex, PoisonError};

use time::{Duration, OffsetDateTime};

use crate::clock::Clock;

/// Clock whose time only changes when told to.
#[derive(Debug)]
pub struct FakeClock {
    /// Current fake time.
    mtx_now: Mutex<OffsetDateTime>,
}

impl FakeClock {
    /// Creates a clock frozen at the given time.
    pub fn new(dt_start: OffsetDateTime) -> Self {
        Self {
            mtx_now: Mutex::new(dt_start),
        }
    }

    /// Sets the current time.
    pub fn set(&self, dt_now: OffsetDateTime) {
        *self.mtx_now.lock().unwrap_or_else(PoisonError::into_inner) = dt_now;
    }

    /// Moves the current time forward (or back, for a negative duration).
    pub fn advance(&self, dur_step: Duration) {
        let mut o_guard = self.mtx_now.lock().unwrap_or_else(PoisonError::into_inner);
        *o_guard = o_guard.saturating_add(dur_step);
    }
}

impl Clock for FakeClock {
    /// Returns the current fake time.
    fn now(&self) -> OffsetDateTime {
        *self.mtx_now.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
