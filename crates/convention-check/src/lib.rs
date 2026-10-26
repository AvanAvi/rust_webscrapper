//! Checks Rust sources against the project's prefix, object, and documentation convention.
#![forbid(unsafe_code)]

mod attr_inspector;
mod check_error;
mod convention_app;
mod convention_checker;
mod depth_tracker;
mod enum_collector;
mod file_checker;
mod file_checker_depth;
mod file_checker_visit;
mod parsed_source;
mod pat_ident_collector;
mod prefix_rules;
mod source_scanner;
mod type_classifier;
mod type_scope;
mod violation;
mod violation_kind;

pub use check_error::CheckError;
pub use convention_app::ConventionApp;
pub use convention_checker::ConventionChecker;
pub use depth_tracker::{DepthTracker, U_MAX_LOOP_DEPTH, U_MAX_NESTING_DEPTH};
pub use parsed_source::ParsedSource;
pub use prefix_rules::PrefixRules;
pub use type_classifier::TypeClassifier;
pub use type_scope::TypeScope;
pub use violation::Violation;
pub use violation_kind::ViolationKind;
