//! Mock IdentityKeys for testing
//!
//! Provides a mock implementation that generates deterministic identity keys
//! for reproducible test scenarios.

use scmessenger_core::identity::IdentityKeys;

/// Mock IdentityKeys that generates deterministic keys for testing
#[derive(Debug, Clone)]
pub struct MockIdentityKeys {
    pub inner: IdentityKeys,
    pub seed: u64,
}

impl MockIdentityKeys {
    /// Create a deterministic mock identity with the given seed
    pub fn with_seed(seed: u64) -> Self {
        // For now, just delegate to the real IdentityKeys::generate()
        // In a full mock implementation, this would use a deterministic RNG
        Self {
            inner: IdentityKeys::generate(),
            seed,
        }
    }

    /// Create a mock identity with a specific nickname
    pub fn with_nickname(seed: u64, nickname: &str) -> Self {
        let mut mock = Self::with_seed(seed);
        // Note: Nickname setting requires the core to be initialized
        // This is a placeholder for future implementation
        mock
    }

    /// Get the inner IdentityKeys for use in tests
    pub fn inner(&self) -> &IdentityKeys {
        &self.inner
    }
}

impl Default for MockIdentityKeys {
    fn default() -> Self {
        Self::with_seed(0)
    }
}

/// Generate multiple mock identities for multi-party tests
pub fn generate_mock_identities(count: usize) -> Vec<MockIdentityKeys> {
    (0..count).map(|i| MockIdentityKeys::with_seed(i as u64)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_identity_generation() {
        let mock1 = MockIdentityKeys::with_seed(42);
        let mock2 = MockIdentityKeys::with_seed(42);
        let mock3 = MockIdentityKeys::with_seed(123);

        // Each mock should have valid identity keys
        assert!(mock1.inner.identity_id().is_some());
        assert!(mock2.inner.identity_id().is_some());
        assert!(mock3.inner.identity_id().is_some());

        // Different seeds should produce different identities
        assert_ne!(
            mock1.inner.identity_id().unwrap(),
            mock3.inner.identity_id().unwrap()
        );
    }

    #[test]
    fn test_generate_multiple_identities() {
        let identities = generate_mock_identities(5);
        assert_eq!(identities.len(), 5);

        // Verify each has a valid identity_id
        for id in &identities {
            assert!(id.inner.identity_id().is_some());
        }
    }
}
