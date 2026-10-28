//! Per-host cache of signal files with single-flight fetching.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex, PoisonError};

use crawler_core::BodyArchive;
use time::OffsetDateTime;
use tokio::sync::Notify;
use tokio::sync::futures::OwnedNotified;

use crate::collector::fetched_file::FetchedFile;
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

/// The three file slots of a host and when the host was last used.
#[derive(Debug)]
struct HostSlots {
    /// Slots indexed by `SignalFile::index`.
    vec_o_slots: [Slot; 3],
    /// Last time any slot was requested.
    dt_last_used: OffsetDateTime,
}

/// Cache keyed by origin with at most `u_max_hosts` hosts. At the limit the least recently used
/// host with nothing in flight is dropped (its archived bodies are released); if every host has a
/// fetch in flight, the caller fetches without caching.
pub struct HostCache {
    /// Hosts by origin (key).
    mtx_hosts: Mutex<HashMap<String, HostSlots>>,
    /// Host limit.
    u_max_hosts: usize,
    /// Archive whose references the cache holds for cached bodies.
    o_archive: Arc<dyn BodyArchive>,
}

impl HostCache {
    /// Creates an empty cache.
    pub fn new(u_max_hosts: usize, o_archive: Arc<dyn BodyArchive>) -> Self {
        Self {
            mtx_hosts: Mutex::new(HashMap::new()),
            u_max_hosts: u_max_hosts.max(1),
            o_archive,
        }
    }

    /// Returns a hit, a wait, the lead, or a bypass for one file of one host.
    pub fn acquire(&self, str_origin: &str, e_file: SignalFile, dt_now: OffsetDateTime) -> Acquire {
        let mut map_o_hosts = self
            .mtx_hosts
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if !map_o_hosts.contains_key(str_origin) && !self.make_room(&mut map_o_hosts) {
            return Acquire::Bypass;
        }
        let o_host = map_o_hosts
            .entry(str_origin.to_owned())
            .or_insert_with(|| HostSlots {
                vec_o_slots: Default::default(),
                dt_last_used: dt_now,
            });
        o_host.dt_last_used = dt_now;
        let o_slot = &mut o_host.vec_o_slots[e_file.index()];
        match &o_slot.e_phase {
            SlotPhase::Ready(o_file, dt_expires) if *dt_expires > dt_now => {
                Acquire::Hit(Arc::clone(o_file))
            }
            SlotPhase::InFlight(o_notify) => Self::wait_on(o_notify),
            _ => self.start_lead(o_slot, str_origin, e_file),
        }
    }

    /// Stores the leader's result, wakes the waiters, and releases the replaced body.
    pub fn complete(
        &self,
        o_ticket: &LeaderTicket,
        o_file: Arc<FetchedFile>,
        o_expiry: (OffsetDateTime, u32),
    ) {
        let mut map_o_hosts = self
            .mtx_hosts
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        match self.own_slot(&mut map_o_hosts, o_ticket) {
            Some(o_slot) => {
                o_slot.e_phase = SlotPhase::Ready(o_file, o_expiry.0);
                o_slot.u_failures = o_expiry.1;
            }
            None => self.release(&o_file),
        }
        o_ticket.o_notify.notify_waiters();
    }

    /// Clears a slot whose leader was cancelled, so a waiter can take over.
    pub fn abandon(&self, o_ticket: &LeaderTicket) {
        let mut map_o_hosts = self
            .mtx_hosts
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(o_slot) = self.own_slot(&mut map_o_hosts, o_ticket) {
            o_slot.e_phase = SlotPhase::Empty;
        }
        o_ticket.o_notify.notify_waiters();
    }

    /// Returns a registered, enabled notification for the in-flight fetch.
    fn wait_on(o_notify: &Arc<Notify>) -> Acquire {
        let mut o_notified = Box::pin(Arc::clone(o_notify).notified_owned());
        o_notified.as_mut().enable();
        Acquire::Wait(o_notified)
    }

    /// Marks the slot in flight, releases an expired body, and hands out the lead.
    fn start_lead(&self, o_slot: &mut Slot, str_origin: &str, e_file: SignalFile) -> Acquire {
        let o_notify = Arc::new(Notify::new());
        let e_old = std::mem::replace(
            &mut o_slot.e_phase,
            SlotPhase::InFlight(Arc::clone(&o_notify)),
        );
        if let SlotPhase::Ready(o_old, _) = e_old {
            self.release(&o_old);
        }
        Acquire::Lead(LeaderTicket {
            str_origin: str_origin.to_owned(),
            e_file,
            o_notify,
            u_failures: o_slot.u_failures,
        })
    }

    /// Returns the slot if it is still in flight under this ticket.
    fn own_slot<'m>(
        &self,
        map_o_hosts: &'m mut HashMap<String, HostSlots>,
        o_ticket: &LeaderTicket,
    ) -> Option<&'m mut Slot> {
        let o_slot = map_o_hosts
            .get_mut(&o_ticket.str_origin)?
            .vec_o_slots
            .get_mut(o_ticket.e_file.index())?;
        let b_ours = matches!(&o_slot.e_phase, SlotPhase::InFlight(o_n) if Arc::ptr_eq(o_n, &o_ticket.o_notify));
        b_ours.then_some(o_slot)
    }

    /// Frees one host slot if the cache is full; returns false when every host is busy.
    fn make_room(&self, map_o_hosts: &mut HashMap<String, HostSlots>) -> bool {
        if map_o_hosts.len() < self.u_max_hosts {
            return true;
        }
        let opt_str_victim = map_o_hosts
            .iter()
            .filter(|(_, o_host)| {
                !o_host
                    .vec_o_slots
                    .iter()
                    .any(|o_s| matches!(o_s.e_phase, SlotPhase::InFlight(_)))
            })
            .min_by_key(|(_, o_host)| o_host.dt_last_used)
            .map(|(str_origin, _)| str_origin.clone());
        let Some(o_host) = opt_str_victim.and_then(|str_origin| map_o_hosts.remove(&str_origin))
        else {
            return false;
        };
        self.release_host(&o_host);
        true
    }

    /// Releases the archived bodies of a dropped host.
    fn release_host(&self, o_host: &HostSlots) {
        for o_slot in &o_host.vec_o_slots {
            if let SlotPhase::Ready(o_file, _) = &o_slot.e_phase {
                self.release(o_file);
            }
        }
    }

    /// Releases the cache's reference to a file's archived body.
    fn release(&self, o_file: &FetchedFile) {
        if let Some(str_sha256) = &o_file.opt_str_sha256 {
            self.o_archive.release(str_sha256);
        }
    }
}
