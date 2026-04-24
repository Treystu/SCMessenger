//! Mock Routing components for testing
//!
//! Provides mock implementations for routing engine and reputation tracking.

use libp2p::PeerId;
use scmessenger_core::routing::engine::{RoutingDecision, RoutingEngine};

/// Mock RoutingEngine that provides controlled routing decisions for testing
#[derive(Debug, Clone)]
pub struct MockRoutingEngine {
    pub decisions: Vec<RoutingDecision>,
    pub round_robin_index: usize,
}

impl MockRoutingEngine {
    /// Create a new mock routing engine with default behavior
    pub fn new() -> Self {
        Self {
            decisions: Vec::new(),
            round_robin_index: 0,
        }
    }

    /// Create a mock routing engine with a specific set of pre-recorded decisions
    pub fn with_decisions(decisions: Vec<RoutingDecision>) -> Self {
        Self {
            decisions,
            round_robin_index: 0,
        }
    }

    /// Get the next decision (round-robin through the list)
    pub fn next_decision(&mut self) -> RoutingDecision {
        if self.decisions.is_empty() {
            // Default decision: direct route to target
            RoutingDecision::Direct
        } else {
            let decision = self.decisions[self.round_robin_index % self.decisions.len()].clone();
            self.round_robin_index = (self.round_robin_index + 1) % self.decisions.len();
            decision
        }
    }

    /// Add a decision to the mock
    pub fn add_decision(&mut self, decision: RoutingDecision) {
        self.decisions.push(decision);
    }

    /// Reset the round-robin index
    pub fn reset(&mut self) {
        self.round_robin_index = 0;
    }
}

impl Default for MockRoutingEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a mock routing engine that always returns direct routes
pub fn create_mock_direct_routing() -> MockRoutingEngine {
    MockRoutingEngine::with_decisions(vec![RoutingDecision::Direct; 10])
}

/// Create a mock routing engine that always returns relay routes
pub fn create_mock_relay_routing() -> MockRoutingEngine {
    MockRoutingEngine::with_decisions(vec![RoutingDecision::Relay; 10])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_routing_engine_creation() {
        let engine = MockRoutingEngine::new();
        assert!(engine.decisions.is_empty());
        assert_eq!(engine.round_robin_index, 0);
    }

    #[test]
    fn test_mock_routing_round_robin() {
        let mut engine = MockRoutingEngine::with_decisions(vec![
            RoutingDecision::Direct,
            RoutingDecision::Relay,
        ]);

        assert_eq!(engine.next_decision(), RoutingDecision::Direct);
        assert_eq!(engine.next_decision(), RoutingDecision::Relay);
        assert_eq!(engine.next_decision(), RoutingDecision::Direct); // wrapped
        assert_eq!(engine.next_decision(), RoutingDecision::Relay); // wrapped
    }

    #[test]
    fn test_mock_routing_with_empty_decisions() {
        let mut engine = MockRoutingEngine::new();
        // Default decision should be Direct when no decisions are configured
        assert_eq!(engine.next_decision(), RoutingDecision::Direct);
    }

    #[test]
    fn test_mock_routing_reset() {
        let mut engine = MockRoutingEngine::with_decisions(vec![
            RoutingDecision::Direct,
            RoutingDecision::Relay,
        ]);

        engine.next_decision(); // advance index
        assert_eq!(engine.round_robin_index, 1);

        engine.reset();
        assert_eq!(engine.round_robin_index, 0);
        assert_eq!(engine.next_decision(), RoutingDecision::Direct);
    }
}
