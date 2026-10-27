//! Describes one GET request.

use url::Url;

use crate::host_scope::HostScope;

/// URL to fetch and how far redirects may go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchRequest {
    /// Target URL.
    pub url_target: Url,
    /// Redirect host scope.
    pub e_scope: HostScope,
}

impl FetchRequest {
    /// Creates a request that may redirect to any allowed host.
    pub fn new(url_target: Url) -> Self {
        Self {
            url_target,
            e_scope: HostScope::AnyHost,
        }
    }

    /// Returns a copy restricted to the given redirect scope.
    pub fn with_scope(mut self, e_scope: HostScope) -> Self {
        self.e_scope = e_scope;
        self
    }
}
