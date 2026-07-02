//! Mock implementations for testing
//!
//! This module provides mock implementations for components that are
//! expensive or impossible to test with real implementations in integration tests.
//!
//! Usage:
//! ```
//! use scmessenger_core::tests::mocks;
//! ```

// Mock IdentityKeys for testing
pub mod identity;

// Mock Transport components
pub mod transport;

// Mock Routing components
pub mod routing;

// Re-export for convenience
pub use identity::MockIdentityKeys;
pub use transport::MockSwarmHandle;
pub use routing::MockRoutingEngine;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_identity_keys() {
        let mock = MockIdentityKeys::with_seed(42);
        assert!(mock.inner.identity_id().is_some());
    }

    #[test]
    fn test_mock_transport() {
        let swarm = MockSwarmHandle::new(PeerId::random());
        assert!(swarm.peer_id().to_bytes().len() > 0);
    }

    #[test]
    fn test_mock_routing() {
        let engine = MockRoutingEngine::new();
        assert_eq!(engine.next_decision(), RoutingDecision::Direct);
    }
}
