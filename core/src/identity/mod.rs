// Identity & Crypto - Foundation for KERI support (Phase 4)

mod keys;
mod store;

pub use keys::{IdentityKeys, KeyPair};
pub use store::IdentityStore;

use anyhow::Result;

/// Manages node identity and cryptographic keys
pub struct IdentityManager {
    store: IdentityStore,
    keys: Option<IdentityKeys>,
    nickname: Option<String>,
}

impl IdentityManager {
    /// Create a new identity manager with in-memory storage
    pub fn new() -> Self {
        Self {
            store: IdentityStore::memory(),
            keys: None,
            nickname: None,
        }
    }

    /// Create a new identity manager with persistent storage
    pub fn with_path(path: &str) -> Result<Self> {
        let mut manager = Self {
            store: IdentityStore::persistent(path)?,
            keys: None,
            nickname: None,
        };
        // Load any previously-persisted identity material without generating
        // a new identity. Fresh installs remain uninitialized.
        manager.hydrate_from_store()?;
        Ok(manager)
    }

    fn hydrate_from_store(&mut self) -> Result<()> {
        if let Some(nickname) = self.store.load_nickname()? {
            self.nickname = Some(nickname);
        }
        if let Some(keys) = self.store.load_keys()? {
            self.keys = Some(keys);
        }
        Ok(())
    }

    /// Generate or load identity keys
    pub fn initialize(&mut self) -> Result<()> {
        self.hydrate_from_store()?;

        if self.keys.is_some() {
            tracing::info!("🔑 Loaded existing identity");
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

    /// Set nickname
    pub fn set_nickname(&mut self, nickname: String) -> Result<()> {
        self.store.save_nickname(&nickname)?;
        self.nickname = Some(nickname);
        Ok(())
    }

    /// Get nickname
    pub fn nickname(&self) -> Option<String> {
        self.nickname.clone()
    }

    /// Export raw identity key bytes for secure platform backup.
    pub fn export_key_bytes(&self) -> Option<Vec<u8>> {
        self.keys.as_ref().map(|keys| keys.to_bytes())
    }

    /// Import raw identity key bytes and persist them in the configured store.
    pub fn import_key_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        let keys = IdentityKeys::from_bytes(bytes)?;
        self.store.save_keys(&keys)?;
        self.keys = Some(keys);
        Ok(())
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
        let invalid = manager
            .verify(b"wrong data", &signature, &public_key)
            .unwrap();
        assert!(!invalid);
    }

    #[test]
    fn test_identity_persistence() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir
            .path()
            .join("test_identity")
            .to_str()
            .unwrap()
            .to_string();

        // Create and initialize
        let mut manager1 = IdentityManager::with_path(&path).unwrap();
        manager1.initialize().unwrap();
        manager1.set_nickname("Alice".to_string()).unwrap();
        let id1 = manager1.identity_id().unwrap();

        drop(manager1);

        // Load existing
        let mut manager2 = IdentityManager::with_path(&path).unwrap();
        manager2.initialize().unwrap();
        let id2 = manager2.identity_id().unwrap();
        let nick2 = manager2.nickname();

        assert_eq!(id1, id2);
        assert_eq!(nick2, Some("Alice".to_string()));
    }

    #[test]
    fn test_identity_import_export_roundtrip() {
        let mut manager1 = IdentityManager::new();
        manager1.initialize().unwrap();
        let exported = manager1.export_key_bytes().unwrap();
        let original_id = manager1.identity_id();
        let original_pub = manager1.public_key_hex();

        let mut manager2 = IdentityManager::new();
        manager2.import_key_bytes(&exported).unwrap();

        assert_eq!(manager2.identity_id(), original_id);
        assert_eq!(manager2.public_key_hex(), original_pub);
    }

    #[test]
    fn test_with_path_hydrates_existing_identity_without_initialize() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir
            .path()
            .join("existing_identity")
            .to_str()
            .unwrap()
            .to_string();

        {
            let mut manager = IdentityManager::with_path(&path).unwrap();
            manager.initialize().unwrap();
            manager.set_nickname("PersistedNick".to_string()).unwrap();
        }

        let manager = IdentityManager::with_path(&path).unwrap();
        assert!(manager.keys().is_some());
        assert_eq!(manager.nickname(), Some("PersistedNick".to_string()));
    }
}
