//! Multi-path delivery (Phase 2 API stub)
//!
//! Placeholder implementation for MultiPathDelivery used by integration tests.
//! Full implementation will be delivered in Phase 2.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a delivery path for multi-path message routing
#[derive(Debug, Clone)]
pub struct DeliveryPath {
    /// Unique path identifier
    pub path_id: u64,
    /// Target peer ID
    pub peer_id: u64,
    /// Latency estimate in milliseconds
    pub estimated_latency_ms: u64,
    /// Whether this path is currently active
    pub active: bool,
}

/// Manages multi-path message delivery across redundant routes
#[derive(Debug, Clone)]
pub struct MultiPathDelivery {
    paths: HashMap<u64, Vec<DeliveryPath>>,
    max_paths_per_peer: usize,
}

impl Default for MultiPathDelivery {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiPathDelivery {
    /// Create a new multi-path delivery manager
    pub fn new() -> Self {
        Self {
            paths: HashMap::new(),
            max_paths_per_peer: 3,
        }
    }

    /// Register a delivery path for a peer
    pub fn register_path(&mut self, peer_id: u64, path: DeliveryPath) {
        let paths = self.paths.entry(peer_id).or_default();
        if paths.len() < self.max_paths_per_peer {
            paths.push(path);
        }
    }

    /// Get all active paths for a peer
    pub fn active_paths(&self, peer_id: u64) -> Vec<&DeliveryPath> {
        self.paths
            .get(&peer_id)
            .map(|paths| paths.iter().filter(|p| p.active).collect())
            .unwrap_or_default()
    }

    /// Mark a path as inactive (failed)
    pub fn mark_path_failed(&mut self, path_id: u64) {
        for paths in self.paths.values_mut() {
            if let Some(path) = paths.iter_mut().find(|p| p.path_id == path_id) {
                path.active = false;
                break;
            }
        }
    }

    /// Get the number of peers with registered paths
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// Check if the delivery manager is empty
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

}