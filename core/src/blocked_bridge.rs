// Blocked identities bridge for mobile platforms (Android/iOS)
//
// Exposes BlockedManager through UniFFI for cross-platform blocking functionality.

use crate::store::blocked::{
    BlockedIdentity as CoreBlockedIdentity, BlockedManager as CoreBlockedManager,
};
use crate::IronCoreError;
use std::sync::Arc;

/// Mobile-friendly BlockedIdentity wrapper for UniFFI
#[derive(Clone)]
pub struct BlockedIdentity {
    pub peer_id: String,
    pub device_id: Option<String>,
    pub blocked_at: u64,
    pub reason: Option<String>,
    pub notes: Option<String>,
    /// When true, the contact has been both blocked AND deleted (cascade purge).
    pub is_deleted: bool,
}

impl From<CoreBlockedIdentity> for BlockedIdentity {
    fn from(core: CoreBlockedIdentity) -> Self {
        Self {
            peer_id: core.peer_id,
            device_id: core.device_id,
            blocked_at: core.blocked_at,
            reason: core.reason,
            notes: core.notes,
            is_deleted: core.is_deleted,
        }
    }
}

impl From<BlockedIdentity> for CoreBlockedIdentity {
    fn from(mobile: BlockedIdentity) -> Self {
        Self {
            peer_id: mobile.peer_id,
            device_id: mobile.device_id,
            blocked_at: mobile.blocked_at,
            reason: mobile.reason,
            notes: mobile.notes,
            is_deleted: mobile.is_deleted,
        }
    }
}

/// BlockedManager for mobile platforms
pub struct BlockedManager {
    inner: CoreBlockedManager,
}

impl Default for BlockedManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockedManager {
    pub fn new() -> Self {
        // Use in-memory storage for mobile - will persist through IronCore later
        let backend = Arc::new(crate::store::backend::MemoryStorage::new());
        Self {
            inner: CoreBlockedManager::new(backend),
        }
    }

    /// Block a peer ID
    pub fn block(&self, blocked: BlockedIdentity) -> Result<(), IronCoreError> {
        self.inner.block(blocked.into())
    }

    /// Unblock a peer ID
    pub fn unblock(&self, peer_id: String, device_id: Option<String>) -> Result<(), IronCoreError> {
        self.inner.unblock(peer_id, device_id)
    }

    /// Check if a peer is blocked
    pub fn is_blocked(
        &self,
        peer_id: String,
        device_id: Option<String>,
    ) -> Result<bool, IronCoreError> {
        self.inner.is_blocked(&peer_id, device_id.as_deref())
    }

    /// Get blocked identity details
    pub fn get(
        &self,
        peer_id: String,
        device_id: Option<String>,
    ) -> Result<Option<BlockedIdentity>, IronCoreError> {
        match self.inner.get(&peer_id, device_id.as_deref())? {
            Some(core_blocked) => Ok(Some(BlockedIdentity::from(core_blocked))),
            None => Ok(None),
        }
    }

    /// List all blocked identities
    pub fn list(&self) -> Result<Vec<BlockedIdentity>, IronCoreError> {
        let core_list = self.inner.list()?;
        Ok(core_list.into_iter().map(BlockedIdentity::from).collect())
    }

    /// Get count of blocked identities
    pub fn count(&self) -> Result<u32, IronCoreError> {
        self.inner.count().map(|c| c as u32)
    }
}

// UniFFI namespace functions for builder pattern
pub fn blocked_identity_new(peer_id: String) -> BlockedIdentity {
    CoreBlockedIdentity::new(peer_id).into()
}

pub fn blocked_identity_with_device_id(
    blocked: BlockedIdentity,
    device_id: String,
) -> BlockedIdentity {
    let mut b: CoreBlockedIdentity = blocked.into();
    b.device_id = Some(device_id);
    b.into()
}

pub fn blocked_identity_with_reason(blocked: BlockedIdentity, reason: String) -> BlockedIdentity {
    let mut b: CoreBlockedIdentity = blocked.into();
    b.reason = Some(reason);
    b.into()
}

pub fn blocked_identity_with_notes(blocked: BlockedIdentity, notes: String) -> BlockedIdentity {
    let mut b: CoreBlockedIdentity = blocked.into();
    b.notes = Some(notes);
    b.into()
}
