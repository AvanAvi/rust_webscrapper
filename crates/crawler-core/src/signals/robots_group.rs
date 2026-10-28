//! Holds one user-agent group of a robots-style file.

use crate::signals::content_signal::ContentSignal;
use crate::signals::robots_rule::RobotsRule;

/// User-agent lines and the records that follow them up to the next group.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RobotsGroup {
    /// User-agent values that start the group, as written.
    pub vec_str_agents: Vec<String>,
    /// Allow and disallow rules in file order.
    pub vec_o_rules: Vec<RobotsRule>,
    /// Largest Crawl-delay in the group, in milliseconds.
    pub opt_u_crawl_delay_ms: Option<u64>,
    /// Merged Content-Signal preferences of the group.
    pub opt_o_content_signal: Option<ContentSignal>,
}

impl RobotsGroup {
    /// Returns an estimate of the heap bytes the group holds (lengths, not allocator slack).
    pub fn heap_bytes(&self) -> usize {
        let u_agents: usize = self
            .vec_str_agents
            .iter()
            .map(|str_agent| size_of::<String>() + str_agent.len())
            .sum();
        let u_rules: usize = self
            .vec_o_rules
            .iter()
            .map(|o_rule| size_of::<RobotsRule>() + o_rule.str_pattern.len())
            .sum();
        u_agents + u_rules
    }

    /// Returns whether a user-agent line equals the product token, ignoring ASCII case.
    pub fn has_agent(&self, str_token: &str) -> bool {
        self.vec_str_agents
            .iter()
            .any(|str_agent| str_agent.eq_ignore_ascii_case(str_token))
    }
}
