//! Records every request the mock site receives, so tests can prove what never arrived.

use std::sync::{Arc, Mutex, PoisonError};

use crate::recorded_request::RecordedRequest;

/// Shared, append-only list of received requests.
#[derive(Debug, Clone, Default)]
pub struct RequestLog {
    /// Requests in arrival order.
    mtx_requests: Arc<Mutex<Vec<RecordedRequest>>>,
}

impl RequestLog {
    /// Creates an empty log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a request.
    pub fn record(&self, str_host: &str, str_path: &str) {
        let o_request = RecordedRequest {
            str_host: str_host.to_owned(),
            str_path: str_path.to_owned(),
        };
        self.mtx_requests
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(o_request);
    }

    /// Returns all recorded requests.
    pub fn all(&self) -> Vec<RecordedRequest> {
        self.mtx_requests
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}
