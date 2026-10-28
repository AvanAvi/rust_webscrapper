//! Holds a parsed robots-style file and answers which rule applies to a path.

use crate::signals::content_signal::ContentSignal;
use crate::signals::pattern_matcher::PatternMatcher;
use crate::signals::robots_finding::RobotsFinding;
use crate::signals::robots_group::RobotsGroup;
use crate::signals::robots_rule::RobotsRule;
use crate::signals::rule_kind::RuleKind;
use crate::signals::signal_context::SignalContext;

/// Path that RFC 9309 section 2.2.2 always allows.
const STR_ROBOTS_PATH: &str = "/robots.txt";

/// Parsed groups of one robots-style file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RobotsTxt {
    /// Groups in file order.
    vec_o_groups: Vec<RobotsGroup>,
    /// Invalid lines and records.
    u_invalid_lines: u32,
    /// Whether input beyond the parsing limit was ignored.
    b_truncated: bool,
}

impl RobotsTxt {
    /// Creates a document from finished groups and counts.
    pub fn new(vec_o_groups: Vec<RobotsGroup>, u_invalid_lines: u32, b_truncated: bool) -> Self {
        Self {
            vec_o_groups,
            u_invalid_lines,
            b_truncated,
        }
    }

    /// Returns an estimate of the heap bytes the parsed groups hold.
    pub fn heap_bytes(&self) -> usize {
        self.vec_o_groups
            .iter()
            .map(|o_group| size_of::<RobotsGroup>() + o_group.heap_bytes())
            .sum()
    }

    /// Returns the groups in file order.
    pub fn groups(&self) -> &[RobotsGroup] {
        &self.vec_o_groups
    }

    /// Returns the robots.txt finding for the context's product token and path.
    pub fn evaluate(&self, o_context: &SignalContext) -> RobotsFinding {
        let (opt_str_agent, vec_o_groups) = self.matching_groups(o_context.product_token());
        let opt_o_rule = match o_context.path() == STR_ROBOTS_PATH {
            true => None,
            false => self.best_rule(&vec_o_groups, o_context.path()).cloned(),
        };
        RobotsFinding {
            b_allowed: opt_o_rule
                .as_ref()
                .is_none_or(|o_rule| o_rule.e_kind == RuleKind::Allow),
            opt_str_matched_agent: opt_str_agent,
            opt_o_matched_rule: opt_o_rule,
            opt_u_crawl_delay_ms: vec_o_groups
                .iter()
                .filter_map(|o_g| o_g.opt_u_crawl_delay_ms)
                .max(),
            opt_o_content_signal: self.merged_signal(&vec_o_groups),
            u_groups: u32::try_from(self.vec_o_groups.len()).unwrap_or(u32::MAX),
            u_invalid_lines: self.u_invalid_lines,
            b_truncated: self.b_truncated,
        }
    }

    /// Returns the matched agent and the deciding rule for a path, with no implicit allows.
    pub fn decide(&self, str_token: &str, str_path: &str) -> (Option<String>, Option<RobotsRule>) {
        let (opt_str_agent, vec_o_groups) = self.matching_groups(str_token);
        (
            opt_str_agent,
            self.best_rule(&vec_o_groups, str_path).cloned(),
        )
    }

    /// Returns the groups for the token, falling back to `*` (RFC 9309 section 2.2.1).
    fn matching_groups(&self, str_token: &str) -> (Option<String>, Vec<&RobotsGroup>) {
        let vec_o_named: Vec<&RobotsGroup> = self
            .vec_o_groups
            .iter()
            .filter(|o_g| o_g.has_agent(str_token))
            .collect();
        if !vec_o_named.is_empty() {
            return (Some(str_token.to_ascii_lowercase()), vec_o_named);
        }
        let vec_o_star: Vec<&RobotsGroup> = self
            .vec_o_groups
            .iter()
            .filter(|o_g| o_g.has_agent("*"))
            .collect();
        let opt_str_agent = (!vec_o_star.is_empty()).then(|| String::from("*"));
        (opt_str_agent, vec_o_star)
    }

    /// Returns the longest matching rule; on equal length an allow rule wins.
    fn best_rule<'g>(
        &self,
        vec_o_groups: &[&'g RobotsGroup],
        str_path: &str,
    ) -> Option<&'g RobotsRule> {
        let o_matcher = PatternMatcher::new();
        vec_o_groups
            .iter()
            .flat_map(|o_group| o_group.vec_o_rules.iter())
            .filter(|o_rule| o_matcher.matches(&o_rule.str_pattern, str_path))
            .max_by_key(|o_rule| (o_rule.str_pattern.len(), o_rule.e_kind == RuleKind::Allow))
    }

    /// Returns the Content-Signal preferences of the groups, merged restrictively.
    fn merged_signal(&self, vec_o_groups: &[&RobotsGroup]) -> Option<ContentSignal> {
        vec_o_groups
            .iter()
            .filter_map(|o_group| o_group.opt_o_content_signal)
            .reduce(|o_left, o_right| o_left.merge(&o_right))
    }
}
