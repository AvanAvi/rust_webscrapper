//! Local multi-host mock website used only by tests.
#![forbid(unsafe_code)]

mod mock_site;
mod raw_server;
mod recorded_request;
mod request_log;
mod scenario_bodies;
mod scenario_router;
mod signal_scenarios;

pub use mock_site::MockSite;
pub use recorded_request::RecordedRequest;
pub use scenario_bodies::{U_BIG_TEXT_BYTES, U_BOMB_BYTES};
