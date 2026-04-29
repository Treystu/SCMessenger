// Contact management storage
//
// Refactored to use generic StorageBackend for cross-platform parity (Sled/IndexedDB/Memory).

use crate::store::backend::StorageBackend;
use crate::store::history::HistoryManager;
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

    /// Reconcile contacts from message history to recover potentially lost records.
    /// Scans all message records and creates a basic contact if the peer_id is unknown.
    pub fn reconcile_from_history(&self, history: &HistoryManager) -> Result<u32, IronCoreError> {
        let all_messages = history.recent_including_hidden(None, 10000)?;
        let mut recovered_count = 0;

        for msg in all_messages {
            if self.get(msg.peer_id.clone()).is_ok() && self.get(msg.peer_id.clone())?.is_none() {
                // We have the peer_id from history, but no contact record.
                // Note: We lack the public key here unless we can derive it from the peer_id.
                // In libp2p, the peer_id typically contains the public key.
                if let Ok(pub_key) = self.derive_public_key_from_peer_id(&msg.peer_id) {
                    let contact = Contact::new(msg.peer_id.clone(), pub_key);
                    self.add(contact)?;
                    recovered_count += 1;
                }
            }
        }
        Ok(recovered_count)
    }

    fn derive_public_key_from_peer_id(&self, peer_id: &str) -> Result<String, IronCoreError> {
        let trimmed = peer_id.trim();

        // If it's 64 hex chars, validate it's a genuine Ed25519 public key.
        // identity_id is also 64 hex chars (Blake3 hash) but NOT a valid Ed25519 key.
        // Rejecting identity_id here prevents reconcile_from_history from creating
        // contacts with public_key = identity_id, which breaks future encryption.
        if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            if let Ok(bytes) = hex::decode(trimmed) {
                if bytes.len() == 32 {
                    if let Ok(arr) = <[u8; 32]>::try_from(bytes.as_slice()) {
                        if ed25519_dalek::VerifyingKey::from_bytes(&arr).is_ok() {
                            return Ok(trimmed.to_lowercase());
                        }
                    }
                }
            }
            // 64-hex but not a valid Ed25519 key → likely identity_id; cannot derive pubkey.
            return Err(IronCoreError::InvalidInput);
        }

        // Try to decode as libp2p PeerId (base58) and extract Ed25519 public key.
        // Matches the protobuf prefix used by libp2p identity multihash:
        // 0x00 0x24 0x08 0x01 0x12 0x20 <32 bytes>
        if let Ok(bytes) = bs58::decode(trimmed).into_vec() {
            if bytes.len() == 38
                && bytes[0] == 0x00
                && bytes[1] == 0x24
                && bytes[2] == 0x08
                && bytes[3] == 0x01
                && bytes[4] == 0x12
                && bytes[5] == 0x20
            {
                return Ok(hex::encode(&bytes[6..38]));
            }
            // Fallback: take last 32 bytes for non-standard PeerIds
            if bytes.len() >= 32 {
                return Ok(hex::encode(&bytes[bytes.len() - 32..]));
            }
        }

        Err(IronCoreError::InvalidInput)
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

    /// Verify database integrity and detect corruption.
    /// Returns an error if the database has data but returns 0 contacts.
    pub fn verify_integrity(&self) -> Result<(), IronCoreError> {
        let contact_count = self.count();
        let db_size = self.backend.count_prefix(b"").unwrap_or(0);

        // If contact count is 0 but database has entries, there may be corruption
        // or the contacts were not properly loaded from the database.
        // We use a threshold of 1024 bytes as a reasonable indicator of data presence.
        if contact_count == 0 && db_size > 0 {
            // Check if we have actual data by scanning a few keys
            let has_data = !self.backend.scan_prefix(b"").unwrap_or_default().is_empty();
            if has_data {
                // Database has data but count() returns 0 - potential corruption
                // This could happen if the data is stored but not properly deserialized
                return Err(IronCoreError::CorruptionDetected);
            }
        }
        Ok(())
    }
}

fn current_timestamp() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
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
