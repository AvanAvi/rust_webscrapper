//! Declares the injected source of unique identifiers.

use uuid::Uuid;

/// Source of unique IDs; injected so tests are deterministic.
pub trait IdSource: Send + Sync {
    /// Returns a new identifier.
    fn next_id(&self) -> Uuid;
}
