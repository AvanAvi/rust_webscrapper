//! Holds the bodies and bookkeeping of the in-memory archive.

use std::collections::HashMap;

use crate::core_error::CoreError;

/// One archived body with its reference count and insertion order.
#[derive(Debug, Clone)]
struct ArchivedBody {
    /// Body bytes.
    by_body: Vec<u8>,
    /// Number of holders; zero means evictable.
    u_refs: u64,
    /// Insertion order, used to evict the oldest unreferenced body first.
    u_order: u64,
}

/// Bodies by SHA-256 hex (key) plus the total stored size.
#[derive(Debug, Default)]
pub struct ArchiveState {
    /// Bodies by hash (key).
    map_o_bodies: HashMap<String, ArchivedBody>,
    /// Sum of stored body sizes.
    u_total_bytes: usize,
    /// Next insertion number.
    u_next_order: u64,
}

impl ArchiveState {
    /// Adds a reference to an existing body; returns whether it existed.
    pub fn add_ref(&mut self, str_sha256: &str) -> bool {
        let Some(o_body) = self.map_o_bodies.get_mut(str_sha256) else {
            return false;
        };
        o_body.u_refs = o_body.u_refs.saturating_add(1);
        true
    }

    /// Drops a reference from a body, if present.
    pub fn drop_ref(&mut self, str_sha256: &str) {
        if let Some(o_body) = self.map_o_bodies.get_mut(str_sha256) {
            o_body.u_refs = o_body.u_refs.saturating_sub(1);
        }
    }

    /// Returns a copy of a body.
    pub fn body(&self, str_sha256: &str) -> Option<Vec<u8>> {
        self.map_o_bodies
            .get(str_sha256)
            .map(|o_body| o_body.by_body.clone())
    }

    /// Returns the stored bytes and the number of stored bodies.
    pub fn usage(&self) -> (usize, usize) {
        (self.u_total_bytes, self.map_o_bodies.len())
    }

    /// Inserts a new body with one reference, evicting unreferenced bodies to stay within the
    /// byte capacity and the entry limit.
    pub fn insert(
        &mut self,
        str_sha256: String,
        by_body: &[u8],
        u_capacity: usize,
        u_max_entries: usize,
    ) -> Result<(), CoreError> {
        while self.map_o_bodies.len() >= u_max_entries {
            if !self.evict_oldest_unreferenced() {
                return Err(CoreError::ArchiveTooManyEntries { u_max_entries });
            }
        }
        while self.u_total_bytes + by_body.len() > u_capacity {
            if !self.evict_oldest_unreferenced() {
                return Err(CoreError::ArchiveFull { u_capacity });
            }
        }
        self.u_total_bytes += by_body.len();
        self.u_next_order += 1;
        let o_body = ArchivedBody {
            by_body: by_body.to_vec(),
            u_refs: 1,
            u_order: self.u_next_order,
        };
        self.map_o_bodies.insert(str_sha256, o_body);
        Ok(())
    }

    /// Removes the oldest body nobody references; returns false when every body is referenced.
    fn evict_oldest_unreferenced(&mut self) -> bool {
        let opt_str_victim = self
            .map_o_bodies
            .iter()
            .filter(|(_, o_body)| o_body.u_refs == 0)
            .min_by_key(|(_, o_body)| o_body.u_order)
            .map(|(str_hash, _)| str_hash.clone());
        let Some(str_victim) = opt_str_victim else {
            return false;
        };
        if let Some(o_body) = self.map_o_bodies.remove(&str_victim) {
            self.u_total_bytes -= o_body.by_body.len();
        }
        true
    }
}
