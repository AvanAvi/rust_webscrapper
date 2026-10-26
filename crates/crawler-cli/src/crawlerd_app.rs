//! Dispatches `crawlerd` subcommands.

use std::process::ExitCode;

/// Usage text printed until the subcommands exist.
const STR_USAGE: &str = "usage: crawlerd <server|worker|crawl|verify|export> (not implemented yet)";

/// Top-level `crawlerd` application.
pub struct CrawlerdApp {
    /// Raw process arguments including the program name.
    vec_str_args: Vec<String>,
}

impl CrawlerdApp {
    /// Creates the application from raw process arguments.
    pub fn new(vec_str_args: Vec<String>) -> Self {
        Self { vec_str_args }
    }

    /// Runs the requested subcommand and returns the process exit code.
    pub fn run(&self) -> ExitCode {
        let u_arg_count = self.vec_str_args.len();
        eprintln!("{STR_USAGE} ({u_arg_count} args given)");
        ExitCode::from(2)
    }
}
