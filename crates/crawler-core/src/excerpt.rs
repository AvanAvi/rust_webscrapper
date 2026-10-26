//! Holds a short, size-capped excerpt of a signal source and records whether it was cut.

use serde::{Deserialize, Serialize};

use crate::core_error::CoreError;
use crate::excerpt_fields::ExcerptFields;

/// Maximum excerpt size in bytes; longer text is cut at a character boundary.
pub const U_MAX_EXCERPT_BYTES: usize = 2048;

/// Bytes a cut may fall short of the cap, because a UTF-8 character is at most 4 bytes.
const U_MAX_CUT_SLACK: usize = 3;

/// Excerpt of an observed signal source, with its original size so partial excerpts show.
///
/// Encodes as `{"original_bytes":N,"text":"...","truncated":bool}`. Decoding rejects
/// combinations that `Excerpt::new` could not have produced, so each excerpt has one encoding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ExcerptFields")]
pub struct Excerpt {
    /// Excerpt text, at most `U_MAX_EXCERPT_BYTES` long.
    #[serde(rename = "text")]
    str_text: String,
    /// Whether the text was cut from a longer original.
    #[serde(rename = "truncated")]
    b_truncated: bool,
    /// Size of the original text in bytes.
    #[serde(rename = "original_bytes")]
    u_original_bytes: u64,
}

impl Excerpt {
    /// Creates an excerpt, cutting the text to the size cap at a character boundary.
    pub fn new(str_source: &str) -> Self {
        let u_end = str_source.floor_char_boundary(U_MAX_EXCERPT_BYTES);
        Self {
            str_text: str_source[..u_end].to_owned(),
            b_truncated: u_end < str_source.len(),
            u_original_bytes: str_source.len() as u64,
        }
    }

    /// Returns the excerpt text.
    pub fn as_str(&self) -> &str {
        &self.str_text
    }

    /// Returns whether the text is a cut-down prefix of the original.
    pub fn is_truncated(&self) -> bool {
        self.b_truncated
    }

    /// Returns the size of the original text in bytes.
    pub fn original_bytes(&self) -> u64 {
        self.u_original_bytes
    }

    /// Checks that decoded fields are exactly what `new` would produce for some original.
    fn validate(o_fields: &ExcerptFields) -> Result<(), CoreError> {
        let u_len = o_fields.str_text.len();
        let u_cap = U_MAX_EXCERPT_BYTES as u64;
        if o_fields.b_truncated != (o_fields.u_original_bytes > u_cap) {
            return Err(Self::invalid(
                "truncated flag does not match original_bytes",
            ));
        }
        if !o_fields.b_truncated && u_len as u64 != o_fields.u_original_bytes {
            return Err(Self::invalid(
                "untruncated text length differs from original_bytes",
            ));
        }
        let b_cut_at_cap =
            (U_MAX_EXCERPT_BYTES - U_MAX_CUT_SLACK..=U_MAX_EXCERPT_BYTES).contains(&u_len);
        if o_fields.b_truncated && !b_cut_at_cap {
            return Err(Self::invalid("truncated text is not cut at the size cap"));
        }
        Ok(())
    }

    /// Builds the error for an inconsistent excerpt.
    fn invalid(str_reason: &str) -> CoreError {
        CoreError::InvalidExcerpt {
            str_reason: str_reason.to_owned(),
        }
    }
}

impl TryFrom<ExcerptFields> for Excerpt {
    /// Error for inconsistent fields.
    type Error = CoreError;

    /// Validates decoded fields and builds the excerpt.
    fn try_from(o_fields: ExcerptFields) -> Result<Self, Self::Error> {
        Self::validate(&o_fields)?;
        Ok(Self {
            str_text: o_fields.str_text,
            b_truncated: o_fields.b_truncated,
            u_original_bytes: o_fields.u_original_bytes,
        })
    }
}
