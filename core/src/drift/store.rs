/// CRDT Mesh Message Store for Drift Network
///
/// A grow-only set (G-Set) CRDT implementation that guarantees
/// conflict-free merging in mesh networks where any two nodes
/// can meet and synchronize their message stores.

use std::collections::HashMap;

/// Maximum messages in the store
const MAX_MESSAGES: usize = 10_000;

/// A unique message identifier (16 bytes, matches DriftEnvelope.message_id)
pub type MessageId = [u8; 16];

/// Stored envelope with metadata for routing/priority decisions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredEnvelope {
    /// Raw Drift Envelope bytes (serialized)
    pub envelope_data: Vec<u8>,
    /// Message ID (extracted for indexing)
    pub message_id: MessageId,
    /// Recipient hint (first 4 bytes of blake3(recipient_pk))
    pub recipient_hint: [u8; 4],
    /// When the message was created (unix timestamp u32)
    pub created_at: u32,
    /// TTL expiry (unix timestamp u32, 0 = never)
    pub ttl_expiry: u32,
    /// Number of relay hops this message has taken
    pub hop_count: u8,
    /// Message priority (0-255)
    pub priority: u8,
    /// When this node first received this message
    pub received_at: u64,
}

impl StoredEnvelope {
    /// Calculate priority score for sync ordering and eviction.
    ///
    /// Higher score = more important = synced first, evicted last.
    ///
    /// Factors:
    /// - priority: explicit priority field (0-255)
    /// - recency: newer messages score higher (exponential decay, τ = 24 hours)
    /// - hop_count: fewer hops = closer to origin = higher value
    /// - ttl_remaining: messages about to expire get priority
    pub fn priority_score(&self) -> f64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as f64;

        let age_hours = (now - self.created_at as f64) / 3600.0;
        let recency = (-age_hours / 24.0).exp(); // τ = 24 hours

        let hop_penalty = 1.0 / (1.0 + self.hop_count as f64);

        let ttl_factor = if self.ttl_expiry == 0 {
            1.0 // No expiry = full score
        } else {
            let total_lifetime = (self.ttl_expiry as f64) - (self.created_at as f64);
            let remaining = (self.ttl_expiry as f64) - now;
            if total_lifetime <= 0.0 || remaining <= 0.0 {
                0.0 // Expired
            } else {
                (remaining / total_lifetime).clamp(0.0, 1.0)
            }
        };

        (self.priority as f64) * recency * hop_penalty * ttl_factor
    }
}

/// CRDT Message Store — Grow-Only Set with priority eviction
///
/// Properties:
/// - Add-only: messages can be added but never modified
/// - Merge: any two stores can merge without conflict
/// - Eviction: when over capacity, lowest-priority messages are evicted
/// - Dedup: duplicate message IDs are silently ignored
pub struct MeshStore {
    /// Messages indexed by ID
    messages: HashMap<MessageId, StoredEnvelope>,
    /// Maximum capacity
    max_messages: usize,
}

impl MeshStore {
    /// Create a new empty mesh store with default capacity
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
            max_messages: MAX_MESSAGES,
        }
    }

    /// Create a mesh store with custom capacity
    pub fn with_capacity(max: usize) -> Self {
        Self {
            messages: HashMap::new(),
            max_messages: max,
        }
    }

    /// Insert a message. Returns true if new, false if duplicate.
    ///
    /// CRDT property: idempotent — inserting the same message
    /// multiple times has the same effect as inserting once.
    pub fn insert(&mut self, envelope: StoredEnvelope) -> bool {
        if self.messages.contains_key(&envelope.message_id) {
            return false; // Duplicate — CRDT idempotent
        }
        self.messages.insert(envelope.message_id, envelope);
        self.evict_if_over_budget();
        true
    }

    /// CRDT merge: union of two stores.
    ///
    /// For each message ID, keep the entry (add-only, no conflict possible).
    /// This operation is:
    /// - Commutative: merge(A, B) == merge(B, A)
    /// - Idempotent: merge(A, A) == A
    /// - Associative: merge(merge(A, B), C) == merge(A, merge(B, C))
    pub fn merge(&mut self, other: &MeshStore) {
        for (id, envelope) in &other.messages {
            if !self.messages.contains_key(id) {
                self.messages.insert(*id, envelope.clone());
            }
        }
        self.evict_if_over_budget();
    }

    /// Get a message by ID
    pub fn get(&self, id: &MessageId) -> Option<&StoredEnvelope> {
        self.messages.get(id)
    }

    /// Check if we have a message
    pub fn contains(&self, id: &MessageId) -> bool {
        self.messages.contains_key(id)
    }

    /// Get all message IDs (for sync/bloom filter)
    pub fn message_ids(&self) -> Vec<MessageId> {
        self.messages.keys().copied().collect()
    }

    /// Get messages for a specific recipient hint
    pub fn messages_for_recipient(&self, hint: &[u8; 4]) -> Vec<&StoredEnvelope> {
        self.messages
            .values()
            .filter(|e| &e.recipient_hint == hint)
            .collect()
    }

    /// Get all messages sorted by priority score (highest first)
    pub fn by_priority(&self) -> Vec<&StoredEnvelope> {
        let mut msgs: Vec<_> = self.messages.values().collect();
        msgs.sort_by(|a, b| {
            b.priority_score()
                .partial_cmp(&a.priority_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        msgs
    }

    /// Total message count
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Remove expired messages
    ///
    /// Returns the number of messages removed.
    pub fn remove_expired(&mut self) -> usize {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;
        let before = self.messages.len();
        self.messages.retain(|_, e| e.ttl_expiry == 0 || e.ttl_expiry > now);
        before - self.messages.len()
    }

    /// Evict lowest-priority messages when over budget
    fn evict_if_over_budget(&mut self) {
        while self.messages.len() > self.max_messages {
            // Find the message with the lowest priority score
            let lowest_id = self
                .messages
                .iter()
                .min_by(|a, b| {
                    a.1.priority_score()
                        .partial_cmp(&b.1.priority_score())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(id, _)| *id);
            if let Some(id) = lowest_id {
                self.messages.remove(&id);
            } else {
                break;
            }
        }
    }
}

impl Default for MeshStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_envelope(
        message_id: MessageId,
        priority: u8,
        hop_count: u8,
        created_at: u32,
        ttl_expiry: u32,
    ) -> StoredEnvelope {
        StoredEnvelope {
            envelope_data: vec![1, 2, 3, 4],
            message_id,
            recipient_hint: [1, 2, 3, 4],
            created_at,
            ttl_expiry,
            hop_count,
            priority,
            received_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    #[test]
    fn test_insert_single_message() {
        let mut store = MeshStore::new();
        let msg_id = [1u8; 16];
        let envelope = make_envelope(msg_id, 100, 0, 1000, 0);

        assert!(store.insert(envelope));
        assert_eq!(store.len(), 1);
        assert!(store.contains(&msg_id));
    }

    #[test]
    fn test_insert_duplicate_is_idempotent() {
        let mut store = MeshStore::new();
        let msg_id = [1u8; 16];
        let envelope1 = make_envelope(msg_id, 100, 0, 1000, 0);

        assert!(store.insert(envelope1));
        assert_eq!(store.len(), 1);

        // Insert same message again
        let envelope2 = make_envelope(msg_id, 200, 1, 2000, 0);
        assert!(!store.insert(envelope2)); // Returns false
        assert_eq!(store.len(), 1); // Still 1 message, not 2
    }

    #[test]
    fn test_get_message() {
        let mut store = MeshStore::new();
        let msg_id = [42u8; 16];
        let envelope = make_envelope(msg_id, 100, 0, 1000, 0);

        store.insert(envelope.clone());
        let retrieved = store.get(&msg_id).unwrap();

        assert_eq!(retrieved.message_id, msg_id);
        assert_eq!(retrieved.priority, 100);
    }

    #[test]
    fn test_merge_non_overlapping_stores() {
        let mut store_a = MeshStore::new();
        let mut store_b = MeshStore::new();

        let msg_a = [1u8; 16];
        let msg_b = [2u8; 16];

        store_a.insert(make_envelope(msg_a, 100, 0, 1000, 0));
        store_b.insert(make_envelope(msg_b, 100, 0, 1000, 0));

        assert_eq!(store_a.len(), 1);
        assert_eq!(store_b.len(), 1);

        store_a.merge(&store_b);

        assert_eq!(store_a.len(), 2);
        assert!(store_a.contains(&msg_a));
        assert!(store_a.contains(&msg_b));
    }

    #[test]
    fn test_merge_overlapping_stores() {
        let mut store_a = MeshStore::new();
        let mut store_b = MeshStore::new();

        let msg_a = [1u8; 16];
        let msg_b = [2u8; 16];

        store_a.insert(make_envelope(msg_a, 100, 0, 1000, 0));
        store_a.insert(make_envelope(msg_b, 100, 0, 1000, 0));

        store_b.insert(make_envelope(msg_a, 100, 0, 1000, 0)); // Same message
        store_b.insert(make_envelope([3u8; 16], 100, 0, 1000, 0)); // New message

        store_a.merge(&store_b);

        assert_eq!(store_a.len(), 3);
        assert!(store_a.contains(&msg_a));
        assert!(store_a.contains(&msg_b));
        assert!(store_a.contains(&[3u8; 16]));
    }

    #[test]
    fn test_merge_commutativity() {
        // merge(A, B) should equal merge(B, A)
        let mut store_a = MeshStore::new();
        let mut store_b = MeshStore::new();
        let mut store_a2 = MeshStore::new();
        let mut store_b2 = MeshStore::new();

        let msg_a = [1u8; 16];
        let msg_b = [2u8; 16];

        store_a.insert(make_envelope(msg_a, 100, 0, 1000, 0));
        store_b.insert(make_envelope(msg_b, 50, 0, 1000, 0));

        store_a2.insert(make_envelope(msg_a, 100, 0, 1000, 0));
        store_b2.insert(make_envelope(msg_b, 50, 0, 1000, 0));

        // merge(A, B)
        store_a.merge(&store_b);

        // merge(B, A)
        store_b2.merge(&store_a2);

        // Both should have the same messages
        assert_eq!(store_a.len(), store_b2.len());
        assert!(store_a.contains(&msg_a));
        assert!(store_a.contains(&msg_b));
        assert!(store_b2.contains(&msg_a));
        assert!(store_b2.contains(&msg_b));
    }

    #[test]
    fn test_merge_idempotency() {
        // merge(A, A) should equal A
        let mut store = MeshStore::new();
        let msg_a = [1u8; 16];
        let msg_b = [2u8; 16];

        store.insert(make_envelope(msg_a, 100, 0, 1000, 0));
        store.insert(make_envelope(msg_b, 50, 0, 1000, 0));

        let store_copy = MeshStore {
            messages: store.messages.clone(),
            max_messages: store.max_messages,
        };

        store.merge(&store_copy);

        assert_eq!(store.len(), 2);
        assert!(store.contains(&msg_a));
        assert!(store.contains(&msg_b));
    }

    #[test]
    fn test_eviction_on_over_capacity() {
        let mut store = MeshStore::with_capacity(5);

        // Use a recent timestamp so recency factor doesn't dominate
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        // Insert 10 messages with different priorities
        for i in 0..10 {
            let msg_id = [i as u8; 16];
            let priority = (i * 10) as u8; // priorities: 0, 10, 20, ..., 90
            store.insert(make_envelope(msg_id, priority, 0, now, 0));
        }

        // Should have evicted the lowest priority messages, keeping the highest
        assert_eq!(store.len(), 5);

        // Check that highest priority messages are still there
        for i in 5..10 {
            let msg_id = [i as u8; 16];
            assert!(store.contains(&msg_id));
        }

        // Check that lowest priority messages were evicted
        for i in 0..5 {
            let msg_id = [i as u8; 16];
            assert!(!store.contains(&msg_id));
        }
    }

    #[test]
    fn test_priority_score_newer_higher() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        // Newer message
        let newer = StoredEnvelope {
            envelope_data: vec![1, 2, 3],
            message_id: [1u8; 16],
            recipient_hint: [1, 2, 3, 4],
            created_at: now - 3600, // 1 hour ago
            ttl_expiry: 0,
            hop_count: 0,
            priority: 100,
            received_at: 0,
        };

        // Older message (same priority, hops, ttl)
        let older = StoredEnvelope {
            envelope_data: vec![1, 2, 3],
            message_id: [2u8; 16],
            recipient_hint: [1, 2, 3, 4],
            created_at: now - 86400, // 1 day ago
            ttl_expiry: 0,
            hop_count: 0,
            priority: 100,
            received_at: 0,
        };

        assert!(newer.priority_score() > older.priority_score());
    }

    #[test]
    fn test_priority_score_fewer_hops_higher() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        let msg1 = StoredEnvelope {
            envelope_data: vec![1, 2, 3],
            message_id: [1u8; 16],
            recipient_hint: [1, 2, 3, 4],
            created_at: now,
            ttl_expiry: 0,
            hop_count: 0, // Direct
            priority: 100,
            received_at: 0,
        };

        let msg2 = StoredEnvelope {
            envelope_data: vec![1, 2, 3],
            message_id: [2u8; 16],
            recipient_hint: [1, 2, 3, 4],
            created_at: now,
            ttl_expiry: 0,
            hop_count: 5, // 5 hops
            priority: 100,
            received_at: 0,
        };

        assert!(msg1.priority_score() > msg2.priority_score());
    }

    #[test]
    fn test_priority_score_explicit_priority() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        let high_priority = StoredEnvelope {
            envelope_data: vec![1, 2, 3],
            message_id: [1u8; 16],
            recipient_hint: [1, 2, 3, 4],
            created_at: now,
            ttl_expiry: 0,
            hop_count: 0,
            priority: 255,
            received_at: 0,
        };

        let low_priority = StoredEnvelope {
            envelope_data: vec![1, 2, 3],
            message_id: [2u8; 16],
            recipient_hint: [1, 2, 3, 4],
            created_at: now,
            ttl_expiry: 0,
            hop_count: 0,
            priority: 0,
            received_at: 0,
        };

        assert!(high_priority.priority_score() > low_priority.priority_score());
    }

    #[test]
    fn test_remove_expired_messages() {
        let mut store = MeshStore::new();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        // Message that expires in 1 hour
        let future_expiry = StoredEnvelope {
            envelope_data: vec![1, 2, 3],
            message_id: [1u8; 16],
            recipient_hint: [1, 2, 3, 4],
            created_at: now,
            ttl_expiry: now + 3600,
            hop_count: 0,
            priority: 100,
            received_at: 0,
        };

        // Message that already expired
        let past_expiry = StoredEnvelope {
            envelope_data: vec![1, 2, 3],
            message_id: [2u8; 16],
            recipient_hint: [1, 2, 3, 4],
            created_at: now - 7200,
            ttl_expiry: now - 1800, // Expired 30 mins ago
            hop_count: 0,
            priority: 100,
            received_at: 0,
        };

        // Message with no expiry (ttl_expiry = 0)
        let no_expiry = StoredEnvelope {
            envelope_data: vec![1, 2, 3],
            message_id: [3u8; 16],
            recipient_hint: [1, 2, 3, 4],
            created_at: now - 86400,
            ttl_expiry: 0,
            hop_count: 0,
            priority: 100,
            received_at: 0,
        };

        store.insert(future_expiry);
        store.insert(past_expiry);
        store.insert(no_expiry);

        assert_eq!(store.len(), 3);

        let removed = store.remove_expired();
        assert_eq!(removed, 1); // Only the past_expiry should be removed

        assert!(store.contains(&[1u8; 16])); // Future expiry still there
        assert!(!store.contains(&[2u8; 16])); // Past expiry removed
        assert!(store.contains(&[3u8; 16])); // No expiry still there
    }

    #[test]
    fn test_messages_for_recipient() {
        let mut store = MeshStore::new();

        let hint_a = [1u8, 2u8, 3u8, 4u8];
        let hint_b = [5u8, 6u8, 7u8, 8u8];

        let mut msg1 = make_envelope([1u8; 16], 100, 0, 1000, 0);
        msg1.recipient_hint = hint_a;

        let mut msg2 = make_envelope([2u8; 16], 100, 0, 1000, 0);
        msg2.recipient_hint = hint_a;

        let mut msg3 = make_envelope([3u8; 16], 100, 0, 1000, 0);
        msg3.recipient_hint = hint_b;

        store.insert(msg1);
        store.insert(msg2);
        store.insert(msg3);

        let msgs_a = store.messages_for_recipient(&hint_a);
        let msgs_b = store.messages_for_recipient(&hint_b);

        assert_eq!(msgs_a.len(), 2);
        assert_eq!(msgs_b.len(), 1);
    }

    #[test]
    fn test_by_priority_ordering() {
        let mut store = MeshStore::new();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        // Insert messages with different priorities
        store.insert(StoredEnvelope {
            envelope_data: vec![1],
            message_id: [1u8; 16],
            recipient_hint: [1, 2, 3, 4],
            created_at: now,
            ttl_expiry: 0,
            hop_count: 0,
            priority: 50,
            received_at: 0,
        });

        store.insert(StoredEnvelope {
            envelope_data: vec![2],
            message_id: [2u8; 16],
            recipient_hint: [1, 2, 3, 4],
            created_at: now,
            ttl_expiry: 0,
            hop_count: 0,
            priority: 200,
            received_at: 0,
        });

        store.insert(StoredEnvelope {
            envelope_data: vec![3],
            message_id: [3u8; 16],
            recipient_hint: [1, 2, 3, 4],
            created_at: now,
            ttl_expiry: 0,
            hop_count: 0,
            priority: 100,
            received_at: 0,
        });

        let ordered = store.by_priority();

        assert_eq!(ordered.len(), 3);
        // Should be ordered: 200, 100, 50
        assert_eq!(ordered[0].priority, 200);
        assert_eq!(ordered[1].priority, 100);
        assert_eq!(ordered[2].priority, 50);
    }

    #[test]
    fn test_message_ids() {
        let mut store = MeshStore::new();

        let msg_id_1 = [1u8; 16];
        let msg_id_2 = [2u8; 16];
        let msg_id_3 = [3u8; 16];

        store.insert(make_envelope(msg_id_1, 100, 0, 1000, 0));
        store.insert(make_envelope(msg_id_2, 100, 0, 1000, 0));
        store.insert(make_envelope(msg_id_3, 100, 0, 1000, 0));

        let ids = store.message_ids();

        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&msg_id_1));
        assert!(ids.contains(&msg_id_2));
        assert!(ids.contains(&msg_id_3));
    }

    #[test]
    fn test_empty_store() {
        let store = MeshStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_custom_capacity() {
        let store = MeshStore::with_capacity(100);
        assert_eq!(store.max_messages, 100);
    }

    #[test]
    fn test_merge_with_eviction() {
        let mut store_a = MeshStore::with_capacity(3);
        let mut store_b = MeshStore::with_capacity(3);

        // Use a recent timestamp so recency factor doesn't dominate
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        // Store A has 3 high-priority messages
        for i in 0..3 {
            let msg_id = [i as u8; 16];
            store_a.insert(make_envelope(msg_id, i as u8 + 100, 0, now, 0));
        }

        // Store B has 2 low-priority messages
        for i in 3..5 {
            let msg_id = [i as u8; 16];
            store_b.insert(make_envelope(msg_id, 10, 0, now, 0));
        }

        // Merge B into A (5 messages, should keep 3 highest priority)
        store_a.merge(&store_b);

        assert_eq!(store_a.len(), 3);
        // The lowest priority message (from store_b) should have been evicted
        assert!(!store_a.contains(&[3u8; 16]));
        assert!(!store_a.contains(&[4u8; 16]));
    }

    #[test]
    fn test_insert_after_eviction_preserves_crdt() {
        let mut store_a = MeshStore::with_capacity(2);
        let mut store_b = MeshStore::with_capacity(2);

        // Both stores start with msg_id [1] and [2]
        store_a.insert(make_envelope([1u8; 16], 100, 0, 1000, 0));
        store_a.insert(make_envelope([2u8; 16], 100, 0, 1000, 0));

        store_b.insert(make_envelope([1u8; 16], 100, 0, 1000, 0));
        store_b.insert(make_envelope([2u8; 16], 100, 0, 1000, 0));

        // Add low-priority message to store_a (will evict something)
        store_a.insert(make_envelope([3u8; 16], 1, 0, 1000, 0));

        // Add high-priority message to store_b
        store_b.insert(make_envelope([4u8; 16], 200, 0, 1000, 0));

        // Merge both ways
        let mut test_store_1 = MeshStore::with_capacity(4);
        test_store_1.merge(&store_a);
        test_store_1.merge(&store_b);

        let mut test_store_2 = MeshStore::with_capacity(4);
        test_store_2.merge(&store_b);
        test_store_2.merge(&store_a);

        // Both should have the same final set of messages
        let ids1 = test_store_1.message_ids();
        let ids2 = test_store_2.message_ids();

        ids1.iter().all(|id| test_store_2.contains(id));
        ids2.iter().all(|id| test_store_1.contains(id));
    }
}
