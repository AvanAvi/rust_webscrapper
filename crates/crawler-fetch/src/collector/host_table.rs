//! Holds the cached hosts and enforces the host bound and the byte bound.

use std::collections::HashMap;
use std::sync::Arc;

use time::OffsetDateTime;
use tokio::sync::Notify;

use crate::collector::fetched_file::FetchedFile;
use crate::collector::host_cache::{Acquire, LeaderTicket};
use crate::collector::signal_file::SignalFile;

/// State of one file of one host.
#[derive(Debug, Default)]
enum SlotPhase {
    /// Never fetched, or abandoned by a cancelled leader.
    #[default]
    Empty,
    /// A leader is fetching.
    InFlight(Arc<Notify>),
    /// Cached result and its expiry.
    Ready(Arc<FetchedFile>, OffsetDateTime),
}

/// One file's cache slot with its failure streak.
#[derive(Debug, Default)]
struct Slot {
    /// Current phase.
    e_phase: SlotPhase,
    /// Consecutive failures, for the backoff.
    u_failures: u32,
}

impl Slot {
    /// Returns a hit, a wait, or the lead, plus the expired file the lead replaced.
    fn acquire(
        &mut self,
        str_origin: &str,
        e_file: SignalFile,
        dt_now: OffsetDateTime,
    ) -> (Acquire, Option<Arc<FetchedFile>>) {
        match &self.e_phase {
            SlotPhase::Ready(o_file, dt_expires) if *dt_expires > dt_now => {
                (Acquire::Hit(Arc::clone(o_file)), None)
            }
            SlotPhase::InFlight(o_notify) => (self.wait_on(o_notify), None),
            _ => self.start_lead(str_origin, e_file),
        }
    }

    /// Returns a registered, enabled notification for the in-flight fetch.
    fn wait_on(&self, o_notify: &Arc<Notify>) -> Acquire {
        let mut o_notified = Box::pin(Arc::clone(o_notify).notified_owned());
        o_notified.as_mut().enable();
        Acquire::Wait(o_notified)
    }

    /// Marks the slot in flight and hands out the lead, returning any replaced cached file.
    fn start_lead(
        &mut self,
        str_origin: &str,
        e_file: SignalFile,
    ) -> (Acquire, Option<Arc<FetchedFile>>) {
        let o_notify = Arc::new(Notify::new());
        let e_old = std::mem::replace(
            &mut self.e_phase,
            SlotPhase::InFlight(Arc::clone(&o_notify)),
        );
        let o_ticket = LeaderTicket {
            str_origin: str_origin.to_owned(),
            e_file,
            o_notify,
            u_failures: self.u_failures,
        };
        let opt_o_old = match e_old {
            SlotPhase::Ready(o_old, _) => Some(o_old),
            _ => None,
        };
        (Acquire::Lead(o_ticket), opt_o_old)
    }
}

/// The three file slots of a host and when the host was last used.
#[derive(Debug)]
struct HostSlots {
    /// Slots indexed by `SignalFile::index`.
    vec_o_slots: [Slot; 3],
    /// Last time any slot was requested.
    dt_last_used: OffsetDateTime,
}

impl HostSlots {
    /// Returns whether any file of the host is being fetched.
    fn is_busy(&self) -> bool {
        self.vec_o_slots
            .iter()
            .any(|o_slot| matches!(o_slot.e_phase, SlotPhase::InFlight(_)))
    }

    /// Returns the cached files of the host.
    fn into_ready_files(self) -> impl Iterator<Item = Arc<FetchedFile>> {
        self.vec_o_slots
            .into_iter()
            .filter_map(|o_slot| match o_slot.e_phase {
                SlotPhase::Ready(o_file, _) => Some(o_file),
                _ => None,
            })
    }
}

/// Hosts by origin (key), bounded by `u_max_hosts` hosts and by `u_max_bytes` of cached file
/// weight (archived body plus parsed form). Over either bound the least recently used host with
/// nothing in flight is dropped; dropped files are returned so their archived bodies can be
/// released. The host being stored is never dropped, so the overshoot is at most one host.
#[derive(Debug)]
pub struct HostTable {
    /// Hosts by origin (key).
    map_o_hosts: HashMap<String, HostSlots>,
    /// Sum of the weights of the cached files.
    u_cached_bytes: usize,
    /// Host limit.
    u_max_hosts: usize,
    /// Byte limit.
    u_max_bytes: usize,
}

impl HostTable {
    /// Creates an empty table.
    pub fn new(u_max_hosts: usize, u_max_bytes: usize) -> Self {
        Self {
            map_o_hosts: HashMap::new(),
            u_cached_bytes: 0,
            u_max_hosts: u_max_hosts.max(1),
            u_max_bytes,
        }
    }

    /// Returns the summed weight of the cached files.
    pub fn cached_bytes(&self) -> usize {
        self.u_cached_bytes
    }

    /// Returns a hit, a wait, the lead, or a bypass, plus the files dropped on the way.
    pub fn acquire(
        &mut self,
        str_origin: &str,
        e_file: SignalFile,
        dt_now: OffsetDateTime,
    ) -> (Acquire, Vec<Arc<FetchedFile>>) {
        let mut vec_o_freed = Vec::new();
        if !self.make_room(str_origin, &mut vec_o_freed) {
            return (Acquire::Bypass, vec_o_freed);
        }
        let o_host = self
            .map_o_hosts
            .entry(str_origin.to_owned())
            .or_insert_with(|| HostSlots {
                vec_o_slots: Default::default(),
                dt_last_used: dt_now,
            });
        o_host.dt_last_used = dt_now;
        let (e_acquire, opt_o_old) =
            o_host.vec_o_slots[e_file.index()].acquire(str_origin, e_file, dt_now);
        if let Some(o_old) = opt_o_old {
            self.u_cached_bytes = self.u_cached_bytes.saturating_sub(o_old.weight());
            vec_o_freed.push(o_old);
        }
        (e_acquire, vec_o_freed)
    }

    /// Caches the leader's result and returns the files dropped to stay within the byte bound
    /// (or the result itself when the slot is no longer the leader's).
    pub fn store(
        &mut self,
        o_ticket: &LeaderTicket,
        o_file: Arc<FetchedFile>,
        o_expiry: (OffsetDateTime, u32),
    ) -> Vec<Arc<FetchedFile>> {
        let u_weight = o_file.weight();
        let Some(o_slot) = self.own_slot(o_ticket) else {
            return vec![o_file];
        };
        o_slot.e_phase = SlotPhase::Ready(o_file, o_expiry.0);
        o_slot.u_failures = o_expiry.1;
        self.u_cached_bytes = self.u_cached_bytes.saturating_add(u_weight);
        let mut vec_o_freed = Vec::new();
        while self.u_cached_bytes > self.u_max_bytes
            && self.evict_lru(Some(&o_ticket.str_origin), &mut vec_o_freed)
        {}
        vec_o_freed
    }

    /// Clears a slot whose leader was cancelled, so a waiter can take over.
    pub fn abandon(&mut self, o_ticket: &LeaderTicket) {
        if let Some(o_slot) = self.own_slot(o_ticket) {
            o_slot.e_phase = SlotPhase::Empty;
        }
    }

    /// Returns the slot if it is still in flight under this ticket.
    fn own_slot(&mut self, o_ticket: &LeaderTicket) -> Option<&mut Slot> {
        let o_slot = self
            .map_o_hosts
            .get_mut(&o_ticket.str_origin)?
            .vec_o_slots
            .get_mut(o_ticket.e_file.index())?;
        let b_ours = matches!(&o_slot.e_phase, SlotPhase::InFlight(o_n) if Arc::ptr_eq(o_n, &o_ticket.o_notify));
        b_ours.then_some(o_slot)
    }

    /// Frees one host if a new origin would exceed the host bound; false when every host is busy.
    fn make_room(&mut self, str_origin: &str, vec_o_freed: &mut Vec<Arc<FetchedFile>>) -> bool {
        if self.map_o_hosts.contains_key(str_origin) || self.map_o_hosts.len() < self.u_max_hosts {
            return true;
        }
        self.evict_lru(None, vec_o_freed)
    }

    /// Drops the least recently used idle host other than `opt_str_keep`; false when none.
    fn evict_lru(
        &mut self,
        opt_str_keep: Option<&str>,
        vec_o_freed: &mut Vec<Arc<FetchedFile>>,
    ) -> bool {
        let opt_str_victim = self
            .map_o_hosts
            .iter()
            .filter(|(str_o, o_host)| !o_host.is_busy() && Some(str_o.as_str()) != opt_str_keep)
            .min_by_key(|(_, o_host)| o_host.dt_last_used)
            .map(|(str_origin, _)| str_origin.clone());
        let Some(o_host) = opt_str_victim.and_then(|str_o| self.map_o_hosts.remove(&str_o)) else {
            return false;
        };
        for o_file in o_host.into_ready_files() {
            self.u_cached_bytes = self.u_cached_bytes.saturating_sub(o_file.weight());
            vec_o_freed.push(o_file);
        }
        true
    }
}
