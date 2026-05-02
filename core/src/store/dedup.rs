// Dedup statistics tracking for inbound message deduplication
//
// Provides core parity with the platform-specific dedup stats APIs (Android
// MessageDedupEntry, iOS equivalent). The Inbox module handles dedup tracking;
// this module exposes queryable statistics for diagnostics and mesh enhancement
// logging.

use serde::{Deserialize, Serialize};

/// Statistics for a deduplicated message entry.
///
/// Mirrors the Android `MessageDedupEntry` and provides cross-platform parity
/// for dedup diagnostics. Entries are keyed by message ID and track first
/// reception time, duplicate count, and which transport first delivered the
/// message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupStats {
    pub message_id: String,
    pub first_received_at: u64,
    pub duplicate_count: u32,
    pub first_transport: Option<String>,
}

impl DedupStats {
    /// Create a new dedup stats entry.
    pub fn new(message_id: String, first_received_at: u64) -> Self {
        Self {
            message_id,
            first_received_at,
            duplicate_count: 0,
            first_transport: None,
        }
    }

    /// Record a duplicate occurrence, incrementing the count.
    pub fn record_duplicate(&mut self) {
        self.duplicate_count = self.duplicate_count.saturating_add(1);
    }
}

/// A dedup stats tracker that records per-message dedup statistics.
///
/// This is a standalone tracker that can be composed with the Inbox or used
/// independently by transport layers for mesh enhancement logging.
#[derive(Debug, Clone)]
pub struct DedupStatsTracker {
    entries: std::collections::HashMap<String, DedupStats>,
    max_entries: usize,
}

impl DedupStatsTracker {
    /// Create a new tracker with a default maximum entry count.
    pub fn new() -> Self {
        Self {
            entries: std::collections::HashMap::new(),
            max_entries: 10_000,
        }
    }

    /// Create a tracker with a custom maximum entry count.
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            entries: std::collections::HashMap::new(),
            max_entries,
        }
    }

    /// Record that a message was received. Returns the updated stats entry.
    /// If this is a first-seen message, creates a new entry. If it's a
    /// duplicate, increments the duplicate count.
    pub fn record_received(
        &mut self,
        message_id: &str,
        first_transport: Option<String>,
    ) -> &DedupStats {
        let now_ms = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if let Some(entry) = self.entries.get_mut(message_id) {
            entry.record_duplicate();
            self.entries.get(message_id).unwrap()
        } else {
            let mut stats = DedupStats::new(message_id.to_string(), now_ms);
            stats.first_transport = first_transport;
            if self.entries.len() >= self.max_entries {
                // Evict the oldest entry
                if let Some(oldest_key) = self
                    .entries
                    .iter()
                    .min_by_key(|(_, v)| v.first_received_at)
                    .map(|(k, _)| k.clone())
                {
                    self.entries.remove(&oldest_key);
                }
            }
            self.entries.insert(message_id.to_string(), stats);
            self.entries.get(message_id).unwrap()
        }
    }

    /// Get dedup statistics for a specific message ID.
    /// Returns None if the message has not been tracked.
    pub fn get_dedup_stats(&self, message_id: &str) -> Option<&DedupStats> {
        self.entries.get(message_id)
    }

    /// Total number of tracked entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the tracker is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all tracked entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Aggregate statistics across all tracked entries.
    pub fn aggregate(&self) -> DedupAggregateStats {
        let total_entries = self.entries.len();
        let total_duplicates: u32 = self.entries.values().map(|e| e.duplicate_count).sum();
        let entries_with_transport = self
            .entries
            .values()
            .filter(|e| e.first_transport.is_some())
            .count();
        DedupAggregateStats {
            total_entries,
            total_duplicates,
            entries_with_transport,
        }
    }
}

impl Default for DedupStatsTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregate statistics across all dedup entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupAggregateStats {
    pub total_entries: usize,
    pub total_duplicates: u32,
    pub entries_with_transport: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_stats_records_first_seen() {
        let mut tracker = DedupStatsTracker::new();
        let stats = tracker.record_received("msg-1", Some("ble".to_string()));
        assert_eq!(stats.message_id, "msg-1");
        assert_eq!(stats.duplicate_count, 0);
        assert_eq!(stats.first_transport.as_deref(), Some("ble"));
    }

    #[test]
    fn dedup_stats_increments_duplicates() {
        let mut tracker = DedupStatsTracker::new();
        tracker.record_received("msg-1", Some("wifi".to_string()));
        tracker.record_received("msg-1", Some("ble".to_string()));
        tracker.record_received("msg-1", None);

        let stats = tracker.get_dedup_stats("msg-1").unwrap();
        assert_eq!(stats.duplicate_count, 2);
        // First transport should remain unchanged
        assert_eq!(stats.first_transport.as_deref(), Some("wifi"));
    }

    #[test]
    fn get_dedup_stats_returns_none_for_unknown() {
        let tracker = DedupStatsTracker::new();
        assert!(tracker.get_dedup_stats("unknown").is_none());
    }

    #[test]
    fn aggregate_stats_are_correct() {
        let mut tracker = DedupStatsTracker::new();
        tracker.record_received("msg-1", Some("wifi".to_string()));
        tracker.record_received("msg-1", Some("ble".to_string()));
        tracker.record_received("msg-2", None);

        let agg = tracker.aggregate();
        assert_eq!(agg.total_entries, 2);
        assert_eq!(agg.total_duplicates, 1);
        assert_eq!(agg.entries_with_transport, 1);
    }

    #[test]
    fn tracker_evicts_oldest_at_capacity() {
        let mut tracker = DedupStatsTracker::with_max_entries(2);
        tracker.record_received("msg-1", None);
        tracker.record_received("msg-2", None);
        tracker.record_received("msg-3", None);

        assert_eq!(tracker.len(), 2);
        // msg-1 should have been evicted
        assert!(tracker.get_dedup_stats("msg-1").is_none());
        assert!(tracker.get_dedup_stats("msg-3").is_some());
    }
}
