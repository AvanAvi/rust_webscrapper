//! Parses X-Robots-Tag headers and robots meta elements.

use std::collections::BTreeSet;

use crate::signal_kind::SignalKind;
use crate::signals::header_block::HeaderBlock;
use crate::signals::html_meta_extractor::HtmlMetaExtractor;
use crate::signals::parsed_signal::ParsedSignal;
use crate::signals::robots_tag_finding::RobotsTagFinding;
use crate::signals::robots_tag_source::RobotsTagSource;
use crate::signals::signal_context::SignalContext;
use crate::signals::signal_parser::SignalParser;

/// Directives whose own syntax contains a colon, so `name:` is not a user-agent scope.
const VEC_STR_VALUED_DIRECTIVES: [&str; 4] = [
    "max-snippet",
    "max-image-preview",
    "max-video-preview",
    "unavailable_after",
];
/// Most directives kept.
const U_MAX_DIRECTIVES: usize = 256;
/// Longest directive kept, in bytes.
const U_MAX_DIRECTIVE_BYTES: usize = 128;

/// Page-level robots directive parser. Unscoped header directives apply to every crawler; a
/// `bot:` segment applies when `bot` is `*` or equals the product token (ASCII case
/// ignored). Meta elements apply when named `robots` or the product token.
#[derive(Debug, Clone, Copy)]
pub struct RobotsTagParser {
    /// Publication method this parser reads.
    e_source: RobotsTagSource,
}

impl RobotsTagParser {
    /// Creates a parser for one publication method.
    pub fn new(e_source: RobotsTagSource) -> Self {
        Self { e_source }
    }

    /// Returns the directive lists from X-Robots-Tag headers that apply to the token.
    fn header_lists(&self, by_input: &[u8], str_token: &str) -> Vec<String> {
        let o_headers = HeaderBlock::parse(by_input);
        o_headers
            .values("x-robots-tag")
            .into_iter()
            .flat_map(|str_value| self.scoped_directives(str_value, str_token))
            .collect()
    }

    /// Returns the directive lists from meta elements that apply to the token.
    fn meta_lists(&self, by_input: &[u8], str_token: &str) -> Vec<String> {
        let str_lower_token = str_token.to_ascii_lowercase();
        HtmlMetaExtractor::new()
            .extract(by_input)
            .into_iter()
            .filter(|o_meta| o_meta.0 == "robots" || o_meta.0 == str_lower_token)
            .map(|o_meta| o_meta.1)
            .collect()
    }

    /// Returns the directives of one header value that apply to the token.
    ///
    /// Directives before any `bot:` prefix apply to every crawler. A `bot:` prefix scopes the
    /// directives after it, up to the next prefix, so `googlebot: noindex, proofcrawl: noai`
    /// gives `noai` to `proofcrawl` only.
    fn scoped_directives(&self, str_value: &str, str_token: &str) -> Vec<String> {
        let mut b_applies = true;
        let mut vec_str_out = Vec::new();
        for str_part in str_value.split(',') {
            let str_directive = match self.scope_prefix(str_part) {
                Some((str_agent, str_rest)) => {
                    b_applies = str_agent == "*" || str_agent.eq_ignore_ascii_case(str_token);
                    str_rest
                }
                None => str_part,
            };
            if b_applies {
                vec_str_out.push(str_directive.to_owned());
            }
        }
        vec_str_out
    }

    /// Splits `bot: rest` into the agent and the rest when the part starts with a user-agent
    /// name; directives whose own syntax uses a colon (such as `max-snippet: 20`) are not agents.
    fn scope_prefix<'p>(&self, str_part: &'p str) -> Option<(&'p str, &'p str)> {
        let (str_head, str_rest) = str_part.split_once(':')?;
        let str_name = str_head.trim();
        let b_agent_like = !str_name.is_empty()
            && str_name.bytes().all(|u_byte| {
                u_byte.is_ascii_alphanumeric() || matches!(u_byte, b'_' | b'-' | b'*')
            });
        let b_valued = VEC_STR_VALUED_DIRECTIVES.contains(&str_name.to_ascii_lowercase().as_str());
        (b_agent_like && !b_valued).then_some((str_name, str_rest))
    }

    /// Builds the finding from comma-separated directive lists.
    fn finding_from(&self, vec_str_lists: &[String]) -> RobotsTagFinding {
        let set_str_directives: BTreeSet<String> = vec_str_lists
            .iter()
            .flat_map(|str_list| str_list.split(','))
            .map(|str_d| str_d.trim().to_ascii_lowercase())
            .filter(|str_d| !str_d.is_empty() && str_d.len() <= U_MAX_DIRECTIVE_BYTES)
            .collect();
        let vec_str_directives: Vec<String> = set_str_directives
            .into_iter()
            .take(U_MAX_DIRECTIVES)
            .collect();
        RobotsTagFinding {
            b_noai: vec_str_directives.iter().any(|str_d| str_d == "noai"),
            b_noimageai: vec_str_directives.iter().any(|str_d| str_d == "noimageai"),
            vec_str_directives,
        }
    }
}

impl SignalParser for RobotsTagParser {
    /// Returns the signal kind for this parser's publication method.
    fn kind(&self) -> SignalKind {
        match self.e_source {
            RobotsTagSource::Header => SignalKind::XRobotsTag,
            RobotsTagSource::Meta => SignalKind::MetaRobots,
        }
    }

    /// Returns the robots directive finding for the context's crawler.
    fn parse(&self, by_input: &[u8], o_context: &SignalContext) -> ParsedSignal {
        let vec_str_lists = match self.e_source {
            RobotsTagSource::Header => self.header_lists(by_input, o_context.product_token()),
            RobotsTagSource::Meta => self.meta_lists(by_input, o_context.product_token()),
        };
        ParsedSignal::RobotsTag(self.finding_from(&vec_str_lists))
    }
}
