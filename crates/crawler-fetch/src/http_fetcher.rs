//! Declares the injected HTTP fetching interface.

use std::future::Future;
use std::pin::Pin;

use crate::fetch_error::FetchError;
use crate::fetch_request::FetchRequest;
use crate::fetch_response::FetchResponse;

/// Future returned by a fetch.
pub type FetchFuture<'a> =
    Pin<Box<dyn Future<Output = Result<FetchResponse, FetchError>> + Send + 'a>>;

/// Fetches one URL within the safety limits; implemented by `SafeHttpClient` and `FakeFetcher`.
pub trait HttpFetcher: Send + Sync {
    /// Performs one GET request.
    fn fetch(&self, o_request: FetchRequest) -> FetchFuture<'_>;
}
