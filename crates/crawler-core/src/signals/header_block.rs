//! Splits a raw HTTP header block into name and value pairs.

/// Largest header block read, in bytes.
const U_MAX_HEADER_BYTES: usize = 65_536;

/// Header fields from a `name: value` block, names lowercased, order kept.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeaderBlock {
    /// Header fields as (lowercase name, trimmed value).
    vec_o_fields: Vec<(String, String)>,
}

impl HeaderBlock {
    /// Parses a header block; lines without a colon are skipped.
    pub fn parse(by_input: &[u8]) -> Self {
        let by_kept = by_input.get(..U_MAX_HEADER_BYTES).unwrap_or(by_input);
        let str_text = String::from_utf8_lossy(by_kept);
        let vec_o_fields = str_text
            .split('\n')
            .filter_map(|str_line| str_line.split_once(':'))
            .map(|(str_name, str_value)| {
                (
                    str_name.trim().to_ascii_lowercase(),
                    str_value.trim().to_owned(),
                )
            })
            .collect();
        Self { vec_o_fields }
    }

    /// Returns every value of a header, in order.
    pub fn values(&self, str_name: &str) -> Vec<&str> {
        self.vec_o_fields
            .iter()
            .filter(|o_field| o_field.0 == str_name)
            .map(|o_field| o_field.1.as_str())
            .collect()
    }
}
