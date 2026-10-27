//! Declares name resolution and the system resolver.

use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;

/// Future returned by a lookup: addresses or a reason the lookup failed.
pub type LookupFuture = Pin<Box<dyn Future<Output = Result<Vec<IpAddr>, String>> + Send>>;

/// Resolves a host name to addresses. Every answer is still filtered by the address policy.
pub trait HostLookup: Send + Sync + std::fmt::Debug {
    /// Returns the addresses of a host.
    fn lookup(&self, str_host: &str) -> LookupFuture;
}

/// Resolver backed by the operating system (`getaddrinfo`).
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemHostLookup;

impl SystemHostLookup {
    /// Creates the system resolver.
    pub fn new() -> Self {
        Self
    }
}

impl HostLookup for SystemHostLookup {
    /// Resolves through the operating system.
    fn lookup(&self, str_host: &str) -> LookupFuture {
        let str_target = format!("{str_host}:0");
        Box::pin(async move {
            let o_addrs = tokio::net::lookup_host(str_target)
                .await
                .map_err(|err_io| err_io.to_string())?;
            Ok(o_addrs.map(|o_sock| o_sock.ip()).collect())
        })
    }
}
