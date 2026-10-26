//! Lists whether a signal source was found.

use serde::{Deserialize, Serialize};

/// Whether a signal source existed when it was checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalStatus {
    /// The source was retrieved and parsed.
    Present,
    /// The source does not exist (for example a 404); never treated as a reservation.
    Absent,
    /// The source could not be retrieved (timeout, 5xx, connection failure).
    Unreachable,
}
