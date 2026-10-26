//! Groups the signals observed for one URL so a policy can read them.

use crate::signal_kind::SignalKind;
use crate::signal_observation::SignalObservation;
use crate::signals::parsed_signal::ParsedSignal;

/// All signal observations made for one URL, in observation order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SignalSet {
    /// Observations, including absent and unreachable sources.
    vec_o_observations: Vec<SignalObservation>,
}

impl SignalSet {
    /// Creates a set from observations.
    pub fn new(vec_o_observations: Vec<SignalObservation>) -> Self {
        Self { vec_o_observations }
    }

    /// Returns every observation.
    pub fn observations(&self) -> &[SignalObservation] {
        &self.vec_o_observations
    }

    /// Returns the parsed findings of one kind, skipping sources that were not parsed.
    pub fn parsed_for(&self, e_kind: SignalKind) -> Vec<&ParsedSignal> {
        self.vec_o_observations
            .iter()
            .filter(|o_obs| o_obs.e_kind == e_kind)
            .filter_map(|o_obs| o_obs.opt_e_parsed.as_ref())
            .collect()
    }

    /// Returns whether no signal was observed.
    pub fn is_empty(&self) -> bool {
        self.vec_o_observations.is_empty()
    }
}
