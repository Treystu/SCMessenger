// Inbox — receive and deduplicate incoming messages
//
// Tracks seen message IDs to prevent replay attacks and duplicate delivery.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;
use tracing::debug;

/// Maximum tracked message IDs (for deduplication)
const MAX_SEEN_IDS: usize = 50_000;

#[derive(Debug, Error, Clone)]
pub enum InboxError {
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// A received message record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivedMessage {
    /// Message ID
    pub message_id: String,
    /// Sender's identity ID
    pub sender_id: String,
    /// Decrypted payload bytes
    pub payload: Vec<u8>,
    /// When this was received (unix timestamp)
    pub received_at: u64,
}

/// In-memory inbound message deduplication and storage
pub struct MemoryInbox {
    /// Set of seen message IDs (for dedup)
    seen_ids: HashSet<String>,
    /// Ordered list of seen IDs (for O(1) FIFO eviction)
    seen_order: VecDeque<String>,
    /// Recent messages by sender (for application retrieval)
    messages: HashMap<String, Vec<ReceivedMessage>>,
    /// Total stored messages
    total: usize,
}

impl MemoryInbox {
    pub fn new() -> Self {
        Self {
            seen_ids: HashSet::new(),
            seen_order: VecDeque::new(),
            messages: HashMap::new(),
            total: 0,
        }
    }

    /// Check if a message ID has already been seen (duplicate)
    pub fn is_duplicate(&self, message_id: &str) -> bool {
        self.seen_ids.contains(message_id)
    }

    /// Record a received message. Returns false if duplicate.
    pub fn receive(&mut self, msg: ReceivedMessage) -> bool {
        if self.seen_ids.contains(&msg.message_id) {
            return false; // Duplicate
        }

        // Track for dedup
        self.seen_ids.insert(msg.message_id.clone());
        self.seen_order.push_back(msg.message_id.clone());

        // Evict old IDs if at capacity (O(1) with VecDeque)
        while self.seen_ids.len() > MAX_SEEN_IDS {
            if let Some(old_id) = self.seen_order.pop_front() {
                self.seen_ids.remove(&old_id);
            }
        }

        // Store message
        self.messages
            .entry(msg.sender_id.clone())
            .or_default()
            .push(msg);
        self.total += 1;

        true // New message
    }

    /// Get all messages from a specific sender
    pub fn messages_from(&self, sender_id: &str) -> Vec<&ReceivedMessage> {
        self.messages
            .get(sender_id)
            .map(|msgs| msgs.iter().collect())
            .unwrap_or_default()
    }

    /// Get all recent messages across all senders
    pub fn all_messages(&self) -> Vec<&ReceivedMessage> {
        self.messages
            .values()
            .flat_map(|msgs| msgs.iter())
            .collect()
    }

    /// Total stored messages
    pub fn total_count(&self) -> usize {
        self.total
    }

    /// Number of unique senders
    pub fn sender_count(&self) -> usize {
        self.messages.len()
    }

    /// Clear all messages (but keep dedup IDs)
    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.total = 0;
    }
}

impl Default for MemoryInbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Persistent sled-backed inbound message deduplication and storage
pub struct SledInbox {
    #[allow(dead_code)] // Keeps sled::Db alive (RAII)
    db: sled::Db,
    seen_tree: sled::Tree,
    messages_tree: sled::Tree,
    total: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl SledInbox {
    /// Create or open a persistent inbox
    pub fn new(db: sled::Db) -> Result<Self, InboxError> {
        let seen_tree = db
            .open_tree("inbox_seen")
            .map_err(|e| InboxError::StorageError(e.to_string()))?;

        let messages_tree = db
            .open_tree("inbox_messages")
            .map_err(|e| InboxError::StorageError(e.to_string()))?;

        // Count existing messages
        let total = messages_tree.iter().count();

        Ok(Self {
            db,
            seen_tree,
            messages_tree,
            total: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(total)),
        })
    }

    fn make_messages_key(sender_id: &str, message_id: &str) -> Vec<u8> {
        format!("{}:{}", sender_id, message_id).into_bytes()
    }

    /// Check if a message ID has already been seen (duplicate)
    pub fn is_duplicate(&self, message_id: &str) -> bool {
        self.seen_tree.contains_key(message_id.as_bytes()).unwrap_or(false)
    }

    /// Record a received message. Returns false if duplicate.
    pub fn receive(&self, msg: ReceivedMessage) -> Result<bool, InboxError> {
        if self.is_duplicate(&msg.message_id) {
            return Ok(false); // Duplicate
        }

        // Store seen ID with timestamp
        let timestamp = msg.received_at;
        let timestamp_bytes = bincode::serialize(&timestamp)
            .map_err(|e| InboxError::SerializationError(e.to_string()))?;

        self.seen_tree
            .insert(msg.message_id.as_bytes(), timestamp_bytes)
            .map_err(|e| InboxError::StorageError(e.to_string()))?;

        // Store message
        let key = Self::make_messages_key(&msg.sender_id, &msg.message_id);
        let value = bincode::serialize(&msg)
            .map_err(|e| InboxError::SerializationError(e.to_string()))?;

        self.messages_tree
            .insert(key, value)
            .map_err(|e| InboxError::StorageError(e.to_string()))?;

        self.total.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        debug!("Received message {} from {}", msg.message_id, msg.sender_id);

        Ok(true) // New message
    }

    /// Get all messages from a specific sender
    pub fn messages_from(&self, sender_id: &str) -> Result<Vec<ReceivedMessage>, InboxError> {
        let prefix = format!("{}:", sender_id);
        let mut messages = Vec::new();

        for entry in self.messages_tree.scan_prefix(prefix.as_bytes()) {
            let (_, value) = entry.map_err(|e: sled::Error| InboxError::StorageError(e.to_string()))?;
            let msg: ReceivedMessage = bincode::deserialize(&value)
                .map_err(|e| InboxError::SerializationError(e.to_string()))?;
            messages.push(msg);
        }

        Ok(messages)
    }

    /// Get all recent messages across all senders
    pub fn all_messages(&self) -> Result<Vec<ReceivedMessage>, InboxError> {
        let mut messages = Vec::new();

        for entry in self.messages_tree.iter() {
            let (_, value) = entry.map_err(|e: sled::Error| InboxError::StorageError(e.to_string()))?;
            let msg: ReceivedMessage = bincode::deserialize(&value)
                .map_err(|e| InboxError::SerializationError(e.to_string()))?;
            messages.push(msg);
        }

        Ok(messages)
    }

    /// Total stored messages
    pub fn total_count(&self) -> usize {
        self.total.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Number of unique senders
    pub fn sender_count(&self) -> Result<usize, InboxError> {
        let mut senders = std::collections::HashSet::new();

        for entry in self.messages_tree.iter() {
            let (_, value) = entry.map_err(|e: sled::Error| InboxError::StorageError(e.to_string()))?;
            let msg: ReceivedMessage = bincode::deserialize(&value)
                .map_err(|e| InboxError::SerializationError(e.to_string()))?;
            senders.insert(msg.sender_id);
        }

        Ok(senders.len())
    }

    /// Clear all messages (but keep dedup IDs)
    pub fn clear_messages(&self) -> Result<(), InboxError> {
        for entry in self.messages_tree.iter() {
            let (key, _) = entry.map_err(|e: sled::Error| InboxError::StorageError(e.to_string()))?;
            self.messages_tree
                .remove(key)
                .map_err(|e: sled::Error| InboxError::StorageError(e.to_string()))?;
        }
        self.total.store(0, std::sync::atomic::Ordering::Relaxed);
        debug!("Cleared all messages from inbox");
        Ok(())
    }
}

/// Inbox storage backend — supports both in-memory and persistent modes
pub enum Inbox {
    Memory(MemoryInbox),
    Persistent(SledInbox),
}

impl Inbox {
    /// Create a new in-memory inbox
    pub fn new() -> Self {
        Inbox::Memory(MemoryInbox::new())
    }

    /// Create a persistent inbox backed by sled
    pub fn with_storage(db: sled::Db) -> Result<Self, InboxError> {
        Ok(Inbox::Persistent(SledInbox::new(db)?))
    }

    /// Check if a message ID has already been seen (duplicate)
    pub fn is_duplicate(&self, message_id: &str) -> bool {
        match self {
            Inbox::Memory(inbox) => inbox.is_duplicate(message_id),
            Inbox::Persistent(inbox) => inbox.is_duplicate(message_id),
        }
    }

    /// Record a received message. Returns false if duplicate.
    pub fn receive(&mut self, msg: ReceivedMessage) -> bool {
        match self {
            Inbox::Memory(inbox) => inbox.receive(msg),
            Inbox::Persistent(inbox) => inbox.receive(msg).unwrap_or(false),
        }
    }

    /// Get all messages from a specific sender
    pub fn messages_from(&self, sender_id: &str) -> Vec<ReceivedMessage> {
        match self {
            Inbox::Memory(inbox) => inbox
                .messages_from(sender_id)
                .into_iter()
                .cloned()
                .collect(),
            Inbox::Persistent(inbox) => inbox.messages_from(sender_id).unwrap_or_default(),
        }
    }

    /// Get all recent messages across all senders
    pub fn all_messages(&self) -> Vec<ReceivedMessage> {
        match self {
            Inbox::Memory(inbox) => inbox
                .all_messages()
                .into_iter()
                .cloned()
                .collect(),
            Inbox::Persistent(inbox) => inbox.all_messages().unwrap_or_default(),
        }
    }

    /// Total stored messages
    pub fn total_count(&self) -> usize {
        match self {
            Inbox::Memory(inbox) => inbox.total_count(),
            Inbox::Persistent(inbox) => inbox.total_count(),
        }
    }

    /// Number of unique senders
    pub fn sender_count(&self) -> usize {
        match self {
            Inbox::Memory(inbox) => inbox.sender_count(),
            Inbox::Persistent(inbox) => inbox.sender_count().unwrap_or(0),
        }
    }

    /// Clear all messages (but keep dedup IDs)
    pub fn clear_messages(&mut self) {
        match self {
            Inbox::Memory(inbox) => inbox.clear_messages(),
            Inbox::Persistent(inbox) => {
                let _ = inbox.clear_messages();
            }
        }
    }
}

impl Default for Inbox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_received(id: &str, sender: &str, payload: &str) -> ReceivedMessage {
        ReceivedMessage {
            message_id: id.to_string(),
            sender_id: sender.to_string(),
            payload: payload.as_bytes().to_vec(),
            received_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    #[test]
    fn test_memory_receive_and_query() {
        let mut inbox = Inbox::new();

        assert!(inbox.receive(make_received("msg1", "alice", "hello")));
        assert!(inbox.receive(make_received("msg2", "alice", "world")));
        assert!(inbox.receive(make_received("msg3", "bob", "hey")));

        assert_eq!(inbox.total_count(), 3);
        assert_eq!(inbox.sender_count(), 2);
        assert_eq!(inbox.messages_from("alice").len(), 2);
        assert_eq!(inbox.messages_from("bob").len(), 1);
    }

    #[test]
    fn test_memory_deduplication() {
        let mut inbox = Inbox::new();

        assert!(inbox.receive(make_received("msg1", "alice", "hello")));
        assert!(!inbox.receive(make_received("msg1", "alice", "hello"))); // Duplicate
        assert!(!inbox.receive(make_received("msg1", "bob", "different sender same id"))); // Still duplicate

        assert_eq!(inbox.total_count(), 1);
    }

    #[test]
    fn test_memory_is_duplicate() {
        let mut inbox = Inbox::new();

        assert!(!inbox.is_duplicate("msg1"));
        inbox.receive(make_received("msg1", "alice", "hello"));
        assert!(inbox.is_duplicate("msg1"));
    }

    #[test]
    fn test_memory_all_messages() {
        let mut inbox = Inbox::new();
        inbox.receive(make_received("msg1", "alice", "hello"));
        inbox.receive(make_received("msg2", "bob", "world"));

        let all = inbox.all_messages();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_memory_clear_messages() {
        let mut inbox = Inbox::new();
        inbox.receive(make_received("msg1", "alice", "hello"));

        inbox.clear_messages();
        assert_eq!(inbox.total_count(), 0);

        // Dedup IDs should still be tracked
        assert!(inbox.is_duplicate("msg1"));
    }

    #[test]
    fn test_persistent_receive_and_query() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let db = sled::open(path).unwrap();
        let mut inbox = Inbox::with_storage(db).unwrap();

        assert!(inbox.receive(make_received("msg1", "alice", "hello")));
        assert!(inbox.receive(make_received("msg2", "alice", "world")));
        assert!(inbox.receive(make_received("msg3", "bob", "hey")));

        assert_eq!(inbox.total_count(), 3);
        assert_eq!(inbox.sender_count(), 2);
        assert_eq!(inbox.messages_from("alice").len(), 2);
        assert_eq!(inbox.messages_from("bob").len(), 1);
    }

    #[test]
    fn test_persistent_deduplication() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let db = sled::open(path).unwrap();
        let mut inbox = Inbox::with_storage(db).unwrap();

        assert!(inbox.receive(make_received("msg1", "alice", "hello")));
        assert!(!inbox.receive(make_received("msg1", "alice", "hello")));
        assert!(!inbox.receive(make_received("msg1", "bob", "different sender same id")));

        assert_eq!(inbox.total_count(), 1);
    }

    #[test]
    fn test_persistent_is_duplicate() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let db = sled::open(path).unwrap();
        let mut inbox = Inbox::with_storage(db).unwrap();

        assert!(!inbox.is_duplicate("msg1"));
        inbox.receive(make_received("msg1", "alice", "hello"));
        assert!(inbox.is_duplicate("msg1"));
    }

    #[test]
    fn test_persistent_all_messages() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let db = sled::open(path).unwrap();
        let mut inbox = Inbox::with_storage(db).unwrap();

        inbox.receive(make_received("msg1", "alice", "hello"));
        inbox.receive(make_received("msg2", "bob", "world"));

        let all = inbox.all_messages();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_persistent_clear_messages() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let db = sled::open(path).unwrap();
        let mut inbox = Inbox::with_storage(db).unwrap();

        inbox.receive(make_received("msg1", "alice", "hello"));
        inbox.clear_messages();

        assert_eq!(inbox.total_count(), 0);
        assert!(inbox.is_duplicate("msg1"));
    }

    #[test]
    fn test_persistent_survives_restart() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        // First instance: write messages
        {
            let db = sled::open(path).unwrap();
            let mut inbox = Inbox::with_storage(db).unwrap();
            inbox.receive(make_received("msg1", "alice", "hello"));
            inbox.receive(make_received("msg2", "bob", "world"));
            assert_eq!(inbox.total_count(), 2);
        }

        // Second instance: read messages and verify dedup
        {
            let db = sled::open(path).unwrap();
            let mut inbox = Inbox::with_storage(db).unwrap();
            assert_eq!(inbox.total_count(), 2);
            assert!(inbox.is_duplicate("msg1"));
            assert!(inbox.is_duplicate("msg2"));

            // Receiving same message again should be rejected
            assert!(!inbox.receive(make_received("msg1", "alice", "hello")));
        }
    }
}
