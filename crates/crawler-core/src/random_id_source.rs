//! Generates random version-4 UUIDs.

use uuid::Uuid;

use crate::id_source::IdSource;

/// ID source backed by the operating system's random number generator.
#[derive(Debug, Clone, Copy, Default)]
pub struct RandomIdSource;

impl RandomIdSource {
    /// Creates a random ID source.
    pub fn new() -> Self {
        Self
    }
}

impl IdSource for RandomIdSource {
    /// Returns a random version-4 UUID.
    fn next_id(&self) -> Uuid {
        Uuid::new_v4()
    }
}
