// Outbox — queue messages for peers that may be offline
//
// Messages are stored locally and retried when the peer comes online.
// This is the foundation for store-and-forward delivery.

use std::collections::{HashMap, VecDeque};

/// Maximum messages queued per peer
const MAX_QUEUE_PER_PEER: usize = 1000;

/// Maximum total messages across all peers
const MAX_TOTAL_QUEUED: usize = 10_000;

/// A queued outbound message
#[derive(Debug, Clone)]
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

/// Outbound message queue
pub struct Outbox {
    /// Messages queued per recipient
    queues: HashMap<String, VecDeque<QueuedMessage>>,
    /// Total message count
    total: usize,
}

impl Outbox {
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
            total: 0,
        }
    }

    /// Queue a message for delivery
    pub fn enqueue(&mut self, msg: QueuedMessage) -> Result<(), String> {
        if self.total >= MAX_TOTAL_QUEUED {
            return Err(format!("Outbox full ({} messages)", MAX_TOTAL_QUEUED));
        }

        let queue = self.queues.entry(msg.recipient_id.clone()).or_default();

        if queue.len() >= MAX_QUEUE_PER_PEER {
            return Err(format!(
                "Queue full for peer {} ({} messages)",
                msg.recipient_id, MAX_QUEUE_PER_PEER
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

impl Default for Outbox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_enqueue_and_peek() {
        let mut outbox = Outbox::new();
        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg3", "peer_b")).unwrap();

        assert_eq!(outbox.total_count(), 3);
        assert_eq!(outbox.peer_count(), 2);
        assert_eq!(outbox.peek_for_peer("peer_a").len(), 2);
        assert_eq!(outbox.peek_for_peer("peer_b").len(), 1);
        assert_eq!(outbox.peek_for_peer("peer_c").len(), 0);
    }

    #[test]
    fn test_remove() {
        let mut outbox = Outbox::new();
        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();

        assert!(outbox.remove("msg1"));
        assert_eq!(outbox.total_count(), 1);
        assert!(!outbox.remove("msg1")); // Already removed
    }

    #[test]
    fn test_drain_for_peer() {
        let mut outbox = Outbox::new();
        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg3", "peer_b")).unwrap();

        let drained = outbox.drain_for_peer("peer_a");
        assert_eq!(drained.len(), 2);
        assert_eq!(outbox.total_count(), 1);
        assert_eq!(outbox.peek_for_peer("peer_a").len(), 0);
    }

    #[test]
    fn test_record_attempt() {
        let mut outbox = Outbox::new();
        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();

        outbox.record_attempt("msg1");
        outbox.record_attempt("msg1");

        let msgs = outbox.peek_for_peer("peer_a");
        assert_eq!(msgs[0].attempts, 2);
    }

    #[test]
    fn test_remove_expired() {
        let mut outbox = Outbox::new();

        let mut old_msg = make_msg("old", "peer_a");
        old_msg.queued_at = 0; // epoch = very old
        outbox.enqueue(old_msg).unwrap();

        let fresh_msg = make_msg("fresh", "peer_a");
        outbox.enqueue(fresh_msg).unwrap();

        let removed = outbox.remove_expired(3600); // 1 hour max age
        assert_eq!(removed, 1);
        assert_eq!(outbox.total_count(), 1);
    }
}
