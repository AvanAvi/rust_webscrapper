//! Signal sets: a receipt's `signal_set_id` resolves to exactly the observations the decision
//! used, shared observations are stored once, and tampering or loss is detected.

#[path = "receipt_support/sample.rs"]
mod sample;

use std::sync::Arc;

use crawler_core::{
    BodyArchive, CoreError, MemoryBodyArchive, PolicyEvaluator, Purpose, SignalKind,
    SignalObservation, SignalSet, SignalSetArchive,
};
use sample::{BoxError, sample_clock, sample_signals, sample_submission};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Archive that serves altered bytes for one hash.
struct TamperingArchive {
    /// Real store.
    o_inner: MemoryBodyArchive,
    /// Hash whose bytes are altered on load.
    str_target: String,
}

impl BodyArchive for TamperingArchive {
    /// Stores through the real archive.
    fn store(&self, by_body: &[u8]) -> Result<String, CoreError> {
        self.o_inner.store(by_body)
    }

    /// Loads, appending a byte to the target body.
    fn load(&self, str_sha256: &str) -> Option<Vec<u8>> {
        let mut by_body = self.o_inner.load(str_sha256)?;
        if str_sha256 == self.str_target {
            by_body.push(b' ');
        }
        Some(by_body)
    }

    /// Retains through the real archive.
    fn retain(&self, str_sha256: &str) -> Result<(), CoreError> {
        self.o_inner.retain(str_sha256)
    }

    /// Releases through the real archive.
    fn release(&self, str_sha256: &str) {
        self.o_inner.release(str_sha256);
    }
}

/// Returns the sample signal plus a variant observed on another page.
fn two_signals() -> Result<Vec<SignalObservation>, BoxError> {
    let vec_o_obs = sample_signals(&sample_clock()?)?;
    let mut o_other = vec_o_obs[0].clone();
    o_other.e_kind = SignalKind::XRobotsTag;
    o_other.str_source_url = String::from("https://example.com/b");
    Ok(vec![vec_o_obs[0].clone(), o_other])
}

#[test]
fn test_set_round_trips_and_its_id_matches_the_receipt() -> TestResult {
    let o_store = SignalSetArchive::new(Arc::new(MemoryBodyArchive::new()));
    let o_clock = sample_clock()?;
    let vec_o_obs = sample_signals(&o_clock)?;
    let str_set_id = o_store.store(&vec_o_obs)?;
    assert_eq!(o_store.load(&str_set_id)?, vec_o_obs);
    let o_submission = sample_submission(&sample_clock()?)?;
    assert_eq!(
        o_submission.str_signal_set_id, str_set_id,
        "receipt points at this set"
    );
    Ok(())
}

#[test]
fn test_decision_can_be_recomputed_from_the_stored_set() -> TestResult {
    let o_store = SignalSetArchive::new(Arc::new(MemoryBodyArchive::new()));
    let o_clock = sample_clock()?;
    let str_set_id = o_store.store(&sample_signals(&o_clock)?)?;
    let o_submission = sample_submission(&sample_clock()?)?;
    let o_again = PolicyEvaluator::for_purpose(Purpose::AiTrain)
        .evaluate(&SignalSet::new(o_store.load(&str_set_id)?))
        .to_record(o_submission.o_decision.dt_decided_at.clone());
    assert_eq!(o_again, o_submission.o_decision);
    Ok(())
}

#[test]
fn test_shared_observations_are_stored_once() -> TestResult {
    let o_archive = Arc::new(MemoryBodyArchive::new());
    let o_store = SignalSetArchive::new(o_archive.clone());
    let vec_o_obs = two_signals()?;
    o_store.store(&vec_o_obs[..1])?;
    o_store.store(&vec_o_obs)?;
    assert_eq!(
        o_archive.usage().1,
        4,
        "2 observations + 2 set records, shared one stored once"
    );
    Ok(())
}

#[test]
fn test_altered_or_missing_bytes_do_not_resolve() -> TestResult {
    let vec_o_obs = two_signals()?;
    let str_set_id =
        SignalSetArchive::new(Arc::new(MemoryBodyArchive::new())).set_id(&vec_o_obs)?;
    let o_tampering = Arc::new(TamperingArchive {
        o_inner: MemoryBodyArchive::new(),
        str_target: str_set_id.clone(),
    });
    let o_store = SignalSetArchive::new(o_tampering);
    assert_eq!(o_store.store(&vec_o_obs)?, str_set_id);
    assert!(matches!(
        o_store.load(&str_set_id),
        Err(CoreError::SignalSetUnresolved { .. })
    ));
    let o_empty = SignalSetArchive::new(Arc::new(MemoryBodyArchive::new()));
    assert!(matches!(
        o_empty.load(&str_set_id),
        Err(CoreError::SignalSetUnresolved { .. })
    ));
    Ok(())
}

#[test]
fn test_failed_store_leaves_nothing_referenced() -> TestResult {
    let o_archive = Arc::new(MemoryBodyArchive::with_limits(1024 * 1024, 2));
    let o_store = SignalSetArchive::new(o_archive.clone());
    assert!(
        o_store.store(&two_signals()?).is_err(),
        "3 entries do not fit in 2"
    );
    o_archive.store(b"first")?;
    o_archive.store(b"second")?;
    assert_eq!(
        o_archive.usage().1,
        2,
        "the partial set was released and evicted"
    );
    Ok(())
}
