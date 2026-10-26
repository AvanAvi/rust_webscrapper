//! Tracks the generic parameters and `Self` type visible while checking an item.

use std::collections::HashMap;

use syn::punctuated::Punctuated;
use syn::{GenericParam, Generics, Token, Type, TypeParamBound, WherePredicate};

/// Trait names whose implementors are closures.
const VEC_STR_CLOSURE_TRAITS: [&str; 3] = ["Fn", "FnMut", "FnOnce"];

/// Generic parameters and `Self` prefix in effect for one item.
#[derive(Debug, Clone, Default)]
pub struct TypeScope {
    /// Prefix per generic parameter name (key); `None` means any prefix.
    map_opt_generics: HashMap<String, Option<&'static str>>,
    /// Prefix required for values of type `Self`, if known.
    opt_str_self_prefix: Option<&'static str>,
}

impl TypeScope {
    /// Creates an empty scope.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy of this scope extended with the given generics.
    pub fn with_generics(&self, o_generics: &Generics) -> Self {
        let mut o_scope = self.clone();
        for e_param in &o_generics.params {
            o_scope.add_param(e_param);
        }
        o_scope.add_where_clause(o_generics);
        o_scope
    }

    /// Returns a copy of this scope with a new `Self` prefix.
    pub fn with_self_prefix(&self, opt_str_prefix: Option<&'static str>) -> Self {
        let mut o_scope = self.clone();
        o_scope.opt_str_self_prefix = opt_str_prefix;
        o_scope
    }

    /// Returns whether a name is a generic parameter in scope.
    pub fn has_generic(&self, str_name: &str) -> bool {
        self.map_opt_generics.contains_key(str_name)
    }

    /// Returns the prefix required for a generic parameter; `None` means any.
    pub fn generic_prefix(&self, str_name: &str) -> Option<&'static str> {
        self.map_opt_generics.get(str_name).copied().flatten()
    }

    /// Returns the prefix required for `Self`, if known.
    pub fn self_prefix(&self) -> Option<&'static str> {
        self.opt_str_self_prefix
    }

    /// Returns whether any bound in the list is a closure trait.
    pub fn has_closure_bound(&self, vec_e_bounds: &Punctuated<TypeParamBound, Token![+]>) -> bool {
        vec_e_bounds
            .iter()
            .filter_map(|e_bound| self.bound_name(e_bound))
            .any(|str_name| VEC_STR_CLOSURE_TRAITS.contains(&str_name.as_str()))
    }

    /// Returns the last path segment of a trait bound, if it is a trait bound.
    pub fn bound_name(&self, e_bound: &TypeParamBound) -> Option<String> {
        let TypeParamBound::Trait(o_trait) = e_bound else {
            return None;
        };
        o_trait
            .path
            .segments
            .last()
            .map(|o_seg| o_seg.ident.to_string())
    }

    /// Records one generic type parameter with its implied prefix.
    fn add_param(&mut self, e_param: &GenericParam) {
        let GenericParam::Type(o_param) = e_param else {
            return;
        };
        let opt_str_prefix = self.has_closure_bound(&o_param.bounds).then_some("cb_");
        self.map_opt_generics
            .insert(o_param.ident.to_string(), opt_str_prefix);
    }

    /// Records closure bounds declared in a where clause.
    fn add_where_clause(&mut self, o_generics: &Generics) {
        let Some(o_where) = &o_generics.where_clause else {
            return;
        };
        for e_pred in &o_where.predicates {
            self.add_predicate(e_pred);
        }
    }

    /// Records one where-clause predicate that bounds a plain generic name.
    fn add_predicate(&mut self, e_pred: &WherePredicate) {
        let WherePredicate::Type(o_pred) = e_pred else {
            return;
        };
        let Type::Path(o_path) = &o_pred.bounded_ty else {
            return;
        };
        let Some(o_ident) = o_path.path.get_ident() else {
            return;
        };
        if self.has_closure_bound(&o_pred.bounds) {
            self.map_opt_generics
                .insert(o_ident.to_string(), Some("cb_"));
        }
    }
}
