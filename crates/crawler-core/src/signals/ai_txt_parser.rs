//! Parses ai.txt with robots.txt-compatible syntax.

use crate::signal_kind::SignalKind;
use crate::signals::ai_txt_finding::AiTxtFinding;
use crate::signals::parsed_signal::ParsedSignal;
use crate::signals::robots_parser::RobotsParser;
use crate::signals::robots_rule::RobotsRule;
use crate::signals::robots_txt::RobotsTxt;
use crate::signals::rule_kind::RuleKind;
use crate::signals::signal_context::SignalContext;
use crate::signals::signal_parser::SignalParser;

/// ai.txt parser (Spawning). There is no published grammar, so the file is read as
/// robots.txt-style groups and rules. If the URL path matches no rule, the path is retried
/// with each assumed page extension appended, so `Disallow: *.html` covers `/about`.
#[derive(Debug, Clone, Copy, Default)]
pub struct AiTxtParser {
    /// Shared robots-style reader and grouping.
    o_robots: RobotsParser,
}

impl AiTxtParser {
    /// Creates a parser.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the rule decided by an assumed page extension, and that extension.
    fn decide_by_extension(
        &self,
        o_doc: &RobotsTxt,
        o_context: &SignalContext,
    ) -> Option<(RobotsRule, String)> {
        let str_base = o_context.path().split('?').next().unwrap_or_default();
        o_context.page_extensions().iter().find_map(|str_ext| {
            let (_, opt_o_rule) =
                o_doc.decide(o_context.product_token(), &format!("{str_base}.{str_ext}"));
            opt_o_rule.map(|o_rule| (o_rule, str_ext.clone()))
        })
    }
}

impl SignalParser for AiTxtParser {
    /// Returns `SignalKind::AiTxt`.
    fn kind(&self) -> SignalKind {
        SignalKind::AiTxt
    }

    /// Returns the ai.txt finding for the context's URL.
    fn parse(&self, by_input: &[u8], o_context: &SignalContext) -> ParsedSignal {
        let o_doc = self.o_robots.parse_document(by_input);
        let (opt_str_agent, opt_o_path_rule) =
            o_doc.decide(o_context.product_token(), o_context.path());
        let (opt_o_rule, opt_str_ext) = match opt_o_path_rule {
            Some(o_rule) => (Some(o_rule), None),
            None => self
                .decide_by_extension(&o_doc, o_context)
                .map_or((None, None), |(o_r, str_e)| (Some(o_r), Some(str_e))),
        };
        let o_report = o_doc.evaluate(o_context);
        ParsedSignal::AiTxt(AiTxtFinding {
            b_reserved: opt_o_rule
                .as_ref()
                .is_some_and(|o_rule| o_rule.e_kind == RuleKind::Disallow),
            opt_str_matched_agent: opt_str_agent,
            opt_o_matched_rule: opt_o_rule,
            opt_str_assumed_extension: opt_str_ext,
            u_invalid_lines: o_report.u_invalid_lines,
            b_truncated: o_report.b_truncated,
        })
    }
}
