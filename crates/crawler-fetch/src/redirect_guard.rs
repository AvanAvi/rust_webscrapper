//! Re-validates every redirect hop: count, scheme, downgrade, host scope, and literal address.

use std::sync::Arc;

use crawler_core::RedirectHop;
use reqwest::redirect::{Action, Attempt};
use url::Url;

use crate::fetch_error::FetchError;
use crate::fetch_log::FetchLog;
use crate::host_scope::HostScope;
use crate::url_safety::UrlSafety;

/// Redirect policy for one fetch; records each hop and refuses unsafe ones.
#[derive(Debug, Clone)]
pub struct RedirectGuard {
    /// URL validator applied to each target.
    o_safety: UrlSafety,
    /// Allowed host scope.
    e_scope: HostScope,
    /// Maximum number of redirects followed.
    u_max_hops: usize,
    /// Per-fetch log.
    o_log: Arc<FetchLog>,
}

impl RedirectGuard {
    /// Creates a guard for one fetch.
    pub fn new(
        o_safety: UrlSafety,
        e_scope: HostScope,
        u_max_hops: usize,
        o_log: Arc<FetchLog>,
    ) -> Self {
        Self {
            o_safety,
            e_scope,
            u_max_hops,
            o_log,
        }
    }

    /// Decides one redirect attempt for reqwest, recording the hop and any refusal.
    pub fn decide(&self, o_attempt: Attempt<'_>) -> Action {
        let vec_url_previous: Vec<Url> = o_attempt.previous().to_vec();
        let url_to = o_attempt.url().clone();
        let (Some(url_origin), Some(url_from)) =
            (vec_url_previous.first(), vec_url_previous.last())
        else {
            return o_attempt.error(FetchError::Internal {
                str_reason: String::from("redirect without origin"),
            });
        };
        self.o_log.record_hop(RedirectHop {
            str_url: url_from.to_string(),
            u_status: o_attempt.status().as_u16(),
        });
        match self.check_hop(url_origin, url_from, &url_to, vec_url_previous.len()) {
            Ok(()) => o_attempt.follow(),
            Err(err_refusal) => {
                self.o_log.refuse(err_refusal.clone());
                o_attempt.error(err_refusal)
            }
        }
    }

    /// Returns `Ok` when the hop from `url_from` to `url_to` may be followed as redirect number `u_hop`.
    pub fn check_hop(
        &self,
        url_origin: &Url,
        url_from: &Url,
        url_to: &Url,
        u_hop: usize,
    ) -> Result<(), FetchError> {
        if u_hop > self.u_max_hops {
            return Err(FetchError::TooManyRedirects {
                u_max: self.u_max_hops,
            });
        }
        if url_from.scheme() == "https" && url_to.scheme() == "http" {
            return Err(FetchError::RedirectDowngrade {
                str_to: self.o_safety.redact(url_to),
            });
        }
        if self.e_scope == HostScope::SameHost && url_origin.host_str() != url_to.host_str() {
            return Err(FetchError::RedirectOutOfScope {
                str_to: self.o_safety.redact(url_to),
            });
        }
        self.o_safety.check(url_to)
    }
}
