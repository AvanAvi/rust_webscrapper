//! Generates predictable IDs for tests.

use std::sync::atomic::{AtomicU64, Ordering};

use uuid::Uuid;

use crate::id_source::IdSource;

/// ID source that counts upward from a starting value.
#[derive(Debug)]
pub struct SequentialIdSource {
    /// Next value to hand out.
    o_next: AtomicU64,
}

impl SequentialIdSource {
    /// Creates a source whose first ID encodes `u_start`.
    pub fn new(u_start: u64) -> Self {
        Self {
            o_next: AtomicU64::new(u_start),
        }
    }
}

impl IdSource for SequentialIdSource {
    /// Returns the next counter value as a UUID.
    fn next_id(&self) -> Uuid {
        Uuid::from_u128(u128::from(self.o_next.fetch_add(1, Ordering::SeqCst)))
    }
}
