// Privacy Enhancements — Phase 7
//
// Provides onion routing, circuit building, traffic analysis resistance,
// timing jitter, and cover traffic generation.

pub mod circuit;
pub mod cover;
pub mod onion;
pub mod padding;
pub mod timing;

pub use circuit::{CircuitBuilder, CircuitConfig, CircuitId, CircuitPath};
pub use cover::{CoverConfig, CoverTrafficGenerator, CoverTrafficScheduler};
pub use onion::{
    construct_onion, peel_layer, OnionEnvelope, OnionHeader, OnionLayer, MAX_ONION_HOPS,
};
pub use padding::{pad_message, pad_to_next_standard_size, unpad_message, PaddingScheme};
pub use timing::{compute_jitter, JitterConfig, RelayTimingPolicy, TimingJitter};
