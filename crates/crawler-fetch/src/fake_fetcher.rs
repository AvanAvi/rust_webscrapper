//! Returns canned responses for unit tests of code that fetches.

use std::collections::HashMap;

use crate::fetch_error::FetchError;
use crate::fetch_request::FetchRequest;
use crate::fetch_response::FetchResponse;
use crate::http_fetcher::{FetchFuture, HttpFetcher};

/// Fetcher answering from a map of URL (key) to canned result; never touches the network.
#[derive(Debug, Clone, Default)]
pub struct FakeFetcher {
    /// Canned results per URL string (key).
    map_res_canned: HashMap<String, Result<FetchResponse, FetchError>>,
}

impl FakeFetcher {
    /// Creates a fetcher with no canned results.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the fetcher with a canned result for a URL.
    pub fn with(mut self, str_url: &str, res_o_canned: Result<FetchResponse, FetchError>) -> Self {
        self.map_res_canned.insert(str_url.to_owned(), res_o_canned);
        self
    }
}

impl HttpFetcher for FakeFetcher {
    /// Returns the canned result, or a connect error for unknown URLs.
    fn fetch(&self, o_request: FetchRequest) -> FetchFuture<'_> {
        let res_o_result = self
            .map_res_canned
            .get(o_request.url_target.as_str())
            .cloned()
            .unwrap_or_else(|| {
                Err(FetchError::Connect {
                    str_reason: String::from("no canned response"),
                })
            });
        Box::pin(async move { res_o_result })
    }
}
