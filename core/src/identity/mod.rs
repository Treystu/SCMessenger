// Identity & Crypto - Foundation for KERI support (Phase 4)

mod store;
mod keys;

pub use store::IdentityStore;
pub use keys::{KeyPair, IdentityKeys};

use anyhow::Result;

/// Manages node identity and cryptographic keys
pub struct IdentityManager {
    store: IdentityStore,
    keys: Option<IdentityKeys>,
}

impl IdentityManager {
    /// Create a new identity manager with in-memory storage
    pub fn new() -> Self {
        Self {
            store: IdentityStore::memory(),
            keys: None,
        }
    }

    /// Create a new identity manager with persistent storage
    pub fn with_path(path: &str) -> Result<Self> {
        Ok(Self {
            store: IdentityStore::persistent(path)?,
            keys: None,
        })
    }

    /// Generate or load identity keys
    pub fn initialize(&mut self) -> Result<()> {
        // Try to load existing keys
        if let Some(keys) = self.store.load_keys()? {
            tracing::info!("🔑 Loaded existing identity");
            self.keys = Some(keys);
        } else {
            // Generate new keys
            tracing::info!("🔑 Generating new identity");
            let keys = IdentityKeys::generate();
            self.store.save_keys(&keys)?;
            self.keys = Some(keys);
        }

        Ok(())
    }

    /// Get identity keys (if initialized)
    pub fn keys(&self) -> Option<&IdentityKeys> {
        self.keys.as_ref()
    }

    /// Get identity public key as hex string
    pub fn public_key_hex(&self) -> Option<String> {
        self.keys.as_ref().map(|k| k.public_key_hex())
    }

    /// Get identity ID (hash of public key)
    pub fn identity_id(&self) -> Option<String> {
        self.keys.as_ref().map(|k| k.identity_id())
    }

    /// Sign data with identity key
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        match &self.keys {
            Some(keys) => keys.sign(data),
            None => Err(anyhow::anyhow!("Identity not initialized")),
        }
    }

    /// Verify signature
    pub fn verify(&self, data: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool> {
        IdentityKeys::verify(data, signature, public_key)
    }
}

impl Default for IdentityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_manager_creation() {
        let manager = IdentityManager::new();
        assert!(manager.keys().is_none());
    }

    #[test]
    fn test_identity_initialization() {
        let mut manager = IdentityManager::new();
        manager.initialize().unwrap();

        assert!(manager.keys().is_some());
        assert!(manager.public_key_hex().is_some());
        assert!(manager.identity_id().is_some());
    }

    #[test]
    fn test_identity_signing() {
        let mut manager = IdentityManager::new();
        manager.initialize().unwrap();

        let data = b"test message";
        let signature = manager.sign(data).unwrap();

        assert!(!signature.is_empty());
    }

    #[test]
    fn test_identity_verification() {
        let mut manager = IdentityManager::new();
        manager.initialize().unwrap();

        let data = b"test message";
        let signature = manager.sign(data).unwrap();

        let keys = manager.keys().unwrap();
        let public_key = keys.signing_key.verifying_key().to_bytes();

        let valid = manager.verify(data, &signature, &public_key).unwrap();
        assert!(valid);

        // Test invalid signature
        let invalid = manager.verify(b"wrong data", &signature, &public_key).unwrap();
        assert!(!invalid);
    }

    #[test]
    fn test_identity_persistence() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("test_identity").to_str().unwrap().to_string();

        // Create and initialize
        let mut manager1 = IdentityManager::with_path(&path).unwrap();
        manager1.initialize().unwrap();
        let id1 = manager1.identity_id().unwrap();

        drop(manager1);

        // Load existing
        let mut manager2 = IdentityManager::with_path(&path).unwrap();
        manager2.initialize().unwrap();
        let id2 = manager2.identity_id().unwrap();

        assert_eq!(id1, id2);
    }
}
