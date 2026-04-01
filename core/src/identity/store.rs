// Identity storage using sled

use super::IdentityKeys;
use crate::store::backend::StorageBackend;
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;
use web_time::{SystemTime, UNIX_EPOCH};

const IDENTITY_KEY: &[u8] = b"identity_keys";
const NICKNAME_KEY: &[u8] = b"identity_nickname";
const DEVICE_ID_KEY: &[u8] = b"identity_device_id";
const SENIORITY_TIMESTAMP_KEY: &[u8] = b"identity_seniority_timestamp";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceMetadata {
    pub device_id: String,
    pub seniority_timestamp: u64,
}

impl DeviceMetadata {
    pub fn generate() -> Self {
        Self {
            device_id: Uuid::new_v4().to_string(),
            seniority_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

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

    /// Save device metadata to storage
    pub fn save_device_metadata(&self, metadata: &DeviceMetadata) -> Result<()> {
        match self {
            Self::Memory => Ok(()),
            Self::Persistent(db) => {
                db.put(DEVICE_ID_KEY, metadata.device_id.as_bytes())
                    .map_err(|e| anyhow::anyhow!(e))?;
                db.put(
                    SENIORITY_TIMESTAMP_KEY,
                    metadata.seniority_timestamp.to_string().as_bytes(),
                )
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

    /// Load persisted device metadata from storage.
    pub fn load_device_metadata(&self) -> Result<Option<DeviceMetadata>> {
        match self {
            Self::Memory => Ok(None),
            Self::Persistent(db) => {
                let device_id = db.get(DEVICE_ID_KEY).map_err(|e| anyhow::anyhow!(e))?;
                let seniority = db
                    .get(SENIORITY_TIMESTAMP_KEY)
                    .map_err(|e| anyhow::anyhow!(e))?;

                match (device_id, seniority) {
                    (None, None) => {
                        tracing::debug!("identity device metadata not yet present in store");
                        Ok(None)
                    }
                    (Some(device_id), Some(seniority)) => {
                        let device_id = String::from_utf8(device_id)?;
                        let parsed_uuid = Uuid::parse_str(&device_id)
                            .map_err(|e| anyhow::anyhow!("invalid stored device_id: {e}"))?;
                        if parsed_uuid.get_version_num() != 4 {
                            return Err(anyhow::anyhow!(
                                "invalid stored device_id version: expected UUIDv4"
                            ));
                        }
                        let seniority_timestamp =
                            String::from_utf8(seniority)?.parse::<u64>().map_err(|e| {
                                anyhow::anyhow!("invalid stored seniority_timestamp: {e}")
                            })?;
                        Ok(Some(DeviceMetadata {
                            device_id,
                            seniority_timestamp,
                        }))
                    }
                    _ => {
                        tracing::warn!(
                            "identity device metadata is partially present; regenerating missing WS13.1 local metadata"
                        );
                        Ok(None)
                    }
                }
            }
        }
    }

    /// Load device metadata or generate and persist a new installation-local value.
    pub fn load_or_create_device_metadata(&self) -> Result<DeviceMetadata> {
        if let Some(metadata) = self.load_device_metadata()? {
            return Ok(metadata);
        }

        let metadata = DeviceMetadata::generate();
        self.save_device_metadata(&metadata)?;
        Ok(metadata)
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

    /// Load device ID from storage
    pub fn load_device_id(&self) -> Result<Option<String>> {
        match self {
            Self::Memory => Ok(None),
            Self::Persistent(db) => {
                if let Some(bytes) = db.get(DEVICE_ID_KEY).map_err(|e| anyhow::anyhow!(e))? {
                    Ok(Some(String::from_utf8(bytes)?))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Load seniority timestamp from storage
    pub fn load_seniority_timestamp(&self) -> Result<Option<u64>> {
        match self {
            Self::Memory => Ok(None),
            Self::Persistent(db) => {
                if let Some(bytes) = db
                    .get(SENIORITY_TIMESTAMP_KEY)
                    .map_err(|e| anyhow::anyhow!(e))?
                {
                    if bytes.len() == 8 {
                        let arr: [u8; 8] = bytes.try_into().unwrap();
                        Ok(Some(u64::from_le_bytes(arr)))
                    } else {
                        Ok(None)
                    }
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
                db.remove(DEVICE_ID_KEY).map_err(|e| anyhow::anyhow!(e))?;
                db.remove(SENIORITY_TIMESTAMP_KEY)
                    .map_err(|e| anyhow::anyhow!(e))?;
                db.flush().map_err(|e| anyhow::anyhow!(e))?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use tempfile::tempdir;
    use web_time::Duration;

    const MAX_REOPEN_ATTEMPTS: u64 = 10;
    const REOPEN_BACKOFF_BASE_MS: u64 = 25;

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
            let backend2 = Arc::new(
                (0..MAX_REOPEN_ATTEMPTS)
                    .find_map(
                        |attempt| match crate::store::backend::SledStorage::new(&path) {
                            Ok(storage) => Some(storage),
                            Err(_) if attempt + 1 < MAX_REOPEN_ATTEMPTS => {
                                thread::sleep(Duration::from_millis(
                                    REOPEN_BACKOFF_BASE_MS * (attempt + 1),
                                ));
                                None
                            }
                            Err(err) => panic!("failed to reopen sled identity store: {err}"),
                        },
                    )
                    .unwrap(),
            );
            let store = IdentityStore::persistent(backend2);
            let loaded = store.load_keys().unwrap().unwrap();
            assert_eq!(id, loaded.identity_id());
        }
    }

    #[test]
    fn test_device_metadata_persistence_across_instances() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_store").to_str().unwrap().to_string();

        let original = {
            let backend = Arc::new(crate::store::backend::SledStorage::new(&path).unwrap());
            let store = IdentityStore::persistent(backend);
            store.load_or_create_device_metadata().unwrap()
        };

        let backend2 = Arc::new(
            (0..MAX_REOPEN_ATTEMPTS)
                .find_map(
                    |attempt| match crate::store::backend::SledStorage::new(&path) {
                        Ok(storage) => Some(storage),
                        Err(_) if attempt + 1 < MAX_REOPEN_ATTEMPTS => {
                            thread::sleep(Duration::from_millis(
                                REOPEN_BACKOFF_BASE_MS * (attempt + 1),
                            ));
                            None
                        }
                        Err(err) => panic!("failed to reopen sled identity store: {err}"),
                    },
                )
                .unwrap(),
        );
        let store = IdentityStore::persistent(backend2);
        let reloaded = store.load_or_create_device_metadata().unwrap();

        assert_eq!(original, reloaded);
        let parsed_uuid = Uuid::parse_str(&reloaded.device_id).unwrap();
        assert_eq!(parsed_uuid.get_version_num(), 4);
        assert!(reloaded.seniority_timestamp > 0);
    }
}
