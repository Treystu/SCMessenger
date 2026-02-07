// Outbox — queue messages for peers that may be offline
//
// Messages are stored locally and retried when the peer comes online.
// This is the foundation for store-and-forward delivery.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;
use tracing::debug;

/// Maximum messages queued per peer
const MAX_QUEUE_PER_PEER: usize = 1000;

/// Maximum total messages across all peers
const MAX_TOTAL_QUEUED: usize = 10_000;

#[derive(Debug, Error, Clone)]
pub enum OutboxError {
    #[error("Outbox full ({0} messages)")]
    OutboxFull(usize),
    #[error("Queue full for peer {0} ({1} messages)")]
    QueueFullForPeer(String, usize),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// A queued outbound message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMessage {
    /// Unique message ID
    pub message_id: String,
    /// Target peer's identity ID
    pub recipient_id: String,
    /// Serialized envelope bytes
    pub envelope_data: Vec<u8>,
    /// When this was queued (unix timestamp)
    pub queued_at: u64,
    /// Number of delivery attempts
    pub attempts: u32,
}

/// In-memory outbound message queue
pub struct MemoryOutbox {
    /// Messages queued per recipient
    queues: HashMap<String, VecDeque<QueuedMessage>>,
    /// Total message count
    total: usize,
}

impl MemoryOutbox {
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
            total: 0,
        }
    }

    /// Queue a message for delivery
    pub fn enqueue(&mut self, msg: QueuedMessage) -> Result<(), OutboxError> {
        if self.total >= MAX_TOTAL_QUEUED {
            return Err(OutboxError::OutboxFull(MAX_TOTAL_QUEUED));
        }

        let queue = self.queues.entry(msg.recipient_id.clone()).or_default();

        if queue.len() >= MAX_QUEUE_PER_PEER {
            return Err(OutboxError::QueueFullForPeer(
                msg.recipient_id.clone(),
                MAX_QUEUE_PER_PEER,
            ));
        }

        queue.push_back(msg);
        self.total += 1;
        Ok(())
    }

    /// Get all queued messages for a peer (without removing them)
    pub fn peek_for_peer(&self, recipient_id: &str) -> Vec<&QueuedMessage> {
        self.queues
            .get(recipient_id)
            .map(|q| q.iter().collect())
            .unwrap_or_default()
    }

    /// Remove a specific message by ID (after successful delivery)
    pub fn remove(&mut self, message_id: &str) -> bool {
        let mut found = false;
        for queue in self.queues.values_mut() {
            if let Some(pos) = queue.iter().position(|m| m.message_id == message_id) {
                queue.remove(pos);
                self.total -= 1;
                found = true;
                break;
            }
        }
        if found {
            // Clean up empty queues so peer_count() stays accurate
            self.queues.retain(|_, q| !q.is_empty());
        }
        found
    }

    /// Drain all messages for a peer (for batch delivery)
    pub fn drain_for_peer(&mut self, recipient_id: &str) -> Vec<QueuedMessage> {
        if let Some(queue) = self.queues.remove(recipient_id) {
            let count = queue.len();
            self.total -= count;
            queue.into()
        } else {
            Vec::new()
        }
    }

    /// Increment attempt count for a message
    pub fn record_attempt(&mut self, message_id: &str) {
        for queue in self.queues.values_mut() {
            if let Some(msg) = queue.iter_mut().find(|m| m.message_id == message_id) {
                msg.attempts += 1;
                return;
            }
        }
    }

    /// Total queued messages
    pub fn total_count(&self) -> usize {
        self.total
    }

    /// Number of peers with queued messages
    pub fn peer_count(&self) -> usize {
        self.queues.len()
    }

    /// Remove expired messages (older than max_age_secs)
    pub fn remove_expired(&mut self, max_age_secs: u64) -> usize {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut removed = 0;

        for queue in self.queues.values_mut() {
            let before = queue.len();
            queue.retain(|msg| now.saturating_sub(msg.queued_at) < max_age_secs);
            removed += before - queue.len();
        }

        self.total -= removed;

        // Clean up empty queues
        self.queues.retain(|_, q| !q.is_empty());

        removed
    }
}

impl Default for MemoryOutbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Persistent sled-backed outbound message queue
pub struct SledOutbox {
    #[allow(dead_code)] // Keeps sled::Db alive (RAII)
    db: sled::Db,
    tree: sled::Tree,
    total: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl SledOutbox {
    /// Create or open a persistent outbox
    pub fn new(db: sled::Db) -> Result<Self, OutboxError> {
        let tree = db
            .open_tree("outbox")
            .map_err(|e| OutboxError::StorageError(e.to_string()))?;

        // Count existing messages
        let total = tree
            .iter()
            .count();

        Ok(Self {
            db,
            tree,
            total: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(total)),
        })
    }

    fn make_key(recipient_id: &str, message_id: &str) -> Vec<u8> {
        format!("{}:{}", recipient_id, message_id).into_bytes()
    }

    /// Queue a message for delivery
    pub fn enqueue(&self, msg: QueuedMessage) -> Result<(), OutboxError> {
        let total = self.total.load(std::sync::atomic::Ordering::Relaxed);
        if total >= MAX_TOTAL_QUEUED {
            return Err(OutboxError::OutboxFull(MAX_TOTAL_QUEUED));
        }

        // Count messages for this peer
        let peer_key_prefix = format!("{}:", msg.recipient_id);
        let peer_count = self
            .tree
            .scan_prefix(peer_key_prefix.as_bytes())
            .count();

        if peer_count >= MAX_QUEUE_PER_PEER {
            return Err(OutboxError::QueueFullForPeer(
                msg.recipient_id.clone(),
                MAX_QUEUE_PER_PEER,
            ));
        }

        let key = Self::make_key(&msg.recipient_id, &msg.message_id);
        let value = bincode::serialize(&msg)
            .map_err(|e| OutboxError::SerializationError(e.to_string()))?;

        self.tree
            .insert(key, value)
            .map_err(|e| OutboxError::StorageError(e.to_string()))?;

        self.total.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        debug!("Enqueued message {} for peer {}", msg.message_id, msg.recipient_id);
        Ok(())
    }

    /// Get all queued messages for a peer (without removing them)
    pub fn peek_for_peer(&self, recipient_id: &str) -> Result<Vec<QueuedMessage>, OutboxError> {
        let prefix = format!("{}:", recipient_id);
        let mut messages = Vec::new();

        for entry in self.tree.scan_prefix(prefix.as_bytes()) {
            let (_, value) = entry.map_err(|e: sled::Error| OutboxError::StorageError(e.to_string()))?;
            let msg: QueuedMessage = bincode::deserialize(&value)
                .map_err(|e| OutboxError::SerializationError(e.to_string()))?;
            messages.push(msg);
        }

        Ok(messages)
    }

    /// Remove a specific message by ID (after successful delivery)
    pub fn remove(&self, message_id: &str) -> Result<bool, OutboxError> {
        // Search for the message to find its recipient
        for entry in self.tree.iter() {
            let (key, value) = entry.map_err(|e: sled::Error| OutboxError::StorageError(e.to_string()))?;
            let msg: QueuedMessage = bincode::deserialize(&value)
                .map_err(|e| OutboxError::SerializationError(e.to_string()))?;

            if msg.message_id == message_id {
                self.tree
                    .remove(key)
                    .map_err(|e: sled::Error| OutboxError::StorageError(e.to_string()))?;
                self.total.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                debug!("Removed message {}", message_id);
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Drain all messages for a peer (for batch delivery)
    pub fn drain_for_peer(&self, recipient_id: &str) -> Result<Vec<QueuedMessage>, OutboxError> {
        let prefix = format!("{}:", recipient_id);
        let mut messages = Vec::new();
        let mut keys_to_remove: Vec<Vec<u8>> = Vec::new();

        for entry in self.tree.scan_prefix(prefix.as_bytes()) {
            let (key, value) = entry.map_err(|e: sled::Error| OutboxError::StorageError(e.to_string()))?;
            let msg: QueuedMessage = bincode::deserialize(&value)
                .map_err(|e| OutboxError::SerializationError(e.to_string()))?;
            keys_to_remove.push(key.to_vec());
            messages.push(msg);
        }

        for key in keys_to_remove {
            self.tree
                .remove(key)
                .map_err(|e: sled::Error| OutboxError::StorageError(e.to_string()))?;
            self.total.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        }

        debug!("Drained {} messages for peer {}", messages.len(), recipient_id);
        Ok(messages)
    }

    /// Increment attempt count for a message
    pub fn record_attempt(&self, message_id: &str) -> Result<(), OutboxError> {
        for entry in self.tree.iter() {
            let (key, value) = entry.map_err(|e: sled::Error| OutboxError::StorageError(e.to_string()))?;
            let mut msg: QueuedMessage = bincode::deserialize(&value)
                .map_err(|e| OutboxError::SerializationError(e.to_string()))?;

            if msg.message_id == message_id {
                msg.attempts += 1;
                let new_value = bincode::serialize(&msg)
                    .map_err(|e| OutboxError::SerializationError(e.to_string()))?;
                self.tree
                    .insert(key, new_value)
                    .map_err(|e: sled::Error| OutboxError::StorageError(e.to_string()))?;
                return Ok(());
            }
        }
        Ok(())
    }

    /// Total queued messages
    pub fn total_count(&self) -> usize {
        self.total.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Number of peers with queued messages
    pub fn peer_count(&self) -> Result<usize, OutboxError> {
        let mut peers = std::collections::HashSet::new();

        for entry in self.tree.iter() {
            let (_, value) = entry.map_err(|e: sled::Error| OutboxError::StorageError(e.to_string()))?;
            let msg: QueuedMessage = bincode::deserialize(&value)
                .map_err(|e| OutboxError::SerializationError(e.to_string()))?;
            peers.insert(msg.recipient_id);
        }

        Ok(peers.len())
    }

    /// Remove expired messages (older than max_age_secs)
    pub fn remove_expired(&self, max_age_secs: u64) -> Result<usize, OutboxError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut removed = 0;
        let mut keys_to_remove: Vec<Vec<u8>> = Vec::new();

        for entry in self.tree.iter() {
            let (key, value) = entry.map_err(|e: sled::Error| OutboxError::StorageError(e.to_string()))?;
            let msg: QueuedMessage = bincode::deserialize(&value)
                .map_err(|e| OutboxError::SerializationError(e.to_string()))?;

            if now.saturating_sub(msg.queued_at) >= max_age_secs {
                keys_to_remove.push(key.to_vec());
            }
        }

        for key in keys_to_remove {
            self.tree
                .remove(key)
                .map_err(|e: sled::Error| OutboxError::StorageError(e.to_string()))?;
            removed += 1;
        }

        self.total.fetch_sub(removed, std::sync::atomic::Ordering::Relaxed);
        debug!("Removed {} expired messages", removed);
        Ok(removed)
    }
}

/// Outbox storage backend — supports both in-memory and persistent modes
pub enum Outbox {
    Memory(MemoryOutbox),
    Persistent(SledOutbox),
}

impl Outbox {
    /// Create a new in-memory outbox
    pub fn new() -> Self {
        Outbox::Memory(MemoryOutbox::new())
    }

    /// Create a persistent outbox backed by sled
    pub fn with_storage(db: sled::Db) -> Result<Self, OutboxError> {
        Ok(Outbox::Persistent(SledOutbox::new(db)?))
    }

    /// Queue a message for delivery
    pub fn enqueue(&mut self, msg: QueuedMessage) -> Result<(), OutboxError> {
        match self {
            Outbox::Memory(inbox) => inbox.enqueue(msg),
            Outbox::Persistent(inbox) => inbox.enqueue(msg),
        }
    }

    /// Get all queued messages for a peer (without removing them)
    pub fn peek_for_peer(&self, recipient_id: &str) -> Result<Vec<QueuedMessage>, OutboxError> {
        match self {
            Outbox::Memory(outbox) => Ok(outbox
                .peek_for_peer(recipient_id)
                .into_iter()
                .cloned()
                .collect()),
            Outbox::Persistent(outbox) => outbox.peek_for_peer(recipient_id),
        }
    }

    /// Remove a specific message by ID (after successful delivery)
    pub fn remove(&mut self, message_id: &str) -> Result<bool, OutboxError> {
        match self {
            Outbox::Memory(outbox) => Ok(outbox.remove(message_id)),
            Outbox::Persistent(outbox) => outbox.remove(message_id),
        }
    }

    /// Drain all messages for a peer (for batch delivery)
    pub fn drain_for_peer(&mut self, recipient_id: &str) -> Result<Vec<QueuedMessage>, OutboxError> {
        match self {
            Outbox::Memory(outbox) => Ok(outbox.drain_for_peer(recipient_id)),
            Outbox::Persistent(outbox) => outbox.drain_for_peer(recipient_id),
        }
    }

    /// Increment attempt count for a message
    pub fn record_attempt(&mut self, message_id: &str) -> Result<(), OutboxError> {
        match self {
            Outbox::Memory(outbox) => {
                outbox.record_attempt(message_id);
                Ok(())
            }
            Outbox::Persistent(outbox) => outbox.record_attempt(message_id),
        }
    }

    /// Total queued messages
    pub fn total_count(&self) -> usize {
        match self {
            Outbox::Memory(outbox) => outbox.total_count(),
            Outbox::Persistent(outbox) => outbox.total_count(),
        }
    }

    /// Number of peers with queued messages
    pub fn peer_count(&self) -> Result<usize, OutboxError> {
        match self {
            Outbox::Memory(outbox) => Ok(outbox.peer_count()),
            Outbox::Persistent(outbox) => outbox.peer_count(),
        }
    }

    /// Remove expired messages (older than max_age_secs)
    pub fn remove_expired(&mut self, max_age_secs: u64) -> Result<usize, OutboxError> {
        match self {
            Outbox::Memory(outbox) => Ok(outbox.remove_expired(max_age_secs)),
            Outbox::Persistent(outbox) => outbox.remove_expired(max_age_secs),
        }
    }
}

impl Default for Outbox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_msg(id: &str, recipient: &str) -> QueuedMessage {
        QueuedMessage {
            message_id: id.to_string(),
            recipient_id: recipient.to_string(),
            envelope_data: vec![1, 2, 3],
            queued_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            attempts: 0,
        }
    }

    #[test]
    fn test_memory_enqueue_and_peek() {
        let mut outbox = Outbox::new();
        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg3", "peer_b")).unwrap();

        assert_eq!(outbox.total_count(), 3);
        assert_eq!(outbox.peer_count().unwrap(), 2);
        assert_eq!(outbox.peek_for_peer("peer_a").unwrap().len(), 2);
        assert_eq!(outbox.peek_for_peer("peer_b").unwrap().len(), 1);
        assert_eq!(outbox.peek_for_peer("peer_c").unwrap().len(), 0);
    }

    #[test]
    fn test_memory_remove() {
        let mut outbox = Outbox::new();
        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();

        assert!(outbox.remove("msg1").unwrap());
        assert_eq!(outbox.total_count(), 1);
        assert!(!outbox.remove("msg1").unwrap()); // Already removed
    }

    #[test]
    fn test_memory_drain_for_peer() {
        let mut outbox = Outbox::new();
        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg3", "peer_b")).unwrap();

        let drained = outbox.drain_for_peer("peer_a").unwrap();
        assert_eq!(drained.len(), 2);
        assert_eq!(outbox.total_count(), 1);
        assert_eq!(outbox.peek_for_peer("peer_a").unwrap().len(), 0);
    }

    #[test]
    fn test_memory_record_attempt() {
        let mut outbox = Outbox::new();
        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();

        outbox.record_attempt("msg1").unwrap();
        outbox.record_attempt("msg1").unwrap();

        let msgs = outbox.peek_for_peer("peer_a").unwrap();
        assert_eq!(msgs[0].attempts, 2);
    }

    #[test]
    fn test_memory_remove_expired() {
        let mut outbox = Outbox::new();

        let mut old_msg = make_msg("old", "peer_a");
        old_msg.queued_at = 0; // epoch = very old
        outbox.enqueue(old_msg).unwrap();

        let fresh_msg = make_msg("fresh", "peer_a");
        outbox.enqueue(fresh_msg).unwrap();

        let removed = outbox.remove_expired(3600).unwrap(); // 1 hour max age
        assert_eq!(removed, 1);
        assert_eq!(outbox.total_count(), 1);
    }

    #[test]
    fn test_persistent_enqueue_and_peek() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let db = sled::open(path).unwrap();
        let mut outbox = Outbox::with_storage(db).unwrap();

        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg3", "peer_b")).unwrap();

        assert_eq!(outbox.total_count(), 3);
        assert_eq!(outbox.peer_count().unwrap(), 2);
        assert_eq!(outbox.peek_for_peer("peer_a").unwrap().len(), 2);
        assert_eq!(outbox.peek_for_peer("peer_b").unwrap().len(), 1);
        assert_eq!(outbox.peek_for_peer("peer_c").unwrap().len(), 0);
    }

    #[test]
    fn test_persistent_remove() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let db = sled::open(path).unwrap();
        let mut outbox = Outbox::with_storage(db).unwrap();

        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();

        assert!(outbox.remove("msg1").unwrap());
        assert_eq!(outbox.total_count(), 1);
        assert!(!outbox.remove("msg1").unwrap());
    }

    #[test]
    fn test_persistent_drain_for_peer() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let db = sled::open(path).unwrap();
        let mut outbox = Outbox::with_storage(db).unwrap();

        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg3", "peer_b")).unwrap();

        let drained = outbox.drain_for_peer("peer_a").unwrap();
        assert_eq!(drained.len(), 2);
        assert_eq!(outbox.total_count(), 1);
        assert_eq!(outbox.peek_for_peer("peer_a").unwrap().len(), 0);
    }

    #[test]
    fn test_persistent_record_attempt() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let db = sled::open(path).unwrap();
        let mut outbox = Outbox::with_storage(db).unwrap();

        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();

        outbox.record_attempt("msg1").unwrap();
        outbox.record_attempt("msg1").unwrap();

        let msgs = outbox.peek_for_peer("peer_a").unwrap();
        assert_eq!(msgs[0].attempts, 2);
    }

    #[test]
    fn test_persistent_remove_expired() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let db = sled::open(path).unwrap();
        let mut outbox = Outbox::with_storage(db).unwrap();

        let mut old_msg = make_msg("old", "peer_a");
        old_msg.queued_at = 0;
        outbox.enqueue(old_msg).unwrap();

        let fresh_msg = make_msg("fresh", "peer_a");
        outbox.enqueue(fresh_msg).unwrap();

        let removed = outbox.remove_expired(3600).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(outbox.total_count(), 1);
    }

    #[test]
    fn test_persistent_survives_restart() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        // First instance: write messages
        {
            let db = sled::open(path).unwrap();
            let mut outbox = Outbox::with_storage(db).unwrap();
            outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
            outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();
            assert_eq!(outbox.total_count(), 2);
        }

        // Second instance: read messages
        {
            let db = sled::open(path).unwrap();
            let outbox = Outbox::with_storage(db).unwrap();
            assert_eq!(outbox.total_count(), 2);
            assert_eq!(outbox.peek_for_peer("peer_a").unwrap().len(), 2);
        }
    }
}
