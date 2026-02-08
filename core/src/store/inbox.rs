// Inbox — receive and deduplicate incoming messages
//
// Tracks seen message IDs to prevent replay attacks and duplicate delivery.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;
use tracing::debug;

/// Maximum tracked message IDs (for deduplication)
const MAX_SEEN_IDS: usize = 50_000;

/// Maximum stored messages across all senders (matches outbox MAX_TOTAL_QUEUED)
const MAX_STORED_MESSAGES: usize = 10_000;

/// Maximum stored messages per sender
const MAX_MESSAGES_PER_SENDER: usize = 1_000;

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
    /// High water mark: timestamp of the most recently evicted message.
    /// Messages older than this are silently rejected to prevent Zombie Loop —
    /// where peers endlessly re-sync messages that were already evicted.
    eviction_high_water_mark: u64,
}

impl MemoryInbox {
    pub fn new() -> Self {
        Self {
            seen_ids: HashSet::new(),
            seen_order: VecDeque::new(),
            messages: HashMap::new(),
            total: 0,
            eviction_high_water_mark: 0,
        }
    }

    /// Check if a message ID has already been seen (duplicate)
    pub fn is_duplicate(&self, message_id: &str) -> bool {
        self.seen_ids.contains(message_id)
    }

    /// Record a received message. Returns false if duplicate.
    ///
    /// Enforces storage quotas:
    /// - `MAX_STORED_MESSAGES` (10,000) total across all senders
    /// - `MAX_MESSAGES_PER_SENDER` (1,000) per individual sender
    ///
    /// When quotas are hit, the oldest messages are evicted to make room.
    pub fn receive(&mut self, msg: ReceivedMessage) -> bool {
        if self.seen_ids.contains(&msg.message_id) {
            return false; // Duplicate
        }

        // Zombie Loop prevention: reject messages older than the high water mark.
        // Once we evict a message due to capacity, any re-sync of that message
        // (or older) from peers is silently dropped to prevent infinite re-sync cycles.
        if self.eviction_high_water_mark > 0 && msg.received_at <= self.eviction_high_water_mark {
            debug!(
                "Rejecting message {} — older than eviction high water mark ({})",
                msg.message_id, self.eviction_high_water_mark
            );
            return false;
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

        // Evict oldest message globally if at total capacity
        if self.total >= MAX_STORED_MESSAGES {
            self.evict_oldest_message();
        }

        // Evict oldest from this sender if at per-sender capacity
        let sender_msgs = self.messages.entry(msg.sender_id.clone()).or_default();
        if sender_msgs.len() >= MAX_MESSAGES_PER_SENDER {
            sender_msgs.remove(0); // Remove oldest (front of vec)
            self.total -= 1;
        }

        // Store message
        sender_msgs.push(msg);
        self.total += 1;

        true // New message
    }

    /// Evict the single oldest message across all senders.
    /// Updates the high water mark to prevent Zombie Loop re-sync.
    fn evict_oldest_message(&mut self) {
        let mut oldest_sender: Option<String> = None;
        let mut oldest_time = u64::MAX;

        for (sender, msgs) in &self.messages {
            if let Some(first) = msgs.first() {
                if first.received_at < oldest_time {
                    oldest_time = first.received_at;
                    oldest_sender = Some(sender.clone());
                }
            }
        }

        if let Some(sender) = oldest_sender {
            if let Some(msgs) = self.messages.get_mut(&sender) {
                msgs.remove(0);
                self.total -= 1;
                if msgs.is_empty() {
                    self.messages.remove(&sender);
                }
                // Advance high water mark — any message at or before this timestamp
                // is now considered "already processed and evicted"
                if oldest_time > self.eviction_high_water_mark {
                    self.eviction_high_water_mark = oldest_time;
                }
            }
        }
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
    /// High water mark: timestamp of the most recently evicted message.
    /// Messages older than this are silently rejected (Zombie Loop prevention).
    eviction_high_water_mark: std::sync::Arc<std::sync::atomic::AtomicU64>,
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

        // Load persisted high water mark if it exists
        let hwm = db.open_tree("inbox_meta")
            .ok()
            .and_then(|tree| tree.get(b"high_water_mark").ok().flatten())
            .and_then(|bytes| bincode::deserialize::<u64>(&bytes).ok())
            .unwrap_or(0);

        Ok(Self {
            db,
            seen_tree,
            messages_tree,
            total: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(total)),
            eviction_high_water_mark: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(hwm)),
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
    ///
    /// Enforces the same storage quotas as MemoryInbox:
    /// - `MAX_STORED_MESSAGES` (10,000) total
    /// - `MAX_MESSAGES_PER_SENDER` (1,000) per sender
    pub fn receive(&self, msg: ReceivedMessage) -> Result<bool, InboxError> {
        if self.is_duplicate(&msg.message_id) {
            return Ok(false); // Duplicate
        }

        // Zombie Loop prevention: reject messages older than the high water mark
        let hwm = self.eviction_high_water_mark.load(std::sync::atomic::Ordering::Relaxed);
        if hwm > 0 && msg.received_at <= hwm {
            debug!(
                "Rejecting message {} — older than eviction high water mark ({})",
                msg.message_id, hwm
            );
            return Ok(false);
        }

        // Store seen ID with timestamp
        let timestamp = msg.received_at;
        let timestamp_bytes = bincode::serialize(&timestamp)
            .map_err(|e| InboxError::SerializationError(e.to_string()))?;

        self.seen_tree
            .insert(msg.message_id.as_bytes(), timestamp_bytes)
            .map_err(|e| InboxError::StorageError(e.to_string()))?;

        // Evict oldest globally if at capacity
        let current_total = self.total.load(std::sync::atomic::Ordering::Relaxed);
        if current_total >= MAX_STORED_MESSAGES {
            self.evict_oldest_message()?;
        }

        // Evict oldest from this sender if at per-sender capacity
        let sender_prefix = format!("{}:", msg.sender_id);
        let sender_count = self.messages_tree.scan_prefix(sender_prefix.as_bytes()).count();
        if sender_count >= MAX_MESSAGES_PER_SENDER {
            // Remove the first (oldest) entry for this sender
            if let Some(Ok((oldest_key, _))) = self.messages_tree.scan_prefix(sender_prefix.as_bytes()).next() {
                self.messages_tree.remove(oldest_key)
                    .map_err(|e| InboxError::StorageError(e.to_string()))?;
                self.total.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            }
        }

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

    /// Evict the single oldest message across all senders.
    /// Updates the high water mark to prevent Zombie Loop re-sync.
    fn evict_oldest_message(&self) -> Result<(), InboxError> {
        let mut oldest_key: Option<sled::IVec> = None;
        let mut oldest_time = u64::MAX;

        for entry in self.messages_tree.iter() {
            let (key, value) = entry.map_err(|e| InboxError::StorageError(e.to_string()))?;
            if let Ok(msg) = bincode::deserialize::<ReceivedMessage>(&value) {
                if msg.received_at < oldest_time {
                    oldest_time = msg.received_at;
                    oldest_key = Some(key);
                }
            }
        }

        if let Some(key) = oldest_key {
            self.messages_tree.remove(key)
                .map_err(|e| InboxError::StorageError(e.to_string()))?;
            self.total.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

            // Advance high water mark and persist it
            let current_hwm = self.eviction_high_water_mark.load(std::sync::atomic::Ordering::Relaxed);
            if oldest_time > current_hwm {
                self.eviction_high_water_mark.store(oldest_time, std::sync::atomic::Ordering::Relaxed);
                // Persist to sled so it survives restarts
                if let Ok(meta_tree) = self.db.open_tree("inbox_meta") {
                    if let Ok(bytes) = bincode::serialize(&oldest_time) {
                        let _ = meta_tree.insert(b"high_water_mark", bytes);
                    }
                }
            }
        }

        Ok(())
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
