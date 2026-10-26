//! Lists the kinds of convention violations.

/// Category of a single convention violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ViolationKind {
    /// A name has no recognised prefix.
    MissingPrefix,
    /// A name has a prefix that does not match its type.
    WrongPrefix,
    /// A serde or sqlx field lacks an explicit rename.
    MissingRename,
    /// An item that needs a `///` doc comment has none.
    MissingDoc,
    /// A free function exists outside `main` and test code.
    FreeFunction,
    /// A test function name does not start with `test_`.
    TestName,
    /// A file is longer than the line limit.
    FileTooLong,
    /// Control flow is nested deeper than the limit inside a function body.
    NestingTooDeep,
    /// A loop is nested inside another loop in the same function body.
    NestedLoop,
}

impl ViolationKind {
    /// Returns the short stable label used in reports.
    pub fn label(&self) -> &'static str {
        match self {
            Self::MissingPrefix => "missing-prefix",
            Self::WrongPrefix => "wrong-prefix",
            Self::MissingRename => "missing-rename",
            Self::MissingDoc => "missing-doc",
            Self::FreeFunction => "free-function",
            Self::TestName => "test-name",
            Self::FileTooLong => "file-too-long",
            Self::NestingTooDeep => "nesting-too-deep",
            Self::NestedLoop => "nested-loop",
        }
    }
}
