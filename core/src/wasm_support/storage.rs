// WASM In-Memory Message Storage
//
// Bounded message store for browser environments with configurable
// eviction strategies (LRU, Priority, Oldest-first).

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use thiserror::Error;

/// Eviction strategy for store overflow
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EvictionStrategy {
    /// Remove least recently used
    LRU,
    /// Remove lowest priority messages first
    Priority,
    /// Remove oldest (by timestamp) messages first
    OldestFirst,
}

/// Configuration for WASM message store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmStoreConfig {
    /// Maximum number of messages
    pub max_messages: usize,
    /// Maximum total bytes (default 50MB)
    pub max_total_bytes: usize,
    /// Strategy for eviction when full
    pub eviction_strategy: EvictionStrategy,
}

impl Default for WasmStoreConfig {
    fn default() -> Self {
        Self {
            max_messages: 1000,
            max_total_bytes: 50_000_000,
            eviction_strategy: EvictionStrategy::LRU,
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum StorageError {
    #[error("Store full")]
    StoreFull,
    #[error("Message not found")]
    NotFound,
}

/// Metadata for a stored message
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MessageEntry {
    message_id: [u8; 16],
    data: Vec<u8>,
    priority: u8,
    timestamp_ms: u64,
    access_count: u64,
}

/// In-memory message store for WASM environments
pub struct WasmStore {
    config: WasmStoreConfig,
    messages: Arc<RwLock<HashMap<[u8; 16], MessageEntry>>>,
    access_order: Arc<RwLock<VecDeque<[u8; 16]>>>,
}

impl WasmStore {
    /// Create a new store with the given configuration
    pub fn new(config: WasmStoreConfig) -> Self {
        Self {
            config,
            messages: Arc::new(RwLock::new(HashMap::new())),
            access_order: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Create store with default configuration
    pub fn default() -> Self {
        Self::new(WasmStoreConfig::default())
    }

    /// Insert a message into the store
    pub fn insert(
        &self,
        message_id: [u8; 16],
        data: Vec<u8>,
        priority: u8,
    ) -> Result<bool, StorageError> {
        let data_len = data.len();

        // Check if eviction is needed
        if self.should_evict(data_len) {
            self.evict_if_needed()?;
        }

        let entry = MessageEntry {
            message_id,
            data,
            priority,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            access_count: 0,
        };

        let mut messages = self.messages.write();
        let is_new = !messages.contains_key(&message_id);

        messages.insert(message_id, entry);

        let mut access_order = self.access_order.write();
        if is_new {
            access_order.push_back(message_id);
        }

        Ok(is_new)
    }

    /// Get a message by ID
    pub fn get(&self, message_id: &[u8; 16]) -> Option<Vec<u8>> {
        let mut messages = self.messages.write();
        if let Some(entry) = messages.get_mut(message_id) {
            entry.access_count += 1;
            let data = entry.data.clone();
            drop(messages);

            // Update access order for LRU
            let mut access_order = self.access_order.write();
            access_order.retain(|&id| id != *message_id);
            access_order.push_back(*message_id);

            return Some(data);
        }
        None
    }

    /// Check if message exists
    pub fn contains(&self, message_id: &[u8; 16]) -> bool {
        self.messages.read().contains_key(message_id)
    }

    /// Remove a message
    pub fn remove(&self, message_id: &[u8; 16]) -> bool {
        self.messages.write().remove(message_id).is_some()
    }

    /// Get number of messages
    pub fn len(&self) -> usize {
        self.messages.read().len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.messages.read().is_empty()
    }

    /// Get total bytes used
    pub fn total_bytes(&self) -> usize {
        self.messages
            .read()
            .values()
            .map(|e| e.data.len())
            .sum()
    }

    /// Get all message IDs
    pub fn all_message_ids(&self) -> Vec<[u8; 16]> {
        self.messages.read().keys().copied().collect()
    }

    /// Get messages matching a recipient hint
    /// (scanning bytes 4-7 of data for hint, which is after msg_id and priority)
    pub fn messages_for_hint(&self, hint: &[u8; 4]) -> Vec<Vec<u8>> {
        self.messages
            .read()
            .values()
            .filter(|e| {
                if e.data.len() >= 8 {
                    &e.data[4..8] == hint
                } else {
                    false
                }
            })
            .map(|e| e.data.clone())
            .collect()
    }

    /// Check if eviction is needed
    fn should_evict(&self, new_data_len: usize) -> bool {
        let messages = self.messages.read();
        let total = self.total_bytes();
        let count = messages.len();

        (count >= self.config.max_messages) || (total + new_data_len > self.config.max_total_bytes)
    }

    /// Evict messages according to strategy
    fn evict_if_needed(&self) -> Result<(), StorageError> {
        match self.config.eviction_strategy {
            EvictionStrategy::LRU => self.evict_lru(),
            EvictionStrategy::Priority => self.evict_priority(),
            EvictionStrategy::OldestFirst => self.evict_oldest(),
        }
    }

    /// Evict least recently used message
    fn evict_lru(&self) -> Result<(), StorageError> {
        let mut access_order = self.access_order.write();
        let mut messages = self.messages.write();

        if let Some(to_remove) = access_order.pop_front() {
            messages.remove(&to_remove);
            Ok(())
        } else {
            Err(StorageError::StoreFull)
        }
    }

    /// Evict lowest priority message
    fn evict_priority(&self) -> Result<(), StorageError> {
        let mut messages = self.messages.write();

        let min_priority_id = messages
            .iter()
            .min_by_key(|(_, e)| e.priority)
            .map(|(id, _)| *id);

        if let Some(id) = min_priority_id {
            messages.remove(&id);

            let mut access_order = self.access_order.write();
            access_order.retain(|&msg_id| msg_id != id);

            Ok(())
        } else {
            Err(StorageError::StoreFull)
        }
    }

    /// Evict oldest message
    fn evict_oldest(&self) -> Result<(), StorageError> {
        let mut messages = self.messages.write();

        let oldest_id = messages
            .iter()
            .min_by_key(|(_, e)| e.timestamp_ms)
            .map(|(id, _)| *id);

        if let Some(id) = oldest_id {
            messages.remove(&id);

            let mut access_order = self.access_order.write();
            access_order.retain(|&msg_id| msg_id != id);

            Ok(())
        } else {
            Err(StorageError::StoreFull)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_creation() {
        let store = WasmStore::default();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
        assert_eq!(store.total_bytes(), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let store = WasmStore::default();
        let msg_id = [1u8; 16];
        let data = b"hello world".to_vec();

        assert!(store.insert(msg_id, data.clone(), 100).unwrap());
        assert_eq!(store.len(), 1);
        assert_eq!(store.get(&msg_id), Some(data));
    }

    #[test]
    fn test_insert_duplicate() {
        let store = WasmStore::default();
        let msg_id = [2u8; 16];
        let data = b"test".to_vec();

        assert!(store.insert(msg_id, data.clone(), 100).unwrap());
        assert!(!store.insert(msg_id, data, 100).unwrap()); // Should return false
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_contains_and_remove() {
        let store = WasmStore::default();
        let msg_id = [3u8; 16];
        let data = b"test".to_vec();

        store.insert(msg_id, data, 100).unwrap();
        assert!(store.contains(&msg_id));

        assert!(store.remove(&msg_id));
        assert!(!store.contains(&msg_id));
    }

    #[test]
    fn test_all_message_ids() {
        let store = WasmStore::default();
        let ids = vec![[4u8; 16], [5u8; 16], [6u8; 16]];

        for (i, &id) in ids.iter().enumerate() {
            store.insert(id, vec![i as u8], 100).unwrap();
        }

        let all_ids = store.all_message_ids();
        assert_eq!(all_ids.len(), 3);
    }

    #[test]
    fn test_eviction_strategy_lru() {
        let config = WasmStoreConfig {
            max_messages: 3,
            max_total_bytes: 1000,
            eviction_strategy: EvictionStrategy::LRU,
        };
        let store = WasmStore::new(config);

        for i in 0..3 {
            let msg_id = [i as u8; 16];
            store.insert(msg_id, vec![i as u8; 10], 100).unwrap();
        }

        assert_eq!(store.len(), 3);

        // Add one more, should evict first (least recently used)
        let new_id = [99u8; 16];
        store.insert(new_id, vec![99u8; 10], 100).unwrap();
        assert_eq!(store.len(), 3);
    }

    #[test]
    fn test_eviction_strategy_priority() {
        let config = WasmStoreConfig {
            max_messages: 3,
            max_total_bytes: 1000,
            eviction_strategy: EvictionStrategy::Priority,
        };
        let store = WasmStore::new(config);

        store.insert([1u8; 16], vec![0; 10], 200).unwrap(); // High priority
        store.insert([2u8; 16], vec![0; 10], 50).unwrap(); // Low priority
        store.insert([3u8; 16], vec![0; 10], 150).unwrap(); // Medium priority

        assert_eq!(store.len(), 3);

        // Add new message, should evict lowest priority (50)
        store.insert([4u8; 16], vec![0; 10], 100).unwrap();
        assert_eq!(store.len(), 3);
        assert!(!store.contains(&[2u8; 16]));
    }

    #[test]
    fn test_eviction_strategy_oldest() {
        let config = WasmStoreConfig {
            max_messages: 2,
            max_total_bytes: 10000,
            eviction_strategy: EvictionStrategy::OldestFirst,
        };
        let store = WasmStore::new(config);

        let id1 = [10u8; 16];
        let id2 = [11u8; 16];

        store.insert(id1, vec![0; 10], 100).unwrap();
        store.insert(id2, vec![0; 10], 100).unwrap();

        // Adding one more should evict the oldest
        store.insert([12u8; 16], vec![0; 10], 100).unwrap();
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_messages_for_hint() {
        let store = WasmStore::default();

        // messages_for_hint checks bytes [4..8] of message data
        let mut msg1 = vec![0; 8];
        msg1[4..8].copy_from_slice(&[0x12, 0x34, 0x56, 0x78]);

        let mut msg2 = vec![0; 8];
        msg2[4..8].copy_from_slice(&[0x12, 0x34, 0x56, 0x78]);

        let mut msg3 = vec![0; 8];
        msg3[4..8].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]);

        store.insert([1u8; 16], msg1, 100).unwrap();
        store.insert([2u8; 16], msg2, 100).unwrap();
        store.insert([3u8; 16], msg3, 100).unwrap();

        let hint = [0x12u8, 0x34, 0x56, 0x78];
        let matches = store.messages_for_hint(&hint);
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_total_bytes() {
        let store = WasmStore::default();

        store.insert([1u8; 16], vec![0; 100], 100).unwrap();
        store.insert([2u8; 16], vec![0; 200], 100).unwrap();
        store.insert([3u8; 16], vec![0; 50], 100).unwrap();

        assert_eq!(store.total_bytes(), 350);
    }

    #[test]
    fn test_custom_config() {
        let config = WasmStoreConfig {
            max_messages: 500,
            max_total_bytes: 25_000_000,
            eviction_strategy: EvictionStrategy::Priority,
        };
        let store = WasmStore::new(config);

        assert!(store.is_empty());
    }

    #[test]
    fn test_byte_limit_eviction() {
        let config = WasmStoreConfig {
            max_messages: 100,
            max_total_bytes: 200,
            eviction_strategy: EvictionStrategy::OldestFirst,
        };
        let store = WasmStore::new(config);

        store.insert([1u8; 16], vec![0; 100], 100).unwrap();
        store.insert([2u8; 16], vec![0; 100], 100).unwrap();

        // Adding 50 more should trigger eviction of oldest
        store.insert([3u8; 16], vec![0; 50], 100).unwrap();

        // Should have 2 messages, not 3
        assert!(store.len() <= 2);
    }

    #[test]
    fn test_access_count_in_lru() {
        let config = WasmStoreConfig {
            max_messages: 2,
            max_total_bytes: 1000,
            eviction_strategy: EvictionStrategy::LRU,
        };
        let store = WasmStore::new(config);

        let id1 = [20u8; 16];
        let id2 = [21u8; 16];

        store.insert(id1, vec![0; 10], 100).unwrap();
        store.insert(id2, vec![0; 10], 100).unwrap();

        // Access id1 to make it recently used
        let _ = store.get(&id1);

        // Add new message, should evict id2 (not recently accessed)
        store.insert([22u8; 16], vec![0; 10], 100).unwrap();

        assert!(store.contains(&id1));
        assert!(!store.contains(&id2));
    }

    #[test]
    fn test_empty_after_remove_all() {
        let store = WasmStore::default();

        let ids = [[30u8; 16], [31u8; 16], [32u8; 16]];
        for id in ids {
            store.insert(id, vec![0; 10], 100).unwrap();
        }

        for id in ids {
            store.remove(&id);
        }

        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }
}
