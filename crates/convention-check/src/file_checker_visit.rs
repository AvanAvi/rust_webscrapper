//! Connects the syntax-tree visitor to the file checker's rules.

use syn::visit::{self, Visit};
use syn::{
    Arm, ExprClosure, ExprForLoop, ExprIf, ExprLet, ExprLoop, ExprMatch, ExprWhile, Field,
    ImplItemConst, ImplItemFn, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStatic,
    ItemStruct, ItemTrait, Local, TraitItemConst, TraitItemFn,
};

use crate::file_checker::FileChecker;

impl<'ast> Visit<'ast> for FileChecker<'_> {
    /// Checks a free function, its parameters, and its body.
    fn visit_item_fn(&mut self, o_item: &'ast ItemFn) {
        self.enter(o_item.sig.ident.to_string());
        self.check_free_fn(o_item);
        let b_prev = self.enter_test_if(&o_item.attrs);
        let o_body_depth = self.begin_body();
        let o_saved = self.push_generics(&o_item.sig.generics);
        self.check_signature(&o_item.sig);
        visit::visit_item_fn(self, o_item);
        self.end_body(o_body_depth);
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
        let o_body_depth = self.begin_body();
        let o_saved = self.push_generics(&o_item.sig.generics);
        self.check_signature(&o_item.sig);
        visit::visit_impl_item_fn(self, o_item);
        self.end_body(o_body_depth);
        self.restore_scope(o_saved);
        self.leave();
    }

    /// Checks a trait method's doc and parameters.
    fn visit_trait_item_fn(&mut self, o_item: &'ast TraitItemFn) {
        self.enter(o_item.sig.ident.to_string());
        self.require_doc(&o_item.attrs, "trait method");
        let o_body_depth = self.begin_body();
        let o_saved = self.push_generics(&o_item.sig.generics);
        self.check_signature(&o_item.sig);
        visit::visit_trait_item_fn(self, o_item);
        self.end_body(o_body_depth);
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

    /// Checks the names bound by a `let` statement and the depth of a `let ... else`.
    fn visit_local(&mut self, o_local: &'ast Local) {
        self.check_pat(&o_local.pat, None);
        if let Some(o_init) = &o_local.init {
            self.walk_local_init(o_init);
        }
    }

    /// Checks a `for` loop's bindings, nesting, and loop level.
    fn visit_expr_for_loop(&mut self, o_expr: &'ast ExprForLoop) {
        self.check_pat(&o_expr.pat, None);
        self.visit_expr(&o_expr.expr);
        self.enter_loop("for");
        self.visit_block(&o_expr.body);
        self.leave_loop();
    }

    /// Checks a `while` loop's nesting and loop level.
    fn visit_expr_while(&mut self, o_expr: &'ast ExprWhile) {
        self.visit_expr(&o_expr.cond);
        self.enter_loop("while");
        self.visit_block(&o_expr.body);
        self.leave_loop();
    }

    /// Checks a `loop`'s nesting and loop level.
    fn visit_expr_loop(&mut self, o_expr: &'ast ExprLoop) {
        self.enter_loop("loop");
        self.visit_block(&o_expr.body);
        self.leave_loop();
    }

    /// Checks an `if` chain's nesting.
    fn visit_expr_if(&mut self, o_expr: &'ast ExprIf) {
        self.walk_if(o_expr);
    }

    /// Checks a `match`'s nesting and its arms.
    fn visit_expr_match(&mut self, o_expr: &'ast ExprMatch) {
        self.visit_expr(&o_expr.expr);
        self.enter_nesting("match");
        for o_arm in &o_expr.arms {
            self.visit_arm(o_arm);
        }
        self.leave_nesting();
    }

    /// Checks closure parameter names, using the type when one is written.
    fn visit_expr_closure(&mut self, o_expr: &'ast ExprClosure) {
        for e_pat in &o_expr.inputs {
            self.check_pat(e_pat, None);
        }
        self.enter_nesting("closure");
        self.visit_expr(&o_expr.body);
        self.leave_nesting();
    }

    /// Checks the names bound by a `match` arm pattern.
    fn visit_arm(&mut self, o_arm: &'ast Arm) {
        self.check_pat(&o_arm.pat, None);
        visit::visit_arm(self, o_arm);
    }

    /// Checks the names bound by `if let`, `while let`, and let chains.
    fn visit_expr_let(&mut self, o_expr: &'ast ExprLet) {
        self.check_pat(&o_expr.pat, None);
        visit::visit_expr_let(self, o_expr);
    }
}
