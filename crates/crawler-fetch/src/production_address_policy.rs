//! Forbids every non-public address range, including embedded IPv4 forms.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::address_policy::{AddressPolicy, AddressRejection, Sealed};
use crate::ip_range::IpRange;

/// IPv4 ranges that are never crawled.
const VEC_O_BLOCKED_V4: [IpRange; 17] = [
    IpRange::v4([0, 0, 0, 0], 8, "unspecified/this-network 0.0.0.0/8"),
    IpRange::v4([10, 0, 0, 0], 8, "private 10.0.0.0/8"),
    IpRange::v4([100, 64, 0, 0], 10, "carrier-grade NAT 100.64.0.0/10"),
    IpRange::v4([127, 0, 0, 0], 8, "loopback 127.0.0.0/8"),
    IpRange::v4(
        [169, 254, 0, 0],
        16,
        "link-local and cloud metadata 169.254.0.0/16",
    ),
    IpRange::v4([172, 16, 0, 0], 12, "private 172.16.0.0/12"),
    IpRange::v4([192, 0, 0, 0], 24, "IETF protocol assignments 192.0.0.0/24"),
    IpRange::v4([192, 0, 2, 0], 24, "documentation 192.0.2.0/24"),
    IpRange::v4([192, 88, 99, 0], 24, "6to4 relay anycast 192.88.99.0/24"),
    IpRange::v4([192, 168, 0, 0], 16, "private 192.168.0.0/16"),
    IpRange::v4([198, 18, 0, 0], 15, "benchmarking 198.18.0.0/15"),
    IpRange::v4([198, 51, 100, 0], 24, "documentation 198.51.100.0/24"),
    IpRange::v4([203, 0, 113, 0], 24, "documentation 203.0.113.0/24"),
    IpRange::v4([224, 0, 0, 0], 4, "multicast 224.0.0.0/4"),
    IpRange::v4([240, 0, 0, 0], 4, "reserved 240.0.0.0/4"),
    IpRange::v4([255, 255, 255, 255], 32, "broadcast 255.255.255.255"),
    IpRange::v4([100, 100, 100, 200], 32, "cloud metadata 100.100.100.200"),
];

/// IPv6 ranges that are never crawled. IPv4-mapped, NAT64, and 6to4 forms are also checked
/// by the IPv4 address they carry; IPv4-compatible addresses are refused outright.
const VEC_O_BLOCKED_V6: [IpRange; 13] = [
    IpRange::v6([0, 0, 0, 0, 0, 0, 0, 0], 128, "unspecified ::"),
    IpRange::v6([0, 0, 0, 0, 0, 0, 0, 1], 128, "loopback ::1"),
    IpRange::v6(
        [0, 0, 0, 0, 0, 0, 0, 0],
        96,
        "IPv4-compatible (deprecated) ::/96",
    ),
    IpRange::v6(
        [0x64, 0xff9b, 1, 0, 0, 0, 0, 0],
        48,
        "local-use NAT64 64:ff9b:1::/48",
    ),
    IpRange::v6([0x100, 0, 0, 0, 0, 0, 0, 0], 64, "discard 100::/64"),
    IpRange::v6(
        [0x2001, 0, 0, 0, 0, 0, 0, 0],
        23,
        "IETF protocol assignments 2001::/23",
    ),
    IpRange::v6(
        [0x2001, 0xdb8, 0, 0, 0, 0, 0, 0],
        32,
        "documentation 2001:db8::/32",
    ),
    IpRange::v6([0x3fff, 0, 0, 0, 0, 0, 0, 0], 20, "documentation 3fff::/20"),
    IpRange::v6(
        [0xfc00, 0, 0, 0, 0, 0, 0, 0],
        7,
        "unique local (incl. fd00:ec2::254 metadata) fc00::/7",
    ),
    IpRange::v6([0xfe80, 0, 0, 0, 0, 0, 0, 0], 10, "link-local fe80::/10"),
    IpRange::v6([0xfec0, 0, 0, 0, 0, 0, 0, 0], 10, "site-local fec0::/10"),
    IpRange::v6([0xff00, 0, 0, 0, 0, 0, 0, 0], 8, "multicast ff00::/8"),
    IpRange::v6([0x5f00, 0, 0, 0, 0, 0, 0, 0], 16, "SRv6 SIDs 5f00::/16"),
];

/// Policy for real crawls: only globally routable unicast addresses are allowed.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProductionAddressPolicy;

impl ProductionAddressPolicy {
    /// Creates the policy.
    pub fn new() -> Self {
        Self
    }

    /// Checks an IPv4 address against the blocked ranges.
    fn check_v4(&self, o_ip: Ipv4Addr, e_original: IpAddr) -> Result<(), AddressRejection> {
        match VEC_O_BLOCKED_V4
            .iter()
            .find(|o_range| o_range.contains_v4(o_ip))
        {
            Some(o_range) => Err(AddressRejection {
                e_ip: e_original,
                str_range: o_range.name(),
            }),
            None => Ok(()),
        }
    }

    /// Checks an IPv6 address, including IPv4 embedded by mapping, compatibility, NAT64, or 6to4.
    fn check_v6(&self, o_ip: Ipv6Addr, e_original: IpAddr) -> Result<(), AddressRejection> {
        if let Some(o_range) = VEC_O_BLOCKED_V6
            .iter()
            .find(|o_range| o_range.contains_v6(o_ip))
        {
            return Err(AddressRejection {
                e_ip: e_original,
                str_range: o_range.name(),
            });
        }
        match self.embedded_v4(o_ip) {
            Some(o_v4) => self.check_v4(o_v4, e_original),
            None => Ok(()),
        }
    }

    /// Returns the IPv4 address carried by an IPv4-mapped, NAT64, or 6to4 IPv6 address.
    fn embedded_v4(&self, o_ip: Ipv6Addr) -> Option<Ipv4Addr> {
        let vec_u_seg = o_ip.segments();
        let o_low = Ipv4Addr::from_bits(o_ip.to_bits() as u32);
        match vec_u_seg {
            [0, 0, 0, 0, 0, 0xffff, _, _] | [0x64, 0xff9b, 0, 0, 0, 0, _, _] => Some(o_low),
            [0x2002, u_hi, u_lo, ..] => Some(Ipv4Addr::from_bits(
                (u32::from(u_hi) << 16) | u32::from(u_lo),
            )),
            _ => None,
        }
    }
}

impl Sealed for ProductionAddressPolicy {}

impl AddressPolicy for ProductionAddressPolicy {
    /// Allows only globally routable unicast addresses.
    fn check(&self, e_ip: IpAddr) -> Result<(), AddressRejection> {
        match e_ip {
            IpAddr::V4(o_v4) => self.check_v4(o_v4, e_ip),
            IpAddr::V6(o_v6) => self.check_v6(o_v6, e_ip),
        }
    }
}
