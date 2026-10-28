//! Stores the signal observations behind a decision and names them by one hash.

use std::sync::Arc;

use crate::body_archive::BodyArchive;
use crate::canonical_encoder::CanonicalEncoder;
use crate::core_error::CoreError;
use crate::hasher::Hasher;
use crate::signal_observation::SignalObservation;

/// Content-addressed store for the signal sets receipts point to.
///
/// Each observation is stored as its canonical JSON, keyed by the SHA-256 of those bytes (its
/// observation hash). The set record is the canonical JSON array of the observation hashes in
/// observation order (reasons follow that order), and `signal_set_id` is the SHA-256 of that
/// record. Identical observations, such as one host's robots.txt finding reused across pages,
/// are stored once. Loading re-checks every hash, so a receipt's `signal_set_id` commits to
/// the exact observations.
pub struct SignalSetArchive {
    /// Body store shared with the collector.
    o_archive: Arc<dyn BodyArchive>,
    /// Canonical JSON encoder.
    o_encoder: CanonicalEncoder,
    /// SHA-256.
    o_hasher: Hasher,
}

impl SignalSetArchive {
    /// Creates a store over a body archive.
    pub fn new(o_archive: Arc<dyn BodyArchive>) -> Self {
        Self {
            o_archive,
            o_encoder: CanonicalEncoder::new(),
            o_hasher: Hasher::new(),
        }
    }

    /// Returns the `signal_set_id` of observations without storing anything.
    pub fn set_id(&self, vec_o_obs: &[SignalObservation]) -> Result<String, CoreError> {
        let vec_str_hashes = vec_o_obs
            .iter()
            .map(|o_obs| Ok(self.o_hasher.sha256_hex(&self.o_encoder.encode(o_obs)?)))
            .collect::<Result<Vec<String>, CoreError>>()?;
        Ok(self
            .o_hasher
            .sha256_hex(&self.o_encoder.encode(&vec_str_hashes)?))
    }

    /// Stores every observation and the set record, taking one archive reference each, and
    /// returns the `signal_set_id`. On failure nothing stays referenced and no receipt may be
    /// written for the decision: its evidence could not be archived.
    pub fn store(&self, vec_o_obs: &[SignalObservation]) -> Result<String, CoreError> {
        let mut vec_str_stored = Vec::with_capacity(vec_o_obs.len() + 1);
        let res_str_id = self.store_all(vec_o_obs, &mut vec_str_stored);
        if res_str_id.is_err() {
            vec_str_stored
                .iter()
                .for_each(|str_sha| self.o_archive.release(str_sha));
        }
        res_str_id
    }

    /// Loads a set and checks every hash against the bytes it names.
    pub fn load(&self, str_set_id: &str) -> Result<Vec<SignalObservation>, CoreError> {
        let by_record = self.verified(str_set_id)?;
        let vec_str_hashes: Vec<String> = serde_json::from_slice(&by_record)?;
        vec_str_hashes
            .iter()
            .map(|str_hash| Ok(serde_json::from_slice(&self.verified(str_hash)?)?))
            .collect()
    }

    /// Stores observations then the record, collecting the keys stored so far.
    fn store_all(
        &self,
        vec_o_obs: &[SignalObservation],
        vec_str_stored: &mut Vec<String>,
    ) -> Result<String, CoreError> {
        for o_obs in vec_o_obs {
            let str_hash = self.o_archive.store(&self.o_encoder.encode(o_obs)?)?;
            vec_str_stored.push(str_hash);
        }
        let str_set_id = self
            .o_archive
            .store(&self.o_encoder.encode(&*vec_str_stored)?)?;
        vec_str_stored.push(str_set_id.clone());
        Ok(str_set_id)
    }

    /// Returns archived bytes after checking that they hash to the requested key.
    fn verified(&self, str_hash: &str) -> Result<Vec<u8>, CoreError> {
        let by_body = self
            .o_archive
            .load(str_hash)
            .ok_or_else(|| Self::unresolved(str_hash, "not in the archive"))?;
        if self.o_hasher.sha256_hex(&by_body) != str_hash {
            return Err(Self::unresolved(
                str_hash,
                "archived bytes do not match the hash",
            ));
        }
        Ok(by_body)
    }

    /// Builds the error for a hash that does not resolve.
    fn unresolved(str_hash: &str, str_reason: &str) -> CoreError {
        CoreError::SignalSetUnresolved {
            str_sha256: str_hash.to_owned(),
            str_reason: str_reason.to_owned(),
        }
    }
}
