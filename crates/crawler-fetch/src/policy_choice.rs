//! Selects the address policy a client is built with.

use std::sync::Arc;

use crate::address_policy::AddressPolicy;
use crate::fetch_error::FetchError;
use crate::production_address_policy::ProductionAddressPolicy;

/// Which address policy to use. `LoopbackForTests` only builds when the `test-policy` feature
/// is compiled in; otherwise the client refuses it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PolicyChoice {
    /// Only globally routable addresses.
    #[default]
    Production,
    /// Loopback only, for reaching the local mock-site in tests.
    LoopbackForTests,
}

impl PolicyChoice {
    /// Returns the policy, or `TestPolicyUnavailable` for the test policy in a default build.
    pub fn build(&self) -> Result<Arc<dyn AddressPolicy>, FetchError> {
        match self {
            Self::Production => Ok(Arc::new(ProductionAddressPolicy::new())),
            Self::LoopbackForTests => Self::build_test_policy(),
        }
    }

    /// Returns whether the test policy is compiled into this build.
    pub fn is_test_policy_compiled() -> bool {
        cfg!(feature = "test-policy")
    }

    /// Builds the loopback test policy (feature `test-policy`).
    #[cfg(feature = "test-policy")]
    fn build_test_policy() -> Result<Arc<dyn AddressPolicy>, FetchError> {
        Ok(Arc::new(
            crate::loopback_test_policy::LoopbackTestPolicy::new(),
        ))
    }

    /// Refuses the test policy: this build does not contain it.
    #[cfg(not(feature = "test-policy"))]
    fn build_test_policy() -> Result<Arc<dyn AddressPolicy>, FetchError> {
        Err(FetchError::TestPolicyUnavailable)
    }
}
