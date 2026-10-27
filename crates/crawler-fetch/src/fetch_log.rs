//! Collects what happened during one fetch: redirect hops, vetted addresses, refusals.

use std::net::IpAddr;
use std::sync::{Mutex, PoisonError};

use crawler_core::RedirectHop;

use crate::fetch_error::FetchError;

/// Per-fetch record shared by the resolver, the redirect guard, and the client.
#[derive(Debug, Default)]
pub struct FetchLog {
    /// Redirects followed or refused, in order.
    mtx_redirects: Mutex<Vec<RedirectHop>>,
    /// Addresses that passed the policy and were handed to the connector.
    mtx_ips: Mutex<Vec<IpAddr>>,
    /// First refusal raised inside reqwest callbacks.
    mtx_refusal: Mutex<Option<FetchError>>,
}

impl FetchLog {
    /// Creates an empty log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a redirect response.
    pub fn record_hop(&self, o_hop: RedirectHop) {
        self.mtx_redirects
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(o_hop);
    }

    /// Records vetted addresses, keeping each once.
    pub fn record_ips(&self, vec_e_ips: &[IpAddr]) {
        let mut vec_e_known = self.mtx_ips.lock().unwrap_or_else(PoisonError::into_inner);
        for e_ip in vec_e_ips {
            if !vec_e_known.contains(e_ip) {
                vec_e_known.push(*e_ip);
            }
        }
    }

    /// Records a refusal; only the first one is kept.
    pub fn refuse(&self, err_refusal: FetchError) {
        let mut opt_err = self
            .mtx_refusal
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if opt_err.is_none() {
            *opt_err = Some(err_refusal);
        }
    }

    /// Returns the recorded refusal, if any.
    pub fn refusal(&self) -> Option<FetchError> {
        self.mtx_refusal
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Returns the recorded redirect hops.
    pub fn redirects(&self) -> Vec<RedirectHop> {
        self.mtx_redirects
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Returns the vetted addresses.
    pub fn ips(&self) -> Vec<IpAddr> {
        self.mtx_ips
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}
