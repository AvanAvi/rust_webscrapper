//! Per-host cache of signal files with single-flight fetching.

use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crawler_core::BodyArchive;
use time::OffsetDateTime;
use tokio::sync::Notify;
use tokio::sync::futures::OwnedNotified;

use crate::collector::fetched_file::FetchedFile;
use crate::collector::host_table::HostTable;
use crate::collector::signal_file::SignalFile;

/// What a caller must do to get a signal file.
pub enum Acquire {
    /// A fresh cached result.
    Hit(Arc<FetchedFile>),
    /// Another task is fetching; wait for it, then ask again.
    Wait(Pin<Box<OwnedNotified>>),
    /// This caller fetches and must report through a `LeaderGuard`.
    Lead(LeaderTicket),
    /// The cache is full of busy hosts; fetch without caching.
    Bypass,
}

/// Permission to fetch one file for one host, held by the single in-flight leader.
#[derive(Debug, Clone)]
pub struct LeaderTicket {
    /// Cache key of the host (its origin URL).
    pub str_origin: String,
    /// File being fetched.
    pub e_file: SignalFile,
    /// Wakes the waiters when the leader finishes or gives up.
    pub o_notify: Arc<Notify>,
    /// Consecutive failures before this attempt.
    pub u_failures: u32,
}

/// Cache keyed by origin, bounded in hosts and in bytes (see `HostTable`). The cache owns one
/// archive reference per cached body and releases it when the file is replaced or dropped; if
/// every host has a fetch in flight, the caller fetches without caching.
pub struct HostCache {
    /// Hosts and bounds.
    mtx_table: Mutex<HostTable>,
    /// Archive whose references the cache holds for cached bodies.
    o_archive: Arc<dyn BodyArchive>,
}

impl HostCache {
    /// Creates an empty cache with a host bound and a byte bound.
    pub fn new(u_max_hosts: usize, u_max_bytes: usize, o_archive: Arc<dyn BodyArchive>) -> Self {
        Self {
            mtx_table: Mutex::new(HostTable::new(u_max_hosts, u_max_bytes)),
            o_archive,
        }
    }

    /// Returns the summed weight of the cached files.
    pub fn cached_bytes(&self) -> usize {
        self.table().cached_bytes()
    }

    /// Returns a hit, a wait, the lead, or a bypass for one file of one host.
    pub fn acquire(&self, str_origin: &str, e_file: SignalFile, dt_now: OffsetDateTime) -> Acquire {
        let (e_acquire, vec_o_freed) = self.table().acquire(str_origin, e_file, dt_now);
        self.release_all(&vec_o_freed);
        e_acquire
    }

    /// Stores the leader's result, wakes the waiters, and releases dropped bodies.
    pub fn complete(
        &self,
        o_ticket: &LeaderTicket,
        o_file: Arc<FetchedFile>,
        o_expiry: (OffsetDateTime, u32),
    ) {
        let vec_o_freed = self.table().store(o_ticket, o_file, o_expiry);
        o_ticket.o_notify.notify_waiters();
        self.release_all(&vec_o_freed);
    }

    /// Clears a slot whose leader was cancelled, so a waiter can take over.
    pub fn abandon(&self, o_ticket: &LeaderTicket) {
        self.table().abandon(o_ticket);
        o_ticket.o_notify.notify_waiters();
    }

    /// Releases the archive reference a fetched file holds (dropped or never cached).
    pub fn release_file(&self, o_file: &FetchedFile) {
        if let Some(str_sha256) = &o_file.opt_str_sha256 {
            self.o_archive.release(str_sha256);
        }
    }

    /// Locks the table, recovering it if a holder panicked.
    fn table(&self) -> MutexGuard<'_, HostTable> {
        self.mtx_table
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Releases the cache's archive references of dropped files.
    fn release_all(&self, vec_o_files: &[Arc<FetchedFile>]) {
        for o_file in vec_o_files {
            self.release_file(o_file);
        }
    }
}
