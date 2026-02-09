// Inbox — receive and deduplicate incoming messages
//
// Tracks seen message IDs to prevent replay attacks and duplicate delivery.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Maximum tracked message IDs (for deduplication)
const MAX_SEEN_IDS: usize = 50_000;

const SEEN_IDS_KEY: &[u8] = b"inbox_seen_ids";
const MESSAGES_PREFIX: &[u8] = b"inbox_msg_";

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

/// Storage backend for inbox
enum InboxBackend {
    Memory {
        seen_ids: HashSet<String>,
        seen_order: Vec<String>,
        messages: HashMap<String, Vec<ReceivedMessage>>,
        total: usize,
    },
    Persistent(sled::Db),
}

/// Inbound message deduplication and storage
pub struct Inbox {
    backend: InboxBackend,
}

impl Inbox {
    /// Create a new in-memory inbox
    pub fn new() -> Self {
        Self {
            backend: InboxBackend::Memory {
                seen_ids: HashSet::new(),
                seen_order: Vec::new(),
                messages: HashMap::new(),
                total: 0,
            },
        }
    }

    /// Create a persistent inbox with sled backend
    pub fn persistent(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self {
            backend: InboxBackend::Persistent(db),
        })
    }

    /// Check if a message ID has already been seen (duplicate)
    pub fn is_duplicate(&self, message_id: &str) -> bool {
        match &self.backend {
            InboxBackend::Memory { seen_ids, .. } => seen_ids.contains(message_id),
            InboxBackend::Persistent(db) => {
                if let Ok(Some(bytes)) = db.get(SEEN_IDS_KEY) {
                    if let Ok(seen_ids) = bincode::deserialize::<HashSet<String>>(&bytes) {
                        return seen_ids.contains(message_id);
                    }
                }
                false
            }
        }
    }

    /// Record a received message. Returns false if duplicate.
    pub fn receive(&mut self, msg: ReceivedMessage) -> bool {
        match &mut self.backend {
            InboxBackend::Memory {
                seen_ids,
                seen_order,
                messages,
                total,
            } => {
                if seen_ids.contains(&msg.message_id) {
                    return false; // Duplicate
                }

                // Track for dedup
                seen_ids.insert(msg.message_id.clone());
                seen_order.push(msg.message_id.clone());

                // Evict old IDs if at capacity
                while seen_ids.len() > MAX_SEEN_IDS {
                    if let Some(old_id) = seen_order.first().cloned() {
                        seen_order.remove(0);
                        seen_ids.remove(&old_id);
                    }
                }

                // Store message
                messages.entry(msg.sender_id.clone()).or_default().push(msg);
                *total += 1;

                true // New message
            }
            InboxBackend::Persistent(db) => {
                // Load seen IDs
                let mut seen_ids: HashSet<String> = db
                    .get(SEEN_IDS_KEY)
                    .ok()
                    .flatten()
                    .and_then(|bytes| bincode::deserialize(&bytes).ok())
                    .unwrap_or_default();

                if seen_ids.contains(&msg.message_id) {
                    return false; // Duplicate
                }

                // Add to seen set
                seen_ids.insert(msg.message_id.clone());

                // Evict if needed (simple approach: keep most recent)
                if seen_ids.len() > MAX_SEEN_IDS {
                    // In a real impl, we'd track order. For now, just clear oldest randomly
                    let to_remove: Vec<_> = seen_ids.iter().take(1000).cloned().collect();
                    for id in to_remove {
                        seen_ids.remove(&id);
                    }
                }

                // Save seen IDs
                if let Ok(bytes) = bincode::serialize(&seen_ids) {
                    let _ = db.insert(SEEN_IDS_KEY, bytes);
                }

                // Store message
                let key = format!("{}{}_{}",
                    String::from_utf8_lossy(MESSAGES_PREFIX),
                    msg.sender_id,
                    msg.message_id
                );
                if let Ok(bytes) = bincode::serialize(&msg) {
                    let _ = db.insert(key.as_bytes(), bytes);
                    let _ = db.flush();
                }

                true // New message
            }
        }
    }

    /// Get all messages from a specific sender
    pub fn messages_from(&self, sender_id: &str) -> Vec<ReceivedMessage> {
        match &self.backend {
            InboxBackend::Memory { messages, .. } => messages
                .get(sender_id)
                .map(|msgs| msgs.clone())
                .unwrap_or_default(),
            InboxBackend::Persistent(db) => {
                let prefix = format!("{}{}_", String::from_utf8_lossy(MESSAGES_PREFIX), sender_id);
                db.scan_prefix(prefix.as_bytes())
                    .filter_map(|result| result.ok())
                    .filter_map(|(_, value)| bincode::deserialize(&value).ok())
                    .collect()
            }
        }
    }

    /// Get all recent messages across all senders
    pub fn all_messages(&self) -> Vec<ReceivedMessage> {
        match &self.backend {
            InboxBackend::Memory { messages, .. } => {
                messages.values().flat_map(|msgs| msgs.clone()).collect()
            }
            InboxBackend::Persistent(db) => db
                .scan_prefix(MESSAGES_PREFIX)
                .filter_map(|result| result.ok())
                .filter_map(|(_, value)| bincode::deserialize(&value).ok())
                .collect(),
        }
    }

    /// Total stored messages
    pub fn total_count(&self) -> usize {
        match &self.backend {
            InboxBackend::Memory { total, .. } => *total,
            InboxBackend::Persistent(db) => {
                db.scan_prefix(MESSAGES_PREFIX).count()
            }
        }
    }

    /// Number of unique senders
    pub fn sender_count(&self) -> usize {
        match &self.backend {
            InboxBackend::Memory { messages, .. } => messages.len(),
            InboxBackend::Persistent(db) => {
                let mut senders: HashSet<String> = HashSet::new();
                for result in db.scan_prefix(MESSAGES_PREFIX) {
                    if let Ok((_, value)) = result {
                        if let Ok(msg) = bincode::deserialize::<ReceivedMessage>(&value) {
                            senders.insert(msg.sender_id);
                        }
                    }
                }
                senders.len()
            }
        }
    }

    /// Clear all messages (but keep dedup IDs)
    pub fn clear_messages(&mut self) {
        match &mut self.backend {
            InboxBackend::Memory { messages, total, .. } => {
                messages.clear();
                *total = 0;
            }
            InboxBackend::Persistent(db) => {
                // Remove all message keys (but keep seen IDs)
                let keys_to_remove: Vec<_> = db
                    .scan_prefix(MESSAGES_PREFIX)
                    .filter_map(|r| r.ok())
                    .map(|(k, _)| k)
                    .collect();

                for key in keys_to_remove {
                    let _ = db.remove(key);
                }
                let _ = db.flush();
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
    fn test_receive_and_query() {
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
    fn test_deduplication() {
        let mut inbox = Inbox::new();

        assert!(inbox.receive(make_received("msg1", "alice", "hello")));
        assert!(!inbox.receive(make_received("msg1", "alice", "hello"))); // Duplicate
        assert!(!inbox.receive(make_received("msg1", "bob", "different sender same id"))); // Still duplicate

        assert_eq!(inbox.total_count(), 1);
    }

    #[test]
    fn test_is_duplicate() {
        let mut inbox = Inbox::new();

        assert!(!inbox.is_duplicate("msg1"));
        inbox.receive(make_received("msg1", "alice", "hello"));
        assert!(inbox.is_duplicate("msg1"));
    }

    #[test]
    fn test_all_messages() {
        let mut inbox = Inbox::new();
        inbox.receive(make_received("msg1", "alice", "hello"));
        inbox.receive(make_received("msg2", "bob", "world"));

        let all = inbox.all_messages();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_clear_messages() {
        let mut inbox = Inbox::new();
        inbox.receive(make_received("msg1", "alice", "hello"));

        inbox.clear_messages();
        assert_eq!(inbox.total_count(), 0);

        // Dedup IDs should still be tracked
        assert!(inbox.is_duplicate("msg1"));
    }

    #[test]
    fn test_persistent_inbox() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("inbox_store").to_str().unwrap().to_string();

        let mut inbox = Inbox::persistent(&path).unwrap();

        // Receive messages
        assert!(inbox.receive(make_received("msg1", "alice", "hello")));
        assert!(inbox.receive(make_received("msg2", "bob", "world")));

        assert_eq!(inbox.total_count(), 2);
        assert_eq!(inbox.sender_count(), 2);

        // Test deduplication
        assert!(!inbox.receive(make_received("msg1", "alice", "duplicate")));

        // Messages should be retrievable
        let alice_msgs = inbox.messages_from("alice");
        assert_eq!(alice_msgs.len(), 1);
        assert_eq!(alice_msgs[0].message_id, "msg1");
    }

    #[test]
    fn test_persistent_inbox_survives_restart() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("inbox_store").to_str().unwrap().to_string();

        // First instance: receive messages
        {
            let mut inbox = Inbox::persistent(&path).unwrap();
            inbox.receive(make_received("msg1", "alice", "hello"));
            inbox.receive(make_received("msg2", "bob", "world"));
        }

        // Second instance: messages should still be there
        {
            let inbox = Inbox::persistent(&path).unwrap();
            assert_eq!(inbox.total_count(), 2);
            assert!(inbox.is_duplicate("msg1"));
            assert!(inbox.is_duplicate("msg2"));

            let all = inbox.all_messages();
            assert_eq!(all.len(), 2);
        }
    }
}
