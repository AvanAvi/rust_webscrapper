//! Counts control-flow nesting and loop nesting inside one function body.

/// Deepest allowed nesting of control-flow constructs inside a function body.
pub const U_MAX_NESTING_DEPTH: usize = 2;

/// Deepest allowed nesting of loops inside a function body.
pub const U_MAX_LOOP_DEPTH: usize = 1;

/// Current nesting and loop depth while walking one function body.
///
/// `if`, `else`, `match`, `for`, `while`, `loop`, closures, and `let ... else` blocks each
/// add one level; `else if` stays at the level of its `if`. Plain, `unsafe`, and `async`
/// blocks add nothing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DepthTracker {
    /// Control-flow constructs currently open.
    u_depth: usize,
    /// Loops currently open.
    u_loop_depth: usize,
}

impl DepthTracker {
    /// Creates a tracker at depth zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens one nesting level; returns whether it exceeds the limit.
    pub fn enter(&mut self) -> bool {
        self.u_depth += 1;
        self.u_depth > U_MAX_NESTING_DEPTH
    }

    /// Closes one nesting level.
    pub fn leave(&mut self) {
        self.u_depth = self.u_depth.saturating_sub(1);
    }

    /// Opens one loop level; returns whether loops are now nested.
    pub fn enter_loop(&mut self) -> bool {
        self.u_loop_depth += 1;
        self.u_loop_depth > U_MAX_LOOP_DEPTH
    }

    /// Closes one loop level.
    pub fn leave_loop(&mut self) {
        self.u_loop_depth = self.u_loop_depth.saturating_sub(1);
    }

    /// Returns the current nesting depth.
    pub fn depth(&self) -> usize {
        self.u_depth
    }
}
