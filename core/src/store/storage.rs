use crate::store::history::HistoryManager;
use crate::store::logs::LogManager;
use crate::IronCoreError;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct DiskStats {
    pub total_bytes: u64,
    pub free_bytes: u64,
}

/// P0_SECURITY_001: Configurable retention policies for message history.
///
/// Provides bounds on database growth and enforces data minimization.
/// Default retention periods are chosen to balance privacy and usability:
/// - 30 days for aggressive privacy (default)
/// - 90 days for standard usage
/// - 365 days for archival needs
/// - 0 means no time-based retention (only count-based cap applies)
#[derive(Debug, Clone)]
pub struct RetentionConfig {
    /// Maximum number of messages to retain. Messages beyond this cap are pruned
    /// oldest-first. Set to 0 to disable count-based retention.
    pub max_messages: u32,
    /// Maximum age of messages in days. Messages older than this are pruned.
    /// Set to 0 to disable time-based retention.
    pub max_age_days: u32,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            max_messages: 50_000,
            max_age_days: 90,
        }
    }
}

impl RetentionConfig {
    pub fn new(max_messages: u32, max_age_days: u32) -> Self {
        Self {
            max_messages,
            max_age_days,
        }
    }
}

pub struct StorageManager {
    stats: RwLock<DiskStats>,
    history: Arc<HistoryManager>,
    logs: Arc<LogManager>,
    pub retention: RetentionConfig,
}

impl StorageManager {
    pub fn new(history: Arc<HistoryManager>, logs: Arc<LogManager>) -> Self {
        Self {
            stats: RwLock::new(DiskStats::default()),
            history,
            logs,
            retention: RetentionConfig::default(),
        }
    }

    pub fn with_retention(
        history: Arc<HistoryManager>,
        logs: Arc<LogManager>,
        retention: RetentionConfig,
    ) -> Self {
        Self {
            stats: RwLock::new(DiskStats::default()),
            history,
            logs,
            retention,
        }
    }

    pub fn update_disk_stats(&self, total: u64, free: u64) {
        let mut stats = self.stats.write();
        stats.total_bytes = total;
        stats.free_bytes = free;

        tracing::debug!("Disk stats updated: free {} / total {}", free, total);
    }

    /// Perform maintenance to enforce retention policies and ensure free disk space.
    ///
    /// Strategy (in priority order):
    /// 1. Time-based retention: prune messages older than `max_age_days`
    /// 2. Count-based retention: enforce `max_messages` cap
    /// 3. Emergency: if disk space is below 20%, prune 10% of messages + logs
    pub fn perform_maintenance(&self) -> Result<(), IronCoreError> {
        let stats = self.stats.read().clone();

        // P0_SECURITY_001: Always enforce time-based retention first
        if self.retention.max_age_days > 0 {
            let cutoff = current_timestamp()
                .saturating_sub(self.retention.max_age_days as u64 * 86400);
            let pruned = self.history.prune_before(cutoff)?;
            if pruned > 0 {
                tracing::info!(
                    "Retention: pruned {} messages older than {} days",
                    pruned,
                    self.retention.max_age_days
                );
            }
        }

        // Count-based retention: enforce max_messages cap
        if self.retention.max_messages > 0 {
            let current_count = self.history.count();
            if current_count > self.retention.max_messages {
                let pruned = self.history.enforce_retention(self.retention.max_messages)?;
                if pruned > 0 {
                    tracing::info!(
                        "Retention: pruned {} messages exceeding cap of {}",
                        pruned,
                        self.retention.max_messages
                    );
                }
            }
        }

        // Emergency: if disk space is critically low, prune more aggressively
        if stats.total_bytes > 0 {
            let buffer_threshold = (stats.total_bytes as f64 * 0.2) as u64;

            if stats.free_bytes < buffer_threshold {
                tracing::warn!(
                    "Free disk space ({}) below 20% threshold ({}). Starting emergency prune.",
                    stats.free_bytes,
                    buffer_threshold
                );

                // Priority 1: Prune logs
                self.logs.prune_oldest(100)?;

                // Priority 2: Prune 10% of messages if we are low on space
                let current_count = self.history.count();
                if current_count > 100 {
                    let to_keep = (current_count as f64 * 0.9) as u32;
                    self.history.enforce_retention(to_keep)?;
                }
            }
        }

        Ok(())
    }
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
    use crate::store::backend::MemoryStorage;
    use crate::store::history::HistoryManager;
    use crate::store::logs::LogManager;

    fn make_storage_manager() -> StorageManager {
        let backend = Arc::new(MemoryStorage::new());
        let history = Arc::new(HistoryManager::new(backend.clone()));
        let logs = Arc::new(LogManager::new(backend));
        StorageManager::new(history, logs)
    }

    #[test]
    fn test_update_disk_stats() {
        let mgr = make_storage_manager();
        mgr.update_disk_stats(1_000_000, 500_000);
        let stats = mgr.stats.read().clone();
        assert_eq!(stats.total_bytes, 1_000_000);
        assert_eq!(stats.free_bytes, 500_000);
    }

    #[test]
    fn test_maintenance_noop_when_zero() {
        let mgr = make_storage_manager();
        // total_bytes = 0 means no stats yet, should be a no-op
        assert!(mgr.perform_maintenance().is_ok());
    }

    #[test]
    fn test_maintenance_noop_when_enough_space() {
        let mgr = make_storage_manager();
        mgr.update_disk_stats(1_000_000, 300_000); // 30% free > 20% threshold
        assert!(mgr.perform_maintenance().is_ok());
    }

    #[test]
    fn test_maintenance_triggers_prune_when_low() {
        let mgr = make_storage_manager();
        mgr.update_disk_stats(1_000_000, 100_000); // 10% free < 20% threshold
                                                   // Should not error even with empty stores
        assert!(mgr.perform_maintenance().is_ok());
    }

    #[test]
    fn test_disk_stats_default() {
        let stats = DiskStats::default();
        assert_eq!(stats.total_bytes, 0);
        assert_eq!(stats.free_bytes, 0);
    }

    #[test]
    fn test_retention_config_default() {
        let config = RetentionConfig::default();
        assert_eq!(config.max_messages, 50_000);
        assert_eq!(config.max_age_days, 90);
    }

    #[test]
    fn test_maintenance_with_retention_time_based() {
        let backend = Arc::new(MemoryStorage::new());
        let history = Arc::new(HistoryManager::new(backend.clone()));
        let logs = Arc::new(LogManager::new(backend));

        // Add a message with an old timestamp
        let old_record = MessageRecord {
            id: "old_msg".to_string(),
            direction: MessageDirection::Sent,
            peer_id: "peer1".to_string(),
            content: "old message".to_string(),
            timestamp: 1_000_000, // Very old timestamp
            sender_timestamp: 1_000_000,
            delivered: true,
            hidden: false,
        };
        history.add(old_record).unwrap();

        // Add a recent message
        let recent_record = MessageRecord {
            id: "recent_msg".to_string(),
            direction: MessageDirection::Received,
            peer_id: "peer1".to_string(),
            content: "recent message".to_string(),
            timestamp: 1_700_000_000, // Recent
            sender_timestamp: 1_700_000_000,
            delivered: true,
            hidden: false,
        };
        history.add(recent_record).unwrap();

        assert_eq!(history.count(), 2);

        // Retain only messages from the last 30 days (2592000 seconds)
        let retention = RetentionConfig::new(0, 30);
        let mgr = StorageManager::with_retention(history.clone(), logs, retention);
        mgr.perform_maintenance().unwrap();

        // Old message should be pruned, recent should remain
        assert_eq!(history.count(), 1);
        assert!(history.get("recent_msg".to_string()).unwrap().is_some());
    }
}
