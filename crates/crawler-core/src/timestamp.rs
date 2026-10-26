//! Holds a UTC instant with exactly one textual form.

use std::fmt;

use serde::{Deserialize, Serialize};
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;
use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};

use crate::core_error::CoreError;

/// The only accepted form: RFC 3339, UTC designator `Z`, exactly six fractional digits.
const VEC_E_FORMAT: &[BorrowedFormatItem<'static>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:6]Z");

/// UTC instant truncated to whole microseconds, so each value has one encoding.
///
/// Microseconds match Postgres `timestamptz` precision, so values survive a database
/// round trip unchanged. Non-UTC offsets, negative years, and any other text form are
/// rejected rather than converted.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Timestamp {
    /// Instant in UTC, truncated to microseconds.
    dt_value: OffsetDateTime,
    /// Canonical text of the instant.
    str_text: String,
}

impl Timestamp {
    /// Creates a timestamp from a UTC date-time, truncating to whole microseconds.
    pub fn from_datetime(dt_value: OffsetDateTime) -> Result<Self, CoreError> {
        if dt_value.offset() != UtcOffset::UTC {
            return Err(Self::invalid("offset is not UTC"));
        }
        if dt_value.year() < 0 {
            return Err(Self::invalid("year is before 0000"));
        }
        let u_micros_as_nanos = dt_value.nanosecond() / 1_000 * 1_000;
        let dt_truncated = dt_value
            .replace_nanosecond(u_micros_as_nanos)
            .map_err(|err_range| Self::invalid(&err_range.to_string()))?;
        let str_text = dt_truncated
            .format(VEC_E_FORMAT)
            .map_err(|err_format| Self::invalid(&err_format.to_string()))?;
        Ok(Self {
            dt_value: dt_truncated,
            str_text,
        })
    }

    /// Parses the one accepted form; other offsets or digit counts are rejected.
    pub fn parse(str_text: &str) -> Result<Self, CoreError> {
        let o_parsed = PrimitiveDateTime::parse(str_text, VEC_E_FORMAT)
            .map_err(|err_parse| Self::invalid(&err_parse.to_string()))?;
        Self::from_datetime(o_parsed.assume_utc())
    }

    /// Returns the instant.
    pub fn as_datetime(&self) -> OffsetDateTime {
        self.dt_value
    }

    /// Returns the canonical text.
    pub fn as_str(&self) -> &str {
        &self.str_text
    }

    /// Builds the error for an unacceptable value.
    fn invalid(str_reason: &str) -> CoreError {
        CoreError::InvalidTimestamp {
            str_reason: str_reason.to_owned(),
        }
    }
}

impl fmt::Display for Timestamp {
    /// Writes the canonical text.
    fn fmt(&self, o_f: &mut fmt::Formatter<'_>) -> fmt::Result {
        o_f.write_str(&self.str_text)
    }
}

impl TryFrom<String> for Timestamp {
    /// Error for text that is not in the accepted form.
    type Error = CoreError;

    /// Parses canonical text.
    fn try_from(str_text: String) -> Result<Self, Self::Error> {
        Self::parse(&str_text)
    }
}

impl From<Timestamp> for String {
    /// Returns the canonical text.
    fn from(dt_value: Timestamp) -> Self {
        dt_value.str_text
    }
}
