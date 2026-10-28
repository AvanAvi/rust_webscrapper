//! Lists the per-host signal files.

use crawler_core::SignalKind;

/// One of the files fetched once per host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalFile {
    /// `/robots.txt` (RFC 9309).
    RobotsTxt,
    /// `/ai.txt`.
    AiTxt,
    /// `/.well-known/tdmrep.json`.
    TdmrepJson,
}

impl SignalFile {
    /// Every signal file, in collection order.
    pub const VEC_E_ALL: [Self; 3] = [Self::RobotsTxt, Self::AiTxt, Self::TdmrepJson];

    /// Returns the path on the host.
    pub fn path(&self) -> &'static str {
        match self {
            Self::RobotsTxt => "/robots.txt",
            Self::AiTxt => "/ai.txt",
            Self::TdmrepJson => "/.well-known/tdmrep.json",
        }
    }

    /// Returns the signal kind recorded in observations.
    pub fn kind(&self) -> SignalKind {
        match self {
            Self::RobotsTxt => SignalKind::RobotsTxt,
            Self::AiTxt => SignalKind::AiTxt,
            Self::TdmrepJson => SignalKind::TdmrepWellKnown,
        }
    }

    /// Returns the slot index in a host's cache entry.
    pub fn index(&self) -> usize {
        match self {
            Self::RobotsTxt => 0,
            Self::AiTxt => 1,
            Self::TdmrepJson => 2,
        }
    }
}
