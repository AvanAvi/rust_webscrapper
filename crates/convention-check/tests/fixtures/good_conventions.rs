//! Fixture that follows every rule and must produce no violations.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Maximum number of items.
const U_MAX_ITEMS: usize = 3;
/// Default timeout.
const DUR_TIMEOUT: Duration = Duration::from_secs(5);
/// Process-wide label.
static STR_LABEL: &str = "fixture";

/// Kind of a record.
#[derive(Serialize)]
pub enum Kind {
    /// First kind.
    Alpha,
    /// Second kind with data.
    Beta {
        /// Payload bytes.
        #[serde(rename = "payload")]
        by_payload: Vec<u8>,
    },
}

/// Record stored as JSON and in a table.
#[derive(Serialize, Deserialize, FromRow)]
pub struct Record {
    /// Record name.
    #[serde(rename = "name")]
    #[sqlx(rename = "name")]
    str_name: String,
    /// Record kind.
    #[serde(skip)]
    #[sqlx(rename = "kind")]
    e_kind: Kind,
    /// Counters keyed by host.
    #[serde(rename = "counts")]
    #[sqlx(rename = "counts")]
    map_u_counts: HashMap<String, u64>,
    /// Shared cache of recent names.
    #[serde(flatten)]
    #[sqlx(rename = "cache")]
    mtx_cache: Arc<Mutex<Vec<String>>>,
}

/// Value serialized through `String`, so its fields never reach JSON.
#[derive(Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Stamp {
    /// Parsed instant.
    dt_value: OffsetDateTime,
    /// Canonical text.
    str_text: String,
}

/// Behaviour shared by records.
pub trait Named {
    /// Returns the display name.
    fn name(&self) -> &str;
}

impl Record {
    /// Returns the name length when it fits the limit.
    pub fn name_len(&self, opt_u_limit: Option<usize>) -> Option<usize> {
        let u_len = self.str_name.len();
        let u_limit: usize = opt_u_limit.unwrap_or(U_MAX_ITEMS);
        if u_len > u_limit {
            return None;
        }
        Some(u_len)
    }

    /// Counts cached names accepted by a filter.
    pub fn count_matching<F>(&self, cb_keep: F, dur_wait: Duration) -> usize
    where
        F: Fn(&str) -> bool,
    {
        let (u_seen, b_done) = (0usize, dur_wait > DUR_TIMEOUT);
        let Some(str_first) = self.str_name.get(0..1) else {
            return u_seen;
        };
        usize::from(cb_keep(str_first) && !b_done)
    }

    /// Returns whether this record has the same kind as another.
    pub fn is_same_kind(&self, e_other: &Kind, o_peer: &Self, by_raw: &[u8]) -> bool {
        let res_u_parsed: Result<u32, String> = Ok(by_raw.len() as u32);
        matches!((e_other, &o_peer.e_kind), (Kind::Alpha, Kind::Alpha)) && res_u_parsed.is_ok()
    }
}

impl Kind {
    /// Sums name lengths using loop, closure, match, and let-chain bindings.
    pub fn total_len(&self, vec_str_names: &[String], opt_u_limit: Option<usize>) -> usize {
        let mut u_total = 0;
        for (u_index, str_name) in vec_str_names.iter().enumerate() {
            u_total += u_index + str_name.len();
        }
        let u_long = vec_str_names.iter().filter(|str_n| str_n.len() > 3).count();
        let u_typed: usize = vec_str_names.iter().map(|str_n: &String| str_n.len()).sum();
        if let Some(u_limit) = opt_u_limit
            && u_limit < u_total
        {
            return u_limit;
        }
        match (self, opt_u_limit) {
            (Kind::Beta { by_payload }, _) => by_payload.len(),
            (Kind::Alpha, Some(u_cap)) if vec_str_names.iter().any(|str_x| str_x.len() > u_cap) => u_cap,
            (_, None) => u_total + u_long + u_typed,
            _ => U_MAX_ITEMS,
        }
    }

    /// Drains a stack of names with `while let`.
    pub fn drain(&self, mut vec_str_stack: Vec<String>) -> usize {
        let mut u_count = 0;
        while let Some(str_last) = vec_str_stack.pop() {
            u_count += str_last.len();
        }
        u_count
    }
}

/// Methods at the nesting and loop limits that must pass.
pub struct Depth;

impl Depth {
    /// Uses exactly two levels: a loop containing a let-else, a match, and a closure.
    pub fn at_limit(&self, vec_u_items: &[u32]) -> u32 {
        let mut u_total = 0;
        for u_item in vec_u_items {
            let Some(u_next) = u_item.checked_add(1) else {
                continue;
            };
            match u_next {
                0 => u_total += 0,
                _ => u_total += 1,
            }
            u_total += vec_u_items.iter().filter(|u_x| **u_x > 1).count() as u32;
        }
        u_total
    }

    /// Long `else if` chains stay at one level; the final `else` holds one more.
    pub fn grade(&self, u_score: u32) -> &'static str {
        if u_score > 90 {
            "a"
        } else if u_score > 80 {
            "b"
        } else if u_score > 70 {
            "c"
        } else if u_score > 60 {
            "d"
        } else {
            if u_score > 50 { "e" } else { "f" }
        }
    }

    /// Two sequential loops are fine; only nested loops break the rule.
    pub fn two_loops(&self, vec_u_items: &[u32]) -> u32 {
        let mut u_total = 0;
        for u_item in vec_u_items {
            u_total += u_item;
        }
        while u_total > 100 {
            u_total /= 2;
        }
        u_total
    }
}

impl Named for Record {
    /// Returns the record name.
    fn name(&self) -> &str {
        STR_LABEL
    }
}

#[cfg(test)]
mod tests {
    fn helper_value() -> u32 {
        1
    }

    #[test]
    fn test_helper_value() {
        let u_value = helper_value();
        assert_eq!(u_value, 1);
    }
}
