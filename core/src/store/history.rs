// Message history persistence and retrieval
//
// Refactored to use generic StorageBackend for cross-platform parity (Sled/IndexedDB/Memory).

use crate::store::backend::StorageBackend;
use crate::IronCoreError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageDirection {
    Sent,
    Received,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRecord {
    pub id: String,
    pub direction: MessageDirection,
    pub peer_id: String,
    pub content: String,
    pub timestamp: u64,
    pub delivered: bool,
}

impl MessageRecord {
    pub fn new_sent(peer_id: String, content: String) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            direction: MessageDirection::Sent,
            peer_id,
            content,
            timestamp: current_timestamp(),
            delivered: false,
        }
    }

    pub fn new_received(peer_id: String, content: String) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            direction: MessageDirection::Received,
            peer_id,
            content,
            timestamp: current_timestamp(),
            delivered: true,
        }
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Debug, Clone, Default)]
pub struct HistoryStats {
    pub total_messages: u32,
    pub sent_count: u32,
    pub received_count: u32,
    pub undelivered_count: u32,
}

#[derive(Clone)]
pub struct HistoryManager {
    backend: Arc<dyn StorageBackend>,
}

impl HistoryManager {
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        Self { backend }
    }

    pub fn add(&self, record: MessageRecord) -> Result<(), IronCoreError> {
        let key = format!("msg_{}", record.id);
        let value = serde_json::to_vec(&record).map_err(|_| IronCoreError::Internal)?;
        self.backend
            .put(key.as_bytes(), &value)
            .map_err(|_| IronCoreError::StorageError)?;
        Ok(())
    }

    pub fn get(&self, id: String) -> Result<Option<MessageRecord>, IronCoreError> {
        let key = format!("msg_{}", id);
        if let Some(data) = self
            .backend
            .get(key.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?
        {
            let record: MessageRecord =
                serde_json::from_slice(&data).map_err(|_| IronCoreError::Internal)?;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    pub fn recent(
        &self,
        peer_filter: Option<String>,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, IronCoreError> {
        let mut records = Vec::new();
        let all = self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| IronCoreError::StorageError)?;

        for (_, value) in all {
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;

            if let Some(ref peer) = peer_filter {
                if &record.peer_id == peer {
                    records.push(record);
                }
            } else {
                records.push(record);
            }
        }

        // Sort by timestamp descending
        records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if records.len() > limit as usize {
            records.truncate(limit as usize);
        }

        Ok(records)
    }

    pub fn conversation(
        &self,
        peer_id: String,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, IronCoreError> {
        self.recent(Some(peer_id), limit)
    }

    pub fn remove_conversation(&self, peer_id: String) -> Result<(), IronCoreError> {
        let all = self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| IronCoreError::StorageError)?;

        for (key, value) in all {
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;

            if record.peer_id == peer_id {
                self.backend
                    .remove(&key)
                    .map_err(|_| IronCoreError::StorageError)?;
            }
        }

        Ok(())
    }

    pub fn mark_delivered(&self, id: String) -> Result<(), IronCoreError> {
        if let Some(mut record) = self.get(id)? {
            record.delivered = true;
            self.add(record)?;
        }
        Ok(())
    }

    pub fn clear(&self) -> Result<(), IronCoreError> {
        let all = self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| IronCoreError::StorageError)?;

        for (key, _) in all {
            self.backend
                .remove(&key)
                .map_err(|_| IronCoreError::StorageError)?;
        }
        Ok(())
    }

    pub fn delete(&self, id: String) -> Result<(), IronCoreError> {
        let key = format!("msg_{}", id);
        self.backend
            .remove(key.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?;
        Ok(())
    }

    pub fn stats(&self) -> Result<HistoryStats, IronCoreError> {
        let mut stats = HistoryStats::default();
        let all = self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| IronCoreError::StorageError)?;

        for (_, value) in all {
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;
            stats.total_messages += 1;
            match record.direction {
                MessageDirection::Sent => {
                    stats.sent_count += 1;
                    if !record.delivered {
                        stats.undelivered_count += 1;
                    }
                }
                MessageDirection::Received => stats.received_count += 1,
            }
        }
        Ok(stats)
    }

    pub fn count(&self) -> u32 {
        self.backend.count_prefix(b"msg_").unwrap_or(0) as u32
    }

    pub fn enforce_retention(&self, max_messages: u32) -> Result<u32, IronCoreError> {
        let mut all = self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| IronCoreError::StorageError)?;

        if all.len() <= max_messages as usize {
            return Ok(0);
        }

        // Parse and sort by timestamp ascending (oldest first)
        let mut records: Vec<(Vec<u8>, MessageRecord)> = all
            .into_iter()
            .map(|(k, v)| {
                let rec: MessageRecord = serde_json::from_slice(&v).unwrap();
                (k, rec)
            })
            .collect();

        records.sort_by(|a, b| a.1.timestamp.cmp(&b.1.timestamp));

        let to_remove = records.len() - max_messages as usize;
        for i in 0..to_remove {
            self.backend
                .remove(&records[i].0)
                .map_err(|_| IronCoreError::StorageError)?;
        }

        Ok(to_remove as u32)
    }

    pub fn prune_before(&self, before_timestamp: u64) -> Result<u32, IronCoreError> {
        let all = self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| IronCoreError::StorageError)?;

        let mut removed = 0u32;
        for (key, value) in all {
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;
            if record.timestamp < before_timestamp {
                self.backend
                    .remove(&key)
                    .map_err(|_| IronCoreError::StorageError)?;
                removed += 1;
            }
        }

        Ok(removed)
    }

    pub fn flush(&self) {
        let _ = self.backend.flush();
    }
}
