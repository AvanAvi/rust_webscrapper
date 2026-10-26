//! Extracts `<meta name content>` pairs from an HTML document.

use scraper::{ElementRef, Html};

/// Largest HTML input parsed, in bytes.
const U_MAX_HTML_BYTES: usize = 5 * 1024 * 1024;
/// Most meta elements collected from one document.
const U_MAX_META_ELEMENTS: usize = 1024;

/// Reads meta elements with a `name` attribute from HTML of any quality.
#[derive(Debug, Clone, Copy, Default)]
pub struct HtmlMetaExtractor;

impl HtmlMetaExtractor {
    /// Creates an extractor.
    pub fn new() -> Self {
        Self
    }

    /// Returns (lowercase name, content) for each named meta element, in document order.
    pub fn extract(&self, by_html: &[u8]) -> Vec<(String, String)> {
        let by_kept = by_html.get(..U_MAX_HTML_BYTES).unwrap_or(by_html);
        let o_document = Html::parse_document(&String::from_utf8_lossy(by_kept));
        let vec_o_elements: Vec<ElementRef<'_>> = o_document
            .root_element()
            .descendants()
            .filter_map(ElementRef::wrap)
            .collect();
        vec_o_elements
            .iter()
            .filter_map(|o_element| self.named_meta(o_element))
            .take(U_MAX_META_ELEMENTS)
            .collect()
    }

    /// Returns the name and content of a meta element, or `None` for anything else.
    fn named_meta(&self, o_element: &ElementRef<'_>) -> Option<(String, String)> {
        if o_element.value().name() != "meta" {
            return None;
        }
        let str_name = o_element.attr("name")?.trim().to_ascii_lowercase();
        let str_content = o_element
            .attr("content")
            .unwrap_or_default()
            .trim()
            .to_owned();
        Some((str_name, str_content))
    }
}
