//! Finds the Rust source files to check under a directory.

use std::fs;
use std::path::{Path, PathBuf};

use crate::check_error::CheckError;

/// Directory names that are never scanned.
const VEC_STR_SKIPPED_DIRS: [&str; 4] = ["target", ".git", "fixtures", "node_modules"];

/// Walks a directory tree and returns `.rs` files in sorted order.
#[derive(Debug, Clone, Copy, Default)]
pub struct SourceScanner;

impl SourceScanner {
    /// Returns every `.rs` file under the root, skipping build and fixture folders.
    pub fn collect_files(&self, path_root: &Path) -> Result<Vec<PathBuf>, CheckError> {
        let mut vec_path_files = Vec::new();
        self.walk_dir(path_root, &mut vec_path_files)?;
        vec_path_files.sort();
        Ok(vec_path_files)
    }

    /// Adds the files of one directory and recurses into its subdirectories.
    fn walk_dir(&self, path_dir: &Path, vec_path_out: &mut Vec<PathBuf>) -> Result<(), CheckError> {
        let o_entries = fs::read_dir(path_dir).map_err(|err_io| self.io_error(path_dir, err_io))?;
        for res_o_entry in o_entries {
            let o_entry = res_o_entry.map_err(|err_io| self.io_error(path_dir, err_io))?;
            self.visit_entry(&o_entry.path(), vec_path_out)?;
        }
        Ok(())
    }

    /// Handles one directory entry: recurse, collect, or ignore.
    fn visit_entry(
        &self,
        path_entry: &Path,
        vec_path_out: &mut Vec<PathBuf>,
    ) -> Result<(), CheckError> {
        if path_entry.is_dir() && !self.is_skipped(path_entry) {
            return self.walk_dir(path_entry, vec_path_out);
        }
        if path_entry
            .extension()
            .is_some_and(|str_ext| str_ext == "rs")
        {
            vec_path_out.push(path_entry.to_path_buf());
        }
        Ok(())
    }

    /// Returns whether a directory must not be scanned.
    fn is_skipped(&self, path_dir: &Path) -> bool {
        path_dir
            .file_name()
            .and_then(|str_name| str_name.to_str())
            .is_some_and(|str_name| VEC_STR_SKIPPED_DIRS.contains(&str_name))
    }

    /// Wraps an I/O error with the path it happened on.
    fn io_error(&self, path_dir: &Path, err_io: std::io::Error) -> CheckError {
        CheckError::Io {
            path_file: path_dir.to_path_buf(),
            err_source: err_io,
        }
    }
}
