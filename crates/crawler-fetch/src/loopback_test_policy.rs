//! Allows only loopback, so tests can reach the local mock-site. Compiled only with `test-policy`.

use std::net::IpAddr;

use crate::address_policy::{AddressPolicy, AddressRejection, Sealed};

/// Test policy: the opposite of production for loopback, and closed for everything else.
#[derive(Debug, Clone, Copy, Default)]
pub struct LoopbackTestPolicy;

impl LoopbackTestPolicy {
    /// Creates the policy.
    pub fn new() -> Self {
        Self
    }
}

impl Sealed for LoopbackTestPolicy {}

impl AddressPolicy for LoopbackTestPolicy {
    /// Allows 127.0.0.0/8 and ::1 (including ::ffff:127.0.0.0/104); refuses everything else.
    fn check(&self, e_ip: IpAddr) -> Result<(), AddressRejection> {
        let b_loopback = match e_ip {
            IpAddr::V4(o_v4) => o_v4.is_loopback(),
            IpAddr::V6(o_v6) => {
                o_v6.is_loopback() || o_v6.to_ipv4_mapped().is_some_and(|o_v4| o_v4.is_loopback())
            }
        };
        if b_loopback {
            return Ok(());
        }
        Err(AddressRejection {
            e_ip,
            str_range: "non-loopback (test policy allows loopback only)",
        })
    }
}
