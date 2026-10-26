//! Records one opt-out signal as observed at one moment.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::excerpt::Excerpt;
use crate::signal_kind::SignalKind;
use crate::signal_status::SignalStatus;
use crate::timestamp::Timestamp;

/// One observed signal source and what was parsed from it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignalObservation {
    /// Kind of signal.
    #[serde(rename = "kind")]
    pub e_kind: SignalKind,
    /// Whether the source was present, absent, or unreachable.
    #[serde(rename = "status")]
    pub e_status: SignalStatus,
    /// URL the signal was read from (the page itself for headers and meta tags).
    #[serde(rename = "source_url")]
    pub str_source_url: String,
    /// Time the signal was observed.
    #[serde(rename = "observed_at")]
    pub dt_observed_at: Timestamp,
    /// HTTP status of the source, when it was fetched.
    #[serde(rename = "http_status", deserialize_with = "Option::deserialize")]
    pub opt_u_http_status: Option<u16>,
    /// SHA-256 hex of the source body, when there was one.
    #[serde(rename = "content_sha256")]
    pub opt_str_content_sha256: Option<String>,
    /// Size-capped excerpt of the relevant part of the source.
    #[serde(rename = "excerpt", deserialize_with = "Option::deserialize")]
    pub opt_o_excerpt: Option<Excerpt>,
    /// Structured result of parsing the source; `null` when nothing was parsed.
    #[serde(rename = "parsed", deserialize_with = "Value::deserialize")]
    pub e_parsed: Value,
}
