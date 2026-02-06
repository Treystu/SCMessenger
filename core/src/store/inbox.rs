// Inbox — receive and deduplicate incoming messages
//
// Tracks seen message IDs to prevent replay attacks and duplicate delivery.

use std::collections::{HashMap, HashSet, VecDeque};

/// Maximum tracked message IDs (for deduplication)
const MAX_SEEN_IDS: usize = 50_000;

/// A received message record
#[derive(Debug, Clone)]
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

/// Inbound message deduplication and storage
pub struct Inbox {
    /// Set of seen message IDs (for dedup)
    seen_ids: HashSet<String>,
    /// Ordered list of seen IDs (for O(1) FIFO eviction)
    seen_order: VecDeque<String>,
    /// Recent messages by sender (for application retrieval)
    messages: HashMap<String, Vec<ReceivedMessage>>,
    /// Total stored messages
    total: usize,
}

impl Inbox {
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
}
