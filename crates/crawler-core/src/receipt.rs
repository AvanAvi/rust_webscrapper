//! Defines the receipt body recorded for every URL the crawler considers.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::canonical_encoder::CanonicalEncoder;
use crate::core_error::CoreError;
use crate::decision_record::DecisionRecord;
use crate::fetch_record::FetchRecord;
use crate::hasher::Hasher;
use crate::identity_record::IdentityRecord;
use crate::signal_observation::SignalObservation;
use crate::target_record::TargetRecord;

/// Schema version written into every receipt of this format.
pub const U_RECEIPT_SCHEMA_VERSION: u32 = 2;

/// Evidence for one considered URL: identity, observed signals, decision, and fetch facts.
///
/// Every field is always serialized; optional ones as explicit `null`. Deserialization
/// rejects missing and unknown fields, so one logical receipt has exactly one encoding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    /// Receipt format version.
    #[serde(rename = "schema_version")]
    pub u_schema_version: u32,
    /// Unique receipt identifier.
    #[serde(rename = "receipt_id")]
    pub uuid_receipt_id: Uuid,
    /// Position in the ledger, assigned by the server.
    #[serde(rename = "sequence")]
    pub u_sequence: u64,
    /// Job the URL belongs to.
    #[serde(rename = "job_id")]
    pub uuid_job_id: Uuid,
    /// Frontier task the receipt answers.
    #[serde(rename = "task_id")]
    pub uuid_task_id: Uuid,
    /// Fencing token of the lease the work was done under.
    #[serde(rename = "lease_epoch")]
    pub u_lease_epoch: u64,
    /// Worker that made the observations.
    #[serde(rename = "worker_id")]
    pub uuid_worker_id: Uuid,
    /// Declared crawler identity.
    #[serde(rename = "identity")]
    pub o_identity: IdentityRecord,
    /// URL forms of the target.
    #[serde(rename = "target")]
    pub o_target: TargetRecord,
    /// Signals observed before the decision.
    #[serde(rename = "signals")]
    pub vec_o_signals: Vec<SignalObservation>,
    /// Allow or deny decision and its rule.
    #[serde(rename = "decision")]
    pub o_decision: DecisionRecord,
    /// Fetch facts; `None` (JSON `null`) when the URL was denied.
    #[serde(rename = "fetch", deserialize_with = "Option::deserialize")]
    pub opt_o_fetch: Option<FetchRecord>,
}

impl Receipt {
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
