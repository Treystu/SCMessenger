// Contact management storage
//
// Refactored to use generic StorageBackend for cross-platform parity (Sled/IndexedDB/Memory).

use crate::store::backend::StorageBackend;
use crate::IronCoreError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub peer_id: String,
    pub nickname: Option<String>, // Federated nickname (from the peer)
    pub local_nickname: Option<String>, // Local override set by the user
    pub public_key: String,
    pub added_at: u64,
    pub last_seen: Option<u64>,
    pub notes: Option<String>,
}

impl Contact {
    pub fn new(peer_id: String, public_key: String) -> Self {
        Self {
            peer_id,
            nickname: None,
            local_nickname: None,
            public_key,
            added_at: current_timestamp(),
            last_seen: None,
            notes: None,
        }
    }

    pub fn with_nickname(mut self, nickname: String) -> Self {
        self.nickname = Some(nickname);
        self
    }

    pub fn display_name(&self) -> &str {
        if let Some(ref local) = self.local_nickname {
            return local;
        }
        self.nickname.as_deref().unwrap_or(&self.peer_id)
    }
}

#[derive(Clone)]
pub struct ContactManager {
    backend: Arc<dyn StorageBackend>,
}

impl ContactManager {
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        Self { backend }
    }

    pub fn add(&self, contact: Contact) -> Result<(), IronCoreError> {
        let key = contact.peer_id.clone();
        let value = serde_json::to_vec(&contact).map_err(|_| IronCoreError::Internal)?;
        self.backend
            .put(key.as_bytes(), &value)
            .map_err(|_| IronCoreError::StorageError)?;
        Ok(())
    }

    pub fn get(&self, peer_id: String) -> Result<Option<Contact>, IronCoreError> {
        let key = peer_id;
        if let Some(data) = self
            .backend
            .get(key.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?
        {
            let contact: Contact =
                serde_json::from_slice(&data).map_err(|_| IronCoreError::Internal)?;
            Ok(Some(contact))
        } else {
            Ok(None)
        }
    }

    pub fn remove(&self, peer_id: String) -> Result<(), IronCoreError> {
        let key = peer_id;
        self.backend
            .remove(key.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<Contact>, IronCoreError> {
        let all = self
            .backend
            .scan_prefix(b"")
            .map_err(|_| IronCoreError::StorageError)?;

        let mut contacts = Vec::new();
        for (_, value) in all {
            let contact: Contact =
                serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;
            contacts.push(contact);
        }

        contacts.sort_by(|a, b| a.display_name().cmp(b.display_name()));
        Ok(contacts)
    }

    pub fn search(&self, query: String) -> Result<Vec<Contact>, IronCoreError> {
        let query_lower = query.to_lowercase();
        let all = self.list()?;

        let results = all
            .into_iter()
            .filter(|contact| {
                contact.peer_id.to_lowercase().contains(&query_lower)
                    || contact.public_key.to_lowercase().contains(&query_lower)
                    || contact
                        .nickname
                        .as_ref()
                        .map_or(false, |n| n.to_lowercase().contains(&query_lower))
                    || contact
                        .local_nickname
                        .as_ref()
                        .map_or(false, |n| n.to_lowercase().contains(&query_lower))
            })
            .collect();

        Ok(results)
    }

    pub fn set_nickname(
        &self,
        peer_id: String,
        nickname: Option<String>,
    ) -> Result<(), IronCoreError> {
        if let Some(mut contact) = self.get(peer_id)? {
            contact.nickname = nickname
                .map(|n| n.trim().to_string())
                .filter(|n| !n.is_empty());
            self.add(contact)?;
            Ok(())
        } else {
            Err(IronCoreError::InvalidInput)
        }
    }

    pub fn set_local_nickname(
        &self,
        peer_id: String,
        nickname: Option<String>,
    ) -> Result<(), IronCoreError> {
        if let Some(mut contact) = self.get(peer_id)? {
            contact.local_nickname = nickname
                .map(|n| n.trim().to_string())
                .filter(|n| !n.is_empty());
            self.add(contact)?;
            Ok(())
        } else {
            Err(IronCoreError::InvalidInput)
        }
    }

    pub fn update_last_seen(&self, peer_id: String) -> Result<(), IronCoreError> {
        if let Some(mut contact) = self.get(peer_id)? {
            contact.last_seen = Some(current_timestamp());
            self.add(contact)?;
        }
        Ok(())
    }

    pub fn count(&self) -> u32 {
        self.backend.count_prefix(b"").unwrap_or(0) as u32
    }

    pub fn flush(&self) {
        let _ = self.backend.flush();
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
