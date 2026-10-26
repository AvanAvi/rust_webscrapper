//! Runs every convention rule over a set of source files.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::check_error::CheckError;
use crate::enum_collector::EnumCollector;
use crate::file_checker::FileChecker;
use crate::parsed_source::ParsedSource;
use crate::prefix_rules::PrefixRules;
use crate::source_scanner::SourceScanner;
use crate::violation::Violation;

/// Entry point that finds, parses, and checks source files.
#[derive(Debug, Clone, Copy, Default)]
pub struct ConventionChecker {
    /// Finds the files to check.
    o_scanner: SourceScanner,
}

impl ConventionChecker {
    /// Creates a checker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns every Rust file under a root, skipping build output and fixtures.
    pub fn collect_files(&self, path_root: &Path) -> Result<Vec<PathBuf>, CheckError> {
        self.o_scanner.collect_files(path_root)
    }

    /// Loads and checks the given files together.
    pub fn check_files(&self, vec_path_files: &[PathBuf]) -> Result<Vec<Violation>, CheckError> {
        let vec_o_sources = vec_path_files
            .iter()
            .map(|path_file| ParsedSource::load(path_file))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(self.check_sources(&vec_o_sources))
    }

    /// Checks already parsed sources, sharing the enum names they declare.
    pub fn check_sources(&self, vec_o_sources: &[ParsedSource]) -> Vec<Violation> {
        let o_rules = PrefixRules::new(self.collect_enums(vec_o_sources));
        vec_o_sources
            .iter()
            .flat_map(|o_source| FileChecker::new(&o_rules, o_source).check())
            .collect()
    }

    /// Returns the names of all enums declared in the sources.
    fn collect_enums(&self, vec_o_sources: &[ParsedSource]) -> HashSet<String> {
        let mut o_collector = EnumCollector::new();
        for o_source in vec_o_sources {
            o_collector.collect(o_source.ast());
        }
        o_collector.into_names()
    }
}
