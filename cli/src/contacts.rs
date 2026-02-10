// Contact list management for SCMessenger CLI
//
// Stores mappings of:
// - PeerID (libp2p network identifier) -> Contact info
// - Nickname -> PeerID
// - Crypto public key -> PeerID

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    /// libp2p PeerID
    pub peer_id: String,

    /// User-friendly nickname
    pub nickname: Option<String>,

    /// Crypto public key (hex)
    pub public_key: String,

    /// When contact was added (unix timestamp)
    pub added_at: u64,

    /// Last seen online (unix timestamp)
    pub last_seen: Option<u64>,

    /// Notes about this contact
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

pub struct ContactList {
    db: Db,
}

impl ContactList {
    /// Open or create contact list database
    pub fn open(path: PathBuf) -> Result<Self> {
        let db = sled::open(path).context("Failed to open contacts database")?;
        Ok(Self { db })
    }

    /// Add a new contact
    pub fn add(&self, contact: Contact) -> Result<()> {
        let key = contact.peer_id.as_bytes();
        let value = serde_json::to_vec(&contact).context("Failed to serialize contact")?;

        self.db
            .insert(key, value)
            .context("Failed to insert contact")?;

        Ok(())
    }

    /// Get a contact by peer ID
    pub fn get(&self, peer_id: &str) -> Result<Option<Contact>> {
        if let Some(data) = self.db.get(peer_id.as_bytes())? {
            let contact: Contact =
                serde_json::from_slice(&data).context("Failed to deserialize contact")?;
            Ok(Some(contact))
        } else {
            Ok(None)
        }
    }

    /// Remove a contact
    pub fn remove(&self, peer_id: &str) -> Result<()> {
        self.db.remove(peer_id.as_bytes())?;
        Ok(())
    }

    /// List all contacts
    pub fn list(&self) -> Result<Vec<Contact>> {
        let mut contacts = Vec::new();

        for item in self.db.iter() {
            let (_, value) = item?;
            let contact: Contact =
                serde_json::from_slice(&value).context("Failed to deserialize contact")?;
            contacts.push(contact);
        }

        // Sort by nickname, then by peer_id
        contacts.sort_by(|a, b| a.display_name().cmp(b.display_name()));

        Ok(contacts)
    }

    /// Find contact by nickname
    pub fn find_by_nickname(&self, nickname: &str) -> Result<Option<Contact>> {
        for item in self.db.iter() {
            let (_, value) = item?;
            let contact: Contact = serde_json::from_slice(&value)?;
            if contact.nickname.as_deref() == Some(nickname) {
                return Ok(Some(contact));
            }
        }
        Ok(None)
    }

    /// Find contact by public key
    pub fn find_by_public_key(&self, public_key: &str) -> Result<Option<Contact>> {
        for item in self.db.iter() {
            let (_, value) = item?;
            let contact: Contact = serde_json::from_slice(&value)?;
            if contact.public_key == public_key {
                return Ok(Some(contact));
            }
        }
        Ok(None)
    }

    /// Update contact's last seen timestamp
    pub fn update_last_seen(&self, peer_id: &str) -> Result<()> {
        if let Some(mut contact) = self.get(peer_id)? {
            contact.last_seen = Some(current_timestamp());
            self.add(contact)?;
        }
        Ok(())
    }

    /// Set contact nickname
    pub fn set_nickname(&self, peer_id: &str, nickname: Option<String>) -> Result<()> {
        if let Some(mut contact) = self.get(peer_id)? {
            contact.nickname = nickname;
            self.add(contact)?;
            Ok(())
        } else {
            anyhow::bail!("Contact not found: {}", peer_id)
        }
    }

    /// Set contact notes
    pub fn set_notes(&self, peer_id: &str, notes: Option<String>) -> Result<()> {
        if let Some(mut contact) = self.get(peer_id)? {
            contact.notes = notes;
            self.add(contact)?;
            Ok(())
        } else {
            anyhow::bail!("Contact not found: {}", peer_id)
        }
    }

    /// Count contacts
    pub fn count(&self) -> usize {
        self.db.len()
    }

    /// Search contacts by query (nickname, peer_id, or public_key)
    pub fn search(&self, query: &str) -> Result<Vec<Contact>> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for item in self.db.iter() {
            let (_, value) = item?;
            let contact: Contact = serde_json::from_slice(&value)?;

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
    fn test_contact_list() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let db_path = temp_dir.path().join("contacts.db");

        let contacts = ContactList::open(db_path)?;

        // Add contact
        let contact = Contact::new("12D3KooTest1".to_string(), "pubkey1".to_string())
            .with_nickname("Alice".to_string());

        contacts.add(contact)?;

        // Retrieve contact
        let retrieved = contacts.get("12D3KooTest1")?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().nickname, Some("Alice".to_string()));

        // List contacts
        let list = contacts.list()?;
        assert_eq!(list.len(), 1);

        Ok(())
    }
}
