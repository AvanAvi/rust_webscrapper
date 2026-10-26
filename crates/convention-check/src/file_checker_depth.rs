//! Applies the nesting-depth and one-loop-level rules while walking a function body.

use syn::visit::Visit;
use syn::{Expr, ExprIf, LocalInit};

use crate::depth_tracker::{DepthTracker, U_MAX_NESTING_DEPTH};
use crate::file_checker::FileChecker;
use crate::violation_kind::ViolationKind;

impl FileChecker<'_> {
    /// Starts a function body at depth zero; returns the enclosing body's depth to restore.
    pub(crate) fn begin_body(&mut self) -> DepthTracker {
        std::mem::take(&mut self.o_depth)
    }

    /// Restores the depth saved by `begin_body`.
    pub(crate) fn end_body(&mut self, o_saved: DepthTracker) {
        self.o_depth = o_saved;
    }

    /// Opens a nesting construct and reports it when it is too deep.
    pub(crate) fn enter_nesting(&mut self, str_what: &str) {
        if !self.o_depth.enter() {
            return;
        }
        let u_depth = self.o_depth.depth();
        let str_detail =
            format!("`{str_what}` at nesting depth {u_depth}, limit is {U_MAX_NESTING_DEPTH}");
        self.report(ViolationKind::NestingTooDeep, str_detail);
    }

    /// Closes a nesting construct.
    pub(crate) fn leave_nesting(&mut self) {
        self.o_depth.leave();
    }

    /// Opens a loop, reporting excess nesting and loops inside loops.
    pub(crate) fn enter_loop(&mut self, str_what: &str) {
        self.enter_nesting(str_what);
        if self.o_depth.enter_loop() {
            let str_detail =
                format!("`{str_what}` loop inside another loop; move it into a method");
            self.report(ViolationKind::NestedLoop, str_detail);
        }
    }

    /// Closes a loop.
    pub(crate) fn leave_loop(&mut self) {
        self.o_depth.leave_loop();
        self.leave_nesting();
    }

    /// Walks an `if` chain; each `else if` stays at the depth of the first `if`.
    pub(crate) fn walk_if(&mut self, o_expr: &ExprIf) {
        self.visit_expr(&o_expr.cond);
        self.enter_nesting("if");
        self.visit_block(&o_expr.then_branch);
        self.leave_nesting();
        let Some((_, e_else)) = &o_expr.else_branch else {
            return;
        };
        self.walk_else(e_else);
    }

    /// Walks the initializer of a `let`, treating a `let ... else` block as one level.
    pub(crate) fn walk_local_init(&mut self, o_init: &LocalInit) {
        self.visit_expr(&o_init.expr);
        let Some((_, e_diverge)) = &o_init.diverge else {
            return;
        };
        self.enter_nesting("let-else");
        self.visit_expr(e_diverge);
        self.leave_nesting();
    }

    /// Walks an `else` branch: another `if` continues the chain, a block adds a level.
    fn walk_else(&mut self, e_else: &Expr) {
        if let Expr::If(o_elif) = e_else {
            self.walk_if(o_elif);
            return;
        }
        self.enter_nesting("else");
        self.visit_expr(e_else);
        self.leave_nesting();
    }
}
