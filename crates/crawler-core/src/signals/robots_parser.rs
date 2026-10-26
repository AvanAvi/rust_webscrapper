//! Parses robots.txt (RFC 9309) including Crawl-delay and Content-Signal records.

use crate::signal_kind::SignalKind;
use crate::signals::parsed_signal::ParsedSignal;
use crate::signals::robots_line_reader::RobotsLineReader;
use crate::signals::robots_txt::RobotsTxt;
use crate::signals::robots_txt_builder::RobotsTxtBuilder;
use crate::signals::signal_context::SignalContext;
use crate::signals::signal_parser::SignalParser;

/// robots.txt parser; the parsed document can be cached and evaluated per URL.
#[derive(Debug, Clone, Copy, Default)]
pub struct RobotsParser {
    /// Line reader with the 500 KiB limit.
    o_reader: RobotsLineReader,
}

impl RobotsParser {
    /// Creates a parser.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the parsed document for caching and repeated evaluation.
    pub fn parse_document(&self, by_input: &[u8]) -> RobotsTxt {
        let o_lines = self.o_reader.read(by_input);
        RobotsTxtBuilder::new(&o_lines).build(&o_lines)
    }
}

impl SignalParser for RobotsParser {
    /// Returns `SignalKind::RobotsTxt`.
    fn kind(&self) -> SignalKind {
        SignalKind::RobotsTxt
    }

    /// Returns the robots.txt finding for the context's URL.
    fn parse(&self, by_input: &[u8], o_context: &SignalContext) -> ParsedSignal {
        ParsedSignal::RobotsTxt(self.parse_document(by_input).evaluate(o_context))
    }
}
