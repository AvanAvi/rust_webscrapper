//! Computes SHA-256 digests and their hex form.

use sha2::{Digest, Sha256};

/// SHA-256 hashing used for receipts, bodies, and signal files.
#[derive(Debug, Clone, Copy, Default)]
pub struct Hasher;

impl Hasher {
    /// Creates a hasher.
    pub fn new() -> Self {
        Self
    }

    /// Returns the SHA-256 digest of the data.
    pub fn sha256(&self, by_data: &[u8]) -> [u8; 32] {
        Sha256::digest(by_data).into()
    }

    /// Returns the SHA-256 digest of the data as lowercase hex.
    pub fn sha256_hex(&self, by_data: &[u8]) -> String {
        self.to_hex(&self.sha256(by_data))
    }

    /// Returns bytes as lowercase hex.
    pub fn to_hex(&self, by_digest: &[u8]) -> String {
        let mut str_hex = String::with_capacity(by_digest.len() * 2);
        for u_byte in by_digest {
            str_hex.push_str(&format!("{u_byte:02x}"));
        }
        str_hex
    }
}
