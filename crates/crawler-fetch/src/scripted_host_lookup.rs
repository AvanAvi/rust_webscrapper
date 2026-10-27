//! Answers lookups from a fixed script, including different answers on later calls.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Mutex, PoisonError};

use crate::host_lookup::{HostLookup, LookupFuture};

/// Successive answers for one host and how many calls have been made.
type ScriptEntry = (Vec<Vec<IpAddr>>, usize);

/// Lookup with scripted answers per host; the n-th call gets the n-th answer, then the last
/// answer repeats. Used to simulate DNS rebinding. Answers still pass the address policy.
#[derive(Debug, Default)]
pub struct ScriptedHostLookup {
    /// Answers per host name (key) and the number of calls made so far.
    mtx_script: Mutex<HashMap<String, ScriptEntry>>,
}

impl ScriptedHostLookup {
    /// Creates an empty script; unknown hosts fail to resolve.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the script with one answer that is always given for a host.
    pub fn with(self, str_host: &str, vec_e_ips: Vec<IpAddr>) -> Self {
        self.with_sequence(str_host, vec![vec_e_ips])
    }

    /// Returns the script with successive answers for a host.
    pub fn with_sequence(self, str_host: &str, vec_vec_answers: Vec<Vec<IpAddr>>) -> Self {
        self.mtx_script
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(str_host.to_ascii_lowercase(), (vec_vec_answers, 0));
        self
    }

    /// Returns the next scripted answer for a host.
    fn next_answer(&self, str_host: &str) -> Result<Vec<IpAddr>, String> {
        let mut map_o_script = self
            .mtx_script
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let Some((vec_vec_answers, u_calls)) = map_o_script.get_mut(&str_host.to_ascii_lowercase())
        else {
            return Err(String::from("no such host in script"));
        };
        let u_index = (*u_calls).min(vec_vec_answers.len().saturating_sub(1));
        *u_calls += 1;
        vec_vec_answers
            .get(u_index)
            .cloned()
            .ok_or_else(|| String::from("empty script"))
    }
}

impl HostLookup for ScriptedHostLookup {
    /// Returns the next scripted answer.
    fn lookup(&self, str_host: &str) -> LookupFuture {
        let res_vec_ips = self.next_answer(str_host);
        Box::pin(async move { res_vec_ips })
    }
}
