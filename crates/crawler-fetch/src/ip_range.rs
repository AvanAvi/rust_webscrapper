//! Describes one CIDR range of IP addresses.

use std::net::{Ipv4Addr, Ipv6Addr};

/// An IPv4 or IPv6 prefix with a name used in error messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IpRange {
    /// Network address as a 128-bit value (IPv4 in the low 32 bits).
    u_network: u128,
    /// Prefix length in bits.
    u_prefix: u32,
    /// Whether the range is IPv4.
    b_is_v4: bool,
    /// Human-readable range name.
    str_name: &'static str,
}

impl IpRange {
    /// Creates an IPv4 range.
    pub const fn v4(by_octets: [u8; 4], u_prefix: u32, str_name: &'static str) -> Self {
        let u_bits = u32::from_be_bytes(by_octets) as u128;
        Self {
            u_network: u_bits,
            u_prefix,
            b_is_v4: true,
            str_name,
        }
    }

    /// Creates an IPv6 range from its eight 16-bit segments.
    pub const fn v6(vec_u_segments: [u16; 8], u_prefix: u32, str_name: &'static str) -> Self {
        let o_addr = Ipv6Addr::new(
            vec_u_segments[0],
            vec_u_segments[1],
            vec_u_segments[2],
            vec_u_segments[3],
            vec_u_segments[4],
            vec_u_segments[5],
            vec_u_segments[6],
            vec_u_segments[7],
        );
        Self {
            u_network: o_addr.to_bits(),
            u_prefix,
            b_is_v4: false,
            str_name,
        }
    }

    /// Returns the range name.
    pub fn name(&self) -> &'static str {
        self.str_name
    }

    /// Returns whether an IPv4 address lies in this range.
    pub fn contains_v4(&self, o_ip: Ipv4Addr) -> bool {
        self.b_is_v4 && self.matches(u128::from(o_ip.to_bits()), 32)
    }

    /// Returns whether an IPv6 address lies in this range.
    pub fn contains_v6(&self, o_ip: Ipv6Addr) -> bool {
        !self.b_is_v4 && self.matches(o_ip.to_bits(), 128)
    }

    /// Compares the top `u_prefix` bits of an address of `u_width` bits.
    fn matches(&self, u_bits: u128, u_width: u32) -> bool {
        let u_shift = u_width - self.u_prefix.min(u_width);
        u_bits.checked_shr(u_shift).unwrap_or(0) == self.u_network.checked_shr(u_shift).unwrap_or(0)
    }
}
