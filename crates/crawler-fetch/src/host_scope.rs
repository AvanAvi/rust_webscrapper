//! Lists how far redirects may move away from the requested host.

/// Hosts a redirect may lead to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HostScope {
    /// Any host whose addresses pass the address policy.
    #[default]
    AnyHost,
    /// Only the host of the original request.
    SameHost,
}
