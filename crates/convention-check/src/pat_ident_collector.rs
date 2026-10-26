//! Collects every name bound by a pattern.

use syn::visit::{self, Visit};
use syn::{Pat, PatIdent};

/// Visitor that records the identifiers a pattern binds.
#[derive(Debug, Clone, Default)]
pub struct PatIdentCollector {
    /// Names found so far, in source order.
    vec_str_names: Vec<String>,
}

impl PatIdentCollector {
    /// Creates an empty collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns all names bound by the pattern.
    pub fn collect(mut self, e_pat: &Pat) -> Vec<String> {
        self.visit_pat(e_pat);
        self.vec_str_names
    }
}

impl<'ast> Visit<'ast> for PatIdentCollector {
    /// Records one bound name and continues into its sub-pattern.
    fn visit_pat_ident(&mut self, o_ident: &'ast PatIdent) {
        self.vec_str_names.push(o_ident.ident.to_string());
        visit::visit_pat_ident(self, o_ident);
    }
}
