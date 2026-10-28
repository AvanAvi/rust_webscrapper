//! Describes one GET request.

use url::Url;

use crate::body_budget::{BodyBudget, BodyOverflow};
use crate::host_scope::HostScope;

/// URL to fetch and how far redirects may go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchRequest {
    /// Target URL.
    pub url_target: Url,
    /// Redirect host scope.
    pub e_scope: HostScope,
    /// Smaller body cap and overflow behaviour for this request; `None` uses the client cap.
    pub opt_o_budget: Option<BodyBudget>,
}

impl FetchRequest {
    /// Creates a request that may redirect to any allowed host.
    pub fn new(url_target: Url) -> Self {
        Self {
            url_target,
            e_scope: HostScope::AnyHost,
            opt_o_budget: None,
        }
    }

    /// Returns a copy with its own body cap (never above the client cap) and overflow behaviour.
    pub fn with_body_cap(mut self, u_cap: usize, e_overflow: BodyOverflow) -> Self {
        self.opt_o_budget = Some(BodyBudget { u_cap, e_overflow });
        self
    }

    /// Returns a copy restricted to the given redirect scope.
    pub fn with_scope(mut self, e_scope: HostScope) -> Self {
        self.e_scope = e_scope;
        self
    }
}
