//! Declares content-addressed storage for signal-file bodies.

use crate::core_error::CoreError;

/// Stores bodies keyed by their SHA-256 so a receipt's `content_sha256` can be resolved and
/// re-parsed later. Bodies are reference-counted and never evicted while referenced.
pub trait BodyArchive: Send + Sync {
    /// Stores a body (deduplicated by hash), takes one reference to it, and returns its SHA-256 hex.
    fn store(&self, by_body: &[u8]) -> Result<String, CoreError>;

    /// Returns the body stored under a SHA-256 hex hash.
    fn load(&self, str_sha256: &str) -> Option<Vec<u8>>;

    /// Adds a reference to a stored body, for example from a receipt.
    fn retain(&self, str_sha256: &str) -> Result<(), CoreError>;

    /// Drops one reference; a body with no references may be evicted when space is needed.
    fn release(&self, str_sha256: &str);
}
