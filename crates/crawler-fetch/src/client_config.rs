//! Collects everything needed to build a `SafeHttpClient`.

use std::sync::Arc;

use crawler_core::{Clock, CrawlerIdentity, SystemClock};

use crate::fetch_limits::FetchLimits;
use crate::host_lookup::{HostLookup, SystemHostLookup};
use crate::policy_choice::PolicyChoice;

/// Identity, address policy, limits, resolver, and clock for a client.
#[derive(Clone)]
pub struct ClientConfig {
    /// Additional root trusted only by the test-policy build.
    #[cfg(feature = "test-policy")]
    pub(crate) opt_o_test_root: Option<reqwest::Certificate>,
    /// Declared crawler identity; its User-Agent is sent on every request.
    pub o_identity: CrawlerIdentity,
    /// Address policy selection.
    pub e_policy: PolicyChoice,
    /// Time and size limits.
    pub o_limits: FetchLimits,
    /// Name lookup.
    pub o_lookup: Arc<dyn HostLookup>,
    /// Clock for fetch timestamps.
    pub o_clock: Arc<dyn Clock>,
}

impl ClientConfig {
    /// Creates a production configuration: production policy, default limits, system resolver.
    pub fn new(o_identity: CrawlerIdentity) -> Self {
        Self {
            #[cfg(feature = "test-policy")]
            opt_o_test_root: None,
            o_identity,
            e_policy: PolicyChoice::Production,
            o_limits: FetchLimits::default(),
            o_lookup: Arc::new(SystemHostLookup::new()),
            o_clock: Arc::new(SystemClock::new()),
        }
    }

    /// Returns the configuration with the loopback test policy (refused without `test-policy`).
    pub fn with_test_policy(mut self) -> Self {
        self.e_policy = PolicyChoice::LoopbackForTests;
        self
    }

    /// Returns the configuration with an extra test root, retaining platform verification.
    #[cfg(feature = "test-policy")]
    pub fn with_test_root(mut self, o_root: reqwest::Certificate) -> Self {
        self.opt_o_test_root = Some(o_root);
        self
    }

    /// Returns the configuration with other limits.
    pub fn with_limits(mut self, o_limits: FetchLimits) -> Self {
        self.o_limits = o_limits;
        self
    }

    /// Returns the configuration with another name lookup.
    pub fn with_lookup(mut self, o_lookup: Arc<dyn HostLookup>) -> Self {
        self.o_lookup = o_lookup;
        self
    }

    /// Returns the configuration with another clock.
    pub fn with_clock(mut self, o_clock: Arc<dyn Clock>) -> Self {
        self.o_clock = o_clock;
        self
    }
}
