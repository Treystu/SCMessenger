// WASM Storage — In-memory message storage with serialization
//
// Provides persistent message storage for WASM environments. Since OPFS (Origin Private
// File System) requires web-sys bindings, we implement in-memory storage here that can be
// serialized/exported for the JS caller to persist using localStorage or OPFS as needed.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use parking_lot::RwLock;

/// Storage eviction policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Remove oldest messages first (FIFO)
    OldestFirst,
    /// Remove messages from unknown senders first
    UnknownSendersFirst,
}

impl Default for EvictionPolicy {
    fn default() -> Self {
        Self::OldestFirst
    }
}

/// Message storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Maximum number of messages to store
    pub max_messages: usize,
    /// Eviction policy when capacity is reached
    pub eviction_policy: EvictionPolicy,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_messages: 10000,
            eviction_policy: EvictionPolicy::OldestFirst,
        }
    }
}

/// Stored message metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    /// Unique message ID
    pub id: String,
    /// Sender's identity ID
    pub sender_id: String,
    /// Recipient hint (can be vague for privacy)
    pub recipient_hint: Option<String>,
    /// Encrypted message payload
    pub payload: Vec<u8>,
    /// Unix timestamp (seconds) when message was stored
    pub stored_at: u64,
    /// Whether message has been read
    pub read: bool,
}

impl StoredMessage {
    /// Create a new stored message
    pub fn new(
        id: String,
        sender_id: String,
        recipient_hint: Option<String>,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            id,
            sender_id,
            recipient_hint,
            payload,
            stored_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            read: false,
        }
    }
}

/// In-memory WASM message storage
#[derive(Debug)]
pub struct WasmStorage {
    config: StorageConfig,
    /// Messages indexed by message ID
    messages: Arc<RwLock<HashMap<String, StoredMessage>>>,
    /// Messages ordered by timestamp for FIFO eviction
    insertion_order: Arc<RwLock<VecDeque<String>>>,
    /// Messages indexed by recipient hint for quick lookup
    by_recipient_hint: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl WasmStorage {
    /// Create a new WASM storage instance
    pub fn new(config: StorageConfig) -> Self {
        Self {
            config,
            messages: Arc::new(RwLock::new(HashMap::new())),
            insertion_order: Arc::new(RwLock::new(VecDeque::new())),
            by_recipient_hint: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store a message
    pub fn store_message(&self, message: StoredMessage) -> Result<(), String> {
        let mut messages = self.messages.write();

        // Check if we need to evict
        if messages.len() >= self.config.max_messages {
            drop(messages); // Release write lock
            self.evict_oldest()?;
            messages = self.messages.write(); // Re-acquire lock
        }

        let message_id = message.id.clone();
        let recipient_hint = message.recipient_hint.clone();

        messages.insert(message_id.clone(), message);

        // Update insertion order
        {
            let mut order = self.insertion_order.write();
            order.push_back(message_id.clone());
        }

        // Update recipient hint index
        if let Some(hint) = recipient_hint {
            let mut by_hint = self.by_recipient_hint.write();
            by_hint
                .entry(hint)
                .or_insert_with(Vec::new)
                .push(message_id);
        }

        Ok(())
    }

    /// Get a message by ID
    pub fn get_message(&self, id: &str) -> Option<StoredMessage> {
        self.messages.read().get(id).cloned()
    }

    /// Get all messages for a recipient hint
    pub fn get_messages_for_hint(&self, hint: &str) -> Vec<StoredMessage> {
        let by_hint = self.by_recipient_hint.read();
        if let Some(ids) = by_hint.get(hint) {
            let messages = self.messages.read();
            ids.iter()
                .filter_map(|id| messages.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all messages
    pub fn get_all_messages(&self) -> Vec<StoredMessage> {
        self.messages
            .read()
            .values()
            .cloned()
            .collect()
    }

    /// Get all unread messages
    pub fn get_unread_messages(&self) -> Vec<StoredMessage> {
        self.messages
            .read()
            .values()
            .filter(|m| !m.read)
            .cloned()
            .collect()
    }

    /// Mark a message as read
    pub fn mark_as_read(&self, id: &str) -> Result<(), String> {
        let mut messages = self.messages.write();
        if let Some(msg) = messages.get_mut(id) {
            msg.read = true;
            Ok(())
        } else {
            Err(format!("Message {} not found", id))
        }
    }

    /// Delete a message by ID
    pub fn delete_message(&self, id: &str) -> Result<(), String> {
        let mut messages = self.messages.write();
        if let Some(msg) = messages.remove(id) {
            // Remove from insertion order
            {
                let mut order = self.insertion_order.write();
                order.retain(|x| x != id);
            }

            // Remove from recipient hint index
            if let Some(hint) = msg.recipient_hint {
                let mut by_hint = self.by_recipient_hint.write();
                if let Some(ids) = by_hint.get_mut(&hint) {
                    ids.retain(|x| x != id);
                }
            }

            Ok(())
        } else {
            Err(format!("Message {} not found", id))
        }
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.read().len()
    }

    /// Evict oldest message based on policy
    fn evict_oldest(&self) -> Result<(), String> {
        match self.config.eviction_policy {
            EvictionPolicy::OldestFirst => {
                let mut order = self.insertion_order.write();
                if let Some(oldest_id) = order.pop_front() {
                    drop(order); // Release lock before deleting
                    let mut messages = self.messages.write();
                    if let Some(msg) = messages.remove(&oldest_id) {
                        // Clean up indexes
                        if let Some(hint) = msg.recipient_hint {
                            drop(messages); // Release lock
                            let mut by_hint = self.by_recipient_hint.write();
                            if let Some(ids) = by_hint.get_mut(&hint) {
                                ids.retain(|x| x != &oldest_id);
                            }
                        }
                        Ok(())
                    } else {
                        Err("Failed to remove oldest message".to_string())
                    }
                } else {
                    Err("No messages to evict".to_string())
                }
            }
            EvictionPolicy::UnknownSendersFirst => {
                let messages = self.messages.read();
                let mut candidates: Vec<_> = messages
                    .iter()
                    .filter(|(_, m)| m.sender_id == "unknown")
                    .map(|(id, m)| (id.clone(), m.stored_at))
                    .collect();

                candidates.sort_by_key(|(_, ts)| *ts);

                if let Some((id_to_remove, _)) = candidates.first() {
                    let id = id_to_remove.clone();
                    drop(messages); // Release lock before deleting
                    self.delete_message(&id)
                } else {
                    // Fall back to oldest first if no unknown senders
                    let mut order = self.insertion_order.write();
                    if let Some(oldest_id) = order.pop_front() {
                        drop(order);
                        let mut messages = self.messages.write();
                        if let Some(msg) = messages.remove(&oldest_id) {
                            if let Some(hint) = msg.recipient_hint {
                                drop(messages);
                                let mut by_hint = self.by_recipient_hint.write();
                                if let Some(ids) = by_hint.get_mut(&hint) {
                                    ids.retain(|x| x != &oldest_id);
                                }
                            }
                            Ok(())
                        } else {
                            Err("Failed to remove oldest message".to_string())
                        }
                    } else {
                        Err("No messages to evict".to_string())
                    }
                }
            }
        }
    }

    /// Export full storage state to JSON (for persistence)
    pub fn export_state(&self) -> Result<String, String> {
        let messages = self.messages.read();
        let state: Vec<StoredMessage> = messages.values().cloned().collect();
        serde_json::to_string(&state).map_err(|e| format!("Serialization error: {}", e))
    }

    /// Import storage state from JSON
    pub fn import_state(&self, json: &str) -> Result<(), String> {
        let state: Vec<StoredMessage> =
            serde_json::from_str(json).map_err(|e| format!("Deserialization error: {}", e))?;

        // Clear existing state
        self.messages.write().clear();
        self.insertion_order.write().clear();
        self.by_recipient_hint.write().clear();

        // Re-import messages
        for message in state {
            self.store_message(message)?;
        }

        Ok(())
    }

    /// Clear all messages
    pub fn clear(&self) {
        self.messages.write().clear();
        self.insertion_order.write().clear();
        self.by_recipient_hint.write().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_message(id: &str, sender_id: &str) -> StoredMessage {
        StoredMessage::new(
            id.to_string(),
            sender_id.to_string(),
            Some("recipient".to_string()),
            vec![1, 2, 3, 4, 5],
        )
    }

    #[test]
    fn test_storage_creation() {
        let storage = WasmStorage::new(StorageConfig::default());
        assert_eq!(storage.message_count(), 0);
    }

    #[test]
    fn test_store_message() {
        let storage = WasmStorage::new(StorageConfig::default());
        let msg = create_test_message("msg-1", "sender-1");
        assert!(storage.store_message(msg).is_ok());
        assert_eq!(storage.message_count(), 1);
    }

    #[test]
    fn test_get_message() {
        let storage = WasmStorage::new(StorageConfig::default());
        let msg = create_test_message("msg-1", "sender-1");
        storage.store_message(msg.clone()).unwrap();

        let retrieved = storage.get_message("msg-1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "msg-1");
    }

    #[test]
    fn test_get_nonexistent_message() {
        let storage = WasmStorage::new(StorageConfig::default());
        assert!(storage.get_message("nonexistent").is_none());
    }

    #[test]
    fn test_get_messages_by_hint() {
        let storage = WasmStorage::new(StorageConfig::default());
        let msg1 = StoredMessage::new(
            "msg-1".to_string(),
            "sender-1".to_string(),
            Some("hint-1".to_string()),
            vec![1, 2, 3],
        );
        let msg2 = StoredMessage::new(
            "msg-2".to_string(),
            "sender-2".to_string(),
            Some("hint-1".to_string()),
            vec![4, 5, 6],
        );

        storage.store_message(msg1).unwrap();
        storage.store_message(msg2).unwrap();

        let messages = storage.get_messages_for_hint("hint-1");
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_mark_as_read() {
        let storage = WasmStorage::new(StorageConfig::default());
        let msg = create_test_message("msg-1", "sender-1");
        storage.store_message(msg).unwrap();

        assert!(!storage.get_message("msg-1").unwrap().read);
        assert!(storage.mark_as_read("msg-1").is_ok());
        assert!(storage.get_message("msg-1").unwrap().read);
    }

    #[test]
    fn test_get_unread_messages() {
        let storage = WasmStorage::new(StorageConfig::default());
        let msg1 = create_test_message("msg-1", "sender-1");
        let msg2 = create_test_message("msg-2", "sender-2");

        storage.store_message(msg1).unwrap();
        storage.store_message(msg2).unwrap();

        assert_eq!(storage.get_unread_messages().len(), 2);

        storage.mark_as_read("msg-1").unwrap();
        assert_eq!(storage.get_unread_messages().len(), 1);
    }

    #[test]
    fn test_delete_message() {
        let storage = WasmStorage::new(StorageConfig::default());
        let msg = create_test_message("msg-1", "sender-1");
        storage.store_message(msg).unwrap();
        assert_eq!(storage.message_count(), 1);

        assert!(storage.delete_message("msg-1").is_ok());
        assert_eq!(storage.message_count(), 0);
    }

    #[test]
    fn test_delete_nonexistent_message() {
        let storage = WasmStorage::new(StorageConfig::default());
        assert!(storage.delete_message("nonexistent").is_err());
    }

    #[test]
    fn test_eviction_oldest_first() {
        let config = StorageConfig {
            max_messages: 3,
            eviction_policy: EvictionPolicy::OldestFirst,
        };
        let storage = WasmStorage::new(config);

        for i in 1..=4 {
            let msg = create_test_message(&format!("msg-{}", i), "sender");
            assert!(storage.store_message(msg).is_ok());
        }

        // msg-1 should have been evicted
        assert!(storage.get_message("msg-1").is_none());
        assert!(storage.get_message("msg-4").is_some());
        assert_eq!(storage.message_count(), 3);
    }

    #[test]
    fn test_export_state() {
        let storage = WasmStorage::new(StorageConfig::default());
        let msg = create_test_message("msg-1", "sender-1");
        storage.store_message(msg).unwrap();

        let json = storage.export_state();
        assert!(json.is_ok());

        let state_json = json.unwrap();
        assert!(state_json.contains("msg-1"));
        assert!(state_json.contains("sender-1"));
    }

    #[test]
    fn test_import_state() {
        let msg = StoredMessage::new(
            "msg-1".to_string(),
            "sender-1".to_string(),
            Some("hint".to_string()),
            vec![1, 2, 3],
        );
        let json = serde_json::to_string(&vec![msg]).unwrap();

        let storage = WasmStorage::new(StorageConfig::default());
        assert!(storage.import_state(&json).is_ok());
        assert_eq!(storage.message_count(), 1);
        assert!(storage.get_message("msg-1").is_some());
    }

    #[test]
    fn test_clear_storage() {
        let storage = WasmStorage::new(StorageConfig::default());
        let msg = create_test_message("msg-1", "sender-1");
        storage.store_message(msg).unwrap();
        assert_eq!(storage.message_count(), 1);

        storage.clear();
        assert_eq!(storage.message_count(), 0);
    }

    #[test]
    fn test_get_all_messages() {
        let storage = WasmStorage::new(StorageConfig::default());
        for i in 1..=5 {
            let msg = create_test_message(&format!("msg-{}", i), "sender");
            storage.store_message(msg).unwrap();
        }

        let all_messages = storage.get_all_messages();
        assert_eq!(all_messages.len(), 5);
    }
}
