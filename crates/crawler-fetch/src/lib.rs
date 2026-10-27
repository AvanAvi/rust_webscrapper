//! SSRF-guarded HTTP fetching with strict size and time limits.
#![forbid(unsafe_code)]

mod address_policy;
mod body_reader;
mod client_config;
mod fake_fetcher;
mod fetch_error;
mod fetch_limits;
mod fetch_log;
mod fetch_request;
mod fetch_response;
mod host_lookup;
mod host_scope;
mod http_fetcher;
mod ip_range;
#[cfg(feature = "test-policy")]
mod loopback_test_policy;
mod policy_choice;
mod production_address_policy;
mod redirect_guard;
mod response_head;
mod safe_http_client;
mod scripted_host_lookup;
mod url_safety;
mod vetting_resolver;

pub use address_policy::{AddressPolicy, AddressRejection};
pub use client_config::ClientConfig;
pub use fake_fetcher::FakeFetcher;
pub use fetch_error::FetchError;
pub use fetch_limits::{
    DUR_CONNECT_TIMEOUT, DUR_READ_TIMEOUT, DUR_TOTAL_TIMEOUT, FetchLimits, U_MAX_BODY_BYTES,
    U_MAX_COMPRESSION_RATIO, U_MAX_REDIRECTS, U_RATIO_FLOOR_BYTES,
};
pub use fetch_log::FetchLog;
pub use fetch_request::FetchRequest;
pub use fetch_response::FetchResponse;
pub use host_lookup::{HostLookup, LookupFuture, SystemHostLookup};
pub use host_scope::HostScope;
pub use http_fetcher::{FetchFuture, HttpFetcher};
#[cfg(feature = "test-policy")]
pub use loopback_test_policy::LoopbackTestPolicy;
pub use policy_choice::PolicyChoice;
pub use production_address_policy::ProductionAddressPolicy;
pub use redirect_guard::RedirectGuard;
pub use safe_http_client::SafeHttpClient;
pub use scripted_host_lookup::ScriptedHostLookup;
pub use url_safety::UrlSafety;
