//! Declares the common interface of all signal parsers.

use crate::signal_kind::SignalKind;
use crate::signals::parsed_signal::ParsedSignal;
use crate::signals::signal_context::SignalContext;

/// Turns the raw bytes of one signal source into a typed finding for one URL.
///
/// Implementations accept arbitrary bytes and never panic: malformed input yields a
/// finding that records how much was invalid rather than an error.
pub trait SignalParser {
    /// Returns the kind of signal this parser reads.
    fn kind(&self) -> SignalKind;

    /// Returns the typed finding for the context's URL.
    fn parse(&self, by_input: &[u8], o_context: &SignalContext) -> ParsedSignal;
}
