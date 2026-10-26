//! Splits robots-style text into records (RFC 9309 section 2.2).

use crate::signals::robots_lines::RobotsLines;
use crate::signals::robots_record::RobotsRecord;

/// Parsing limit in bytes; RFC 9309 section 2.5 requires at least 500 KiB.
pub const U_MAX_ROBOTS_BYTES: usize = 500 * 1024;

/// Reads records from bytes of any encoding quality: lossy UTF-8, BOM, CR/LF/CRLF, comments.
#[derive(Debug, Clone, Copy, Default)]
pub struct RobotsLineReader;

impl RobotsLineReader {
    /// Creates a reader.
    pub fn new() -> Self {
        Self
    }

    /// Returns the records of the first 500 KiB of input.
    pub fn read(&self, by_input: &[u8]) -> RobotsLines {
        let by_kept = by_input.get(..U_MAX_ROBOTS_BYTES).unwrap_or(by_input);
        let str_text = String::from_utf8_lossy(by_kept);
        let str_body = str_text.strip_prefix('\u{feff}').unwrap_or(&str_text);
        let mut o_lines = RobotsLines {
            b_truncated: by_input.len() > U_MAX_ROBOTS_BYTES,
            ..RobotsLines::default()
        };
        for str_line in str_body.split(['\n', '\r']) {
            self.add_line(str_line, &mut o_lines);
        }
        o_lines
    }

    /// Adds one line as a record, skips it as blank, or counts it as invalid.
    fn add_line(&self, str_line: &str, o_lines: &mut RobotsLines) {
        let str_content = str_line.split('#').next().unwrap_or_default().trim();
        if str_content.is_empty() {
            return;
        }
        let Some((str_key, str_value)) = str_content.split_once(':') else {
            o_lines.u_invalid_lines = o_lines.u_invalid_lines.saturating_add(1);
            return;
        };
        o_lines.vec_o_records.push(RobotsRecord {
            str_key: str_key.trim().to_ascii_lowercase(),
            str_value: str_value.trim().to_owned(),
        });
    }
}
