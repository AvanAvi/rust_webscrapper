//! Runs the `crawlerd` binary.
#![forbid(unsafe_code)]

mod crawlerd_app;

use std::process::ExitCode;

use crawlerd_app::CrawlerdApp;

fn main() -> ExitCode {
    CrawlerdApp::new(std::env::args().collect()).run()
}
