//! Parses the value of a Content-Signal line.

use crate::signals::content_signal::ContentSignal;

/// Reads `key=yes|no` pairs; unknown keys and values are counted as invalid and ignored.
#[derive(Debug, Clone, Copy, Default)]
pub struct ContentSignalParser;

impl ContentSignalParser {
    /// Creates a parser.
    pub fn new() -> Self {
        Self
    }

    /// Returns the preferences in a line value and how many pairs were invalid.
    pub fn parse(&self, str_value: &str) -> (ContentSignal, u32) {
        let mut o_signal = ContentSignal::default();
        let mut u_invalid = 0u32;
        for str_pair in str_value
            .split(',')
            .filter(|str_p| !str_p.trim().is_empty())
        {
            if !self.apply_pair(str_pair, &mut o_signal) {
                u_invalid = u_invalid.saturating_add(1);
            }
        }
        (o_signal, u_invalid)
    }

    /// Applies one `key=value` pair; returns false when it is not understood.
    fn apply_pair(&self, str_pair: &str, o_signal: &mut ContentSignal) -> bool {
        let Some((str_key, str_flag)) = str_pair.split_once('=') else {
            return false;
        };
        let opt_b_value = match str_flag.trim().to_ascii_lowercase().as_str() {
            "yes" => Some(true),
            "no" => Some(false),
            _ => return false,
        };
        let o_single = match str_key.trim().to_ascii_lowercase().as_str() {
            "search" => ContentSignal {
                opt_b_search: opt_b_value,
                ..ContentSignal::default()
            },
            "ai-input" => ContentSignal {
                opt_b_ai_input: opt_b_value,
                ..ContentSignal::default()
            },
            "ai-train" => ContentSignal {
                opt_b_ai_train: opt_b_value,
                ..ContentSignal::default()
            },
            _ => return false,
        };
        *o_signal = o_signal.merge(&o_single);
        true
    }
}
