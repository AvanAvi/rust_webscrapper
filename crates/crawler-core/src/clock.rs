//! Declares the injected source of the current time.

use time::OffsetDateTime;

use crate::core_error::CoreError;
use crate::timestamp::Timestamp;

/// Source of the current UTC time; injected so tests are deterministic.
pub trait Clock: Send + Sync {
    /// Returns the current time in UTC.
    fn now(&self) -> OffsetDateTime;

    /// Returns the current time as a receipt timestamp.
    fn timestamp(&self) -> Result<Timestamp, CoreError> {
        Timestamp::from_datetime(self.now())
    }
}
