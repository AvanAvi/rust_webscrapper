//! Holds the typed result of parsing one signal source.

use serde::{Deserialize, Serialize};

use crate::signals::ai_txt_finding::AiTxtFinding;
use crate::signals::robots_finding::RobotsFinding;
use crate::signals::robots_tag_finding::RobotsTagFinding;
use crate::signals::tdmrep_finding::TdmrepFinding;

/// Typed finding from one signal source; contains no floats, so it always encodes canonically.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum ParsedSignal {
    /// Finding from robots.txt.
    RobotsTxt(RobotsFinding),
    /// Finding from ai.txt.
    AiTxt(AiTxtFinding),
    /// Finding from a TDMRep header, meta tag, or well-known file.
    Tdmrep(TdmrepFinding),
    /// Finding from an X-Robots-Tag header or robots meta tag.
    RobotsTag(RobotsTagFinding),
}
