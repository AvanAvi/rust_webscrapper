//! Records the TDMRep reservation that applies to one URL.

use serde::{Deserialize, Serialize};

/// TDM reservation state; `None` means unset, which is never a reservation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TdmrepFinding {
    /// `Some(true)` for `1`, `Some(false)` for `0`, `None` when unset or invalid.
    #[serde(rename = "reserved", deserialize_with = "Option::deserialize")]
    pub opt_b_reserved: Option<bool>,
    /// TDM policy URL, capped in length.
    #[serde(rename = "policy_url", deserialize_with = "Option::deserialize")]
    pub opt_str_policy_url: Option<String>,
    /// Location pattern of the matching tdmrep.json rule.
    #[serde(rename = "matched_location", deserialize_with = "Option::deserialize")]
    pub opt_str_matched_location: Option<String>,
    /// Values or entries that broke the protocol and were treated as unset.
    #[serde(rename = "protocol_errors")]
    pub u_protocol_errors: u32,
}
