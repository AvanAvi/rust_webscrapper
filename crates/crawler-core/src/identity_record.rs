//! Records who the crawler declared itself to be.

use serde::{Deserialize, Serialize};

use crate::purpose::Purpose;

/// Crawler identity as declared on every request of the job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityRecord {
    /// Exact User-Agent header sent.
    #[serde(rename = "user_agent")]
    pub str_user_agent: String,
    /// RFC 9309 product token matched against robots.txt groups.
    #[serde(rename = "product_token")]
    pub str_product_token: String,
    /// Declared purpose of the crawl.
    #[serde(rename = "purpose")]
    pub e_purpose: Purpose,
    /// Contact URL included in the User-Agent.
    #[serde(rename = "contact_url")]
    pub str_contact_url: String,
}
