//! Matches robots.txt-style path patterns (RFC 9309 section 2.2.3).

use crate::percent_normalizer::PercentNormalizer;
use crate::signals::glob_cursor::GlobCursor;

/// Matches patterns where `*` is any run of characters and a final `$` anchors the end.
///
/// Both sides are percent-normalized first, so `%7E` and `~` compare equal while reserved
/// escapes such as `%2F` stay distinct from `/`. Matching is case-sensitive and anchored at
/// the start of the path, as RFC 9309 section 2.2.2 requires.
#[derive(Debug, Clone, Copy, Default)]
pub struct PatternMatcher {
    /// Percent-encoding normalizer shared with URL normalization.
    o_percent: PercentNormalizer,
}

impl PatternMatcher {
    /// Creates a matcher.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether the pattern matches the path.
    pub fn matches(&self, str_pattern: &str, str_path: &str) -> bool {
        let str_pat = self.o_percent.normalize(str_pattern);
        let str_target = self.o_percent.normalize(str_path);
        let Some(str_anchored) = str_pat.strip_suffix('$') else {
            let str_prefix_pat = format!("{str_pat}*");
            return GlobCursor::new().full_match(str_prefix_pat.as_bytes(), str_target.as_bytes());
        };
        GlobCursor::new().full_match(str_anchored.as_bytes(), str_target.as_bytes())
    }
}
