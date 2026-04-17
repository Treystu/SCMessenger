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
pub use timing::{compute_jitter, JitterConfig, MessagePriority, RelayTimingPolicy, TimingJitter};

use serde::{Deserialize, Serialize};

/// Runtime privacy configuration — toggles for each privacy submodule.
///
/// All features default to `false` for backward compatibility. Enable
/// them individually or via `PrivacyConfig::full()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Pad plaintext payloads to standard sizes before encryption.
    /// Hides true message lengths from traffic analysis.
    pub message_padding_enabled: bool,
    /// Onion-route messages through relay hops for sender/recipient anonymity.
    pub onion_routing_enabled: bool,
    /// Generate periodic cover-traffic messages to mask real traffic patterns.
    pub cover_traffic_enabled: bool,
    /// Apply random jitter delays to relay forwarding.
    /// Thwarts timing correlation attacks.
    pub timing_obfuscation_enabled: bool,
    /// Cover traffic generation rate (messages per minute).
    pub cover_traffic_rate_per_minute: u32,
    /// Timing jitter configuration for relay forwarding.
    pub relay_timing_policy: RelayTimingPolicy,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            message_padding_enabled: false,
            onion_routing_enabled: false,
            cover_traffic_enabled: false,
            timing_obfuscation_enabled: false,
            cover_traffic_rate_per_minute: 10,
            relay_timing_policy: RelayTimingPolicy::default(),
        }
    }
}

impl PrivacyConfig {
    /// Enable all privacy features with sensible defaults.
    pub fn full() -> Self {
        Self {
            message_padding_enabled: true,
            onion_routing_enabled: true,
            cover_traffic_enabled: true,
            timing_obfuscation_enabled: true,
            cover_traffic_rate_per_minute: 10,
            relay_timing_policy: RelayTimingPolicy::default(),
        }
    }
}
