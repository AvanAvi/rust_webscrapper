//! Records which crawler build produced a receipt.

use serde::{Deserialize, Serialize};

/// Product name of the crawler.
pub const STR_CRAWLER_NAME: &str = "proofcrawl";

/// Name and version of the crawler build. The version comes from the build
/// (`CARGO_PKG_VERSION`, shared by every workspace crate), never from configuration, so it
/// names the parser and collector code that produced the findings. Only `from_build` creates
/// one; decoding accepts any version so older receipts stay readable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CrawlerRecord {
    /// Product name.
    #[serde(rename = "name")]
    str_name: String,
    /// Build version.
    #[serde(rename = "version")]
    str_version: String,
}

impl CrawlerRecord {
    /// Returns the record of the running build.
    pub fn from_build() -> Self {
        Self {
            str_name: STR_CRAWLER_NAME.to_owned(),
            str_version: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }

    /// Returns the product name.
    pub fn name(&self) -> &str {
        &self.str_name
    }

    /// Returns the build version.
    pub fn version(&self) -> &str {
        &self.str_version
    }
}
