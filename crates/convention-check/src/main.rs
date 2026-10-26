//! Runs the convention checker over a directory tree and exits non-zero on violations.
#![forbid(unsafe_code)]

use std::process::ExitCode;

use convention_check::ConventionApp;

fn main() -> ExitCode {
    ConventionApp::new(std::env::args().collect()).run()
}
