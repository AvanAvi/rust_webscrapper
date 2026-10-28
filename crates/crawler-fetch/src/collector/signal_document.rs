//! Holds a parsed signal file so it can be cached and evaluated for many paths.

use crawler_core::{
    AiTxtParser, ParsedSignal, RobotsParser, RobotsTxt, SignalContext, TdmrepRules,
};

use crate::collector::signal_file::SignalFile;

/// Parsed form of one signal file; nothing is kept for absent or unreachable files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalDocument {
    /// No parsed content.
    Unparsed,
    /// robots.txt groups.
    Robots(RobotsTxt),
    /// ai.txt read as robots-style groups.
    AiTxt(RobotsTxt),
    /// tdmrep.json entries.
    Tdmrep(TdmrepRules),
}

impl SignalDocument {
    /// Parses a present file's body.
    pub fn parse(e_file: SignalFile, by_body: &[u8]) -> Self {
        match e_file {
            SignalFile::RobotsTxt => Self::Robots(RobotsParser::new().parse_document(by_body)),
            SignalFile::AiTxt => Self::AiTxt(RobotsParser::new().parse_document(by_body)),
            SignalFile::TdmrepJson => Self::Tdmrep(TdmrepRules::parse(by_body)),
        }
    }

    /// Returns the finding for one URL, or `None` when nothing was parsed.
    pub fn evaluate(&self, o_context: &SignalContext) -> Option<ParsedSignal> {
        match self {
            Self::Unparsed => None,
            Self::Robots(o_doc) => Some(ParsedSignal::RobotsTxt(o_doc.evaluate(o_context))),
            Self::AiTxt(o_doc) => Some(AiTxtParser::new().evaluate_document(o_doc, o_context)),
            Self::Tdmrep(o_rules) => Some(ParsedSignal::Tdmrep(o_rules.evaluate(o_context.path()))),
        }
    }
}
