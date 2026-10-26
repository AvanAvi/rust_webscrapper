//! Holds the records read from a robots-style file plus counts of what was skipped.

use crate::signals::robots_record::RobotsRecord;

/// Records in file order, invalid-line count, and whether the input was cut at the size limit.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RobotsLines {
    /// Valid records in order.
    pub vec_o_records: Vec<RobotsRecord>,
    /// Non-blank lines that were not `key: value` records.
    pub u_invalid_lines: u32,
    /// Whether input beyond the parsing limit was ignored.
    pub b_truncated: bool,
}
