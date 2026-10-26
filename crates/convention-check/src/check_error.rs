//! Defines the single error type of the convention checker.

use std::path::PathBuf;

/// Errors raised while reading or parsing source files.
#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    /// Reading a file or directory failed.
    #[error("cannot read {path_file:?}: {err_source}")]
    Io {
        /// Path that could not be read.
        path_file: PathBuf,
        /// Underlying I/O error.
        #[source]
        err_source: std::io::Error,
    },
    /// A file is not valid Rust syntax.
    #[error("cannot parse {path_file:?}: {str_message}")]
    Parse {
        /// Path that failed to parse.
        path_file: PathBuf,
        /// Message reported by the parser.
        str_message: String,
    },
}
