// Contact management bridge for mobile platforms
//
// Wraps CLI contact storage logic (sled-based) for UniFFI exposure to Android/iOS.
// Ensures cross-platform database compatibility via JSON serialization.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Public contact structure exposed via UniFFI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub peer_id: String,
    pub nickname: Option<String>,
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
        self.nickname.as_deref().unwrap_or(&self.peer_id)
    }
}

/// Contact manager with thread-safe sled database backend
pub struct ContactManager {
    db: Arc<Mutex<Db>>,
}

impl ContactManager {
    /// Create or open contact database at the given path
    pub fn new(storage_path: String) -> Result<Self, crate::IronCoreError> {
        let path = PathBuf::from(storage_path).join("contacts.db");
        let db = sled::open(path)
            .context("Failed to open contacts database")
            .map_err(|_| crate::IronCoreError::StorageError)?;

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }

    /// Add a contact to the database
    pub fn add(&self, contact: Contact) -> Result<(), crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        let key = contact.peer_id.as_bytes();
        let value = serde_json::to_vec(&contact)
            .context("Failed to serialize contact")
            .map_err(|_| crate::IronCoreError::Internal)?;

        db.insert(key, value)
            .context("Failed to insert contact")
            .map_err(|_| crate::IronCoreError::StorageError)?;

        Ok(())
    }

    /// Get a contact by peer ID
    pub fn get(&self, peer_id: String) -> Result<Option<Contact>, crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        if let Some(data) = db
            .get(peer_id.as_bytes())
            .map_err(|_| crate::IronCoreError::StorageError)?
        {
            let contact: Contact = serde_json::from_slice(&data)
                .context("Failed to deserialize contact")
                .map_err(|_| crate::IronCoreError::Internal)?;
            Ok(Some(contact))
        } else {
            Ok(None)
        }
    }

    /// Remove a contact
    pub fn remove(&self, peer_id: String) -> Result<(), crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        db.remove(peer_id.as_bytes())
            .map_err(|_| crate::IronCoreError::StorageError)?;
        Ok(())
    }

    /// List all contacts, sorted by display name
    pub fn list(&self) -> Result<Vec<Contact>, crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        let mut contacts = Vec::new();

        for item in db.iter() {
            let (_, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let contact: Contact = serde_json::from_slice(&value)
                .context("Failed to deserialize contact")
                .map_err(|_| crate::IronCoreError::Internal)?;
            contacts.push(contact);
        }

        contacts.sort_by(|a, b| a.display_name().cmp(b.display_name()));
        Ok(contacts)
    }

    /// Search contacts by query (matches nickname, peer_id, public_key, or notes)
    pub fn search(&self, query: String) -> Result<Vec<Contact>, crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for item in db.iter() {
            let (_, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let contact: Contact =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;

            let matches = contact.peer_id.to_lowercase().contains(&query_lower)
                || contact.public_key.to_lowercase().contains(&query_lower)
                || contact
                    .nickname
                    .as_ref()
                    .map_or(false, |n| n.to_lowercase().contains(&query_lower))
                || contact
                    .notes
                    .as_ref()
                    .map_or(false, |n| n.to_lowercase().contains(&query_lower));

            if matches {
                results.push(contact);
            }
        }

        results.sort_by(|a, b| a.display_name().cmp(b.display_name()));
        Ok(results)
    }

    /// Set or update contact nickname
    pub fn set_nickname(
        &self,
        peer_id: String,
        nickname: Option<String>,
    ) -> Result<(), crate::IronCoreError> {
        if let Some(mut contact) = self.get(peer_id.clone())? {
            contact.nickname = nickname;
            self.add(contact)?;
            Ok(())
        } else {
            Err(crate::IronCoreError::InvalidInput)
        }
    }

    /// Update contact's last seen timestamp to now
    pub fn update_last_seen(&self, peer_id: String) -> Result<(), crate::IronCoreError> {
        if let Some(mut contact) = self.get(peer_id.clone())? {
            contact.last_seen = Some(current_timestamp());
            self.add(contact)?;
            Ok(())
        } else {
            // Silently ignore if contact doesn't exist
            Ok(())
        }
    }

    /// Count total contacts
    pub fn count(&self) -> u32 {
        let db = self.db.lock().unwrap();
        db.len() as u32
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

    #[test]
    fn test_contact_creation() {
        let contact = Contact::new("12D3KooTest".to_string(), "abcd1234".to_string())
            .with_nickname("Alice".to_string());

        assert_eq!(contact.display_name(), "Alice");
        assert_eq!(contact.peer_id, "12D3KooTest");
    }

    #[test]
    fn test_contact_manager() -> Result<(), crate::IronCoreError> {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage_path = temp_dir.path().to_str().unwrap().to_string();

        let manager = ContactManager::new(storage_path)?;

        // Add contact
        let contact = Contact::new("12D3KooTest1".to_string(), "pubkey1".to_string())
            .with_nickname("Alice".to_string());

        manager.add(contact)?;

        // Retrieve contact
        let retrieved = manager.get("12D3KooTest1".to_string())?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().nickname, Some("Alice".to_string()));

        // List contacts
        let list = manager.list()?;
        assert_eq!(list.len(), 1);

        // Search
        let results = manager.search("alice".to_string())?;
        assert_eq!(results.len(), 1);

        // Count
        assert_eq!(manager.count(), 1);

        Ok(())
    }
}
