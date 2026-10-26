//! Records one redirect followed during a fetch.

use serde::{Deserialize, Serialize};

/// One redirect response and the location it pointed to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedirectHop {
    /// URL that answered with the redirect.
    #[serde(rename = "url")]
    pub str_url: String,
    /// Redirect status code.
    #[serde(rename = "status")]
    pub u_status: u16,
}
