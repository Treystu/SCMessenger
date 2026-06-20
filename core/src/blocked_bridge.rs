// Blocked identities bridge for mobile platforms (Android/iOS)
//
// Exposes BlockedManager through UniFFI for cross-platform blocking functionality.
// Includes device-ID pairing for multi-device blocking.

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

    /// Block a peer ID (also blocks all known device IDs for that peer)
    pub fn block(&self, blocked: BlockedIdentity) -> Result<(), IronCoreError> {
        self.inner.block(blocked.into())
    }

    /// Unblock a peer ID (also unblocks all device IDs if device_id is None)
    pub fn unblock(&self, peer_id: String, device_id: Option<String>) -> Result<(), IronCoreError> {
        self.inner.unblock(peer_id, device_id)
    }

    /// Check if a peer is blocked (checks device-specific then peer-level)
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

    /// List all blocked identities (peer-level and device-specific)
    pub fn list(&self) -> Result<Vec<BlockedIdentity>, IronCoreError> {
        let core_list = self.inner.list()?;
        Ok(core_list.into_iter().map(BlockedIdentity::from).collect())
    }

    /// Get count of blocked identities
    pub fn count(&self) -> Result<u32, IronCoreError> {
        self.inner.count().map(|c| c as u32)
    }

    /// Register a device ID as belonging to a peer.
    /// If the peer is blocked, the device is automatically blocked.
    pub fn register_device_id(
        &self,
        peer_id: String,
        device_id: String,
    ) -> Result<bool, IronCoreError> {
        self.inner.register_device_id(&peer_id, &device_id)
    }

    /// Get all known device IDs registered for a peer.
    pub fn get_known_devices(&self, peer_id: String) -> Result<Vec<String>, IronCoreError> {
        self.inner.get_known_devices(&peer_id)
    }

    /// Check if a specific device of a peer is blocked.
    pub fn is_device_blocked(
        &self,
        peer_id: String,
        device_id: String,
    ) -> Result<bool, IronCoreError> {
        self.inner.is_device_blocked(&peer_id, &device_id)
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

/// Compute a 60-digit safety number from two sorted public keys.
///
/// Implements standard out-of-band identity verification:
/// 1. Sort the two public keys lexically (ensures order-invariance)
/// 2. Hash their concatenation using BLAKE3 (32 bytes output)
/// 3. Split the 32 bytes into 5 groups of 12 digits (60 digits total)
/// 4. Format groups with spaces for readability
pub fn compute_safety_number(first_pubkey_hex: String, second_pubkey_hex: String) -> String {
    let mut keys = vec![first_pubkey_hex, second_pubkey_hex];
    keys.sort();

    let concat = format!("{}{}", keys[0], keys[1]);
    let hash = blake3::hash(concat.as_bytes());
    let hash_bytes = hash.as_bytes();

    // Format 32 bytes into 60 digits.
    // 5 groups of 12 digits. Each group is formed by parsing 6 bytes as u48 / u64 and formatting it.
    // Let's use 5 groups, 6 bytes per group (covers 30 bytes total).
    // Let's take the first 30 bytes of the hash:
    let mut groups = Vec::new();
    for i in 0..5 {
        let chunk = &hash_bytes[(i * 6)..(i * 6 + 6)];
        // parse 6 bytes as a u64 (big endian)
        let val = ((chunk[0] as u64) << 40)
            | ((chunk[1] as u64) << 32)
            | ((chunk[2] as u64) << 24)
            | ((chunk[3] as u64) << 16)
            | ((chunk[4] as u64) << 8)
            | (chunk[5] as u64);
        
        // Format as a 12-digit number with leading zeros (max value of 6 bytes is 281,474,976,710,655 which is 15 digits, so % 10^12)
        let formatted = format!("{:012}", val % 100_000_000_000u64);
        groups.push(formatted);
    }

    groups.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_safety_number() {
        let pub1 = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".to_string();
        let pub2 = "9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba".to_string();

        let num1 = compute_safety_number(pub1.clone(), pub2.clone());
        let num2 = compute_safety_number(pub2, pub1);

        // Verify order invariance
        assert_eq!(num1, num2);

        // Verify length and structure (5 groups of 12 digits separated by spaces -> 5 * 12 + 4 = 64 characters)
        assert_eq!(num1.len(), 64);
        let parts: Vec<&str> = num1.split(' ').collect();
        assert_eq!(parts.len(), 5);
        for part in parts {
            assert_eq!(part.len(), 12);
            assert!(part.chars().all(|c| c.is_ascii_digit()));
        }
    }
}


