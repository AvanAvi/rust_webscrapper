//! Renders the signal headers of a response in one canonical text form for hashing and archiving.

/// Canonical header text: only the requested header names, lowercased, values trimmed, one
/// `name: value` line each, each line ending in `\n`, sorted by name. The sort is stable:
/// lines that share a name keep their response order, because RFC 9110 section 5.3 makes that
/// order significant (TDMRep takes the first `tdm-reservation`). The order of different names
/// is not significant, so it does not change the text or its hash.
#[derive(Debug, Clone, Copy, Default)]
pub struct CanonicalHeaders;

impl CanonicalHeaders {
    /// Creates the renderer.
    pub fn new() -> Self {
        Self
    }

    /// Returns the canonical text of the headers whose lowercased name is in `vec_str_names`.
    pub fn render(&self, vec_o_headers: &[(String, String)], vec_str_names: &[&str]) -> String {
        let mut vec_o_lines: Vec<(String, String)> = vec_o_headers
            .iter()
            .map(|(str_name, str_value)| {
                (
                    str_name.trim().to_ascii_lowercase(),
                    str_value.trim().to_owned(),
                )
            })
            .filter(|(str_name, _)| vec_str_names.contains(&str_name.as_str()))
            .collect();
        vec_o_lines.sort_by(|o_a, o_b| o_a.0.cmp(&o_b.0));
        vec_o_lines
            .iter()
            .map(|(str_name, str_value)| format!("{str_name}: {str_value}\n"))
            .collect()
    }
}
