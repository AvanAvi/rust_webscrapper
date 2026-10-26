//! Describes the crawler and URL a signal is evaluated for.

/// Extensions assumed for pages when ai.txt rules name file types.
const VEC_STR_PAGE_EXTENSIONS: [&str; 2] = ["html", "htm"];

/// Facts a parser needs to turn a signal source into a finding for one URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalContext {
    /// Product token from the User-Agent, matched case-insensitively against groups.
    str_product_token: String,
    /// URL path plus `?query` when present, as robots.txt rules compare it.
    str_path: String,
    /// File extensions the fetched page is assumed to have, without the dot.
    vec_str_page_extensions: Vec<String>,
}

impl SignalContext {
    /// Creates a context for a product token and path, assuming HTML pages.
    pub fn new(str_product_token: &str, str_path: &str) -> Self {
        Self {
            str_product_token: str_product_token.to_owned(),
            str_path: str_path.to_owned(),
            vec_str_page_extensions: VEC_STR_PAGE_EXTENSIONS.map(String::from).to_vec(),
        }
    }

    /// Returns a copy that assumes the given page extensions instead.
    pub fn with_page_extensions(&self, vec_str_extensions: &[&str]) -> Self {
        let mut o_context = self.clone();
        o_context.vec_str_page_extensions = vec_str_extensions
            .iter()
            .map(|str_ext| (*str_ext).to_owned())
            .collect();
        o_context
    }

    /// Returns the product token.
    pub fn product_token(&self) -> &str {
        &self.str_product_token
    }

    /// Returns the path (with query) being evaluated.
    pub fn path(&self) -> &str {
        &self.str_path
    }

    /// Returns the assumed page extensions.
    pub fn page_extensions(&self) -> &[String] {
        &self.vec_str_page_extensions
    }
}
