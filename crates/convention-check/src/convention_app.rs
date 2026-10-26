//! Command-line front end of the convention checker.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::check_error::CheckError;
use crate::convention_checker::ConventionChecker;
use crate::violation::Violation;

/// Root scanned when no path argument is given.
const STR_DEFAULT_ROOT: &str = ".";

/// Parses arguments, runs the checker, and prints a report.
#[derive(Debug, Clone)]
pub struct ConventionApp {
    /// Raw process arguments including the program name.
    vec_str_args: Vec<String>,
}

impl ConventionApp {
    /// Creates the application from raw process arguments.
    pub fn new(vec_str_args: Vec<String>) -> Self {
        Self { vec_str_args }
    }

    /// Runs the check and returns 0 when clean, 1 on violations, 2 on errors.
    pub fn run(&self) -> ExitCode {
        let path_root = self.root_path();
        match self.check(&path_root) {
            Ok(0) => ExitCode::SUCCESS,
            Ok(_) => ExitCode::FAILURE,
            Err(err_check) => self.fail(&err_check),
        }
    }

    /// Returns the directory to scan from the first argument or the default.
    fn root_path(&self) -> PathBuf {
        PathBuf::from(
            self.vec_str_args
                .get(1)
                .map_or(STR_DEFAULT_ROOT, String::as_str),
        )
    }

    /// Checks the tree, prints the report, and returns the violation count.
    fn check(&self, path_root: &Path) -> Result<usize, CheckError> {
        let o_checker = ConventionChecker::new();
        let vec_path_files = o_checker.collect_files(path_root)?;
        let vec_o_violations = o_checker.check_files(&vec_path_files)?;
        self.print_report(vec_path_files.len(), &vec_o_violations);
        Ok(vec_o_violations.len())
    }

    /// Prints each violation and a one-line summary.
    fn print_report(&self, u_file_count: usize, vec_o_violations: &[Violation]) {
        for o_violation in vec_o_violations {
            println!("{o_violation}");
        }
        let u_violation_count = vec_o_violations.len();
        println!(
            "convention-check: {u_file_count} files, {u_violation_count} violations, 0 waivers"
        );
    }

    /// Prints an error and returns the error exit code.
    fn fail(&self, err_check: &CheckError) -> ExitCode {
        eprintln!("convention-check: error: {err_check}");
        ExitCode::from(2)
    }
}
