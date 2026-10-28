//! Says how many body bytes a request may read and what happens past the limit.

/// What to do when a body is larger than its cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BodyOverflow {
    /// Fail the fetch with `BodyTooLarge` (pages).
    #[default]
    Reject,
    /// Keep the first `cap` bytes and mark the response truncated (signal files, whose parsers
    /// read a prefix anyway; RFC 9309 section 2.5 requires parsing at least 500 KiB).
    Truncate,
}

/// Byte cap for one request plus the overflow behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BodyBudget {
    /// Most bytes kept, counted after decompression.
    pub u_cap: usize,
    /// Behaviour past the cap.
    pub e_overflow: BodyOverflow,
}
