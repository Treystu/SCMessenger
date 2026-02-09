// Outbox — queue messages for peers that may be offline
//
// Messages are stored locally and retried when the peer comes online.
// This is the foundation for store-and-forward delivery.

use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Maximum messages queued per peer
const MAX_QUEUE_PER_PEER: usize = 1000;

/// Maximum total messages across all peers
const MAX_TOTAL_QUEUED: usize = 10_000;

const QUEUE_PREFIX: &[u8] = b"outbox_";

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

/// Storage backend for outbox
enum OutboxBackend {
    Memory {
        queues: HashMap<String, VecDeque<QueuedMessage>>,
        total: usize,
    },
    Persistent(sled::Db),
}

/// Outbound message queue
pub struct Outbox {
    backend: OutboxBackend,
}

impl Outbox {
    /// Create a new in-memory outbox
    pub fn new() -> Self {
        Self {
            backend: OutboxBackend::Memory {
                queues: HashMap::new(),
                total: 0,
            },
        }
    }

    /// Create a persistent outbox with sled backend
    pub fn persistent(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self {
            backend: OutboxBackend::Persistent(db),
        })
    }

    /// Queue a message for delivery
    pub fn enqueue(&mut self, msg: QueuedMessage) -> std::result::Result<(), String> {
        match &mut self.backend {
            OutboxBackend::Memory { queues, total } => {
                if *total >= MAX_TOTAL_QUEUED {
                    return Err(format!("Outbox full ({} messages)", MAX_TOTAL_QUEUED));
                }

                let queue = queues.entry(msg.recipient_id.clone()).or_default();

                if queue.len() >= MAX_QUEUE_PER_PEER {
                    return Err(format!(
                        "Queue full for peer {} ({} messages)",
                        msg.recipient_id, MAX_QUEUE_PER_PEER
                    ));
                }

                queue.push_back(msg);
                *total += 1;
                Ok(())
            }
            OutboxBackend::Persistent(db) => {
                // Check total limit
                let current_total = db.scan_prefix(QUEUE_PREFIX).count();
                if current_total >= MAX_TOTAL_QUEUED {
                    return Err(format!("Outbox full ({} messages)", MAX_TOTAL_QUEUED));
                }

                // Check per-peer limit
                let peer_prefix = format!("{}{}_", String::from_utf8_lossy(QUEUE_PREFIX), msg.recipient_id);
                let peer_count = db.scan_prefix(peer_prefix.as_bytes()).count();
                if peer_count >= MAX_QUEUE_PER_PEER {
                    return Err(format!(
                        "Queue full for peer {} ({} messages)",
                        msg.recipient_id, MAX_QUEUE_PER_PEER
                    ));
                }

                // Store message
                let key = format!("{}{}_{}",
                    String::from_utf8_lossy(QUEUE_PREFIX),
                    msg.recipient_id,
                    msg.message_id
                );
                if let Ok(bytes) = bincode::serialize(&msg) {
                    db.insert(key.as_bytes(), bytes).map_err(|e| e.to_string())?;
                    db.flush().map_err(|e| e.to_string())?;
                }
                Ok(())
            }
        }
    }

    /// Get all queued messages for a peer (without removing them)
    pub fn peek_for_peer(&self, recipient_id: &str) -> Vec<QueuedMessage> {
        match &self.backend {
            OutboxBackend::Memory { queues, .. } => queues
                .get(recipient_id)
                .map(|q| q.iter().cloned().collect())
                .unwrap_or_default(),
            OutboxBackend::Persistent(db) => {
                let prefix = format!("{}{}_", String::from_utf8_lossy(QUEUE_PREFIX), recipient_id);
                db.scan_prefix(prefix.as_bytes())
                    .filter_map(|result| result.ok())
                    .filter_map(|(_, value)| bincode::deserialize(&value).ok())
                    .collect()
            }
        }
    }

    /// Remove a specific message by ID (after successful delivery)
    pub fn remove(&mut self, message_id: &str) -> bool {
        match &mut self.backend {
            OutboxBackend::Memory { queues, total } => {
                for queue in queues.values_mut() {
                    if let Some(pos) = queue.iter().position(|m| m.message_id == message_id) {
                        queue.remove(pos);
                        *total -= 1;
                        return true;
                    }
                }
                false
            }
            OutboxBackend::Persistent(db) => {
                // Find and remove the message
                for result in db.scan_prefix(QUEUE_PREFIX) {
                    if let Ok((key, value)) = result {
                        if let Ok(msg) = bincode::deserialize::<QueuedMessage>(&value) {
                            if msg.message_id == message_id {
                                let _ = db.remove(key);
                                let _ = db.flush();
                                return true;
                            }
                        }
                    }
                }
                false
            }
        }
    }

    /// Drain all messages for a peer (for batch delivery)
    pub fn drain_for_peer(&mut self, recipient_id: &str) -> Vec<QueuedMessage> {
        match &mut self.backend {
            OutboxBackend::Memory { queues, total } => {
                if let Some(queue) = queues.remove(recipient_id) {
                    let count = queue.len();
                    *total -= count;
                    queue.into()
                } else {
                    Vec::new()
                }
            }
            OutboxBackend::Persistent(db) => {
                let prefix = format!("{}{}_", String::from_utf8_lossy(QUEUE_PREFIX), recipient_id);
                let mut messages = Vec::new();
                let mut keys_to_remove = Vec::new();

                for result in db.scan_prefix(prefix.as_bytes()) {
                    if let Ok((key, value)) = result {
                        if let Ok(msg) = bincode::deserialize::<QueuedMessage>(&value) {
                            messages.push(msg);
                            keys_to_remove.push(key);
                        }
                    }
                }

                for key in keys_to_remove {
                    let _ = db.remove(key);
                }
                let _ = db.flush();

                messages
            }
        }
    }

    /// Increment attempt count for a message
    pub fn record_attempt(&mut self, message_id: &str) {
        match &mut self.backend {
            OutboxBackend::Memory { queues, .. } => {
                for queue in queues.values_mut() {
                    if let Some(msg) = queue.iter_mut().find(|m| m.message_id == message_id) {
                        msg.attempts += 1;
                        return;
                    }
                }
            }
            OutboxBackend::Persistent(db) => {
                // Find, update, and save the message
                for result in db.scan_prefix(QUEUE_PREFIX) {
                    if let Ok((key, value)) = result {
                        if let Ok(mut msg) = bincode::deserialize::<QueuedMessage>(&value) {
                            if msg.message_id == message_id {
                                msg.attempts += 1;
                                if let Ok(bytes) = bincode::serialize(&msg) {
                                    let _ = db.insert(key, bytes);
                                    let _ = db.flush();
                                }
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Total queued messages
    pub fn total_count(&self) -> usize {
        match &self.backend {
            OutboxBackend::Memory { total, .. } => *total,
            OutboxBackend::Persistent(db) => db.scan_prefix(QUEUE_PREFIX).count(),
        }
    }

    /// Number of peers with queued messages
    pub fn peer_count(&self) -> usize {
        match &self.backend {
            OutboxBackend::Memory { queues, .. } => queues.len(),
            OutboxBackend::Persistent(db) => {
                use std::collections::HashSet;
                let mut peers: HashSet<String> = HashSet::new();
                for result in db.scan_prefix(QUEUE_PREFIX) {
                    if let Ok((_, value)) = result {
                        if let Ok(msg) = bincode::deserialize::<QueuedMessage>(&value) {
                            peers.insert(msg.recipient_id);
                        }
                    }
                }
                peers.len()
            }
        }
    }

    /// Remove expired messages (older than max_age_secs)
    pub fn remove_expired(&mut self, max_age_secs: u64) -> usize {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        match &mut self.backend {
            OutboxBackend::Memory { queues, total } => {
                let mut removed = 0;

                for queue in queues.values_mut() {
                    let before = queue.len();
                    queue.retain(|msg| now.saturating_sub(msg.queued_at) < max_age_secs);
                    removed += before - queue.len();
                }

                *total -= removed;

                // Clean up empty queues
                queues.retain(|_, q| !q.is_empty());

                removed
            }
            OutboxBackend::Persistent(db) => {
                let mut keys_to_remove = Vec::new();

                for result in db.scan_prefix(QUEUE_PREFIX) {
                    if let Ok((key, value)) = result {
                        if let Ok(msg) = bincode::deserialize::<QueuedMessage>(&value) {
                            if now.saturating_sub(msg.queued_at) >= max_age_secs {
                                keys_to_remove.push(key);
                            }
                        }
                    }
                }

                let removed = keys_to_remove.len();
                for key in keys_to_remove {
                    let _ = db.remove(key);
                }
                let _ = db.flush();

                removed
            }
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

    #[test]
    fn test_persistent_outbox() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("outbox_store").to_str().unwrap().to_string();

        let mut outbox = Outbox::persistent(&path).unwrap();

        // Enqueue messages
        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg3", "peer_b")).unwrap();

        assert_eq!(outbox.total_count(), 3);
        assert_eq!(outbox.peer_count(), 2);

        // Peek messages
        let peer_a_msgs = outbox.peek_for_peer("peer_a");
        assert_eq!(peer_a_msgs.len(), 2);

        // Remove a message
        assert!(outbox.remove("msg1"));
        assert_eq!(outbox.total_count(), 2);
    }

    #[test]
    fn test_persistent_outbox_survives_restart() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("outbox_store").to_str().unwrap().to_string();

        // First instance: enqueue messages
        {
            let mut outbox = Outbox::persistent(&path).unwrap();
            outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
            outbox.enqueue(make_msg("msg2", "peer_b")).unwrap();
        }

        // Second instance: messages should still be there
        {
            let outbox = Outbox::persistent(&path).unwrap();
            assert_eq!(outbox.total_count(), 2);
            assert_eq!(outbox.peer_count(), 2);

            let peer_a_msgs = outbox.peek_for_peer("peer_a");
            assert_eq!(peer_a_msgs.len(), 1);
            assert_eq!(peer_a_msgs[0].message_id, "msg1");
        }
    }

    #[test]
    fn test_persistent_outbox_drain() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("outbox_store").to_str().unwrap().to_string();

        let mut outbox = Outbox::persistent(&path).unwrap();

        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg3", "peer_b")).unwrap();

        let drained = outbox.drain_for_peer("peer_a");
        assert_eq!(drained.len(), 2);
        assert_eq!(outbox.total_count(), 1);
        assert_eq!(outbox.peek_for_peer("peer_a").len(), 0);
    }
}
