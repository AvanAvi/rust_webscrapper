//! Names the rules a decision can rest on, in precedence order.

use serde::{Deserialize, Serialize};

/// Rule that decided a crawl decision. When several signals deny, the first in this order is
/// reported as the matched rule; every denying signal is still listed in the reasons.
///
/// Encodes as its stable id (`as_str`); decoding rejects any other id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PolicyRule {
    /// A signal was fetched but its body could not be archived: fail closed, for every purpose.
    #[serde(rename = "evidence.not_archived")]
    EvidenceNotArchived,
    /// robots.txt answered 5xx or could not be fetched: full disallow (RFC 9309 2.3.1.4).
    #[serde(rename = "robots_txt.unreachable")]
    RobotsUnreachable,
    /// A robots.txt rule disallows the path.
    #[serde(rename = "robots_txt.disallow")]
    RobotsDisallow,
    /// A robots.txt Content-Signal says `no` for the declared purpose.
    #[serde(rename = "robots_txt.content_signal")]
    ContentSignalNo,
    /// A TDMRep source sets `tdm-reservation: 1` and the purpose is an AI use.
    #[serde(rename = "tdmrep.reservation")]
    TdmrepReserved,
    /// ai.txt disallows the page and the purpose is an AI use.
    #[serde(rename = "ai_txt.disallow")]
    AiTxtDisallow,
    /// An X-Robots-Tag or robots meta `noai` applies and the purpose is an AI use.
    #[serde(rename = "robots_tag.noai")]
    RobotsTagNoai,
    /// An X-Robots-Tag or robots meta `noimageai` applies and the purpose is an AI use.
    #[serde(rename = "robots_tag.noimageai")]
    RobotsTagNoimageai,
    /// No observed signal reserves the URL against the purpose.
    #[serde(rename = "no_reservation")]
    NoReservation,
}

impl PolicyRule {
    /// Every rule, in precedence order.
    pub const VEC_E_ALL: [Self; 9] = [
        Self::EvidenceNotArchived,
        Self::RobotsUnreachable,
        Self::RobotsDisallow,
        Self::ContentSignalNo,
        Self::TdmrepReserved,
        Self::AiTxtDisallow,
        Self::RobotsTagNoai,
        Self::RobotsTagNoimageai,
        Self::NoReservation,
    ];

    /// Returns the stable identifier stored in receipts.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EvidenceNotArchived => "evidence.not_archived",
            Self::RobotsUnreachable => "robots_txt.unreachable",
            Self::RobotsDisallow => "robots_txt.disallow",
            Self::ContentSignalNo => "robots_txt.content_signal",
            Self::TdmrepReserved => "tdmrep.reservation",
            Self::AiTxtDisallow => "ai_txt.disallow",
            Self::RobotsTagNoai => "robots_tag.noai",
            Self::RobotsTagNoimageai => "robots_tag.noimageai",
            Self::NoReservation => "no_reservation",
        }
    }
}
