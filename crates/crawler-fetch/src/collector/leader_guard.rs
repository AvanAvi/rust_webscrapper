//! Ensures a cancelled single-flight leader never leaves waiters hanging.

use std::sync::Arc;

use time::OffsetDateTime;

use crate::collector::fetched_file::FetchedFile;
use crate::collector::host_cache::{HostCache, LeaderTicket};

/// Held by the task fetching a signal file. Finishing stores the result; dropping it unfinished
/// (task cancelled, panicked, or failed internally) clears the slot and wakes the waiters, one of
/// which then takes over.
pub struct LeaderGuard {
    /// Cache to report to.
    o_cache: Arc<HostCache>,
    /// Ticket, taken when the result is stored.
    opt_o_ticket: Option<LeaderTicket>,
}

impl LeaderGuard {
    /// Creates a guard for a ticket.
    pub fn new(o_cache: Arc<HostCache>, o_ticket: LeaderTicket) -> Self {
        Self {
            o_cache,
            opt_o_ticket: Some(o_ticket),
        }
    }

    /// Stores the result with its expiry and new failure streak, and wakes the waiters.
    pub fn finish(mut self, o_file: Arc<FetchedFile>, o_expiry: (OffsetDateTime, u32)) {
        if let Some(o_ticket) = self.opt_o_ticket.take() {
            self.o_cache.complete(&o_ticket, o_file, o_expiry);
        }
    }
}

impl Drop for LeaderGuard {
    /// Abandons the slot if the result was never stored.
    fn drop(&mut self) {
        if let Some(o_ticket) = self.opt_o_ticket.take() {
            self.o_cache.abandon(&o_ticket);
        }
    }
}
