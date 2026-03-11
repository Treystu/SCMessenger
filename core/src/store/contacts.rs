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
    /// WS13 tight-pair: most-recently-observed device UUID for this contact.
    /// Updated when an inbound message carries WS13 device metadata.
    /// Used as `intended_device_id` when sending to this contact.
    #[serde(default)]
    pub last_known_device_id: Option<String>,
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
            last_known_device_id: None,
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
                        .is_some_and(|n| n.to_lowercase().contains(&query_lower))
                    || contact
                        .local_nickname
                        .as_ref()
                        .is_some_and(|n| n.to_lowercase().contains(&query_lower))
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

    /// Update the most-recently-observed device ID for a contact (WS13 tight-pair).
    ///
    /// Called when an inbound message or ledger exchange reveals the sender's current device UUID.
    /// The stored value is used as `intended_device_id` when routing future messages to this peer.
    /// A `None` value clears any previously-stored device ID (e.g., after a factory reset signal).
    /// `Some` values are normalized (`trim`) and only persisted when non-empty and valid UUIDs;
    /// malformed values are ignored to avoid replacing a previously known-good device ID.
    pub fn update_last_known_device_id(
        &self,
        peer_id: String,
        device_id: Option<String>,
    ) -> Result<(), IronCoreError> {
        if let Some(mut contact) = self.get(peer_id)? {
            match device_id {
                None => {
                    contact.last_known_device_id = None;
                    self.add(contact)?;
                }
                Some(device_id) => {
                    let normalized = device_id.trim();
                    if !normalized.is_empty() && uuid::Uuid::parse_str(normalized).is_ok() {
                        contact.last_known_device_id = Some(normalized.to_string());
                        self.add(contact)?;
                    }
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::backend::MemoryStorage;
    use std::sync::Arc;

    fn make_manager() -> ContactManager {
        ContactManager::new(Arc::new(MemoryStorage::new()))
    }

    #[test]
    fn contact_new_has_no_last_known_device_id() {
        let c = Contact::new("peer-1".to_string(), "pubkey-hex".to_string());
        assert!(c.last_known_device_id.is_none());
    }

    #[test]
    fn update_last_known_device_id_persists_and_is_readable() {
        let mgr = make_manager();
        mgr.add(Contact::new("peer-1".to_string(), "pubkey".to_string()))
            .unwrap();

        mgr.update_last_known_device_id(
            "peer-1".to_string(),
            Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
        )
        .unwrap();

        let contact = mgr.get("peer-1".to_string()).unwrap().unwrap();
        assert_eq!(
            contact.last_known_device_id.as_deref(),
            Some("550e8400-e29b-41d4-a716-446655440000")
        );
    }

    #[test]
    fn update_last_known_device_id_can_clear() {
        let mgr = make_manager();
        let mut c = Contact::new("peer-2".to_string(), "pubkey".to_string());
        c.last_known_device_id = Some("old-device".to_string());
        mgr.add(c).unwrap();

        mgr.update_last_known_device_id("peer-2".to_string(), None)
            .unwrap();

        let contact = mgr.get("peer-2".to_string()).unwrap().unwrap();
        assert!(contact.last_known_device_id.is_none());
    }

    #[test]
    fn contact_roundtrips_through_serde_with_default_device_id() {
        // Simulate a pre-WS13 contact record (no last_known_device_id field).
        let json = r#"{"peer_id":"peer-old","nickname":null,"local_nickname":null,"public_key":"pk","added_at":0,"last_seen":null,"notes":null}"#;
        let c: Contact = serde_json::from_str(json).unwrap();
        assert!(
            c.last_known_device_id.is_none(),
            "legacy records must default to None"
        );
    }

    #[test]
    fn update_last_known_device_id_trims_valid_uuid() {
        let mgr = make_manager();
        mgr.add(Contact::new("peer-3".to_string(), "pubkey".to_string()))
            .unwrap();

        mgr.update_last_known_device_id(
            "peer-3".to_string(),
            Some("  550e8400-e29b-41d4-a716-446655440000  ".to_string()),
        )
        .unwrap();

        let contact = mgr.get("peer-3".to_string()).unwrap().unwrap();
        assert_eq!(
            contact.last_known_device_id.as_deref(),
            Some("550e8400-e29b-41d4-a716-446655440000")
        );
    }

    #[test]
    fn update_last_known_device_id_ignores_invalid_values() {
        let mgr = make_manager();
        let mut c = Contact::new("peer-4".to_string(), "pubkey".to_string());
        c.last_known_device_id = Some("550e8400-e29b-41d4-a716-446655440000".to_string());
        mgr.add(c).unwrap();

        mgr.update_last_known_device_id("peer-4".to_string(), Some("   ".to_string()))
            .unwrap();
        mgr.update_last_known_device_id("peer-4".to_string(), Some("not-a-uuid".to_string()))
            .unwrap();

        let contact = mgr.get("peer-4".to_string()).unwrap().unwrap();
        assert_eq!(
            contact.last_known_device_id.as_deref(),
            Some("550e8400-e29b-41d4-a716-446655440000")
        );
    }
}
