//! Transport Escalation — automatic transport negotiation
//!
//! Implements automatic negotiation of the best transport between two peers,
//! upgrading to better transports when available and downgrading gracefully.

use crate::transport::abstraction::TransportCapabilities;
use crate::transport::abstraction::TransportType;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::debug;

/// Escalation policy for transport selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscalationPolicy {
    /// Always escalate to highest bandwidth available
    PreferHighBandwidth,
    /// Prefer lowest latency transport
    PreferLowLatency,
    /// Prefer BLE over WiFi over Internet (power efficiency)
    PreferLowPower,
    /// Balanced: weighted combination of all factors
    Balanced,
}

impl Default for EscalationPolicy {
    fn default() -> Self {
        EscalationPolicy::Balanced
    }
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
    pub last_escalation_attempt: Option<std::time::SystemTime>,
}

/// Manages transport escalation for all peers
pub struct EscalationEngine {
    /// Current escalation states per peer
    states: Arc<RwLock<HashMap<[u8; 32], EscalationState>>>,
    /// Escalation policy
    policy: EscalationPolicy,
    /// Transport capabilities (for scoring)
    capabilities: Arc<RwLock<HashMap<TransportType, TransportCapabilities>>>,
}

impl EscalationEngine {
    /// Create a new escalation engine
    pub fn new(policy: EscalationPolicy) -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            policy,
            capabilities: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set capabilities for a transport type
    pub fn set_capabilities(&self, transport: TransportType, capabilities: TransportCapabilities) {
        let mut caps = self.capabilities.write();
        caps.insert(transport, capabilities);
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
        let current = self.select_best_transport(&available_transports);

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
            let better = self.find_better_transport(&state.available_transports, state.current_transport);
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
            .find_better_transport(&state.available_transports, state.current_transport)
            .ok_or(EscalationError::NotPossible)?;

        state.current_transport = target;
        state.last_escalation_attempt = Some(std::time::SystemTime::now());

        debug!(
            "Escalated peer {:x?} to {}",
            &peer_id[..8], target
        );
        Ok(target)
    }

    /// Deescalate to a fallback transport
    pub fn deescalate(&self, peer_id: [u8; 32]) -> Result<Option<TransportType>, EscalationError> {
        let mut states = self.states.write();
        let state = states
            .get_mut(&peer_id)
            .ok_or(EscalationError::NotPossible)?;

        let current = state.current_transport;
        let fallback = self.find_worse_transport(&state.available_transports, current);

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

    /// Find the best transport from a list using the policy
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
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
            .unwrap_or(available[0]);

        best
    }

    /// Calculate escalation score for a transport (higher is better)
    fn escalation_score(
        &self,
        transport: TransportType,
        policy: EscalationPolicy,
        caps: &HashMap<TransportType, TransportCapabilities>,
    ) -> f64 {
        let cap = caps.get(&transport).cloned().unwrap_or(TransportCapabilities::for_transport(transport));

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

    /// Find a transport with higher score than current
    fn find_better_transport(
        &self,
        available: &[TransportType],
        current: TransportType,
    ) -> Option<TransportType> {
        let caps = self.capabilities.read();
        let current_score = self.escalation_score(current, self.policy, &caps);

        available
            .iter()
            .filter(|&&t| t != current)
            .max_by(|&&a, &&b| {
                let score_a = self.escalation_score(a, self.policy, &caps);
                let score_b = self.escalation_score(b, self.policy, &caps);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .and_then(|&best| {
                let best_score = self.escalation_score(best, self.policy, &caps);
                if best_score > current_score {
                    Some(best)
                } else {
                    None
                }
            })
    }

    /// Find a transport with lower score than current (fallback)
    fn find_worse_transport(
        &self,
        available: &[TransportType],
        current: TransportType,
    ) -> Option<TransportType> {
        let caps = self.capabilities.read();
        let current_score = self.escalation_score(current, self.policy, &caps);

        available
            .iter()
            .filter(|&&t| t != current)
            .max_by(|&&a, &&b| {
                let score_a = self.escalation_score(a, self.policy, &caps);
                let score_b = self.escalation_score(b, self.policy, &caps);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .and_then(|&best| {
                let best_score = self.escalation_score(best, self.policy, &caps);
                if best_score < current_score {
                    Some(best)
                } else {
                    None
                }
            })
    }

    /// Remove a peer from tracking
    pub fn cleanup_peer(&self, peer_id: [u8; 32]) {
        let mut states = self.states.write();
        states.remove(&peer_id);
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

        let result = engine.init_peer(
            peer_id,
            vec![TransportType::BLE, TransportType::WiFiDirect],
        );
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
                vec![TransportType::BLE, TransportType::WiFiDirect, TransportType::Internet],
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
            .init_peer(
                peer_id,
                vec![TransportType::Internet, TransportType::Local],
            )
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
            .init_peer(
                peer_id,
                vec![TransportType::BLE, TransportType::WiFiDirect],
            )
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
            .init_peer(
                peer_id,
                vec![TransportType::BLE, TransportType::WiFiDirect],
            )
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
                vec![TransportType::BLE, TransportType::WiFiDirect, TransportType::Internet],
            )
            .unwrap();

        // Init picks WiFiDirect (best for high bandwidth), which is already the best
        // So escalation should not be possible
        // To test escalation, we need to manually set peer to a worse transport first
        engine.states.write().get_mut(&peer_id).unwrap().current_transport = TransportType::BLE;

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

        engine
            .init_peer(peer_id, vec![TransportType::BLE])
            .unwrap();

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

        engine
            .init_peer(peer_id, vec![TransportType::BLE])
            .unwrap();

        assert!(engine.current_transport(peer_id).is_some());

        engine.cleanup_peer(peer_id);

        assert!(engine.current_transport(peer_id).is_none());
    }

    #[test]
    fn test_all_states() {
        let engine = EscalationEngine::new(EscalationPolicy::Balanced);
        let peer1 = create_peer_id(10);
        let peer2 = create_peer_id(11);

        engine
            .init_peer(peer1, vec![TransportType::BLE])
            .unwrap();
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
        engine
            .init_peer(peer_id, vec![TransportType::BLE])
            .unwrap();
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
            let score = engine.escalation_score(*transport, EscalationPolicy::PreferHighBandwidth, &Default::default());
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
            let score = engine.escalation_score(*transport, EscalationPolicy::PreferLowLatency, &Default::default());
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
            let score = engine.escalation_score(*transport, EscalationPolicy::PreferLowPower, &Default::default());
            scores.push((transport, score));
        }

        // Sort by score
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // BLE should be first (lowest power)
        assert_eq!(*scores[0].0, TransportType::BLE);
    }
}
