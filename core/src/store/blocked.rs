// Blocked identities and device management
//
// Implements identity blocking with device-ID pairing for multi-device blocking.
// When a peer identity is blocked, all known device IDs for that peer are also
// blocked. New device IDs discovered for a blocked peer are auto-blocked.

use crate::store::backend::StorageBackend;
use crate::IronCoreError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Storage key prefix for blocked identity entries
const BLOCKED_PREFIX: &str = "blocked:";
/// Storage key prefix for device registry entries (peer -> known device IDs)
const DEVICE_REGISTRY_PREFIX: &str = "blocked_devs:";

/// A blocked identity entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedIdentity {
    /// The peer ID (identity hash) being blocked
    pub peer_id: String,
    /// Optional device ID for granular blocking.
    /// When present, only this device of the peer is blocked.
    /// When absent, all devices of the peer are blocked.
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
            Some(device_id) => format!("{}{}:{}", BLOCKED_PREFIX, self.peer_id, device_id),
            None => format!("{}{}", BLOCKED_PREFIX, self.peer_id),
        }
    }
}

/// Manager for blocked identities with device-ID pairing support.
///
/// When a peer identity is blocked, all known device IDs for that peer are
/// also blocked via device-specific entries. New device IDs registered for a
/// blocked peer are automatically blocked.
#[derive(Clone)]
pub struct BlockedManager {
    backend: Arc<dyn StorageBackend>,
}

impl BlockedManager {
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        Self { backend }
    }

    // -----------------------------------------------------------------------
    // Core blocking operations
    // -----------------------------------------------------------------------

    /// Block a peer ID, also blocking all known device IDs for that peer.
    ///
    /// If the `BlockedIdentity` has no `device_id`, this creates a peer-level
    /// block AND individual device-specific blocks for every device ID in the
    /// device registry for this peer. If a `device_id` is set, only that
    /// specific device is blocked.
    pub fn block(&self, blocked: BlockedIdentity) -> Result<(), IronCoreError> {
        let key = blocked.storage_key();
        let value = serde_json::to_vec(&blocked).map_err(|_| IronCoreError::Internal)?;
        self.backend
            .put(key.as_bytes(), &value)
            .map_err(|_| IronCoreError::StorageError)?;

        // Peer-level block: also block every known device for this peer
        if blocked.device_id.is_none() {
            let devices = self.get_known_devices(&blocked.peer_id)?;
            for device_id in devices {
                let device_blocked = BlockedIdentity {
                    peer_id: blocked.peer_id.clone(),
                    device_id: Some(device_id),
                    blocked_at: blocked.blocked_at,
                    reason: blocked.reason.clone(),
                    notes: blocked.notes.clone(),
                    is_deleted: blocked.is_deleted,
                };
                let dkey = device_blocked.storage_key();
                let dvalue =
                    serde_json::to_vec(&device_blocked).map_err(|_| IronCoreError::Internal)?;
                self.backend
                    .put(dkey.as_bytes(), &dvalue)
                    .map_err(|_| IronCoreError::StorageError)?;
            }
        }

        Ok(())
    }

    /// Block a peer AND mark them as deleted (cascade purge variant).
    ///
    /// This sets `is_deleted = true` on the block record so that the ingress
    /// layer knows to drop all future payloads without persisting them, and so
    /// that the caller can trigger a cascade purge of existing stored messages.
    /// Also blocks all known device IDs for the peer.
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

    /// Unblock a peer ID.
    ///
    /// If `device_id` is `None`, removes both the peer-level block AND all
    /// device-specific blocks for that peer, and clears the device registry.
    /// If `device_id` is `Some(...)`, only removes that specific device block.
    pub fn unblock(&self, peer_id: String, device_id: Option<String>) -> Result<(), IronCoreError> {
        match device_id {
            Some(did) => {
                // Remove only the device-specific block
                let key = format!("{}{}:{}", BLOCKED_PREFIX, peer_id, did);
                self.backend
                    .remove(key.as_bytes())
                    .map_err(|_| IronCoreError::StorageError)?;
                // Also remove from device registry
                self.unregister_device_from_registry(&peer_id, &did)?;
            }
            None => {
                // Remove peer-level block
                let key = format!("{}{}", BLOCKED_PREFIX, peer_id);
                self.backend
                    .remove(key.as_bytes())
                    .map_err(|_| IronCoreError::StorageError)?;

                // Remove all device-specific blocks for this peer
                let devices = self.get_known_devices(&peer_id)?;
                for did in devices {
                    let dkey = format!("{}{}:{}", BLOCKED_PREFIX, peer_id, did);
                    self.backend
                        .remove(dkey.as_bytes())
                        .map_err(|_| IronCoreError::StorageError)?;
                }
                // Clear device registry for this peer
                let reg_key = format!("{}{}", DEVICE_REGISTRY_PREFIX, peer_id);
                self.backend
                    .remove(reg_key.as_bytes())
                    .map_err(|_| IronCoreError::StorageError)?;
            }
        }
        Ok(())
    }

    /// Check if a peer ID is blocked.
    ///
    /// If `device_id` is provided, checks for a device-specific block first,
    /// then falls back to the peer-level block. A peer-level block covers all
    /// devices of that peer.
    pub fn is_blocked(
        &self,
        peer_id: &str,
        device_id: Option<&str>,
    ) -> Result<bool, IronCoreError> {
        // Check for device-specific block first
        if let Some(device_id) = device_id {
            let key = format!("{}{}:{}", BLOCKED_PREFIX, peer_id, device_id);
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
        let key = format!("{}{}", BLOCKED_PREFIX, peer_id);
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
            Some(device_id) => format!("{}{}:{}", BLOCKED_PREFIX, peer_id, device_id),
            None => format!("{}{}", BLOCKED_PREFIX, peer_id),
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

    /// List all blocked identities (peer-level and device-specific)
    pub fn list(&self) -> Result<Vec<BlockedIdentity>, IronCoreError> {
        let all = self
            .backend
            .scan_prefix(BLOCKED_PREFIX.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?;

        let mut blocked_list = Vec::new();
        for (_, value) in all {
            let blocked: BlockedIdentity =
                serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;
            blocked_list.push(blocked);
        }

        blocked_list.sort_by_key(|b| std::cmp::Reverse(b.blocked_at));
        Ok(blocked_list)
    }

    /// Get count of blocked identities
    pub fn count(&self) -> Result<usize, IronCoreError> {
        Ok(self.list()?.len())
    }

    /// Check if a peer is both blocked AND deleted (cascade purge state).
    ///
    /// Checks the peer-level block. Returns `true` only when `is_deleted = true`
    /// on the peer-level block record.
    pub fn is_blocked_and_deleted(&self, peer_id: &str) -> Result<bool, IronCoreError> {
        let key = format!("{}{}", BLOCKED_PREFIX, peer_id);
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
    /// results without purging them (evidentiary retention). Includes both
    /// peer-level and device-specific blocked entries.
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

    // -----------------------------------------------------------------------
    // Device-ID pairing and registry
    // -----------------------------------------------------------------------

    /// Register a device ID as belonging to a peer identity.
    ///
    /// If the peer is currently blocked (either peer-level or device-specific),
    /// the newly registered device is automatically blocked as well.
    /// This enables multi-device blocking: when a new device is discovered for
    /// an already-blocked peer, that device is immediately blocked too.
    pub fn register_device_id(
        &self,
        peer_id: &str,
        device_id: &str,
    ) -> Result<bool, IronCoreError> {
        let mut devices = self.get_known_devices(peer_id)?;
        if devices.contains(&device_id.to_string()) {
            return Ok(false); // Already registered
        }
        devices.push(device_id.to_string());
        let reg_key = format!("{}{}", DEVICE_REGISTRY_PREFIX, peer_id);
        let encoded = serde_json::to_vec(&devices).map_err(|_| IronCoreError::Internal)?;
        self.backend
            .put(reg_key.as_bytes(), &encoded)
            .map_err(|_| IronCoreError::StorageError)?;

        // If the peer is blocked, auto-block this device
        if self.is_blocked(peer_id, None)? {
            let peer_block = self.get(peer_id, None)?;
            let blocked = BlockedIdentity {
                peer_id: peer_id.to_string(),
                device_id: Some(device_id.to_string()),
                blocked_at: peer_block
                    .as_ref()
                    .map(|b| b.blocked_at)
                    .unwrap_or_else(current_timestamp),
                reason: peer_block.as_ref().and_then(|b| b.reason.clone()),
                notes: None,
                is_deleted: peer_block.as_ref().map(|b| b.is_deleted).unwrap_or(false),
            };
            let dkey = blocked.storage_key();
            let dvalue = serde_json::to_vec(&blocked).map_err(|_| IronCoreError::Internal)?;
            self.backend
                .put(dkey.as_bytes(), &dvalue)
                .map_err(|_| IronCoreError::StorageError)?;
        }

        Ok(true) // Newly registered
    }

    /// Get all known device IDs registered for a peer.
    pub fn get_known_devices(&self, peer_id: &str) -> Result<Vec<String>, IronCoreError> {
        let reg_key = format!("{}{}", DEVICE_REGISTRY_PREFIX, peer_id);
        if let Some(data) = self
            .backend
            .get(reg_key.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?
        {
            let devices: Vec<String> =
                serde_json::from_slice(&data).map_err(|_| IronCoreError::Internal)?;
            Ok(devices)
        } else {
            Ok(Vec::new())
        }
    }

    /// Remove a device ID from the registry for a peer.
    fn unregister_device_from_registry(
        &self,
        peer_id: &str,
        device_id: &str,
    ) -> Result<(), IronCoreError> {
        let mut devices = self.get_known_devices(peer_id)?;
        devices.retain(|d| d != device_id);
        let reg_key = format!("{}{}", DEVICE_REGISTRY_PREFIX, peer_id);
        if devices.is_empty() {
            self.backend
                .remove(reg_key.as_bytes())
                .map_err(|_| IronCoreError::StorageError)?;
        } else {
            let encoded = serde_json::to_vec(&devices).map_err(|_| IronCoreError::Internal)?;
            self.backend
                .put(reg_key.as_bytes(), &encoded)
                .map_err(|_| IronCoreError::StorageError)?;
        }
        Ok(())
    }

    /// Register multiple device IDs for a peer at once.
    /// Returns the number of newly registered device IDs.
    pub fn register_device_ids(
        &self,
        peer_id: &str,
        device_ids: &[String],
    ) -> Result<usize, IronCoreError> {
        let mut registered = 0;
        for device_id in device_ids {
            if self.register_device_id(peer_id, device_id)? {
                registered += 1;
            }
        }
        Ok(registered)
    }

    /// Check if a specific device of a peer is blocked.
    /// This checks both the device-specific block and the peer-level block.
    pub fn is_device_blocked(&self, peer_id: &str, device_id: &str) -> Result<bool, IronCoreError> {
        self.is_blocked(peer_id, Some(device_id))
    }
}

fn current_timestamp() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
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
        assert!(list.len() >= 2);
    }

    // -----------------------------------------------------------------------
    // Multi-device blocking tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_block_peer_auto_blocks_registered_devices() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-multi-dev";
        let device_a = "device-a";
        let device_b = "device-b";

        // Register devices before blocking
        manager.register_device_id(peer_id, device_a).unwrap();
        manager.register_device_id(peer_id, device_b).unwrap();

        // Block the peer (no device_id = peer-level block)
        manager
            .block(BlockedIdentity::new(peer_id.to_string()))
            .unwrap();

        // Peer-level block
        assert!(manager.is_blocked(peer_id, None).unwrap());
        // Device-specific blocks auto-created
        assert!(manager.is_blocked(peer_id, Some(device_a)).unwrap());
        assert!(manager.is_blocked(peer_id, Some(device_b)).unwrap());
    }

    #[test]
    fn test_register_device_auto_blocks_if_peer_blocked() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-auto-block";
        let device_first = "device-first";
        let device_later = "device-later";

        // Block the peer first
        manager
            .block(BlockedIdentity::new(peer_id.to_string()))
            .unwrap();

        // Register first device (should auto-block)
        manager.register_device_id(peer_id, device_first).unwrap();
        assert!(manager.is_blocked(peer_id, Some(device_first)).unwrap());

        // Register another device later (should also auto-block)
        manager.register_device_id(peer_id, device_later).unwrap();
        assert!(manager.is_blocked(peer_id, Some(device_later)).unwrap());
    }

    #[test]
    fn test_register_device_no_auto_block_if_peer_not_blocked() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-not-blocked";
        let device_id = "device-x";

        // Register device without blocking the peer
        manager.register_device_id(peer_id, device_id).unwrap();

        // Device is in registry but NOT blocked
        assert!(!manager.is_blocked(peer_id, Some(device_id)).unwrap());
        assert!(!manager.is_blocked(peer_id, None).unwrap());
    }

    #[test]
    fn test_register_device_idempotent() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-idem";
        let device_id = "device-idem";

        let first = manager.register_device_id(peer_id, device_id).unwrap();
        let second = manager.register_device_id(peer_id, device_id).unwrap();

        assert!(first); // Newly registered
        assert!(!second); // Already registered
    }

    #[test]
    fn test_unblock_peer_removes_all_device_blocks() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-unblock-all";
        let device_a = "device-a";
        let device_b = "device-b";

        // Register devices and block peer
        manager.register_device_id(peer_id, device_a).unwrap();
        manager.register_device_id(peer_id, device_b).unwrap();
        manager
            .block(BlockedIdentity::new(peer_id.to_string()))
            .unwrap();

        // Verify blocked
        assert!(manager.is_blocked(peer_id, None).unwrap());
        assert!(manager.is_blocked(peer_id, Some(device_a)).unwrap());
        assert!(manager.is_blocked(peer_id, Some(device_b)).unwrap());

        // Unblock the peer (no device_id = full unblock)
        manager.unblock(peer_id.to_string(), None).unwrap();

        // Everything unblocked
        assert!(!manager.is_blocked(peer_id, None).unwrap());
        assert!(!manager.is_blocked(peer_id, Some(device_a)).unwrap());
        assert!(!manager.is_blocked(peer_id, Some(device_b)).unwrap());

        // Device registry cleared
        assert!(manager.get_known_devices(peer_id).unwrap().is_empty());
    }

    #[test]
    fn test_unblock_single_device_when_no_peer_block() {
        // When only a device-specific block exists (no peer-level block),
        // unblocking that device should make the device unblocked.
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-dev-only-unblock";
        let device_a = "device-a";
        let device_b = "device-b";

        // Register devices
        manager.register_device_id(peer_id, device_a).unwrap();
        manager.register_device_id(peer_id, device_b).unwrap();

        // Block only device_a (not the whole peer)
        manager
            .block(BlockedIdentity::new(peer_id.to_string()).with_device_id(device_a.to_string()))
            .unwrap();

        // device_a is blocked, device_b and peer are NOT blocked
        assert!(manager.is_blocked(peer_id, Some(device_a)).unwrap());
        assert!(!manager.is_blocked(peer_id, Some(device_b)).unwrap());
        assert!(!manager.is_blocked(peer_id, None).unwrap());

        // Unblock device_a
        manager
            .unblock(peer_id.to_string(), Some(device_a.to_string()))
            .unwrap();

        // Now device_a is not blocked either
        assert!(!manager.is_blocked(peer_id, Some(device_a)).unwrap());
        assert!(!manager.is_blocked(peer_id, None).unwrap());

        // device_b was never blocked
        assert!(!manager.is_blocked(peer_id, Some(device_b)).unwrap());
    }

    #[test]
    fn test_peer_level_block_overrides_device_unblock() {
        // When a peer-level block exists, unblocking a specific device
        // does NOT make the device accessible because the peer-level block
        // covers all devices. To selectively unblock a device, the caller
        // must first unblock the peer entirely, then block individual devices.
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-full-block";
        let device_a = "device-a";
        let device_b = "device-b";

        manager.register_device_id(peer_id, device_a).unwrap();
        manager.register_device_id(peer_id, device_b).unwrap();
        manager
            .block(BlockedIdentity::new(peer_id.to_string()))
            .unwrap();

        // Attempt to unblock just device_a while peer-level block exists
        manager
            .unblock(peer_id.to_string(), Some(device_a.to_string()))
            .unwrap();

        // Peer-level block still active: all devices remain blocked
        assert!(manager.is_blocked(peer_id, None).unwrap());
        assert!(manager.is_blocked(peer_id, Some(device_a)).unwrap());
        assert!(manager.is_blocked(peer_id, Some(device_b)).unwrap());
    }

    #[test]
    fn test_block_and_delete_propagates_to_devices() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-del";
        let device_id = "device-del";

        manager.register_device_id(peer_id, device_id).unwrap();
        manager
            .block_and_delete(peer_id.to_string(), Some("harassment".to_string()))
            .unwrap();

        assert!(manager.is_blocked_and_deleted(peer_id).unwrap());
        assert!(manager.is_blocked(peer_id, Some(device_id)).unwrap());
    }

    #[test]
    fn test_blocked_only_peer_ids_includes_devices() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        manager
            .block(BlockedIdentity::new("peer-1".to_string()))
            .unwrap();
        manager
            .block(BlockedIdentity::new("peer-2".to_string()))
            .unwrap();

        let peer_ids = manager.blocked_only_peer_ids().unwrap();
        assert!(peer_ids.contains("peer-1"));
        assert!(peer_ids.contains("peer-2"));
    }

    #[test]
    fn test_multi_device_blocking_scenario() {
        // Simulate a real scenario: user blocks a contact who has 3 devices
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "12D3KooWMultiDevice";
        let phone = "device-phone";
        let laptop = "device-laptop";
        let tablet = "device-tablet";

        // Step 1: Register known devices for the contact
        manager.register_device_id(peer_id, phone).unwrap();
        manager.register_device_id(peer_id, laptop).unwrap();
        // tablet not registered yet (will be discovered later)

        // Step 2: User blocks the contact
        manager
            .block(BlockedIdentity::new(peer_id.to_string()).with_reason("spam".to_string()))
            .unwrap();

        // All known devices are blocked
        assert!(manager.is_blocked(peer_id, None).unwrap());
        assert!(manager.is_blocked(peer_id, Some(phone)).unwrap());
        assert!(manager.is_blocked(peer_id, Some(laptop)).unwrap());

        // Step 3: New device discovered later - auto-blocked
        manager.register_device_id(peer_id, tablet).unwrap();
        assert!(manager.is_blocked(peer_id, Some(tablet)).unwrap());

        // Step 4: User unblocks the contact - all devices cleared
        manager.unblock(peer_id.to_string(), None).unwrap();
        assert!(!manager.is_blocked(peer_id, None).unwrap());
        assert!(!manager.is_blocked(peer_id, Some(phone)).unwrap());
        assert!(!manager.is_blocked(peer_id, Some(laptop)).unwrap());
        assert!(!manager.is_blocked(peer_id, Some(tablet)).unwrap());
    }

    #[test]
    fn test_device_specific_block_does_not_block_other_devices() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-dev-only";
        let device_a = "device-a";
        let device_b = "device-b";

        manager.register_device_id(peer_id, device_a).unwrap();
        manager.register_device_id(peer_id, device_b).unwrap();

        // Block only device_a (not the whole peer)
        manager
            .block(BlockedIdentity::new(peer_id.to_string()).with_device_id(device_a.to_string()))
            .unwrap();

        // device_a is blocked, device_b and peer are NOT blocked
        assert!(manager.is_blocked(peer_id, Some(device_a)).unwrap());
        assert!(!manager.is_blocked(peer_id, Some(device_b)).unwrap());
        assert!(!manager.is_blocked(peer_id, None).unwrap());
    }

    #[test]
    fn test_get_known_devices() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-devices";
        assert!(manager.get_known_devices(peer_id).unwrap().is_empty());

        manager.register_device_id(peer_id, "d1").unwrap();
        manager.register_device_id(peer_id, "d2").unwrap();

        let devices = manager.get_known_devices(peer_id).unwrap();
        assert_eq!(devices.len(), 2);
        assert!(devices.contains(&"d1".to_string()));
        assert!(devices.contains(&"d2".to_string()));
    }

    #[test]
    fn test_is_device_blocked() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-dib";
        manager.register_device_id(peer_id, "d1").unwrap();
        manager
            .block(BlockedIdentity::new(peer_id.to_string()))
            .unwrap();

        assert!(manager.is_device_blocked(peer_id, "d1").unwrap());
        // Unknown device also blocked by peer-level block
        assert!(manager.is_device_blocked(peer_id, "unknown-dev").unwrap());
    }
}
