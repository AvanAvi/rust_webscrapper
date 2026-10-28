//! Keeps archived bodies in memory (the Postgres archive arrives in F6).

use std::sync::{Mutex, PoisonError};

use crate::archive_state::ArchiveState;
use crate::body_archive::BodyArchive;
use crate::core_error::CoreError;
use crate::hasher::Hasher;

/// Largest body accepted, in bytes; equals the signal-file fetch cap.
pub const U_MAX_ARCHIVED_BODY_BYTES: usize = 1024 * 1024;
/// Default total capacity, in bytes.
pub const U_DEFAULT_ARCHIVE_CAPACITY: usize = 256 * 1024 * 1024;

/// In-memory, deduplicated, reference-counted archive. When full it evicts the oldest body
/// with no references; if every body is referenced, `store` fails with `ArchiveFull` instead.
#[derive(Debug)]
pub struct MemoryBodyArchive {
    /// Bodies and bookkeeping.
    mtx_state: Mutex<ArchiveState>,
    /// Total capacity in bytes.
    u_capacity: usize,
    /// Hasher for content addresses.
    o_hasher: Hasher,
}

impl MemoryBodyArchive {
    /// Creates an archive with the default capacity.
    pub fn new() -> Self {
        Self::with_capacity(U_DEFAULT_ARCHIVE_CAPACITY)
    }

    /// Creates an archive holding at most `u_capacity` bytes.
    pub fn with_capacity(u_capacity: usize) -> Self {
        Self {
            mtx_state: Mutex::new(ArchiveState::default()),
            u_capacity,
            o_hasher: Hasher::new(),
        }
    }
}

impl Default for MemoryBodyArchive {
    /// Creates an archive with the default capacity.
    fn default() -> Self {
        Self::new()
    }
}

impl BodyArchive for MemoryBodyArchive {
    /// Stores a body once per hash and takes a reference to it.
    fn store(&self, by_body: &[u8]) -> Result<String, CoreError> {
        if by_body.len() > U_MAX_ARCHIVED_BODY_BYTES {
            return Err(CoreError::ArchiveBodyTooLarge {
                u_limit: U_MAX_ARCHIVED_BODY_BYTES,
            });
        }
        let str_sha256 = self.o_hasher.sha256_hex(by_body);
        let mut o_state = self
            .mtx_state
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if !o_state.add_ref(&str_sha256) {
            o_state.insert(str_sha256.clone(), by_body, self.u_capacity)?;
        }
        Ok(str_sha256)
    }

    /// Returns a stored body.
    fn load(&self, str_sha256: &str) -> Option<Vec<u8>> {
        self.mtx_state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .body(str_sha256)
    }

    /// Adds a reference; fails for an unknown hash.
    fn retain(&self, str_sha256: &str) -> Result<(), CoreError> {
        let b_known = self
            .mtx_state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .add_ref(str_sha256);
        if b_known {
            Ok(())
        } else {
            Err(CoreError::UnknownBody {
                str_sha256: str_sha256.to_owned(),
            })
        }
    }

    /// Drops one reference.
    fn release(&self, str_sha256: &str) {
        self.mtx_state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .drop_ref(str_sha256);
    }
}
