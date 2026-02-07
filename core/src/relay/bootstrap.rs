//! Bootstrap Protocol — helps new nodes join the network

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Bootstrap error types
#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error("Invalid invite: {0}")]
    InvalidInvite(String),
    #[error("Invite expired")]
    InviteExpired,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Verification failed")]
    VerificationFailed,
    #[error("No seed peers configured")]
    NoSeedPeers,
}

/// Bootstrap method for joining the network
#[derive(Debug, Clone)]
pub enum BootstrapMethod {
    /// Join via QR code (scanned invite)
    QrCode,
    /// Join via invite link (web or deep link)
    InviteLink,
    /// Discover via Bluetooth LE advertisements
    BleDiscovery,
    /// Connect to hardcoded seed peers
    SeedPeers,
    /// Manually provide a peer address
    ManualAddress,
}

/// A seed peer for network bootstrapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedPeer {
    /// Peer address (IP:port or DNS name:port)
    pub address: String,
    /// Peer's public key (Ed25519, hex encoded)
    pub public_key: String,
    /// Human-readable name
    pub name: String,
}

impl SeedPeer {
    /// Create a new seed peer
    pub fn new(address: String, public_key: String, name: String) -> Self {
        Self {
            address,
            public_key,
            name,
        }
    }
}

/// Payload data in an invite (for QR codes, links, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitePayload {
    /// Inviter's peer ID (Blake3 hash of public key)
    pub peer_id: String,
    /// Inviter's addresses
    pub addresses: Vec<String>,
    /// Group key hint (optional, for groups)
    pub group_key_hint: Option<String>,
    /// When the invite was created (Unix timestamp)
    pub created_at: u64,
    /// When the invite expires (Unix timestamp)
    pub expires_at: u64,
    /// Inviter's Ed25519 public key (for signature verification)
    pub inviter_public_key: Vec<u8>,
}

impl InvitePayload {
    /// Create a new invite payload
    pub fn new(
        peer_id: String,
        addresses: Vec<String>,
        inviter_public_key: Vec<u8>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            peer_id,
            addresses,
            group_key_hint: None,
            created_at: now,
            expires_at: now + 24 * 3600, // Valid for 24 hours
            inviter_public_key,
        }
    }

    /// Set group key hint
    pub fn with_group_key(mut self, hint: String) -> Self {
        self.group_key_hint = Some(hint);
        self
    }

    /// Set expiry duration in seconds
    pub fn with_expiry(mut self, duration_secs: u64) -> Self {
        self.expires_at = self.created_at + duration_secs;
        self
    }

    /// Check if invite is still valid
    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        now < self.expires_at
    }

    /// Serialize to bytes (for QR codes)
    pub fn to_bytes(&self) -> Result<Vec<u8>, BootstrapError> {
        bincode::serialize(self)
            .map_err(|e| BootstrapError::SerializationError(e.to_string()))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BootstrapError> {
        bincode::deserialize(bytes)
            .map_err(|e| BootstrapError::SerializationError(e.to_string()))
    }

    /// Serialize to JSON (for links)
    pub fn to_json(&self) -> Result<String, BootstrapError> {
        serde_json::to_string(self)
            .map_err(|e| BootstrapError::SerializationError(e.to_string()))
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, BootstrapError> {
        serde_json::from_str(json)
            .map_err(|e| BootstrapError::SerializationError(e.to_string()))
    }
}

/// Bootstrap manager for joining the network
pub struct BootstrapManager {
    /// Our peer ID
    our_peer_id: String,
    /// Our public key (for creating invites)
    our_public_key: Vec<u8>,
    /// Our listening addresses
    our_addresses: Vec<String>,
    /// Known seed peers
    seed_peers: Vec<SeedPeer>,
}

impl BootstrapManager {
    /// Create a new bootstrap manager
    pub fn new(
        peer_id: String,
        public_key: Vec<u8>,
        addresses: Vec<String>,
    ) -> Self {
        Self {
            our_peer_id: peer_id,
            our_public_key: public_key,
            our_addresses: addresses,
            seed_peers: Vec::new(),
        }
    }

    /// Add seed peers
    pub fn with_seed_peers(mut self, seed_peers: Vec<SeedPeer>) -> Self {
        self.seed_peers = seed_peers;
        self
    }

    /// Get seed peers to connect to
    pub fn get_seed_peers(&self) -> Result<Vec<SeedPeer>, BootstrapError> {
        if self.seed_peers.is_empty() {
            return Err(BootstrapError::NoSeedPeers);
        }
        Ok(self.seed_peers.clone())
    }

    /// Generate an invite for others to join
    pub fn generate_invite(&self) -> Result<InvitePayload, BootstrapError> {
        Ok(InvitePayload::new(
            self.our_peer_id.clone(),
            self.our_addresses.clone(),
            self.our_public_key.clone(),
        ))
    }

    /// Generate an invite as QR code data
    pub fn generate_qr_data(&self) -> Result<Vec<u8>, BootstrapError> {
        let invite = self.generate_invite()?;
        invite.to_bytes()
    }

    /// Accept an invite and get the inviter's info
    pub fn accept_invite(&self, invite_payload: InvitePayload) -> Result<InvitePayload, BootstrapError> {
        // Verify invite is still valid
        if !invite_payload.is_valid() {
            return Err(BootstrapError::InviteExpired);
        }

        Ok(invite_payload)
    }

    /// Parse QR code data into an invite
    pub fn parse_qr_data(&self, data: &[u8]) -> Result<InvitePayload, BootstrapError> {
        let invite = InvitePayload::from_bytes(data)?;
        self.accept_invite(invite)
    }

    /// Get addresses for a given peer (checks seed peers and other sources)
    pub fn get_peer_addresses(&self, peer_id: &str) -> Option<Vec<String>> {
        // Check seed peers
        for seed in &self.seed_peers {
            if seed.address.contains(peer_id) {
                return Some(vec![seed.address.clone()]);
            }
        }
        None
    }

    /// Set the list of known addresses for this node
    pub fn set_addresses(&mut self, addresses: Vec<String>) {
        self.our_addresses = addresses;
    }

    /// Get our current addresses
    pub fn get_addresses(&self) -> Vec<String> {
        self.our_addresses.clone()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_seed_peer() -> SeedPeer {
        SeedPeer::new(
            "127.0.0.1:8080".to_string(),
            "abcdef0123456789".to_string(),
            "seed-node-1".to_string(),
        )
    }

    fn test_bootstrap_manager() -> BootstrapManager {
        BootstrapManager::new(
            "peer1".to_string(),
            vec![1, 2, 3, 4, 5],
            vec!["127.0.0.1:9000".to_string()],
        )
    }

    #[test]
    fn test_seed_peer_creation() {
        let peer = test_seed_peer();
        assert_eq!(peer.address, "127.0.0.1:8080");
        assert_eq!(peer.name, "seed-node-1");
    }

    #[test]
    fn test_invite_payload_creation() {
        let payload = InvitePayload::new(
            "peer1".to_string(),
            vec!["127.0.0.1:8080".to_string()],
            vec![1, 2, 3, 4, 5],
        );

        assert_eq!(payload.peer_id, "peer1");
        assert_eq!(payload.addresses.len(), 1);
        assert!(payload.is_valid());
    }

    #[test]
    fn test_invite_payload_with_group_key() {
        let payload = InvitePayload::new(
            "peer1".to_string(),
            vec!["127.0.0.1:8080".to_string()],
            vec![1, 2, 3, 4, 5],
        )
        .with_group_key("group_key_123".to_string());

        assert!(payload.group_key_hint.is_some());
        assert_eq!(payload.group_key_hint.unwrap(), "group_key_123");
    }

    #[test]
    fn test_invite_payload_serialization() {
        let payload = InvitePayload::new(
            "peer1".to_string(),
            vec!["127.0.0.1:8080".to_string()],
            vec![1, 2, 3, 4, 5],
        );

        let bytes = payload.to_bytes().expect("Failed to serialize");
        let restored = InvitePayload::from_bytes(&bytes).expect("Failed to deserialize");

        assert_eq!(payload.peer_id, restored.peer_id);
        assert_eq!(payload.addresses, restored.addresses);
    }

    #[test]
    fn test_invite_payload_json_serialization() {
        let payload = InvitePayload::new(
            "peer1".to_string(),
            vec!["127.0.0.1:8080".to_string()],
            vec![1, 2, 3, 4, 5],
        );

        let json = payload.to_json().expect("Failed to serialize");
        let restored = InvitePayload::from_json(&json).expect("Failed to deserialize");

        assert_eq!(payload.peer_id, restored.peer_id);
    }

    #[test]
    fn test_invite_payload_expiry() {
        let payload = InvitePayload::new(
            "peer1".to_string(),
            vec!["127.0.0.1:8080".to_string()],
            vec![1, 2, 3, 4, 5],
        )
        .with_expiry(0);

        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(!payload.is_valid());
    }

    #[test]
    fn test_bootstrap_manager_creation() {
        let manager = test_bootstrap_manager();
        assert_eq!(manager.our_peer_id, "peer1");
        assert_eq!(manager.our_addresses.len(), 1);
    }

    #[test]
    fn test_bootstrap_manager_with_seed_peers() {
        let seed = test_seed_peer();
        let manager = BootstrapManager::new(
            "peer1".to_string(),
            vec![1, 2, 3],
            vec!["127.0.0.1:9000".to_string()],
        )
        .with_seed_peers(vec![seed]);

        assert_eq!(manager.seed_peers.len(), 1);
    }

    #[test]
    fn test_get_seed_peers() {
        let seed = test_seed_peer();
        let manager = test_bootstrap_manager().with_seed_peers(vec![seed.clone()]);

        let peers = manager.get_seed_peers().expect("Failed to get seed peers");
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].address, seed.address);
    }

    #[test]
    fn test_get_seed_peers_empty() {
        let manager = test_bootstrap_manager();

        let result = manager.get_seed_peers();
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_invite() {
        let manager = test_bootstrap_manager();
        let invite = manager.generate_invite().expect("Failed to generate invite");

        assert_eq!(invite.peer_id, "peer1");
        assert_eq!(invite.addresses.len(), 1);
        assert!(invite.is_valid());
    }

    #[test]
    fn test_generate_qr_data() {
        let manager = test_bootstrap_manager();
        let data = manager.generate_qr_data().expect("Failed to generate QR data");

        assert!(!data.is_empty());

        // Should be deserializable
        let invite = InvitePayload::from_bytes(&data).expect("Failed to parse QR data");
        assert_eq!(invite.peer_id, "peer1");
    }

    #[test]
    fn test_parse_qr_data() {
        let manager = test_bootstrap_manager();
        let data = manager.generate_qr_data().expect("Failed to generate QR data");

        let invite = manager.parse_qr_data(&data).expect("Failed to parse QR data");
        assert_eq!(invite.peer_id, "peer1");
    }

    #[test]
    fn test_accept_invite_valid() {
        let manager = test_bootstrap_manager();
        let payload = InvitePayload::new(
            "peer2".to_string(),
            vec!["127.0.0.1:8081".to_string()],
            vec![5, 4, 3, 2, 1],
        );

        let result = manager.accept_invite(payload);
        assert!(result.is_ok());
    }

    #[test]
    fn test_accept_invite_expired() {
        let manager = test_bootstrap_manager();
        let payload = InvitePayload::new(
            "peer2".to_string(),
            vec!["127.0.0.1:8081".to_string()],
            vec![5, 4, 3, 2, 1],
        )
        .with_expiry(0);

        std::thread::sleep(std::time::Duration::from_millis(10));

        let result = manager.accept_invite(payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_addresses() {
        let mut manager = test_bootstrap_manager();
        let new_addresses = vec!["127.0.0.1:9001".to_string(), "127.0.0.1:9002".to_string()];

        manager.set_addresses(new_addresses.clone());

        assert_eq!(manager.get_addresses(), new_addresses);
    }

    #[test]
    fn test_get_peer_addresses() {
        let seed = SeedPeer::new(
            "relay1:8080".to_string(),
            "key123".to_string(),
            "relay-1".to_string(),
        );
        let manager = test_bootstrap_manager().with_seed_peers(vec![seed]);

        let addresses = manager.get_peer_addresses("relay1");
        assert!(addresses.is_some());
    }

    #[test]
    fn test_bootstrap_method_enum() {
        let methods = vec![
            BootstrapMethod::QrCode,
            BootstrapMethod::InviteLink,
            BootstrapMethod::BleDiscovery,
            BootstrapMethod::SeedPeers,
            BootstrapMethod::ManualAddress,
        ];

        assert_eq!(methods.len(), 5);
    }
}
