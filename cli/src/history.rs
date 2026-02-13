// Message history management for SCMessenger CLI
//
// Stores sent and received messages with search capability

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRecord {
    /// Unique message ID
    pub id: String,

    /// Peer ID of sender
    pub from_peer: String,

    /// Peer ID of recipient
    pub to_peer: String,

    /// Message content
    pub content: String,

    /// Timestamp (unix seconds)
    pub timestamp: u64,

    /// Direction from our perspective
    pub direction: Direction,

    /// Whether message was successfully delivered/received
    pub delivered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Sent,
    Received,
}

impl MessageRecord {
    pub fn new_sent(to_peer: String, content: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from_peer: "me".to_string(), // Will be set by caller
            to_peer,
            content,
            timestamp: current_timestamp(),
            direction: Direction::Sent,
            delivered: false,
        }
    }

    pub fn new_received(from_peer: String, content: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from_peer,
            to_peer: "me".to_string(),
            content,
            timestamp: current_timestamp(),
            direction: Direction::Received,
            delivered: true,
        }
    }

    pub fn formatted_time(&self) -> String {
        let dt = DateTime::from_timestamp(self.timestamp as i64, 0).unwrap_or_else(Utc::now);
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn peer(&self) -> &str {
        match self.direction {
            Direction::Sent => &self.to_peer,
            Direction::Received => &self.from_peer,
        }
    }
}

pub struct MessageHistory {
    db: Db,
}

impl MessageHistory {
    /// Open or create message history database
    pub fn open(path: PathBuf) -> Result<Self> {
        let db = sled::open(path).context("Failed to open message history database")?;
        Ok(Self { db })
    }

    /// Add a message to history
    pub fn add(&self, record: MessageRecord) -> Result<()> {
        // Generate key: timestamp_id for chronological ordering
        let key = format!("{:020}_{}", record.timestamp, record.id);

        let value = serde_json::to_vec(&record).context("Failed to serialize message record")?;

        self.db
            .insert(key.as_bytes(), value)
            .context("Failed to insert message record")?;

        Ok(())
    }

    /// Get a specific message by ID
    #[allow(dead_code)]
    pub fn get(&self, id: &str) -> Result<Option<MessageRecord>> {
        for item in self.db.iter() {
            let (_, value) = item?;
            let record: MessageRecord = serde_json::from_slice(&value)?;
            if record.id == id {
                return Ok(Some(record));
            }
        }
        Ok(None)
    }

    /// Get recent messages (all or filtered by peer)
    pub fn recent(&self, peer_filter: Option<&str>, limit: usize) -> Result<Vec<MessageRecord>> {
        let mut messages = Vec::new();

        // Iterate in reverse (most recent first)
        for item in self.db.iter().rev() {
            let (_, value) = item?;
            let record: MessageRecord = serde_json::from_slice(&value)?;

            // Apply peer filter if specified
            if let Some(peer) = peer_filter {
                if record.peer() != peer {
                    continue;
                }
            }

            messages.push(record);

            if messages.len() >= limit {
                break;
            }
        }

        Ok(messages)
    }

    /// Get conversation with a specific peer
    pub fn conversation(&self, peer_id: &str, limit: usize) -> Result<Vec<MessageRecord>> {
        self.recent(Some(peer_id), limit)
    }

    /// Search messages by content
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<MessageRecord>> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for item in self.db.iter().rev() {
            let (_, value) = item?;
            let record: MessageRecord = serde_json::from_slice(&value)?;

            if record.content.to_lowercase().contains(&query_lower) {
                results.push(record);

                if results.len() >= limit {
                    break;
                }
            }
        }

        Ok(results)
    }

    /// Count total messages
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.db.len()
    }

    /// Count messages with a specific peer
    #[allow(dead_code)]
    pub fn count_with_peer(&self, peer_id: &str) -> Result<usize> {
        let mut count = 0;

        for item in self.db.iter() {
            let (_, value) = item?;
            let record: MessageRecord = serde_json::from_slice(&value)?;
            if record.peer() == peer_id {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Mark message as delivered
    #[allow(dead_code)]
    pub fn mark_delivered(&self, id: &str) -> Result<()> {
        if let Some(mut record) = self.get(id)? {
            record.delivered = true;
            self.add(record)?;
        }
        Ok(())
    }

    /// Delete all messages
    #[allow(dead_code)]
    pub fn clear(&self) -> Result<()> {
        self.db.clear()?;
        Ok(())
    }

    /// Delete messages with a specific peer
    #[allow(dead_code)]
    pub fn clear_conversation(&self, peer_id: &str) -> Result<usize> {
        let mut deleted = 0;
        let mut keys_to_delete = Vec::new();

        for item in self.db.iter() {
            let (key, value) = item?;
            let record: MessageRecord = serde_json::from_slice(&value)?;
            if record.peer() == peer_id {
                keys_to_delete.push(key.to_vec());
            }
        }

        for key in keys_to_delete {
            self.db.remove(key)?;
            deleted += 1;
        }

        Ok(deleted)
    }

    /// Get statistics
    pub fn stats(&self) -> Result<HistoryStats> {
        let mut stats = HistoryStats::default();

        for item in self.db.iter() {
            let (_, value) = item?;
            let record: MessageRecord = serde_json::from_slice(&value)?;

            stats.total_messages += 1;

            match record.direction {
                Direction::Sent => stats.sent_messages += 1,
                Direction::Received => stats.received_messages += 1,
            }

            if record.delivered {
                stats.delivered_messages += 1;
            }

            // Track unique peers
            let peer = record.peer().to_string();
            if !stats.unique_peers.contains(&peer) {
                stats.unique_peers.push(peer);
            }
        }

        Ok(stats)
    }
}

#[derive(Debug, Default)]
pub struct HistoryStats {
    pub total_messages: usize,
    pub sent_messages: usize,
    pub received_messages: usize,
    pub delivered_messages: usize,
    pub unique_peers: Vec<String>,
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_record() {
        let msg = MessageRecord::new_sent("peer123".to_string(), "Hello!".to_string());

        assert_eq!(msg.direction, Direction::Sent);
        assert_eq!(msg.peer(), "peer123");
        assert!(!msg.delivered);
    }

    #[test]
    fn test_message_history() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let db_path = temp_dir.path().join("history.db");

        let history = MessageHistory::open(db_path)?;

        // Add sent message
        let msg1 = MessageRecord::new_sent("peer1".to_string(), "Hello".to_string());
        history.add(msg1)?;

        // Add received message
        let msg2 = MessageRecord::new_received("peer1".to_string(), "Hi there".to_string());
        history.add(msg2)?;

        // Check count
        assert_eq!(history.count(), 2);

        // Get conversation
        let conv = history.conversation("peer1", 10)?;
        assert_eq!(conv.len(), 2);

        // Search
        let results = history.search("Hello", 10)?;
        assert_eq!(results.len(), 1);

        Ok(())
    }
}
