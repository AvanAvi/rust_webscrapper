//! Declares which IP addresses the client may connect to.

use std::fmt::Debug;
use std::net::IpAddr;

/// Restricts the policies to those defined in this crate, so no outside type can widen access.
mod sealed {
    /// Marker implemented only by this crate's policies.
    pub trait Sealed {}
}

pub(crate) use sealed::Sealed;

/// An address refused by a policy, with the name of the range it fell in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddressRejection {
    /// The refused address.
    pub e_ip: IpAddr,
    /// Name of the forbidden range.
    pub str_range: &'static str,
}

/// Decides whether the client may connect to an address. Sealed: only `ProductionAddressPolicy`
/// and, with the `test-policy` feature, `LoopbackTestPolicy` implement it.
pub trait AddressPolicy: Sealed + Send + Sync + Debug {
    /// Returns `Ok` when the address may be connected to.
    fn check(&self, e_ip: IpAddr) -> Result<(), AddressRejection>;
}
