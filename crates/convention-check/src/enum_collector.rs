//! Collects the names of all enums declared in a set of files.

use std::collections::HashSet;

use syn::visit::{self, Visit};
use syn::{File, ItemEnum};

/// Visitor that records enum names so their values can require the `e_` prefix.
#[derive(Debug, Clone, Default)]
pub struct EnumCollector {
    /// Enum names seen so far.
    set_str_enums: HashSet<String>,
}

impl EnumCollector {
    /// Creates an empty collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds every enum declared in a file.
    pub fn collect(&mut self, o_file: &File) {
        self.visit_file(o_file);
    }

    /// Returns the collected enum names.
    pub fn into_names(self) -> HashSet<String> {
        self.set_str_enums
    }
}

impl<'ast> Visit<'ast> for EnumCollector {
    /// Records one enum name and continues into nested items.
    fn visit_item_enum(&mut self, o_item: &'ast ItemEnum) {
        self.set_str_enums.insert(o_item.ident.to_string());
        visit::visit_item_enum(self, o_item);
    }
}
