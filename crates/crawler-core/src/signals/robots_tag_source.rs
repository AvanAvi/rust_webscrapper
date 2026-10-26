//! Lists where page-level robots directives can be published.

/// Publication method of page-level robots directives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RobotsTagSource {
    /// `X-Robots-Tag` HTTP response headers.
    Header,
    /// `<meta name="robots">` or `<meta name="<product token>">` elements.
    Meta,
}
