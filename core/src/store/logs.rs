use crate::store::backend::StorageBackend;
use crate::IronCoreError;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSummary {
    pub content: String,
    /// Timestamps as deltas.
    /// [0] = seconds since install_time.
    /// [i] = seconds since previous occurrence.
    pub deltas: Vec<u32>,
}

pub struct LogManager {
    backend: Arc<dyn StorageBackend>,
    install_time: u64,
    /// In-memory cache for fast logging, periodically flushed.
    cache: RwLock<HashMap<u64, LogSummary>>,
}

impl LogManager {
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        let install_time = match backend.get(b"metadata_install_time") {
            Ok(Some(data)) => {
                let s = String::from_utf8_lossy(&data);
                s.parse()
                    .unwrap_or_else(|_| Self::init_install_time(&*backend))
            }
            _ => Self::init_install_time(&*backend),
        };

        Self {
            backend,
            install_time,
            cache: RwLock::new(HashMap::new()),
        }
    }

    fn init_install_time(backend: &dyn StorageBackend) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = backend.put(b"metadata_install_time", now.to_string().as_bytes());
        now
    }

    pub fn record_log(&self, line: String) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Use a simple hash for the line content
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::Hasher;
        hasher.write(line.as_bytes());
        let hash = hasher.finish();

        let mut cache = self.cache.write();
        let entry = cache.entry(hash).or_insert_with(|| {
            // Try to load from backend if not in cache
            if let Ok(Some(data)) = self.backend.get(format!("log_sum_{}", hash).as_bytes()) {
                if let Ok(sum) = serde_json::from_slice::<LogSummary>(&data) {
                    return sum;
                }
            }
            LogSummary {
                content: line,
                deltas: Vec::new(),
            }
        });

        if entry.deltas.is_empty() {
            let offset = (now.saturating_sub(self.install_time)) as u32;
            entry.deltas.push(offset);
        } else {
            // Calculate delta from last occurrence
            // We need to know the absolute time of the last occurrence.
            // Since we only store deltas from install_time, we'd have to sum them all.
            // Efficient way: store last_abs_time in-memory only or store it in LogSummary too.
            // Let's calculate it by summing deltas (usually few deltas per log window).
            let mut last_abs = self.install_time;
            for d in &entry.deltas {
                last_abs += *d as u64;
            }
            let delta = (now.saturating_sub(last_abs)) as u32;
            entry.deltas.push(delta);
        }

        // Limit deltas per log type to avoid memory bloat before flush
        if entry.deltas.len() > 1000 {
            entry.deltas.drain(..500);
        }
    }

    pub fn flush(&self) -> Result<(), IronCoreError> {
        let mut cache = self.cache.write();
        for (hash, summary) in cache.drain() {
            let key = format!("log_sum_{}", hash);
            let value = serde_json::to_vec(&summary).map_err(|_| IronCoreError::Internal)?;
            self.backend
                .put(key.as_bytes(), &value)
                .map_err(|_| IronCoreError::StorageError)?;
        }
        if let Err(e) = self.backend.flush() {
            tracing::warn!("Log backend flush failed: {:?}", e);
        }
        Ok(())
    }

    pub fn prune_oldest(&self, count: usize) -> Result<u32, IronCoreError> {
        // Prune logs to save space.
        // For now, let's just clear many logs.
        let mut pruned = 0;
        let all = self
            .backend
            .scan_prefix(b"log_sum_")
            .map_err(|_| IronCoreError::StorageError)?;

        for (key, _) in all.iter().take(count) {
            self.backend
                .remove(key)
                .map_err(|_| IronCoreError::StorageError)?;
            pruned += 1;
        }
        Ok(pruned)
    }

    pub fn export_all(&self) -> Result<String, IronCoreError> {
        self.flush()?;
        let all = self
            .backend
            .scan_prefix(b"log_sum_")
            .map_err(|_| IronCoreError::StorageError)?;
        let mut logs = Vec::new();
        for (_, value) in all {
            let sum: LogSummary =
                serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;
            logs.push(sum);
        }
        serde_json::to_string_pretty(&logs).map_err(|_| IronCoreError::Internal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::backend::MemoryStorage;

    fn make_manager() -> LogManager {
        let backend = Arc::new(MemoryStorage::new());
        LogManager::new(backend)
    }

    #[test]
    fn test_record_and_export() {
        let mgr = make_manager();
        mgr.record_log("test line 1".to_string());
        mgr.record_log("test line 2".to_string());
        mgr.record_log("test line 1".to_string()); // duplicate

        let exported = mgr.export_all().unwrap();
        let logs: Vec<LogSummary> = serde_json::from_str(&exported).unwrap();
        // Two unique lines
        assert_eq!(logs.len(), 2);
        // "test line 1" should have 2 deltas (recorded twice)
        let line1 = logs.iter().find(|l| l.content == "test line 1").unwrap();
        assert_eq!(line1.deltas.len(), 2);
        // "test line 2" should have 1 delta
        let line2 = logs.iter().find(|l| l.content == "test line 2").unwrap();
        assert_eq!(line2.deltas.len(), 1);
    }

    #[test]
    fn test_flush_and_reload() {
        let backend = Arc::new(MemoryStorage::new());
        let mgr = LogManager::new(backend.clone());
        mgr.record_log("persistent line".to_string());
        mgr.flush().unwrap();

        // Create new manager with same backend
        let mgr2 = LogManager::new(backend);
        mgr2.record_log("persistent line".to_string());
        let exported = mgr2.export_all().unwrap();
        let logs: Vec<LogSummary> = serde_json::from_str(&exported).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].deltas.len(), 2); // original + new
    }

    #[test]
    fn test_prune_oldest() {
        let mgr = make_manager();
        for i in 0..20 {
            mgr.record_log(format!("log line {}", i));
        }
        mgr.flush().unwrap();
        let pruned = mgr.prune_oldest(10).unwrap();
        assert_eq!(pruned, 10);

        let exported = mgr.export_all().unwrap();
        let logs: Vec<LogSummary> = serde_json::from_str(&exported).unwrap();
        assert_eq!(logs.len(), 10);
    }

    #[test]
    fn test_install_time_persisted() {
        let backend = Arc::new(MemoryStorage::new());
        let mgr1 = LogManager::new(backend.clone());
        let time1 = mgr1.install_time;

        // Second manager should read same install time
        let mgr2 = LogManager::new(backend);
        assert_eq!(mgr2.install_time, time1);
    }

    #[test]
    fn test_empty_export() {
        let mgr = make_manager();
        let exported = mgr.export_all().unwrap();
        assert_eq!(exported, "[]");
    }

    #[test]
    fn test_delta_pruning_under_limit() {
        let mgr = make_manager();
        // Record same line many times - should not overflow
        for _ in 0..50 {
            mgr.record_log("repeated".to_string());
        }
        let exported = mgr.export_all().unwrap();
        let logs: Vec<LogSummary> = serde_json::from_str(&exported).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].deltas.len(), 50);
    }
}
