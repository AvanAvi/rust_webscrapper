//! Names the rules a decision can rest on, in precedence order.

/// Rule that decided a crawl decision. When several signals deny, the first in this order is
/// reported as the matched rule; every denying signal is still listed in the reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PolicyRule {
    /// A signal was fetched but its body could not be archived: fail closed, for every purpose.
    EvidenceNotArchived,
    /// robots.txt answered 5xx or could not be fetched: full disallow (RFC 9309 2.3.1.4).
    RobotsUnreachable,
    /// A robots.txt rule disallows the path.
    RobotsDisallow,
    /// A robots.txt Content-Signal says `no` for the declared purpose.
    ContentSignalNo,
    /// A TDMRep source sets `tdm-reservation: 1` and the purpose is an AI use.
    TdmrepReserved,
    /// ai.txt disallows the page and the purpose is an AI use.
    AiTxtDisallow,
    /// An X-Robots-Tag or robots meta `noai` applies and the purpose is an AI use.
    RobotsTagNoai,
    /// An X-Robots-Tag or robots meta `noimageai` applies and the purpose is an AI use.
    RobotsTagNoimageai,
    /// No observed signal reserves the URL against the purpose.
    NoReservation,
}

impl PolicyRule {
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
