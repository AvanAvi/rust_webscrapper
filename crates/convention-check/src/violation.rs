//! Describes one convention violation found in a file.

use std::fmt;
use std::path::{Path, PathBuf};

use crate::violation_kind::ViolationKind;

/// One rule broken at one place in one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// File containing the violation.
    path_file: PathBuf,
    /// Rule that was broken.
    e_kind: ViolationKind,
    /// Item path where the rule was broken, outermost first.
    str_location: String,
    /// Human-readable explanation.
    str_detail: String,
}

impl Violation {
    /// Creates a violation record.
    pub fn new(
        path_file: PathBuf,
        e_kind: ViolationKind,
        str_location: String,
        str_detail: String,
    ) -> Self {
        Self {
            path_file,
            e_kind,
            str_location,
            str_detail,
        }
    }

    /// Returns the broken rule.
    pub fn kind(&self) -> ViolationKind {
        self.e_kind
    }

    /// Returns the file containing the violation.
    pub fn path(&self) -> &Path {
        &self.path_file
    }

    /// Returns the item path where the rule was broken.
    pub fn location(&self) -> &str {
        &self.str_location
    }

    /// Returns the human-readable explanation.
    pub fn detail(&self) -> &str {
        &self.str_detail
    }
}

impl fmt::Display for Violation {
    /// Formats the violation as `path: [kind] location: detail`.
    fn fmt(&self, o_f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            o_f,
            "{}: [{}] {}: {}",
            self.path_file.display(),
            self.e_kind.label(),
            self.str_location,
            self.str_detail
        )
    }
}
