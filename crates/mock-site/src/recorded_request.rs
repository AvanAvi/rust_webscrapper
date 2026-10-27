//! Holds one request the mock site received.

/// Host (without port) and path of a received request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedRequest {
    /// Host header value without the port.
    pub str_host: String,
    /// Request path.
    pub str_path: String,
}
