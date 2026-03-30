// Blocked identities and device management
//
// Implements identity blocking with future device ID pairing support.
// TODO: Add device ID to identity pairing for multi-device blocking.

use crate::store::backend::StorageBackend;
use crate::IronCoreError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A blocked identity entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedIdentity {
    /// The peer ID (identity hash) being blocked
    pub peer_id: String,
    /// Optional device ID for granular blocking
    /// TODO: Implement device ID pairing with identity
    /// When implemented, this enables blocking specific devices while allowing others
    pub device_id: Option<String>,
    /// When this identity was blocked
    pub blocked_at: u64,
    /// Optional reason for blocking
    pub reason: Option<String>,
    /// Notes about this block
    pub notes: Option<String>,
    /// When true, the contact has been both blocked AND deleted.
    ///
    /// Blocked-only (`is_deleted = false`): messages are still received and
    /// persisted for evidentiary purposes, but filtered out of all UI queries.
    ///
    /// Blocked + Deleted (`is_deleted = true`): all existing stored messages
    /// are purged and any future incoming network payloads are dropped at the
    /// ingress layer without being persisted.
    #[serde(default)]
    pub is_deleted: bool,
}

impl BlockedIdentity {
    /// Create a new blocked identity
    pub fn new(peer_id: String) -> Self {
        Self {
            peer_id,
            device_id: None,
            blocked_at: current_timestamp(),
            reason: None,
            notes: None,
            is_deleted: false,
        }
    }

    /// Create a full relay capability (for WASM compatibility)
    /// This is a stub implementation for WASM targets where relay functionality
    /// is not available but the type is used for compatibility.
    pub fn full_relay() -> Self {
        Self::new("relay-stub".to_string())
    }

    /// Block a specific device of this identity
    /// TODO: Requires device ID infrastructure
    pub fn with_device_id(mut self, device_id: String) -> Self {
        self.device_id = Some(device_id);
        self
    }

    /// Add a reason for blocking
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }

    /// Get a storage key for this block
    fn storage_key(&self) -> String {
        match &self.device_id {
            Some(device_id) => format!("blocked:{}:{}", self.peer_id, device_id),
            None => format!("blocked:{}", self.peer_id),
        }
    }
}

/// Manager for blocked identities
#[derive(Clone)]
pub struct BlockedManager {
    backend: Arc<dyn StorageBackend>,
}

impl BlockedManager {
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        Self { backend }
    }

    /// Block a peer ID
    pub fn block(&self, blocked: BlockedIdentity) -> Result<(), IronCoreError> {
        let key = blocked.storage_key();
        let value = serde_json::to_vec(&blocked).map_err(|_| IronCoreError::Internal)?;
        self.backend
            .put(key.as_bytes(), &value)
            .map_err(|_| IronCoreError::StorageError)?;
        Ok(())
    }

    /// Block a peer AND mark them as deleted (cascade purge variant).
    ///
    /// This sets `is_deleted = true` on the block record so that the ingress
    /// layer knows to drop all future payloads without persisting them, and so
    /// that the caller can trigger a cascade purge of existing stored messages.
    pub fn block_and_delete(
        &self,
        peer_id: String,
        reason: Option<String>,
    ) -> Result<(), IronCoreError> {
        let mut blocked = BlockedIdentity::new(peer_id);
        blocked.is_deleted = true;
        if let Some(r) = reason {
            blocked.reason = Some(r);
        }
        self.block(blocked)
    }

    /// Unblock a peer ID
    pub fn unblock(&self, peer_id: String, device_id: Option<String>) -> Result<(), IronCoreError> {
        let key = match device_id {
            Some(device_id) => format!("blocked:{}:{}", peer_id, device_id),
            None => format!("blocked:{}", peer_id),
        };
        self.backend
            .remove(key.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?;
        Ok(())
    }

    /// Check if a peer ID is blocked
    pub fn is_blocked(
        &self,
        peer_id: &str,
        device_id: Option<&str>,
    ) -> Result<bool, IronCoreError> {
        // Check for device-specific block first
        if let Some(device_id) = device_id {
            let key = format!("blocked:{}:{}", peer_id, device_id);
            if self
                .backend
                .get(key.as_bytes())
                .map_err(|_| IronCoreError::StorageError)?
                .is_some()
            {
                return Ok(true);
            }
        }

        // Check for peer-level block
        let key = format!("blocked:{}", peer_id);
        let blocked = self
            .backend
            .get(key.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?
            .is_some();
        Ok(blocked)
    }

    /// Get blocked identity details
    pub fn get(
        &self,
        peer_id: &str,
        device_id: Option<&str>,
    ) -> Result<Option<BlockedIdentity>, IronCoreError> {
        let key = match device_id {
            Some(device_id) => format!("blocked:{}:{}", peer_id, device_id),
            None => format!("blocked:{}", peer_id),
        };

        if let Some(data) = self
            .backend
            .get(key.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?
        {
            let blocked: BlockedIdentity =
                serde_json::from_slice(&data).map_err(|_| IronCoreError::Internal)?;
            Ok(Some(blocked))
        } else {
            Ok(None)
        }
    }

    /// List all blocked identities
    pub fn list(&self) -> Result<Vec<BlockedIdentity>, IronCoreError> {
        let all = self
            .backend
            .scan_prefix(b"blocked:")
            .map_err(|_| IronCoreError::StorageError)?;

        let mut blocked_list = Vec::new();
        for (_, value) in all {
            let blocked: BlockedIdentity =
                serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;
            blocked_list.push(blocked);
        }

        blocked_list.sort_by(|a, b| b.blocked_at.cmp(&a.blocked_at));
        Ok(blocked_list)
    }

    /// Get count of blocked identities
    pub fn count(&self) -> Result<usize, IronCoreError> {
        Ok(self.list()?.len())
    }

    /// Check if a peer is both blocked AND deleted (cascade purge state).
    ///
    /// Returns `true` only when `is_deleted = true` on the block record.
    /// Blocked-only peers (where `is_deleted = false`) return `false`.
    pub fn is_blocked_and_deleted(&self, peer_id: &str) -> Result<bool, IronCoreError> {
        let key = format!("blocked:{}", peer_id);
        if let Some(data) = self
            .backend
            .get(key.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?
        {
            let blocked: BlockedIdentity =
                serde_json::from_slice(&data).map_err(|_| IronCoreError::Internal)?;
            Ok(blocked.is_deleted)
        } else {
            Ok(false)
        }
    }

    /// Return the peer IDs of all blocked-only (not deleted) identities.
    ///
    /// Used by the query layer to filter messages from blocked peers out of UI
    /// results without purging them (evidentiary retention).
    pub fn blocked_only_peer_ids(
        &self,
    ) -> Result<std::collections::HashSet<String>, IronCoreError> {
        let list = self.list()?;
        Ok(list
            .into_iter()
            .filter(|b| !b.is_deleted)
            .map(|b| b.peer_id)
            .collect())
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::backend::MemoryStorage;

    #[test]
    fn test_block_unblock() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "12D3KooWTest123";

        // Initially not blocked
        assert!(!manager.is_blocked(peer_id, None).unwrap());

        // Block the peer
        let blocked = BlockedIdentity::new(peer_id.to_string()).with_reason("Spam".to_string());
        manager.block(blocked).unwrap();

        // Now blocked
        assert!(manager.is_blocked(peer_id, None).unwrap());

        // Unblock
        manager.unblock(peer_id.to_string(), None).unwrap();
        assert!(!manager.is_blocked(peer_id, None).unwrap());
    }

    #[test]
    fn test_device_specific_block() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "12D3KooWTest123";
        let device_id = "device-abc-123";

        // Block specific device
        let blocked =
            BlockedIdentity::new(peer_id.to_string()).with_device_id(device_id.to_string());
        manager.block(blocked).unwrap();

        // Device-specific check
        assert!(manager.is_blocked(peer_id, Some(device_id)).unwrap());
        // Peer without device not blocked
        assert!(!manager.is_blocked(peer_id, None).unwrap());
    }

    #[test]
    fn test_list_blocked() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        manager
            .block(BlockedIdentity::new("peer1".to_string()))
            .unwrap();
        manager
            .block(BlockedIdentity::new("peer2".to_string()))
            .unwrap();

        let list = manager.list().unwrap();
        assert_eq!(list.len(), 2);
    }
}
