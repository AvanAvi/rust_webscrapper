//! Checks binding names against the prefix their type requires.

use std::collections::HashSet;

use syn::Type;

use crate::type_classifier::TypeClassifier;
use crate::type_scope::TypeScope;
use crate::violation_kind::ViolationKind;

/// Every prefix the convention recognises.
const VEC_STR_PREFIXES: [&str; 25] = [
    "b_", "i_", "u_", "f_", "ch_", "str_", "by_", "e_", "o_", "vec_", "map_", "set_", "opt_",
    "res_", "err_", "dt_", "dur_", "uuid_", "path_", "url_", "tx_", "rx_", "mtx_", "jh_", "cb_",
];

/// One naming problem: the broken rule and an explanation.
pub type NameIssue = (ViolationKind, String);

/// Applies the Hungarian prefix rules to names.
#[derive(Debug, Clone, Default)]
pub struct PrefixRules {
    /// Classifier that maps types to prefixes.
    o_classifier: TypeClassifier,
}

impl PrefixRules {
    /// Creates rules that know the given workspace enums.
    pub fn new(set_str_enums: HashSet<String>) -> Self {
        Self {
            o_classifier: TypeClassifier::new(set_str_enums),
        }
    }

    /// Returns the prefix a type requires; `None` means any prefix.
    pub fn expected_for_type(&self, e_ty: &Type, o_scope: &TypeScope) -> Option<&'static str> {
        self.o_classifier.expected_for_type(e_ty, o_scope)
    }

    /// Returns the issue with a typed binding name, if any.
    pub fn check_typed(
        &self,
        str_name: &str,
        e_ty: &Type,
        o_scope: &TypeScope,
    ) -> Option<NameIssue> {
        let str_bare = str_name.trim_start_matches('_');
        if str_bare.is_empty() || str_bare == "self" {
            return None;
        }
        let Some(str_expected) = self.expected_for_type(e_ty, o_scope) else {
            return self.check_untyped(str_name);
        };
        if str_bare.starts_with(str_expected) {
            return None;
        }
        Some(self.mismatch(str_name, str_bare, str_expected))
    }

    /// Returns the issue with a binding whose type is not written, if any.
    pub fn check_untyped(&self, str_name: &str) -> Option<NameIssue> {
        let str_bare = str_name.trim_start_matches('_');
        if str_bare.is_empty() || str_bare == "self" || self.has_any_prefix(str_bare) {
            return None;
        }
        Some((
            ViolationKind::MissingPrefix,
            format!("`{str_name}` has no type prefix"),
        ))
    }

    /// Returns the issue with a constant or static name, if any.
    pub fn check_const(
        &self,
        str_name: &str,
        e_ty: &Type,
        o_scope: &TypeScope,
    ) -> Option<NameIssue> {
        let str_lower = str_name.to_lowercase();
        self.check_typed(&str_lower, e_ty, o_scope)
            .map(|(e_kind, str_detail)| (e_kind, str_detail.replace(&str_lower, str_name)))
    }

    /// Returns whether a name starts with any recognised prefix.
    pub fn has_any_prefix(&self, str_name: &str) -> bool {
        VEC_STR_PREFIXES
            .iter()
            .any(|str_prefix| str_name.starts_with(str_prefix))
    }

    /// Builds the issue for a name whose prefix does not match its type.
    fn mismatch(&self, str_name: &str, str_bare: &str, str_expected: &str) -> NameIssue {
        if self.has_any_prefix(str_bare) {
            let str_detail = format!("`{str_name}` should use prefix `{str_expected}`");
            return (ViolationKind::WrongPrefix, str_detail);
        }
        let str_detail = format!("`{str_name}` needs prefix `{str_expected}`");
        (ViolationKind::MissingPrefix, str_detail)
    }
}
