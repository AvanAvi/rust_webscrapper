//! Holds a short, size-capped excerpt of a signal source.

use serde::{Deserialize, Serialize};

/// Maximum excerpt size in bytes; longer text is cut at a character boundary.
pub const U_MAX_EXCERPT_BYTES: usize = 2048;

/// Excerpt of an observed signal source, never longer than `U_MAX_EXCERPT_BYTES`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub struct Excerpt(String);

impl Excerpt {
    /// Creates an excerpt, cutting the text to the size cap at a character boundary.
    pub fn new(str_text: &str) -> Self {
        let u_end = str_text.floor_char_boundary(U_MAX_EXCERPT_BYTES);
        Self(str_text[..u_end].to_owned())
    }

    /// Returns the excerpt text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Excerpt {
    /// Creates an excerpt from owned text, applying the size cap.
    fn from(str_text: String) -> Self {
        Self::new(&str_text)
    }
}

impl From<Excerpt> for String {
    /// Returns the excerpt text.
    fn from(o_excerpt: Excerpt) -> Self {
        o_excerpt.0
    }
}
