//! Assembles user-agent groups from records (RFC 9309 section 2.2).

use crate::signals::content_signal_parser::ContentSignalParser;
use crate::signals::crawl_delay_parser::CrawlDelayParser;
use crate::signals::robots_group::RobotsGroup;
use crate::signals::robots_lines::RobotsLines;
use crate::signals::robots_record::RobotsRecord;
use crate::signals::robots_rule::RobotsRule;
use crate::signals::robots_txt::RobotsTxt;
use crate::signals::rule_kind::RuleKind;

/// Longest accepted rule pattern, in bytes; longer patterns are counted as invalid.
pub const U_MAX_PATTERN_BYTES: usize = 4096;

/// Groups records: consecutive user-agent lines open a group, and a user-agent line after
/// any member record opens the next one. Unknown records (such as Sitemap) neither open nor
/// close a group. Member records before the first user-agent line are invalid.
#[derive(Debug, Clone, Default)]
pub struct RobotsTxtBuilder {
    /// Finished groups.
    vec_o_groups: Vec<RobotsGroup>,
    /// Group being filled, if any.
    opt_o_current: Option<RobotsGroup>,
    /// Whether the current group already has member records.
    b_in_members: bool,
    /// Invalid lines and records so far.
    u_invalid: u32,
    /// Whether the input was cut at the size limit.
    b_truncated: bool,
}

impl RobotsTxtBuilder {
    /// Starts from the reader's counts.
    pub fn new(o_lines: &RobotsLines) -> Self {
        Self {
            u_invalid: o_lines.u_invalid_lines,
            b_truncated: o_lines.b_truncated,
            ..Self::default()
        }
    }

    /// Returns the document built from all records.
    pub fn build(mut self, o_lines: &RobotsLines) -> RobotsTxt {
        for o_record in &o_lines.vec_o_records {
            self.add(o_record);
        }
        self.close_group();
        RobotsTxt::new(self.vec_o_groups, self.u_invalid, self.b_truncated)
    }

    /// Applies one record.
    fn add(&mut self, o_record: &RobotsRecord) {
        let str_value = o_record.str_value.as_str();
        match o_record.str_key.as_str() {
            "user-agent" => self.add_agent(str_value),
            "allow" => self.add_rule(RuleKind::Allow, str_value),
            "disallow" => self.add_rule(RuleKind::Disallow, str_value),
            "crawl-delay" => self.add_crawl_delay(str_value),
            "content-signal" => self.add_content_signal(str_value),
            _ => {}
        }
    }

    /// Adds a user-agent, opening a new group if the current one already has members.
    fn add_agent(&mut self, str_agent: &str) {
        if self.b_in_members {
            self.close_group();
        }
        let o_group = self.opt_o_current.get_or_insert_with(RobotsGroup::default);
        o_group.vec_str_agents.push(str_agent.to_owned());
    }

    /// Adds an allow or disallow rule; an empty pattern is a member with no rule.
    fn add_rule(&mut self, e_kind: RuleKind, str_pattern: &str) {
        let b_valid_pattern = str_pattern.is_empty()
            || (str_pattern.len() <= U_MAX_PATTERN_BYTES && str_pattern.starts_with(['/', '*']));
        let Some(o_group) = self.member_group(b_valid_pattern) else {
            return;
        };
        if !str_pattern.is_empty() {
            o_group.vec_o_rules.push(RobotsRule {
                e_kind,
                str_pattern: str_pattern.to_owned(),
            });
        }
    }

    /// Records a Crawl-delay, keeping the largest one in the group.
    fn add_crawl_delay(&mut self, str_value: &str) {
        let opt_u_delay = CrawlDelayParser::new().parse(str_value);
        let Some(o_group) = self.member_group(opt_u_delay.is_some()) else {
            return;
        };
        o_group.opt_u_crawl_delay_ms = o_group.opt_u_crawl_delay_ms.max(opt_u_delay);
    }

    /// Records Content-Signal preferences, merging repeated lines restrictively.
    fn add_content_signal(&mut self, str_value: &str) {
        let (o_signal, u_bad_pairs) = ContentSignalParser::new().parse(str_value);
        self.u_invalid = self.u_invalid.saturating_add(u_bad_pairs);
        let Some(o_group) = self.member_group(true) else {
            return;
        };
        let o_merged = o_group
            .opt_o_content_signal
            .map_or(o_signal, |o_old| o_old.merge(&o_signal));
        o_group.opt_o_content_signal = Some(o_merged);
    }

    /// Returns the current group for a member record, counting invalid or orphan records.
    fn member_group(&mut self, b_valid: bool) -> Option<&mut RobotsGroup> {
        if !b_valid || self.opt_o_current.is_none() {
            self.u_invalid = self.u_invalid.saturating_add(1);
            return None;
        }
        self.b_in_members = true;
        self.opt_o_current.as_mut()
    }

    /// Moves the current group into the finished list.
    fn close_group(&mut self) {
        if let Some(o_group) = self.opt_o_current.take() {
            self.vec_o_groups.push(o_group);
        }
        self.b_in_members = false;
    }
}
