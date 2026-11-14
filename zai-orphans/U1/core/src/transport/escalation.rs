//! Transport Escalation — automatic transport negotiation
//!
//! Implements automatic negotiation of the best transport between two peers,
//! upgrading to better transports when available and downgrading gracefully.
//!
//! AUTHORITY (PHIL-012, chunk U1): this engine is the SINGLE decision-maker
//! for transport ordering. Platforms (Android `SmartTransportRouter`, iOS
//! `SmartTransportRouter.swift`, `MeshRepository` heuristics) are executors
//! and inputs: they report delivery outcomes via `record_delivery_outcome`
//! and ask for ordering via `preferred_order` / the UniFFI string-typed
//! surface in `mobile_bridge.rs`. They must not keep rival ordering policy.

use crate::transport::abstraction::TransportCapabilities;
use crate::transport::abstraction::TransportType;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::debug;

/// Parse a transport name as serialized over UniFFI (the `TransportType`
/// `Display` form). Returns None for unknown names -- callers keep unknowns
/// losslessly at the end of their ordering instead of dropping them.
pub fn transport_type_from_name(name: &str) -> Option<TransportType> {
    match name {
        "BLE" => Some(TransportType::BLE),
        "WiFiAware" => Some(TransportType::WiFiAware),
        "WiFiDirect" => Some(TransportType::WiFiDirect),
        "Internet" => Some(TransportType::Internet),
        "Local" => Some(TransportType::Local),
        _ => None,
    }
}

/// Live delivery health observed for one (peer, transport) pair.
///
/// This is the empirical input that reweights the static policy scores:
/// a transport whose recent deliveries keep failing is demoted even when
/// its paper capabilities look best.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TransportHealthStats {
    /// Successful deliveries recorded.
    pub success_count: u64,
    /// Failed deliveries recorded.
    pub failure_count: u64,
    /// Exponentially weighted moving average of successful delivery
    /// latency in milliseconds (failures carry no latency sample).
    pub latency_ewma_ms: f64,
}

impl TransportHealthStats {
    /// Laplace-smoothed success rate clamped to [0.25, 1.0].
    ///
    /// NEUTRALITY CONTRACT: with ZERO observations this is EXACTLY 1.0 --
    /// no smoothing happens, no clamping happens -- so the weighted score
    /// equals the raw policy score bit-for-bit and every behavior locked
    /// in by the pre-health unit tests is preserved. Health weighting can
    /// only change a comparison once at least one outcome has been
    /// recorded for one of the transports being compared.
    pub fn health_factor(&self) -> f64 {
        let successes = self.success_count as f64;
        let failures = self.failure_count as f64;
        if successes == 0.0 && failures == 0.0 {
            return 1.0;
        }
        // Laplace smoothing (one phantom success + one phantom failure)
        // keeps a single failure from flooring the factor to the clamp.
        let smoothed = (successes + 1.0) / (successes + failures + 2.0);
        smoothed.clamp(0.25, 1.0)
    }
}

/// EWMA weight applied to each new latency sample (0.25 = recent samples
/// dominate; a quarter of the history decays per observation).
const HEALTH_LATENCY_EWMA_ALPHA: f64 = 0.25;

/// Escalation policy for transport selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EscalationPolicy {
    /// Always escalate to highest bandwidth available
    PreferHighBandwidth,
    /// Prefer lowest latency transport
    PreferLowLatency,
    /// Prefer BLE over WiFi over Internet (power efficiency)
    PreferLowPower,
    /// Balanced: weighted combination of all factors
    #[default]
    Balanced,
}

/// Errors that can occur during escalation
#[derive(Error, Debug, Clone)]
pub enum EscalationError {
    #[error("No transports available")]
    NoTransportsAvailable,

    #[error("Escalation not possible")]
    NotPossible,

    #[error("Escalation failed: {0}")]
    Failed(String),
}

/// State for a peer's escalation
#[derive(Debug, Clone)]
pub struct EscalationState {
    /// Current transport being used
    pub current_transport: TransportType,
    /// All available transports
    pub available_transports: Vec<TransportType>,
    /// Last escalation attempt timestamp
    pub last_escalation_attempt: Option<web_time::SystemTime>,
}

/// Manages transport escalation for all peers
pub struct EscalationEngine {
    /// Current escalation states per peer
    states: Arc<RwLock<HashMap<[u8; 32], EscalationState>>>,
    /// Escalation policy
    policy: EscalationPolicy,
    /// Transport capabilities (for scoring)
    capabilities: Arc<RwLock<HashMap<TransportType, TransportCapabilities>>>,
    /// Live delivery health per (peer, transport) -- the empirical input
    /// that reweights policy scores (chunk U1 single-authority change).
    health: Arc<RwLock<HashMap<([u8; 32], TransportType), TransportHealthStats>>>,
}

impl EscalationEngine {
    /// Create a new escalation engine
    pub fn new(policy: EscalationPolicy) -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            policy,
            capabilities: Arc::new(RwLock::new(HashMap::new())),
            health: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set capabilities for a transport type
    pub fn set_capabilities(&self, transport: TransportType, capabilities: TransportCapabilities) {
        let mut caps = self.capabilities.write();
        caps.insert(transport, capabilities);
    }

    /// Record a live delivery outcome for (peer, transport).
    ///
    /// This is the single entry point through which platforms feed observed
    /// reality into the authority (chunk U1). Successes update the latency
    /// EWMA; both outcomes update the counts that drive `health_factor`.
    /// Recording never changes a peer's tracked state -- a transport with
    /// recorded health but no escalation state is simply scored when the
    /// platform asks for ordering.
    pub fn record_delivery_outcome(
        &self,
        peer: [u8; 32],
        transport: TransportType,
        success: bool,
        latency_ms: u64,
    ) {
        let mut health = self.health.write();
        let stats = health.entry((peer, transport)).or_default();
        if success {
            stats.success_count += 1;
            let sample = latency_ms as f64;
            stats.latency_ewma_ms = if stats.latency_ewma_ms <= 0.0 {
                sample
            } else {
                stats.latency_ewma_ms
                    + HEALTH_LATENCY_EWMA_ALPHA * (sample - stats.latency_ewma_ms)
            };
        } else {
            stats.failure_count += 1;
        }
        debug!(
            "Recorded {} delivery for peer {:x?} via {}: success={} failure={} factor={:.3}",
            if success { "successful" } else { "failed" },
            &peer[..8],
            transport,
            stats.success_count,
            stats.failure_count,
            stats.health_factor()
        );
    }

    /// Snapshot of live health recorded for (peer, transport), if any.
    /// Diagnostics/read-only -- ordering decisions go through scoring.
    pub fn health_stats(
        &self,
        peer: &[u8; 32],
        transport: TransportType,
    ) -> Option<TransportHealthStats> {
        self.health.read().get(&(*peer, transport)).copied()
    }

    /// Health factor applied to a transport's policy score for a peer.
    /// EXACTLY 1.0 when no outcomes have been recorded (neutrality proof:
    /// `raw * 1.0 == raw`), below 1.0 once failures are observed.
    fn health_factor_for(&self, peer: &[u8; 32], transport: TransportType) -> f64 {
        match self.health.read().get(&(*peer, transport)) {
            Some(stats) => stats.health_factor(),
            None => 1.0,
        }
    }

    /// Initialize escalation state for a peer
    pub fn init_peer(
        &self,
        peer_id: [u8; 32],
        available_transports: Vec<TransportType>,
    ) -> Result<(), EscalationError> {
        if available_transports.is_empty() {
            return Err(EscalationError::NoTransportsAvailable);
        }

        let mut states = self.states.write();
        let current = self.select_best_transport_for_peer(&peer_id, &available_transports);

        states.insert(
            peer_id,
            EscalationState {
                current_transport: current,
                available_transports,
                last_escalation_attempt: None,
            },
        );

        debug!("Initialized escalation for peer {:x?}", &peer_id[..8]);
        Ok(())
    }

    /// Check if escalation should be attempted
    pub fn should_escalate(&self, peer_id: [u8; 32]) -> bool {
        let states = self.states.read();
        if let Some(state) = states.get(&peer_id) {
            let better = self.find_better_transport(
                &peer_id,
                &state.available_transports,
                state.current_transport,
            );
            better.is_some()
        } else {
            false
        }
    }

    /// Attempt to escalate to a better transport
    pub fn escalate(&self, peer_id: [u8; 32]) -> Result<TransportType, EscalationError> {
        let mut states = self.states.write();
        let state = states
            .get_mut(&peer_id)
            .ok_or(EscalationError::NotPossible)?;

        let target = self
            .find_better_transport(&peer_id, &state.available_transports, state.current_transport)
            .ok_or(EscalationError::NotPossible)?;

        state.current_transport = target;
        state.last_escalation_attempt = Some(web_time::SystemTime::now());

        debug!("Escalated peer {:x?} to {}", &peer_id[..8], target);
        Ok(target)
    }

    /// Deescalate to a fallback transport.
    /// Returns the new fallback transport if successful, or None if already
    /// at the lowest tier.
    ///
    /// NOTE (review F7, TRANSPORT_FAILOVER_AUDIT_QWENPAID_2026-08-05): the
    /// fallback chosen is the BEST worse transport (closest tier below
    /// current), i.e. graceful one-step degradation. If that tier is also
    /// silently dead, Phase-2 health integration must drive multi-step
    /// deescalation.
    pub fn deescalate(&self, peer_id: [u8; 32]) -> Result<Option<TransportType>, EscalationError> {
        let mut states = self.states.write();
        let state = states
            .get_mut(&peer_id)
            .ok_or(EscalationError::NotPossible)?;

        let current = state.current_transport;
        let fallback = self.find_worse_transport(&peer_id, &state.available_transports, current);

        if let Some(fallback) = fallback {
            state.current_transport = fallback;
            debug!("Deescalated peer {:x?} to {}", &peer_id[..8], fallback);
        } else {
            debug!(
                "Cannot deescalate peer {:x?}, using current transport",
                &peer_id[..8]
            );
        }

        Ok(fallback)
    }

    /// Get the current transport for a peer
    pub fn current_transport(&self, peer_id: [u8; 32]) -> Option<TransportType> {
        let states = self.states.read();
        states.get(&peer_id).map(|s| s.current_transport)
    }

    /// Update available transports for a peer
    pub fn update_available_transports(
        &self,
        peer_id: [u8; 32],
        transports: Vec<TransportType>,
    ) -> Result<(), EscalationError> {
        let mut states = self.states.write();
        if let Some(state) = states.get_mut(&peer_id) {
            state.available_transports = transports;
            Ok(())
        } else {
            Err(EscalationError::NotPossible)
        }
    }

    /// Find the best transport from a list using the policy.
    ///
    /// Health-blind (policy-only): kept for callers that have no peer
    /// context. Peer-aware callers must use
    /// [`EscalationEngine::select_best_transport_for_peer`], which applies
    /// recorded live health on top of the same policy score.
    pub fn select_best_transport(&self, available: &[TransportType]) -> TransportType {
        if available.is_empty() {
            return TransportType::Local;
        }

        let caps = self.capabilities.read();

        let best = available
            .iter()
            .max_by(|a, b| {
                let score_a = self.escalation_score(**a, self.policy, &caps);
                let score_b = self.escalation_score(**b, self.policy, &caps);
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
            .unwrap_or(available[0]);

        best
    }

    /// Find the best transport for a peer from a list using the policy
    /// weighted by that peer's recorded live health (chunk U1).
    pub fn select_best_transport_for_peer(
        &self,
        peer_id: &[u8; 32],
        available: &[TransportType],
    ) -> TransportType {
        if available.is_empty() {
            return TransportType::Local;
        }

        let caps = self.capabilities.read();

        let best = available
            .iter()
            .max_by(|a, b| {
                let score_a = self.weighted_score(peer_id, **a, &caps);
                let score_b = self.weighted_score(peer_id, **b, &caps);
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
            .unwrap_or(available[0]);

        best
    }

    /// Order `available` transports best-first for a peer (policy score
    /// weighted by recorded live health). Stable: transports with equal
    /// weighted score keep their input order.
    ///
    /// This is the ordering the UniFFI `escalation_preferred_order`
    /// surface exposes to platforms; unknown transports are appended by
    /// the bridge layer (loud, not lossy), not silently dropped here.
    pub fn preferred_order(
        &self,
        peer_id: &[u8; 32],
        available: &[TransportType],
    ) -> Vec<TransportType> {
        let caps = self.capabilities.read();
        let mut ordered = available.to_vec();
        ordered.sort_by(|a, b| {
            let score_a = self.weighted_score(peer_id, *a, &caps);
            let score_b = self.weighted_score(peer_id, *b, &caps);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        ordered
    }

    /// Policy score for a transport multiplied by the peer's live health
    /// factor for it. With no recorded outcomes the factor is exactly 1.0
    /// and the result equals `escalation_score` bit-for-bit (the
    /// no-health-neutrality proof every legacy test pins).
    fn weighted_score(
        &self,
        peer_id: &[u8; 32],
        transport: TransportType,
        caps: &HashMap<TransportType, TransportCapabilities>,
    ) -> f64 {
        self.escalation_score(transport, self.policy, caps)
            * self.health_factor_for(peer_id, transport)
    }

    /// Calculate escalation score for a transport (higher is better)
    fn escalation_score(
        &self,
        transport: TransportType,
        policy: EscalationPolicy,
        caps: &HashMap<TransportType, TransportCapabilities>,
    ) -> f64 {
        let cap = caps
            .get(&transport)
            .cloned()
            .unwrap_or(TransportCapabilities::for_transport(transport));

        match policy {
            EscalationPolicy::PreferHighBandwidth => {
                // Internet > WiFiAware > WiFiDirect > BLE > Local
                // Higher bandwidth = higher score
                cap.estimated_bandwidth_bps as f64 / 1_000_000_000.0
            }
            EscalationPolicy::PreferLowLatency => {
                // Local > BLE > WiFiDirect > WiFiAware > Internet
                // Lower latency = higher score
                1000.0 - cap.estimated_latency_ms as f64
            }
            EscalationPolicy::PreferLowPower => {
                // BLE > WiFiAware > WiFiDirect > Internet
                match transport {
                    TransportType::BLE => 400.0,
                    TransportType::WiFiAware => 300.0,
                    TransportType::WiFiDirect => 200.0,
                    TransportType::Internet => 100.0,
                    TransportType::Local => 0.0,
                }
            }
            EscalationPolicy::Balanced => {
                // Weighted combination
                let bandwidth_score = (cap.estimated_bandwidth_bps as f64 / 1_000_000_000.0) * 0.4;
                let latency_score = (1000.0 - cap.estimated_latency_ms as f64) * 0.3;
                let streaming_score = if cap.supports_streaming { 100.0 } else { 0.0 } * 0.3;

                bandwidth_score + latency_score + streaming_score
            }
        }
    }

    /// Find a transport with higher weighted score than current
    fn find_better_transport(
        &self,
        peer_id: &[u8; 32],
        available: &[TransportType],
        current: TransportType,
    ) -> Option<TransportType> {
        let caps = self.capabilities.read();
        let current_score = self.weighted_score(peer_id, current, &caps);

        available
            .iter()
            .filter(|&&t| t != current)
            .max_by(|&&a, &&b| {
                let score_a = self.weighted_score(peer_id, a, &caps);
                let score_b = self.weighted_score(peer_id, b, &caps);
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .and_then(|&best| {
                let best_score = self.weighted_score(peer_id, best, &caps);
                if best_score > current_score {
                    Some(best)
                } else {
                    None
                }
            })
    }

    /// Find a transport with lower weighted score than current (fallback)
    fn find_worse_transport(
        &self,
        peer_id: &[u8; 32],
        available: &[TransportType],
        current: TransportType,
    ) -> Option<TransportType> {
        let caps = self.capabilities.read();
        let current_score = self.weighted_score(peer_id, current, &caps);

        available
            .iter()
            .filter(|&&t| t != current)
            .max_by(|&&a, &&b| {
                let score_a = self.weighted_score(peer_id, a, &caps);
                let score_b = self.weighted_score(peer_id, b, &caps);
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .and_then(|&best| {
                let best_score = self.weighted_score(peer_id, best, &caps);
                if best_score < current_score {
                    Some(best)
                } else {
                    None
                }
            })
    }

    /// Get the recommended transport for a peer based on current state and policy.
    /// Returns None if the peer is not tracked.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn recommended_transport(&self, peer_id: &[u8; 32]) -> Option<crate::ProximityTransport> {
        let states = self.states.read();
        states.get(peer_id).map(|state| {
            let best = self.select_best_transport_for_peer(peer_id, &state.available_transports);
            match best {
                TransportType::BLE => crate::ProximityTransport::Ble,
                TransportType::WiFiAware => crate::ProximityTransport::WifiAware,
                TransportType::WiFiDirect => crate::ProximityTransport::WifiDirect,
                _ => crate::ProximityTransport::Ble, // fallback to BLE for Internet/Local
            }
        })
    }

    /// Remove a peer from tracking (state AND recorded live health, so the
    /// health map stays bounded with the peer set)
    pub fn cleanup_peer(&self, peer_id: [u8; 32]) {
        let mut states = self.states.write();
        states.remove(&peer_id);
        let mut health = self.health.write();
        health.retain(|(peer, _transport), _| *peer != peer_id);
    }

    /// Get all active peer states
    pub fn all_states(&self) -> Vec<([u8; 32], EscalationState)> {
        let states = self.states.read();
        states
            .iter()
            .map(|(peer_id, state)| (*peer_id, state.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_peer_id(val: u8) -> [u8; 32] {
        let mut id = [0u8; 32];
        id[0] = val;
        id
    }

    #[test]
    fn test_escalation_engine_creation() {
        let engine = EscalationEngine::new(EscalationPolicy::Balanced);
        assert_eq!(engine.policy, EscalationPolicy::Balanced);
    }

    #[test]
    fn test_init_peer_empty_transports() {
        let engine = EscalationEngine::new(EscalationPolicy::Balanced);
        let peer_id = create_peer_id(1);

        let result = engine.init_peer(peer_id, vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_peer_success() {
        let engine = EscalationEngine::new(EscalationPolicy::Balanced);
        let peer_id = create_peer_id(1);

        let result = engine.init_peer(peer_id, vec![TransportType::BLE, TransportType::WiFiDirect]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_select_best_transport_high_bandwidth() {
        let engine = EscalationEngine::new(EscalationPolicy::PreferHighBandwidth);

        let best = engine.select_best_transport(&[
            TransportType::BLE,
            TransportType::WiFiDirect,
            TransportType::Internet,
        ]);

        assert_eq!(best, TransportType::WiFiDirect);
    }

    #[test]
    fn test_select_best_transport_low_latency() {
        let engine = EscalationEngine::new(EscalationPolicy::PreferLowLatency);

        let best = engine.select_best_transport(&[
            TransportType::BLE,
            TransportType::WiFiDirect,
            TransportType::Local,
        ]);

        assert_eq!(best, TransportType::Local);
    }

    #[test]
    fn test_select_best_transport_low_power() {
        let engine = EscalationEngine::new(EscalationPolicy::PreferLowPower);

        let best = engine.select_best_transport(&[
            TransportType::BLE,
            TransportType::WiFiDirect,
            TransportType::Internet,
        ]);

        assert_eq!(best, TransportType::BLE);
    }

    #[test]
    fn test_select_best_transport_balanced() {
        let engine = EscalationEngine::new(EscalationPolicy::Balanced);

        let best = engine.select_best_transport(&[
            TransportType::BLE,
            TransportType::WiFiAware,
            TransportType::WiFiDirect,
        ]);

        // Balanced should prefer higher bandwidth with streaming support
        assert_ne!(best, TransportType::BLE);
    }

    #[test]
    fn test_escalation_high_bandwidth_policy() {
        let engine = EscalationEngine::new(EscalationPolicy::PreferHighBandwidth);
        let peer_id = create_peer_id(1);

        engine
            .init_peer(
                peer_id,
                vec![
                    TransportType::BLE,
                    TransportType::WiFiDirect,
                    TransportType::Internet,
                ],
            )
            .unwrap();

        let current = engine.current_transport(peer_id).unwrap();
        // Should start with highest bandwidth (WiFiDirect)
        assert_eq!(current, TransportType::WiFiDirect);
    }

    #[test]
    fn test_escalation_low_latency_policy() {
        let engine = EscalationEngine::new(EscalationPolicy::PreferLowLatency);
        let peer_id = create_peer_id(2);

        engine
            .init_peer(peer_id, vec![TransportType::Internet, TransportType::Local])
            .unwrap();

        let current = engine.current_transport(peer_id).unwrap();
        // Should start with lowest latency (Local)
        assert_eq!(current, TransportType::Local);
    }

    #[test]
    fn test_escalation_low_power_policy() {
        let engine = EscalationEngine::new(EscalationPolicy::PreferLowPower);
        let peer_id = create_peer_id(3);

        engine
            .init_peer(
                peer_id,
                vec![
                    TransportType::WiFiDirect,
                    TransportType::BLE,
                    TransportType::Internet,
                ],
            )
            .unwrap();

        let current = engine.current_transport(peer_id).unwrap();
        // Should start with lowest power (BLE)
        assert_eq!(current, TransportType::BLE);
    }

    #[test]
    fn test_escalate_to_better_transport() {
        let engine = EscalationEngine::new(EscalationPolicy::PreferHighBandwidth);
        let peer_id = create_peer_id(4);

        engine
            .init_peer(peer_id, vec![TransportType::BLE, TransportType::WiFiDirect])
            .unwrap();

        assert_eq!(
            engine.current_transport(peer_id).unwrap(),
            TransportType::WiFiDirect
        );
    }

    #[test]
    fn test_deescalate_to_fallback() {
        let engine = EscalationEngine::new(EscalationPolicy::PreferHighBandwidth);
        let peer_id = create_peer_id(5);

        engine
            .init_peer(peer_id, vec![TransportType::BLE, TransportType::WiFiDirect])
            .unwrap();

        let fallback = engine.deescalate(peer_id).unwrap();
        assert!(fallback.is_some());
    }

    #[test]
    fn test_should_escalate_true() {
        let engine = EscalationEngine::new(EscalationPolicy::PreferHighBandwidth);
        let peer_id = create_peer_id(6);

        engine
            .init_peer(
                peer_id,
                vec![
                    TransportType::BLE,
                    TransportType::WiFiDirect,
                    TransportType::Internet,
                ],
            )
            .unwrap();

        // Init picks WiFiDirect (best for high bandwidth), which is already the best
        // So escalation should not be possible
        // To test escalation, we need to manually set peer to a worse transport first
        engine
            .states
            .write()
            .get_mut(&peer_id)
            .unwrap()
            .current_transport = TransportType::BLE;

        // Now should be able to escalate from BLE to WiFiDirect or Internet
        assert!(engine.should_escalate(peer_id));
    }

    #[test]
    fn test_should_escalate_false() {
        let engine = EscalationEngine::new(EscalationPolicy::PreferHighBandwidth);
        let peer_id = create_peer_id(7);

        engine
            .init_peer(peer_id, vec![TransportType::Internet])
            .unwrap();

        // Cannot escalate when at best
        assert!(!engine.should_escalate(peer_id));
    }

    #[test]
    fn test_update_available_transports() {
        let engine = EscalationEngine::new(EscalationPolicy::Balanced);
        let peer_id = create_peer_id(8);

        engine.init_peer(peer_id, vec![TransportType::BLE]).unwrap();

        let result = engine.update_available_transports(
            peer_id,
            vec![TransportType::BLE, TransportType::WiFiDirect],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_cleanup_peer() {
        let engine = EscalationEngine::new(EscalationPolicy::Balanced);
        let peer_id = create_peer_id(9);

        engine.init_peer(peer_id, vec![TransportType::BLE]).unwrap();

        assert!(engine.current_transport(peer_id).is_some());

        engine.cleanup_peer(peer_id);

        assert!(engine.current_transport(peer_id).is_none());
    }

    #[test]
    fn test_all_states() {
        let engine = EscalationEngine::new(EscalationPolicy::Balanced);
        let peer1 = create_peer_id(10);
        let peer2 = create_peer_id(11);

        engine.init_peer(peer1, vec![TransportType::BLE]).unwrap();
        engine
            .init_peer(peer2, vec![TransportType::WiFiDirect])
            .unwrap();

        let all = engine.all_states();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_escalation_policy_default() {
        let default = EscalationPolicy::default();
        assert_eq!(default, EscalationPolicy::Balanced);
    }

    #[test]
    fn test_set_capabilities() {
        let engine = EscalationEngine::new(EscalationPolicy::Balanced);
        let caps = TransportCapabilities::for_transport(TransportType::BLE);

        engine.set_capabilities(TransportType::BLE, caps);

        let peer_id = create_peer_id(12);
        engine.init_peer(peer_id, vec![TransportType::BLE]).unwrap();
        assert!(engine.current_transport(peer_id).is_some());
    }

    #[test]
    fn test_escalation_order_high_bandwidth() {
        let engine = EscalationEngine::new(EscalationPolicy::PreferHighBandwidth);

        let mut scores = vec![];
        for transport in &[
            TransportType::BLE,
            TransportType::WiFiAware,
            TransportType::WiFiDirect,
            TransportType::Internet,
            TransportType::Local,
        ] {
            let score = engine.escalation_score(
                *transport,
                EscalationPolicy::PreferHighBandwidth,
                &Default::default(),
            );
            scores.push((transport, score));
        }

        // Sort by score
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Local should be first (highest bandwidth at 10 Gbps)
        assert_eq!(*scores[0].0, TransportType::Local);
    }

    #[test]
    fn test_escalation_order_low_latency() {
        let engine = EscalationEngine::new(EscalationPolicy::PreferLowLatency);

        let mut scores = vec![];
        for transport in &[
            TransportType::BLE,
            TransportType::WiFiAware,
            TransportType::WiFiDirect,
            TransportType::Internet,
            TransportType::Local,
        ] {
            let score = engine.escalation_score(
                *transport,
                EscalationPolicy::PreferLowLatency,
                &Default::default(),
            );
            scores.push((transport, score));
        }

        // Sort by score
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Local should be first (lowest latency)
        assert_eq!(*scores[0].0, TransportType::Local);
    }

    #[test]
    fn test_escalation_order_low_power() {
        let engine = EscalationEngine::new(EscalationPolicy::PreferLowPower);

        let mut scores = vec![];
        for transport in &[
            TransportType::BLE,
            TransportType::WiFiAware,
            TransportType::WiFiDirect,
            TransportType::Internet,
        ] {
            let score = engine.escalation_score(
                *transport,
                EscalationPolicy::PreferLowPower,
                &Default::default(),
            );
            scores.push((transport, score));
        }

        // Sort by score
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // BLE should be first (lowest power)
        assert_eq!(*scores[0].0, TransportType::BLE);
    }

    // ========================================================================
    // Chunk U1: live-health weighting (single authority for transport order)
    // ========================================================================

    #[test]
    fn test_health_factor_neutral_with_zero_observations() {
        // The neutrality proof: no observations => factor is EXACTLY 1.0,
        // never a smoothed/clamped approximation of it.
        let stats = TransportHealthStats::default();
        assert_eq!(stats.health_factor(), 1.0);
        assert_eq!(stats.success_count + stats.failure_count, 0);
    }

    #[test]
    fn test_health_factor_laplace_smoothed_and_clamped() {
        // All failures: smoothed (0+1)/(0+10+2) = 0.0833... clamps up to 0.25.
        let mut stats = TransportHealthStats {
            failure_count: 10,
            ..Default::default()
        };
        assert!((stats.health_factor() - 0.25_f64).abs() < 1e-12);

        // All successes: smoothed (10+1)/(10+0+2) = 11/12, no clamp needed.
        stats = TransportHealthStats {
            success_count: 10,
            ..Default::default()
        };
        assert!((stats.health_factor() - 11.0 / 12.0).abs() < 1e-12);

        // Single failure is NOT a demotion to the clamp: (5+1)/(5+1+2) = 0.75.
        stats = TransportHealthStats {
            success_count: 5,
            failure_count: 1,
            ..Default::default()
        };
        assert!((stats.health_factor() - 0.75).abs() < 1e-12);
    }

    #[test]
    fn test_unhealthy_preferred_transport_demoted() {
        // Policy prefers WiFiDirect (0.25 Gbps-scale score) over Internet
        // (0.1). After enough WiFiDirect failures the clamp floor (0.25
        // factor) drags WiFiDirect to 0.0625 -- below healthy Internet.
        let engine = EscalationEngine::new(EscalationPolicy::PreferHighBandwidth);
        let peer = create_peer_id(20);
        let available = [TransportType::Internet, TransportType::WiFiDirect];

        // Sanity: without health data the policy winner is WiFiDirect.
        assert_eq!(
            engine.select_best_transport_for_peer(&peer, &available),
            TransportType::WiFiDirect
        );

        for _ in 0..10 {
            engine.record_delivery_outcome(peer, TransportType::WiFiDirect, false, 0);
        }

        assert_eq!(
            engine.select_best_transport_for_peer(&peer, &available),
            TransportType::Internet
        );

        // And the ordering surface agrees (best-first).
        assert_eq!(
            engine.preferred_order(&peer, &available),
            vec![TransportType::Internet, TransportType::WiFiDirect]
        );
    }

    #[test]
    fn test_unhealthy_preferred_transport_demoted_balanced() {
        // Same demotion under the default (Balanced) policy -- the policy
        // every mobile bridge engine instance uses.
        let engine = EscalationEngine::new(EscalationPolicy::Balanced);
        let peer = create_peer_id(21);
        let available = [TransportType::WiFiDirect, TransportType::BLE];

        assert_eq!(
            engine.select_best_transport_for_peer(&peer, &available),
            TransportType::WiFiDirect
        );

        for _ in 0..20 {
            engine.record_delivery_outcome(peer, TransportType::WiFiDirect, false, 0);
        }

        assert_eq!(
            engine.select_best_transport_for_peer(&peer, &available),
            TransportType::BLE
        );
    }

    #[test]
    fn test_neutral_health_preserves_legacy_selection_exactly() {
        // The legacy-behavior pin: with zero observations for EVERY transport
        // in the comparison set, the peer-aware weighted winner equals the
        // health-blind policy winner for all policies and lists the legacy
        // tests exercised. This is the regression proof for the no-health
        // path: multiply-by-exactly-1.0 cannot reorder anything.
        for policy in [
            EscalationPolicy::PreferHighBandwidth,
            EscalationPolicy::PreferLowLatency,
            EscalationPolicy::PreferLowPower,
            EscalationPolicy::Balanced,
        ] {
            let engine = EscalationEngine::new(policy);
            let peer = create_peer_id(22);
            let lists = [
                vec![TransportType::BLE, TransportType::WiFiDirect, TransportType::Internet],
                vec![TransportType::BLE, TransportType::WiFiAware, TransportType::Local],
                vec![TransportType::Internet, TransportType::Local],
            ];
            for list in lists {
                assert_eq!(
                    engine.select_best_transport_for_peer(&peer, &list),
                    engine.select_best_transport(&list),
                    "policy {:?} list {:?}: weighted != policy-only with zero health",
                    policy,
                    list
                );
            }
        }
    }

    #[test]
    fn test_recorded_health_is_per_peer() {
        // Health is (peer, transport) keyed: peer 2's failures must not
        // demote the transport for peer 3.
        let engine = EscalationEngine::new(EscalationPolicy::PreferHighBandwidth);
        let unhealthy_peer = create_peer_id(23);
        let healthy_peer = create_peer_id(24);
        let available = [TransportType::BLE, TransportType::WiFiDirect];

        for _ in 0..10 {
            engine.record_delivery_outcome(unhealthy_peer, TransportType::WiFiDirect, false, 0);
        }

        assert_eq!(
            engine.select_best_transport_for_peer(&healthy_peer, &available),
            TransportType::WiFiDirect
        );
        assert!(engine.health_stats(&healthy_peer, TransportType::WiFiDirect).is_none());
        assert_eq!(
            engine
                .health_stats(&unhealthy_peer, TransportType::WiFiDirect)
                .map(|s| s.failure_count),
            Some(10)
        );
    }

    #[test]
    fn test_latency_ewma_recorded_on_success_only() {
        let engine = EscalationEngine::new(EscalationPolicy::Balanced);
        let peer = create_peer_id(25);

        engine.record_delivery_outcome(peer, TransportType::Internet, true, 100);
        engine.record_delivery_outcome(peer, TransportType::Internet, true, 300);
        engine.record_delivery_outcome(peer, TransportType::Internet, false, 9999);

        let stats = engine.health_stats(&peer, TransportType::Internet).unwrap();
        assert_eq!(stats.success_count, 2);
        assert_eq!(stats.failure_count, 1);
        // First sample seeds (100), second pulls a quarter toward 300: 150.
        // The failure carries no latency sample.
        assert!((stats.latency_ewma_ms - 150.0).abs() < 1e-9);
    }

    #[test]
    fn test_preferred_order_stable_and_best_first() {
        let engine = EscalationEngine::new(EscalationPolicy::Balanced);
        let peer = create_peer_id(26);

        let order = engine.preferred_order(
            &peer,
            &[
                TransportType::Internet,
                TransportType::WiFiDirect,
                TransportType::BLE,
            ],
        );
        // Balanced raw order: WiFiDirect > Internet > BLE (stable ties keep input order).
        assert_eq!(
            order,
            vec![
                TransportType::WiFiDirect,
                TransportType::Internet,
                TransportType::BLE
            ]
        );

        // Demote WiFiDirect for this peer: 328.6 * 0.25 = 82.1 drops it
        // below Internet (300.0) AND BLE (285.0).
        for _ in 0..20 {
            engine.record_delivery_outcome(peer, TransportType::WiFiDirect, false, 0);
        }
        let order = engine.preferred_order(
            &peer,
            &[
                TransportType::Internet,
                TransportType::WiFiDirect,
                TransportType::BLE,
            ],
        );
        assert_eq!(
            order,
            vec![
                TransportType::Internet,
                TransportType::BLE,
                TransportType::WiFiDirect
            ]
        );
    }

    #[test]
    fn test_health_cleared_with_cleanup_peer() {
        let engine = EscalationEngine::new(EscalationPolicy::Balanced);
        let peer = create_peer_id(27);
        engine.record_delivery_outcome(peer, TransportType::BLE, true, 10);
        assert!(engine.health_stats(&peer, TransportType::BLE).is_some());
        engine.cleanup_peer(peer);
        assert!(engine.health_stats(&peer, TransportType::BLE).is_none());
    }

    #[test]
    fn test_transport_type_from_name_round_trip() {
        for t in [
            TransportType::BLE,
            TransportType::WiFiAware,
            TransportType::WiFiDirect,
            TransportType::Internet,
            TransportType::Local,
        ] {
            assert_eq!(transport_type_from_name(&t.to_string()), Some(t));
        }
        assert_eq!(transport_type_from_name("tcp_mdns"), None);
        assert_eq!(transport_type_from_name(""), None);
    }

    #[test]
    fn test_deescalate_uses_health_weighted_scores() {
        // Deescalation must also honor live health: a "worse" tier whose
        // health collapsed is no longer the chosen fallback. Balanced raw
        // scores: WiFiDirect 328.6 > Internet 300.0 > BLE 285.0, so the
        // natural fallback from WiFiDirect is Internet; 20 failures on
        // Internet demote it to 75.0 and BLE (285.0) takes over.
        let engine = EscalationEngine::new(EscalationPolicy::Balanced);
        let peer = create_peer_id(28);

        engine
            .init_peer(
                peer,
                vec![TransportType::BLE, TransportType::WiFiDirect, TransportType::Internet],
            )
            .unwrap();
        assert_eq!(
            engine.deescalate(peer).unwrap(),
            Some(TransportType::Internet)
        );

        // Reset current back to WiFiDirect before poisoning Internet.
        engine
            .states
            .write()
            .get_mut(&peer)
            .unwrap()
            .current_transport = TransportType::WiFiDirect;
        for _ in 0..20 {
            engine.record_delivery_outcome(peer, TransportType::Internet, false, 0);
        }

        let fallback = engine.deescalate(peer).unwrap();
        assert_eq!(fallback, Some(TransportType::BLE));
    }
}
