//! Lists the machine-readable opt-out signals the crawler observes.

use serde::{Deserialize, Serialize};

/// Source and format of one observed opt-out signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalKind {
    /// `/robots.txt`, including Crawl-delay and Content-Signal lines.
    RobotsTxt,
    /// `/ai.txt`.
    AiTxt,
    /// `/.well-known/tdmrep.json`.
    TdmrepWellKnown,
    /// `tdm-reservation` HTTP response header.
    TdmrepHeader,
    /// `tdm-reservation` HTML meta tag.
    TdmrepMeta,
    /// `X-Robots-Tag` HTTP response header.
    XRobotsTag,
    /// `<meta name="robots">` HTML tag.
    MetaRobots,
}
