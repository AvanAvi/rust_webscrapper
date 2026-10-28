//! Keeps archived bodies in memory (the Postgres archive arrives in F6).

use std::sync::{Mutex, PoisonError};

use crate::archive_state::ArchiveState;
use crate::body_archive::BodyArchive;
use crate::core_error::CoreError;
use crate::hasher::Hasher;

/// Largest body accepted, in bytes. Equals the signal-file fetch cap: twice the 500 KiB that
/// RFC 9309 section 2.5 requires parsers to handle, so the archived bytes cover what was parsed.
pub const U_MAX_ARCHIVED_BODY_BYTES: usize = 1024 * 1024;
/// Default total capacity, in bytes: twice the collector's 128 MiB cache budget, so bodies held
/// by the cache never fill the archive and the other half absorbs bodies of in-flight fetches.
pub const U_DEFAULT_ARCHIVE_CAPACITY: usize = 256 * 1024 * 1024;
/// Default most bodies kept: above the 30,000 bodies a full 10,000-host cache can reference
/// (three files each), so tiny bodies cannot exhaust the map; bookkeeping stays near 13 MiB.
pub const U_DEFAULT_ARCHIVE_MAX_ENTRIES: usize = 65_536;

/// In-memory, deduplicated, reference-counted archive, bounded in bytes and in entries. When a
/// bound is reached it evicts the oldest body with no references; if every body is referenced,
/// `store` fails (`ArchiveFull` or `ArchiveTooManyEntries`) and the caller must fail closed.
#[derive(Debug)]
pub struct MemoryBodyArchive {
    /// Bodies and bookkeeping.
    mtx_state: Mutex<ArchiveState>,
    /// Total capacity in bytes.
    u_capacity: usize,
    /// Most bodies kept.
    u_max_entries: usize,
    /// Hasher for content addresses.
    o_hasher: Hasher,
}

impl MemoryBodyArchive {
    /// Creates an archive with the default limits.
    pub fn new() -> Self {
        Self::with_limits(U_DEFAULT_ARCHIVE_CAPACITY, U_DEFAULT_ARCHIVE_MAX_ENTRIES)
    }

    /// Creates an archive holding at most `u_capacity` bytes and the default entry count.
    pub fn with_capacity(u_capacity: usize) -> Self {
        Self::with_limits(u_capacity, U_DEFAULT_ARCHIVE_MAX_ENTRIES)
    }

    /// Creates an archive holding at most `u_capacity` bytes in at most `u_max_entries` bodies.
    pub fn with_limits(u_capacity: usize, u_max_entries: usize) -> Self {
        Self {
            mtx_state: Mutex::new(ArchiveState::default()),
            u_capacity,
            u_max_entries,
            o_hasher: Hasher::new(),
        }
    }

    /// Returns the stored bytes and the number of stored bodies.
    pub fn usage(&self) -> (usize, usize) {
        self.mtx_state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .usage()
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
            o_state.insert(
                str_sha256.clone(),
                by_body,
                self.u_capacity,
                self.u_max_entries,
            )?;
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
