//! Defines what a worker submits for one considered URL; the server seals it into a receipt.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::clock::Clock;
use crate::core_error::CoreError;
use crate::crawler_record::CrawlerRecord;
use crate::decision_record::DecisionRecord;
use crate::fetch_failure::FetchFailure;
use crate::fetch_record::FetchRecord;
use crate::identity_record::IdentityRecord;
use crate::outcome::Outcome;
use crate::receipt::Receipt;
use crate::target_record::TargetRecord;

/// Everything a worker claims about one considered URL. It has no `sequence` and no
/// `recorded_at`: only the server sets those, in `seal`. Decoding rejects them as unknown
/// fields, so a worker cannot supply them, and they sit outside the submission, which is the
/// part a worker could sign.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptSubmission {
    /// Unique receipt identifier, also the completion idempotency key.
    #[serde(rename = "receipt_id")]
    pub uuid_receipt_id: Uuid,
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
    /// Crawler build that produced the findings.
    #[serde(rename = "crawler")]
    pub o_crawler: CrawlerRecord,
    /// Declared crawler identity.
    #[serde(rename = "identity")]
    pub o_identity: IdentityRecord,
    /// URL forms of the target.
    #[serde(rename = "target")]
    pub o_target: TargetRecord,
    /// SHA-256 of the signal set the decision used (see `SignalSetArchive`); the observations
    /// themselves are stored once in the archive, shared by every receipt that used them.
    #[serde(rename = "signal_set_id")]
    pub str_signal_set_id: String,
    /// Allow or deny decision and its rule.
    #[serde(rename = "decision")]
    pub o_decision: DecisionRecord,
    /// Page fetch facts; `None` when denied or when the fetch failed.
    #[serde(rename = "fetch", deserialize_with = "Option::deserialize")]
    pub opt_o_fetch: Option<FetchRecord>,
    /// Why an allowed fetch produced no response; `None` otherwise.
    #[serde(rename = "fetch_failure", deserialize_with = "Option::deserialize")]
    pub opt_e_fetch_failure: Option<FetchFailure>,
}

impl ReceiptSubmission {
    /// Stamps the submission with the server's sequence and clock and validates the result.
    pub fn seal(self, u_sequence: u64, o_clock: &dyn Clock) -> Result<Receipt, CoreError> {
        let o_receipt = Receipt::new(u_sequence, o_clock.timestamp()?, self);
        o_receipt.validate()?;
        Ok(o_receipt)
    }

    /// Checks the consistency rules: `signal_set_id` is a SHA-256 digest, a denied URL has
    /// neither fetch nor fetch failure, an allowed one has exactly one, and the decision is made
    /// before the fetch starts.
    pub fn validate(&self) -> Result<(), CoreError> {
        self.check_set_id()?;
        self.check_fetch_shape()?;
        self.check_decision_time()
    }

    /// Checks that `signal_set_id` is a lowercase SHA-256 hex digest.
    fn check_set_id(&self) -> Result<(), CoreError> {
        let b_hex = self.str_signal_set_id.len() == 64
            && self
                .str_signal_set_id
                .bytes()
                .all(|u_b| u_b.is_ascii_digit() || (b'a'..=b'f').contains(&u_b));
        if b_hex {
            return Ok(());
        }
        Err(Self::invalid(
            "signal_set_id is not a lowercase SHA-256 hex digest",
        ))
    }

    /// Checks that fetch and fetch failure match the outcome.
    fn check_fetch_shape(&self) -> Result<(), CoreError> {
        let b_fetch = self.opt_o_fetch.is_some();
        let b_failure = self.opt_e_fetch_failure.is_some();
        match self.o_decision.e_outcome {
            Outcome::Deny if b_fetch || b_failure => Err(Self::invalid(
                "a denied URL has neither fetch nor fetch_failure",
            )),
            Outcome::Allow if b_fetch == b_failure => Err(Self::invalid(
                "an allowed URL has exactly one of fetch and fetch_failure",
            )),
            _ => Ok(()),
        }
    }

    /// Checks that the decision was made strictly before the page fetch started.
    fn check_decision_time(&self) -> Result<(), CoreError> {
        match &self.opt_o_fetch {
            Some(o_fetch) if self.o_decision.dt_decided_at >= o_fetch.dt_started_at => Err(
                Self::invalid("decision.decided_at must be earlier than fetch.started_at"),
            ),
            _ => Ok(()),
        }
    }

    /// Builds the error for a broken rule.
    fn invalid(str_reason: &str) -> CoreError {
        CoreError::InvalidReceipt {
            str_reason: str_reason.to_owned(),
        }
    }
}
