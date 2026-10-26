//! Holds the state and rules used while walking one parsed file.

use syn::visit::Visit;
use syn::{Attribute, Field, FnArg, Generics, Ident, ItemFn, ItemImpl, Pat, Signature, Type};

use crate::attr_inspector::AttrInspector;
use crate::depth_tracker::DepthTracker;
use crate::parsed_source::ParsedSource;
use crate::pat_ident_collector::PatIdentCollector;
use crate::prefix_rules::{NameIssue, PrefixRules};
use crate::type_scope::TypeScope;
use crate::violation::Violation;
use crate::violation_kind::ViolationKind;

/// Maximum number of lines allowed in one source file.
const U_MAX_FILE_LINES: usize = 300;

/// Visitor state for checking one file.
pub struct FileChecker<'a> {
    /// Prefix rules shared by every file.
    o_rules: &'a PrefixRules,
    /// Attribute helper.
    o_attrs: AttrInspector,
    /// Source being checked.
    o_source: &'a ParsedSource,
    /// Violations found so far.
    vec_o_violations: Vec<Violation>,
    /// Generic and `Self` context of the current item.
    o_scope: TypeScope,
    /// Whether the visitor is inside test-only code.
    b_in_test: bool,
    /// Whether fields of the current item need serde renames.
    b_needs_serde_rename: bool,
    /// Whether fields of the current item need sqlx renames.
    b_needs_sqlx_rename: bool,
    /// Names of enclosing items, outermost first.
    vec_str_path: Vec<String>,
    /// Nesting and loop depth inside the current function body.
    pub(crate) o_depth: DepthTracker,
}

impl<'a> FileChecker<'a> {
    /// Creates a checker for one source file.
    pub fn new(o_rules: &'a PrefixRules, o_source: &'a ParsedSource) -> Self {
        Self {
            o_rules,
            o_attrs: AttrInspector::new(),
            o_source,
            vec_o_violations: Vec::new(),
            o_scope: TypeScope::new(),
            b_in_test: o_source.is_test_file(),
            b_needs_serde_rename: false,
            b_needs_sqlx_rename: false,
            vec_str_path: Vec::new(),
            o_depth: DepthTracker::new(),
        }
    }

    /// Checks the whole file and returns its violations.
    pub fn check(mut self) -> Vec<Violation> {
        let o_source = self.o_source;
        self.visit_file(o_source.ast());
        self.check_length();
        self.vec_o_violations
    }

    /// Pushes an item name onto the location path.
    pub(crate) fn enter(&mut self, str_name: String) {
        self.vec_str_path.push(str_name);
    }

    /// Pops the innermost item name.
    pub(crate) fn leave(&mut self) {
        self.vec_str_path.pop();
    }

    /// Reports a missing doc comment unless inside test code.
    pub(crate) fn require_doc(&mut self, vec_o_attrs: &[Attribute], str_what: &str) {
        if self.b_in_test || self.o_attrs.has_doc(vec_o_attrs) {
            return;
        }
        self.report(
            ViolationKind::MissingDoc,
            format!("{str_what} has no `///` doc comment"),
        );
    }

    /// Checks every typed parameter of a signature.
    pub(crate) fn check_signature(&mut self, o_sig: &Signature) {
        for e_arg in &o_sig.inputs {
            let FnArg::Typed(o_typed) = e_arg else {
                continue;
            };
            self.check_pat(&o_typed.pat, Some(&o_typed.ty));
        }
    }

    /// Checks the names a pattern binds, using the type when it is a single binding.
    pub(crate) fn check_pat(&mut self, e_pat: &Pat, opt_e_ty: Option<&Type>) {
        match (e_pat, opt_e_ty) {
            (Pat::Ident(o_ident), Some(e_ty)) => self.check_name(&o_ident.ident.to_string(), e_ty),
            (Pat::Type(o_typed), _) => self.check_pat(&o_typed.pat, Some(&o_typed.ty)),
            _ => self.check_untyped_names(e_pat),
        }
    }

    /// Checks a constant or static item's doc and name.
    pub(crate) fn check_const_item(
        &mut self,
        vec_o_attrs: &[Attribute],
        o_ident: &Ident,
        e_ty: &Type,
    ) {
        self.require_doc(vec_o_attrs, &format!("constant `{o_ident}`"));
        let opt_issue = self
            .o_rules
            .check_const(&o_ident.to_string(), e_ty, &self.o_scope);
        self.apply(opt_issue);
    }

    /// Checks that a free function is `main`, a test, or test-only code.
    pub(crate) fn check_free_fn(&mut self, o_item: &ItemFn) {
        let str_name = o_item.sig.ident.to_string();
        if self.o_attrs.is_test_fn(&o_item.attrs) {
            self.check_test_name(&str_name);
            return;
        }
        if self.b_in_test
            || self.o_attrs.is_cfg_test(&o_item.attrs)
            || self.is_entry_main(&str_name)
        {
            return;
        }
        let str_detail = format!("free function `{str_name}` must be a method");
        self.report(ViolationKind::FreeFunction, str_detail);
    }

    /// Checks a named field's doc, prefix, and serialization rename.
    pub(crate) fn check_field(&mut self, o_field: &Field) {
        let Some(o_ident) = &o_field.ident else {
            return;
        };
        let str_name = o_ident.to_string();
        self.require_doc(&o_field.attrs, &format!("field `{str_name}`"));
        self.check_name(&str_name, &o_field.ty);
        self.check_renames(&o_field.attrs, &str_name);
    }

    /// Sets rename requirements from an item's derives and returns the previous ones.
    pub(crate) fn push_derives(&mut self, vec_o_attrs: &[Attribute]) -> (bool, bool) {
        let o_prev = (self.b_needs_serde_rename, self.b_needs_sqlx_rename);
        self.b_needs_serde_rename = (self.o_attrs.derives(vec_o_attrs, "Serialize")
            || self.o_attrs.derives(vec_o_attrs, "Deserialize"))
            && !self.o_attrs.is_serde_proxied(vec_o_attrs);
        self.b_needs_sqlx_rename = self.o_attrs.derives(vec_o_attrs, "FromRow");
        o_prev
    }

    /// Restores rename requirements saved by `push_derives`.
    pub(crate) fn pop_derives(&mut self, o_prev: (bool, bool)) {
        (self.b_needs_serde_rename, self.b_needs_sqlx_rename) = o_prev;
    }

    /// Marks the visitor as inside test code when the attributes say so; returns the old flag.
    pub(crate) fn enter_test_if(&mut self, vec_o_attrs: &[Attribute]) -> bool {
        let b_prev = self.b_in_test;
        self.b_in_test = b_prev || self.o_attrs.is_cfg_test(vec_o_attrs);
        b_prev
    }

    /// Restores the test flag saved by `enter_test_if`.
    pub(crate) fn restore_test(&mut self, b_prev: bool) {
        self.b_in_test = b_prev;
    }

    /// Extends the scope with generics and returns the previous scope.
    pub(crate) fn push_generics(&mut self, o_generics: &Generics) -> TypeScope {
        let o_scope = self.o_scope.with_generics(o_generics);
        std::mem::replace(&mut self.o_scope, o_scope)
    }

    /// Enters an impl block's generics and `Self` type; returns the previous scope.
    pub(crate) fn push_impl_scope(&mut self, o_item: &ItemImpl) -> TypeScope {
        let opt_str_self = self
            .o_rules
            .expected_for_type(&o_item.self_ty, &self.o_scope);
        let o_scope = self
            .o_scope
            .with_generics(&o_item.generics)
            .with_self_prefix(opt_str_self);
        std::mem::replace(&mut self.o_scope, o_scope)
    }

    /// Restores a scope saved by `push_generics` or `push_impl_scope`.
    pub(crate) fn restore_scope(&mut self, o_scope: TypeScope) {
        self.o_scope = o_scope;
    }

    /// Returns a short label for an impl's self type.
    pub(crate) fn type_label(&self, e_ty: &Type) -> String {
        let Type::Path(o_path) = e_ty else {
            return String::from("impl");
        };
        o_path
            .path
            .segments
            .last()
            .map_or_else(|| String::from("impl"), |o_seg| o_seg.ident.to_string())
    }

    /// Checks one name against its declared type.
    fn check_name(&mut self, str_name: &str, e_ty: &Type) {
        let opt_issue = self.o_rules.check_typed(str_name, e_ty, &self.o_scope);
        self.apply(opt_issue);
    }

    /// Checks every name bound by a pattern that has no usable type.
    fn check_untyped_names(&mut self, e_pat: &Pat) {
        let vec_str_names = PatIdentCollector::new().collect(e_pat);
        for str_name in vec_str_names {
            let opt_issue = self.o_rules.check_untyped(&str_name);
            self.apply(opt_issue);
        }
    }

    /// Reports serde or sqlx fields that lack an explicit rename.
    fn check_renames(&mut self, vec_o_attrs: &[Attribute], str_name: &str) {
        if self.b_needs_serde_rename && !self.o_attrs.has_serde_rename(vec_o_attrs) {
            let str_detail = format!("serde field `{str_name}` needs #[serde(rename = \"...\")]");
            self.report(ViolationKind::MissingRename, str_detail);
        }
        if self.b_needs_sqlx_rename && !self.o_attrs.has_tool_word(vec_o_attrs, "sqlx", "rename") {
            let str_detail = format!("sqlx field `{str_name}` needs #[sqlx(rename = \"...\")]");
            self.report(ViolationKind::MissingRename, str_detail);
        }
    }

    /// Reports a test function whose name lacks the `test_` prefix.
    fn check_test_name(&mut self, str_name: &str) {
        if str_name.starts_with("test_") {
            return;
        }
        self.report(
            ViolationKind::TestName,
            format!("test `{str_name}` must start with `test_`"),
        );
    }

    /// Returns whether a function is the top-level `main` of an entry file.
    fn is_entry_main(&self, str_name: &str) -> bool {
        str_name == "main" && self.o_source.is_entry_file() && self.vec_str_path.len() == 1
    }

    /// Reports a file that exceeds the line limit.
    fn check_length(&mut self) {
        let u_lines = self.o_source.line_count();
        if u_lines > U_MAX_FILE_LINES {
            let str_detail = format!("{u_lines} lines, limit is {U_MAX_FILE_LINES}");
            self.report(ViolationKind::FileTooLong, str_detail);
        }
    }

    /// Records a naming issue when one was found.
    fn apply(&mut self, opt_issue: Option<NameIssue>) {
        let Some((e_kind, str_detail)) = opt_issue else {
            return;
        };
        self.report(e_kind, str_detail);
    }

    /// Records a violation at the current item path.
    pub(crate) fn report(&mut self, e_kind: ViolationKind, str_detail: String) {
        let str_location = match self.vec_str_path.is_empty() {
            true => String::from("<file>"),
            false => self.vec_str_path.join("::"),
        };
        let path_file = self.o_source.path().to_path_buf();
        self.vec_o_violations
            .push(Violation::new(path_file, e_kind, str_location, str_detail));
    }
}
