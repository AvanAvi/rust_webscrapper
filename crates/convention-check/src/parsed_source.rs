//! Holds one parsed source file and facts about where it lives.

use std::fs;
use std::path::{Path, PathBuf};

use crate::check_error::CheckError;

/// A parsed Rust file plus the path facts the rules depend on.
pub struct ParsedSource {
    /// Path used in reports and for path-based rules.
    path_file: PathBuf,
    /// Parsed syntax tree.
    o_ast: syn::File,
    /// Number of lines in the original text.
    u_line_count: usize,
    /// Whether the file lives under a `tests` directory.
    b_is_test_file: bool,
    /// Whether the file is a `main.rs` or `build.rs` entry point.
    b_is_entry_file: bool,
}

impl ParsedSource {
    /// Reads and parses a file from disk.
    pub fn load(path_file: &Path) -> Result<Self, CheckError> {
        let str_text = fs::read_to_string(path_file).map_err(|err_io| CheckError::Io {
            path_file: path_file.to_path_buf(),
            err_source: err_io,
        })?;
        Self::from_text(path_file.to_path_buf(), &str_text)
    }

    /// Parses source text that is reported under the given path.
    pub fn from_text(path_file: PathBuf, str_text: &str) -> Result<Self, CheckError> {
        let o_ast = syn::parse_file(str_text).map_err(|err_syn| CheckError::Parse {
            path_file: path_file.clone(),
            str_message: err_syn.to_string(),
        })?;
        let b_is_test_file = path_file
            .components()
            .any(|e_part| e_part.as_os_str() == "tests");
        let b_is_entry_file = path_file
            .file_name()
            .is_some_and(|str_file| str_file == "main.rs" || str_file == "build.rs");
        let u_line_count = str_text.lines().count();
        Ok(Self {
            path_file,
            o_ast,
            u_line_count,
            b_is_test_file,
            b_is_entry_file,
        })
    }

    /// Returns the report path.
    pub fn path(&self) -> &Path {
        &self.path_file
    }

    /// Returns the parsed syntax tree.
    pub fn ast(&self) -> &syn::File {
        &self.o_ast
    }

    /// Returns the number of lines in the file.
    pub fn line_count(&self) -> usize {
        self.u_line_count
    }

    /// Returns whether the file is test code by location.
    pub fn is_test_file(&self) -> bool {
        self.b_is_test_file
    }

    /// Returns whether the file is a binary or build-script entry point.
    pub fn is_entry_file(&self) -> bool {
        self.b_is_entry_file
    }
}
