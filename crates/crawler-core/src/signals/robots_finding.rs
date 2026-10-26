//! Records what robots.txt says about one URL for one crawler.

use serde::{Deserialize, Serialize};

use crate::signals::content_signal::ContentSignal;
use crate::signals::robots_rule::RobotsRule;

/// The deciding group and rule plus group-level directives; the full file is identified by
/// the observation's content hash instead of being copied into every receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RobotsFinding {
    /// Whether the path may be crawled under RFC 9309.
    #[serde(rename = "allowed")]
    pub b_allowed: bool,
    /// User-agent group that applied: the product token, `*`, or `None` when no group applied.
    #[serde(rename = "matched_agent", deserialize_with = "Option::deserialize")]
    pub opt_str_matched_agent: Option<String>,
    /// Rule that decided, or `None` when no rule matched.
    #[serde(rename = "matched_rule", deserialize_with = "Option::deserialize")]
    pub opt_o_matched_rule: Option<RobotsRule>,
    /// Largest Crawl-delay among the applying groups, in milliseconds.
    #[serde(rename = "crawl_delay_ms", deserialize_with = "Option::deserialize")]
    pub opt_u_crawl_delay_ms: Option<u64>,
    /// Content-Signal preferences of the applying groups.
    #[serde(rename = "content_signal", deserialize_with = "Option::deserialize")]
    pub opt_o_content_signal: Option<ContentSignal>,
    /// Number of groups in the file.
    #[serde(rename = "groups")]
    pub u_groups: u32,
    /// Lines and records that could not be used.
    #[serde(rename = "invalid_lines")]
    pub u_invalid_lines: u32,
    /// Whether the file exceeded the 500 KiB parsing limit.
    #[serde(rename = "truncated")]
    pub b_truncated: bool,
}
