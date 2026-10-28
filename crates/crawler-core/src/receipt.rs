//! Defines the receipt recorded for every URL the crawler considers.

use serde::{Deserialize, Serialize};

use crate::canonical_encoder::CanonicalEncoder;
use crate::core_error::CoreError;
use crate::hasher::Hasher;
use crate::receipt_submission::ReceiptSubmission;
use crate::timestamp::Timestamp;

/// Schema version written into every receipt of this format.
pub const U_RECEIPT_SCHEMA_VERSION: u32 = 3;

/// Evidence for one considered URL: the worker's submission plus the server's stamp
/// (`sequence`, `recorded_at`). Only `ReceiptSubmission::seal` creates one, so the stamp always
/// comes from the server; the fields are private to keep it that way.
///
/// Every field is always serialized; optional ones as explicit `null`. Deserialization
/// rejects missing and unknown fields, so one logical receipt has exactly one encoding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    /// Receipt format version.
    #[serde(rename = "schema_version")]
    u_schema_version: u32,
    /// Position in the ledger, assigned by the server.
    #[serde(rename = "sequence")]
    u_sequence: u64,
    /// Server time the receipt was recorded.
    #[serde(rename = "recorded_at")]
    dt_recorded_at: Timestamp,
    /// What the worker submitted.
    #[serde(rename = "submission")]
    o_submission: ReceiptSubmission,
}

impl Receipt {
    /// Creates a receipt of the current schema version; `ReceiptSubmission::seal` is the caller.
    pub(crate) fn new(
        u_sequence: u64,
        dt_recorded_at: Timestamp,
        o_submission: ReceiptSubmission,
    ) -> Self {
        Self {
            u_schema_version: U_RECEIPT_SCHEMA_VERSION,
            u_sequence,
            dt_recorded_at,
            o_submission,
        }
    }

    /// Returns the schema version.
    pub fn schema_version(&self) -> u32 {
        self.u_schema_version
    }

    /// Returns the ledger position.
    pub fn sequence(&self) -> u64 {
        self.u_sequence
    }

    /// Returns the server time the receipt was recorded.
    pub fn recorded_at(&self) -> &Timestamp {
        &self.dt_recorded_at
    }

    /// Returns the worker's submission.
    pub fn submission(&self) -> &ReceiptSubmission {
        &self.o_submission
    }

    /// Checks the schema version and the submission's consistency rules.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.u_schema_version != U_RECEIPT_SCHEMA_VERSION {
            return Err(CoreError::InvalidReceipt {
                str_reason: format!(
                    "schema_version {} is not {U_RECEIPT_SCHEMA_VERSION}",
                    self.u_schema_version
                ),
            });
        }
        self.o_submission.validate()
    }

    /// Returns the RFC 8785 canonical JSON bytes of the receipt.
    pub fn canonical_bytes(&self, o_encoder: &CanonicalEncoder) -> Result<Vec<u8>, CoreError> {
        o_encoder.encode(self)
    }

    /// Returns the SHA-256 of the canonical bytes.
    pub fn content_hash(
        &self,
        o_encoder: &CanonicalEncoder,
        o_hasher: &Hasher,
    ) -> Result<[u8; 32], CoreError> {
        Ok(o_hasher.sha256(&self.canonical_bytes(o_encoder)?))
    }
}
