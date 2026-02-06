// Identity storage using sled

use super::IdentityKeys;
use anyhow::Result;

const IDENTITY_KEY: &[u8] = b"identity_keys";

/// Storage backend for identity keys
pub enum IdentityStore {
    Memory,
    Persistent(sled::Db),
}

impl IdentityStore {
    /// Create in-memory storage
    pub fn memory() -> Self {
        Self::Memory
    }

    /// Create persistent storage
    pub fn persistent(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self::Persistent(db))
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
                db.insert(IDENTITY_KEY, bytes.as_slice())?;
                db.flush()?;
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
                if let Some(bytes) = db.get(IDENTITY_KEY)? {
                    let keys = IdentityKeys::from_bytes(&bytes)?;
                    Ok(Some(keys))
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
                db.remove(IDENTITY_KEY)?;
                db.flush()?;
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

        let store = IdentityStore::persistent(&path).unwrap();
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

        let store = IdentityStore::persistent(&path).unwrap();
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
            let store = IdentityStore::persistent(&path).unwrap();
            store.save_keys(&keys).unwrap();
        }

        // Load in second instance
        {
            let store = IdentityStore::persistent(&path).unwrap();
            let loaded = store.load_keys().unwrap().unwrap();
            assert_eq!(id, loaded.identity_id());
        }
    }
}
