//! Tracks progress of one wildcard match without recursion.

/// Positions in pattern and text plus the last `*` to backtrack to.
#[derive(Debug, Clone, Copy, Default)]
pub struct GlobCursor {
    /// Next pattern byte.
    u_pat: usize,
    /// Next text byte.
    u_text: usize,
    /// Pattern index of the last `*` and the text index it currently absorbs up to.
    opt_o_star: Option<(usize, usize)>,
}

impl GlobCursor {
    /// Creates a cursor at the start of both inputs.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether the whole pattern matches the whole text; `*` matches any run.
    ///
    /// Greedy matching with a single backtrack point: O(pattern * text) in the worst case.
    pub fn full_match(mut self, by_pat: &[u8], by_text: &[u8]) -> bool {
        while self.u_text < by_text.len() {
            if !self.advance(by_pat, by_text) {
                return false;
            }
        }
        by_pat
            .get(self.u_pat..)
            .is_some_and(|by_rest| by_rest.iter().all(|u_b| *u_b == b'*'))
    }

    /// Consumes one step; returns false when no backtrack point remains.
    fn advance(&mut self, by_pat: &[u8], by_text: &[u8]) -> bool {
        let opt_u_pat_byte = by_pat.get(self.u_pat).copied();
        if opt_u_pat_byte == Some(b'*') {
            self.opt_o_star = Some((self.u_pat, self.u_text));
            self.u_pat += 1;
            return true;
        }
        if opt_u_pat_byte.is_some() && opt_u_pat_byte == by_text.get(self.u_text).copied() {
            self.u_pat += 1;
            self.u_text += 1;
            return true;
        }
        self.backtrack()
    }

    /// Lets the last `*` absorb one more text byte; returns false when there is none.
    fn backtrack(&mut self) -> bool {
        let Some((u_star_pat, u_star_text)) = self.opt_o_star else {
            return false;
        };
        self.u_pat = u_star_pat + 1;
        self.u_text = u_star_text + 1;
        self.opt_o_star = Some((u_star_pat, u_star_text + 1));
        true
    }
}
