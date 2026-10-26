//! Connects the syntax-tree visitor to the file checker's rules.

use syn::visit::{self, Visit};
use syn::{
    Field, ImplItemConst, ImplItemFn, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStatic,
    ItemStruct, ItemTrait, Local, TraitItemConst, TraitItemFn,
};

use crate::file_checker::FileChecker;

impl<'ast> Visit<'ast> for FileChecker<'_> {
    /// Checks a free function, its parameters, and its body.
    fn visit_item_fn(&mut self, o_item: &'ast ItemFn) {
        self.enter(o_item.sig.ident.to_string());
        self.check_free_fn(o_item);
        let b_prev = self.enter_test_if(&o_item.attrs);
        let o_saved = self.push_generics(&o_item.sig.generics);
        self.check_signature(&o_item.sig);
        visit::visit_item_fn(self, o_item);
        self.restore_scope(o_saved);
        self.restore_test(b_prev);
        self.leave();
    }

    /// Checks an impl block with its `Self` type and generics in scope.
    fn visit_item_impl(&mut self, o_item: &'ast ItemImpl) {
        self.enter(self.type_label(&o_item.self_ty));
        let o_saved = self.push_impl_scope(o_item);
        visit::visit_item_impl(self, o_item);
        self.restore_scope(o_saved);
        self.leave();
    }

    /// Checks a method's doc, parameters, and body.
    fn visit_impl_item_fn(&mut self, o_item: &'ast ImplItemFn) {
        self.enter(o_item.sig.ident.to_string());
        self.require_doc(&o_item.attrs, "method");
        let o_saved = self.push_generics(&o_item.sig.generics);
        self.check_signature(&o_item.sig);
        visit::visit_impl_item_fn(self, o_item);
        self.restore_scope(o_saved);
        self.leave();
    }

    /// Checks a trait method's doc and parameters.
    fn visit_trait_item_fn(&mut self, o_item: &'ast TraitItemFn) {
        self.enter(o_item.sig.ident.to_string());
        self.require_doc(&o_item.attrs, "trait method");
        let o_saved = self.push_generics(&o_item.sig.generics);
        self.check_signature(&o_item.sig);
        visit::visit_trait_item_fn(self, o_item);
        self.restore_scope(o_saved);
        self.leave();
    }

    /// Checks a struct's doc, fields, and serialization renames.
    fn visit_item_struct(&mut self, o_item: &'ast ItemStruct) {
        self.enter(o_item.ident.to_string());
        self.require_doc(&o_item.attrs, "struct");
        let o_prev = self.push_derives(&o_item.attrs);
        visit::visit_item_struct(self, o_item);
        self.pop_derives(o_prev);
        self.leave();
    }

    /// Checks an enum's doc and the named fields of its variants.
    fn visit_item_enum(&mut self, o_item: &'ast ItemEnum) {
        self.enter(o_item.ident.to_string());
        self.require_doc(&o_item.attrs, "enum");
        let o_prev = self.push_derives(&o_item.attrs);
        visit::visit_item_enum(self, o_item);
        self.pop_derives(o_prev);
        self.leave();
    }

    /// Checks a trait's doc and its items.
    fn visit_item_trait(&mut self, o_item: &'ast ItemTrait) {
        self.enter(o_item.ident.to_string());
        self.require_doc(&o_item.attrs, "trait");
        visit::visit_item_trait(self, o_item);
        self.leave();
    }

    /// Checks one struct or variant field.
    fn visit_field(&mut self, o_field: &'ast Field) {
        self.check_field(o_field);
        visit::visit_field(self, o_field);
    }

    /// Checks a module-level constant.
    fn visit_item_const(&mut self, o_item: &'ast ItemConst) {
        self.check_const_item(&o_item.attrs, &o_item.ident, &o_item.ty);
        visit::visit_item_const(self, o_item);
    }

    /// Checks an associated constant in an impl block.
    fn visit_impl_item_const(&mut self, o_item: &'ast ImplItemConst) {
        self.check_const_item(&o_item.attrs, &o_item.ident, &o_item.ty);
        visit::visit_impl_item_const(self, o_item);
    }

    /// Checks an associated constant in a trait.
    fn visit_trait_item_const(&mut self, o_item: &'ast TraitItemConst) {
        self.check_const_item(&o_item.attrs, &o_item.ident, &o_item.ty);
        visit::visit_trait_item_const(self, o_item);
    }

    /// Checks a static item like a constant.
    fn visit_item_static(&mut self, o_item: &'ast ItemStatic) {
        self.check_const_item(&o_item.attrs, &o_item.ident, &o_item.ty);
        visit::visit_item_static(self, o_item);
    }

    /// Checks an inline module, treating `#[cfg(test)]` modules as test code.
    fn visit_item_mod(&mut self, o_item: &'ast ItemMod) {
        let b_prev = self.enter_test_if(&o_item.attrs);
        self.enter(o_item.ident.to_string());
        visit::visit_item_mod(self, o_item);
        self.leave();
        self.restore_test(b_prev);
    }

    /// Checks the names bound by a `let` statement.
    fn visit_local(&mut self, o_local: &'ast Local) {
        self.check_pat(&o_local.pat, None);
        visit::visit_local(self, o_local);
    }
}
