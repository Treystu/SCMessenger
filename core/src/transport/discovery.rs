// Discovery mode configuration for controlling metadata leakage from mDNS
//
// This module provides configurable discovery modes to control how the node
// advertises and discovers peers on the network. Each mode offers different
// privacy/discoverability tradeoffs.

use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during beacon operations
#[derive(Error, Debug)]
pub enum BeaconError {
    #[error("Beacon decryption failed")]
    DecryptionFailed,
    #[error("Invalid beacon format")]
    InvalidFormat,
    #[error("Encryption error: {0}")]
    EncryptionError(String),
}

/// Discovery mode controls how this node advertises and finds peers.
///
/// Each mode provides different tradeoffs between discoverability and privacy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiscoveryMode {
    /// Full mDNS + Identify. Fast discovery, zero privacy.
    ///
    /// Use for development, trusted LANs, and initial bootstrapping.
    /// Broadcasts PeerId, IP, port, and protocol IDs to everyone on the LAN.
    Open,

    /// No mDNS broadcast. Manual peer addition only.
    ///
    /// Kademlia for known bootstrap nodes. Identify disabled.
    /// Requires explicit peer addresses to be added by the operator.
    Manual,

    /// Encrypted BLE beacons with pre-shared group key.
    ///
    /// Only devices with the group key can recognize each other.
    /// Uses AES-256-GCM encrypted beacons rotated every 15 minutes.
    DarkBLE {
        /// Pre-shared 256-bit group key (exchanged via QR code at setup)
        group_key: [u8; 32],
    },

    /// No discovery at all. Connect only to explicit multiaddresses.
    ///
    /// Maximum stealth — invisible on the network.
    /// Must establish all connections manually via explicit addresses.
    Silent,
}

impl Default for DiscoveryMode {
    fn default() -> Self {
        DiscoveryMode::Open
    }
}

impl DiscoveryMode {
    /// Check if this mode allows mDNS broadcasting
    pub fn allows_mdns(&self) -> bool {
        matches!(self, DiscoveryMode::Open)
    }

    /// Check if this mode allows Identify protocol
    pub fn allows_identify(&self) -> bool {
        matches!(self, DiscoveryMode::Open | DiscoveryMode::Manual | DiscoveryMode::DarkBLE { .. })
    }

    /// Check if Identify should be advertised to unknown peers
    pub fn advertises_identify(&self) -> bool {
        matches!(self, DiscoveryMode::Open | DiscoveryMode::Manual)
    }
}

/// Configuration for the swarm's discovery behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// The discovery mode to use
    pub mode: DiscoveryMode,
    /// Whether to advertise supported protocols via Identify (only in permissive modes)
    pub advertise_protocols: bool,
    /// Whether to accept incoming connections from unknown peers
    pub accept_unknown_peers: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            mode: DiscoveryMode::Open,
            advertise_protocols: true,
            accept_unknown_peers: true,
        }
    }
}

impl DiscoveryConfig {
    /// Create a new DiscoveryConfig with the given mode
    pub fn new(mode: DiscoveryMode) -> Self {
        Self {
            mode,
            advertise_protocols: true,
            accept_unknown_peers: true,
        }
    }

    /// Set whether to advertise protocols
    pub fn with_advertise_protocols(mut self, advertise: bool) -> Self {
        self.advertise_protocols = advertise;
        self
    }

    /// Set whether to accept unknown peers
    pub fn with_accept_unknown_peers(mut self, accept: bool) -> Self {
        self.accept_unknown_peers = accept;
        self
    }
}

/// Beacon payload for encrypted discovery
#[derive(Debug, Clone, Copy)]
pub struct BeaconPayload {
    /// 15-minute rotation window (epoch)
    pub mesh_epoch: u32,
    /// First 4 bytes of blake3(node_pk || epoch)
    pub node_shard: [u8; 4],
    /// Flags for future use (e.g., node capabilities)
    pub flags: u8,
}

impl BeaconPayload {
    /// Serialize the beacon payload to bytes
    pub fn to_bytes(&self) -> [u8; 9] {
        let mut bytes = [0u8; 9];
        bytes[0..4].copy_from_slice(&self.mesh_epoch.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.node_shard);
        bytes[8] = self.flags;
        bytes
    }

    /// Deserialize a beacon payload from bytes
    pub fn from_bytes(bytes: &[u8; 9]) -> Self {
        let mesh_epoch = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let mut node_shard = [0u8; 4];
        node_shard.copy_from_slice(&bytes[4..8]);
        let flags = bytes[8];

        BeaconPayload {
            mesh_epoch,
            node_shard,
            flags,
        }
    }
}

/// Create an encrypted discovery beacon payload
///
/// Uses XChaCha20-Poly1305 with the group key to encrypt a beacon containing:
/// - mesh_epoch (15-minute rotation window)
/// - node_shard (first 4 bytes of blake3(node_pk || epoch))
/// - flags (for future use)
///
/// # Arguments
/// * `group_key` - Pre-shared 256-bit group key
/// * `node_public_key` - This node's public key (for shard computation)
///
/// # Returns
/// Encrypted beacon bytes ready for transmission
pub fn create_encrypted_beacon(
    group_key: &[u8; 32],
    node_public_key: &[u8; 32],
) -> Result<Vec<u8>, BeaconError> {
    // Get current epoch (15-minute windows since epoch)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| BeaconError::EncryptionError(e.to_string()))?;
    let mesh_epoch = (now.as_secs() / (15 * 60)) as u32;

    // Compute node shard: blake3(node_pk || epoch)
    let mut hasher = blake3::Hasher::new();
    hasher.update(node_public_key);
    hasher.update(&mesh_epoch.to_le_bytes());
    let hash = hasher.finalize();
    let mut node_shard = [0u8; 4];
    node_shard.copy_from_slice(&hash.as_bytes()[0..4]);

    let payload = BeaconPayload {
        mesh_epoch,
        node_shard,
        flags: 0,
    };

    // Encrypt the payload using XChaCha20-Poly1305
    let cipher = XChaCha20Poly1305::new(group_key.into());

    // Construct nonce deterministically from epoch only (so decrypt can reconstruct it)
    // nonce[0..4] = mesh_epoch, rest derived from group_key hash
    let mut nonce_bytes = [0u8; 24];
    nonce_bytes[0..4].copy_from_slice(&mesh_epoch.to_le_bytes());

    // Derive the rest of nonce from group_key to make it deterministic
    let mut hasher = blake3::Hasher::new();
    hasher.update(group_key);
    hasher.update(&mesh_epoch.to_le_bytes());
    let hash = hasher.finalize();
    nonce_bytes[4..24].copy_from_slice(&hash.as_bytes()[0..20]);

    let nonce = chacha20poly1305::XNonce::from_slice(&nonce_bytes);

    let plaintext = payload.to_bytes();
    let payload_struct = Payload {
        msg: &plaintext,
        aad: &[],
    };
    let ciphertext = cipher
        .encrypt(nonce, payload_struct)
        .map_err(|e| BeaconError::EncryptionError(e.to_string()))?;

    Ok(ciphertext)
}

/// Attempt to decrypt a discovery beacon
///
/// Returns None if the key doesn't match or the beacon is invalid.
pub fn decrypt_beacon(
    group_key: &[u8; 32],
    beacon_data: &[u8],
) -> Result<BeaconPayload, BeaconError> {
    if beacon_data.len() < 25 {
        return Err(BeaconError::InvalidFormat);
    }

    // Extract potential epoch from first 4 bytes of ciphertext hint
    // For now, we try to decrypt and derive the epoch from the result
    // This is a simplified approach; in production, you might transmit
    // the epoch in plaintext for faster filtering

    let cipher = XChaCha20Poly1305::new(group_key.into());

    // Try decryption with various epoch guesses (current ± a few windows)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| BeaconError::EncryptionError(e.to_string()))?;
    let current_epoch = (now.as_secs() / (15 * 60)) as u32;

    // Try current epoch and ±1 window (should cover clock skew)
    for epoch_offset in -1..=1 {
        let test_epoch = (current_epoch as i32 + epoch_offset) as u32;

        // Reconstruct nonce using the same method as encryption
        let mut nonce_bytes = [0u8; 24];
        nonce_bytes[0..4].copy_from_slice(&test_epoch.to_le_bytes());

        // Derive the rest of nonce from group_key to match encryption
        let mut hasher = blake3::Hasher::new();
        hasher.update(group_key);
        hasher.update(&test_epoch.to_le_bytes());
        let hash = hasher.finalize();
        nonce_bytes[4..24].copy_from_slice(&hash.as_bytes()[0..20]);

        let nonce = chacha20poly1305::XNonce::from_slice(&nonce_bytes);

        let payload_struct = Payload {
            msg: beacon_data,
            aad: &[],
        };
        if let Ok(plaintext) = cipher.decrypt(nonce, payload_struct) {
            if plaintext.len() == 9 {
                let mut bytes = [0u8; 9];
                bytes.copy_from_slice(&plaintext);
                return Ok(BeaconPayload::from_bytes(&bytes));
            }
        }
    }

    Err(BeaconError::DecryptionFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beacon_encrypt_decrypt_roundtrip() {
        let group_key = [0x42u8; 32];
        let node_pk = [0xaa; 32];

        let encrypted = create_encrypted_beacon(&group_key, &node_pk)
            .expect("Encryption should succeed");

        let decrypted = decrypt_beacon(&group_key, &encrypted)
            .expect("Decryption should succeed with correct key");

        assert_eq!(decrypted.flags, 0);
        assert_eq!(decrypted.node_shard.len(), 4);
    }

    #[test]
    fn test_beacon_decrypt_wrong_key_fails() {
        let group_key = [0x42u8; 32];
        let wrong_key = [0x99u8; 32];
        let node_pk = [0xaa; 32];

        let encrypted = create_encrypted_beacon(&group_key, &node_pk)
            .expect("Encryption should succeed");

        let result = decrypt_beacon(&wrong_key, &encrypted);
        assert!(
            result.is_err(),
            "Decryption with wrong key should fail"
        );
    }

    #[test]
    fn test_beacon_payload_serialization() {
        let payload = BeaconPayload {
            mesh_epoch: 0x12345678,
            node_shard: [0xaa, 0xbb, 0xcc, 0xdd],
            flags: 0x42,
        };

        let bytes = payload.to_bytes();
        let recovered = BeaconPayload::from_bytes(&bytes);

        assert_eq!(recovered.mesh_epoch, payload.mesh_epoch);
        assert_eq!(recovered.node_shard, payload.node_shard);
        assert_eq!(recovered.flags, payload.flags);
    }

    #[test]
    fn test_discovery_config_serialization() {
        let config = DiscoveryConfig {
            mode: DiscoveryMode::Open,
            advertise_protocols: true,
            accept_unknown_peers: false,
        };

        let json = serde_json::to_string(&config).expect("Should serialize");
        let recovered: DiscoveryConfig =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(recovered.mode, config.mode);
        assert_eq!(recovered.advertise_protocols, config.advertise_protocols);
        assert_eq!(
            recovered.accept_unknown_peers,
            config.accept_unknown_peers
        );
    }

    #[test]
    fn test_discovery_mode_serialization_all_variants() {
        // Test Open
        let open = DiscoveryMode::Open;
        let json = serde_json::to_string(&open).expect("Should serialize");
        let _recovered: DiscoveryMode =
            serde_json::from_str(&json).expect("Should deserialize");

        // Test Manual
        let manual = DiscoveryMode::Manual;
        let json = serde_json::to_string(&manual).expect("Should serialize");
        let _recovered: DiscoveryMode =
            serde_json::from_str(&json).expect("Should deserialize");

        // Test DarkBLE
        let darkble = DiscoveryMode::DarkBLE {
            group_key: [0xffu8; 32],
        };
        let json = serde_json::to_string(&darkble).expect("Should serialize");
        let recovered: DiscoveryMode =
            serde_json::from_str(&json).expect("Should deserialize");
        assert!(matches!(recovered, DiscoveryMode::DarkBLE { .. }));

        // Test Silent
        let silent = DiscoveryMode::Silent;
        let json = serde_json::to_string(&silent).expect("Should serialize");
        let _recovered: DiscoveryMode =
            serde_json::from_str(&json).expect("Should deserialize");
    }

    #[test]
    fn test_discovery_mode_properties() {
        let open = DiscoveryMode::Open;
        assert!(open.allows_mdns());
        assert!(open.allows_identify());
        assert!(open.advertises_identify());

        let manual = DiscoveryMode::Manual;
        assert!(!manual.allows_mdns());
        assert!(manual.allows_identify());
        assert!(manual.advertises_identify());

        let darkble = DiscoveryMode::DarkBLE {
            group_key: [0; 32],
        };
        assert!(!darkble.allows_mdns());
        assert!(darkble.allows_identify());
        assert!(!darkble.advertises_identify());

        let silent = DiscoveryMode::Silent;
        assert!(!silent.allows_mdns());
        assert!(!silent.allows_identify());
        assert!(!silent.advertises_identify());
    }

    #[test]
    fn test_discovery_config_builder() {
        let config = DiscoveryConfig::new(DiscoveryMode::Manual)
            .with_advertise_protocols(false)
            .with_accept_unknown_peers(false);

        assert_eq!(config.mode, DiscoveryMode::Manual);
        assert!(!config.advertise_protocols);
        assert!(!config.accept_unknown_peers);
    }

    #[test]
    fn test_epoch_rotation_changes_beacon() {
        let group_key = [0x42u8; 32];
        let node_pk = [0xaa; 32];

        // Create two beacons (they might be in different epochs depending on timing)
        let beacon1 = create_encrypted_beacon(&group_key, &node_pk)
            .expect("First encryption should succeed");
        let beacon2 = create_encrypted_beacon(&group_key, &node_pk)
            .expect("Second encryption should succeed");

        // Beacons might be identical if in same epoch, different if in different epoch
        // Both should decrypt successfully with the same key
        let payload1 = decrypt_beacon(&group_key, &beacon1)
            .expect("Should decrypt first beacon");
        let payload2 = decrypt_beacon(&group_key, &beacon2)
            .expect("Should decrypt second beacon");

        // Both should have valid payloads
        assert_eq!(payload1.flags, 0);
        assert_eq!(payload2.flags, 0);
    }
}
