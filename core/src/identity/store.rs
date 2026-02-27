// Identity storage using sled

use super::IdentityKeys;
use crate::store::backend::StorageBackend;
use anyhow::Result;
use std::sync::Arc;

const IDENTITY_KEY: &[u8] = b"identity_keys";
const NICKNAME_KEY: &[u8] = b"identity_nickname";

/// Storage backend for identity keys
pub enum IdentityStore {
    Memory,
    Persistent(Arc<dyn StorageBackend>),
}

impl IdentityStore {
    /// Create in-memory storage
    pub fn memory() -> Self {
        Self::Memory
    }

    /// Create persistent storage
    pub fn persistent(backend: Arc<dyn StorageBackend>) -> Self {
        Self::Persistent(backend)
    }

    /// Save keys to storage
    pub fn save_keys(&self, keys: &IdentityKeys) -> Result<()> {
        match self {
            Self::Memory => {
                // In-memory storage doesn't persist
                Ok(())
            }
            Self::Persistent(db) => {
                let bytes = keys.to_bytes();
                db.put(IDENTITY_KEY, &bytes)
                    .map_err(|e| anyhow::anyhow!(e))?;
                db.flush().map_err(|e| anyhow::anyhow!(e))?;
                Ok(())
            }
        }
    }

    /// Save nickname to storage
    pub fn save_nickname(&self, nickname: &str) -> Result<()> {
        match self {
            Self::Memory => Ok(()), // Memory store doesn't persist
            Self::Persistent(db) => {
                db.put(NICKNAME_KEY, nickname.as_bytes())
                    .map_err(|e| anyhow::anyhow!(e))?;
                db.flush().map_err(|e| anyhow::anyhow!(e))?;
                Ok(())
            }
        }
    }

    /// Load keys from storage
    pub fn load_keys(&self) -> Result<Option<IdentityKeys>> {
        match self {
            Self::Memory => {
                // In-memory storage always returns None
                Ok(None)
            }
            Self::Persistent(db) => {
                if let Some(bytes) = db.get(IDENTITY_KEY).map_err(|e| anyhow::anyhow!(e))? {
                    let keys = IdentityKeys::from_bytes(&bytes)?;
                    Ok(Some(keys))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Load nickname from storage
    pub fn load_nickname(&self) -> Result<Option<String>> {
        match self {
            Self::Memory => Ok(None),
            Self::Persistent(db) => {
                if let Some(bytes) = db.get(NICKNAME_KEY).map_err(|e| anyhow::anyhow!(e))? {
                    Ok(Some(String::from_utf8(bytes)?))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Clear stored keys
    pub fn clear(&self) -> Result<()> {
        match self {
            Self::Memory => Ok(()),
            Self::Persistent(db) => {
                db.remove(IDENTITY_KEY).map_err(|e| anyhow::anyhow!(e))?;
                db.remove(NICKNAME_KEY).map_err(|e| anyhow::anyhow!(e))?;
                db.flush().map_err(|e| anyhow::anyhow!(e))?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_memory_store() {
        let store = IdentityStore::memory();
        let keys = IdentityKeys::generate();

        store.save_keys(&keys).unwrap();

        // Memory store doesn't persist
        let loaded = store.load_keys().unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_persistent_store() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_store").to_str().unwrap().to_string();

        let backend = Arc::new(crate::store::backend::SledStorage::new(&path).unwrap());
        let store = IdentityStore::persistent(backend);
        let keys = IdentityKeys::generate();

        // Save
        store.save_keys(&keys).unwrap();

        // Load
        let loaded = store.load_keys().unwrap();
        assert!(loaded.is_some());

        let loaded_keys = loaded.unwrap();
        assert_eq!(keys.identity_id(), loaded_keys.identity_id());
    }

    #[test]
    fn test_store_clear() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_store").to_str().unwrap().to_string();

        let backend = Arc::new(crate::store::backend::SledStorage::new(&path).unwrap());
        let store = IdentityStore::persistent(backend);
        let keys = IdentityKeys::generate();

        store.save_keys(&keys).unwrap();
        store.clear().unwrap();

        let loaded = store.load_keys().unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_store_persistence_across_instances() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_store").to_str().unwrap().to_string();

        let keys = IdentityKeys::generate();
        let id = keys.identity_id();

        // Save in first instance
        {
            let backend = Arc::new(crate::store::backend::SledStorage::new(&path).unwrap());
            let store = IdentityStore::persistent(backend);
            store.save_keys(&keys).unwrap();
        }

        // Load in second instance
        {
            let backend2 = Arc::new(crate::store::backend::SledStorage::new(&path).unwrap());
            let store = IdentityStore::persistent(backend2);
            let loaded = store.load_keys().unwrap().unwrap();
            assert_eq!(id, loaded.identity_id());
        }
    }
}
