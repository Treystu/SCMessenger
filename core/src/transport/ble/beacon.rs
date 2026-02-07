/// BLE Beacon abstraction for DarkBLE discovery
///
/// This module provides BLE beacon construction, parsing, and rotation logic.
/// Beacons are encrypted advertisements that rotate every epoch (default 15 minutes).
/// Only devices with the group key can decrypt and recognize beacons.

use crate::transport::discovery::{decrypt_beacon_with_period, BeaconPayload};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305,
};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// BLE Beacon service UUID (0xDF01)
pub const BLE_BEACON_SERVICE_UUID: u16 = 0xDF01;

/// Default beacon rotation period in seconds (15 minutes)
pub const DEFAULT_BEACON_ROTATION_SECS: u64 = 15 * 60;

/// Errors for beacon operations
#[derive(Error, Debug)]
pub enum BleBeaconError {
    #[error("Beacon creation failed: {0}")]
    CreationFailed(String),
    #[error("Invalid beacon format")]
    InvalidFormat,
    #[error("Decryption failed")]
    DecryptionFailed,
    #[error("System time error: {0}")]
    SystemTimeError(String),
}

/// A BLE Beacon advertisement
#[derive(Debug, Clone)]
pub struct BleBeacon {
    /// GATT service UUID
    pub service_uuid: u16,
    /// Encrypted payload (includes BeaconPayload + tag)
    pub encrypted_payload: Vec<u8>,
    /// Rotation epoch (determines when beacon was created)
    pub rotation_epoch: u32,
}

impl BleBeacon {
    /// Get the size of the beacon in bytes
    pub fn size(&self) -> usize {
        2 + self.encrypted_payload.len() // service_uuid (2) + payload
    }
}

/// Builder for constructing BLE beacons with identity hints
pub struct BeaconBuilder {
    group_key: [u8; 32],
    node_public_key: [u8; 32],
    rotation_period_secs: u64,
}

impl BeaconBuilder {
    /// Create a new beacon builder
    pub fn new(group_key: [u8; 32], node_public_key: [u8; 32]) -> Self {
        Self {
            group_key,
            node_public_key,
            rotation_period_secs: DEFAULT_BEACON_ROTATION_SECS,
        }
    }

    /// Set the beacon rotation period in seconds
    pub fn with_rotation_period(mut self, secs: u64) -> Self {
        self.rotation_period_secs = secs;
        self
    }

    /// Build a beacon for the current epoch
    pub fn build(&self) -> Result<BleBeacon, BleBeaconError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| BleBeaconError::SystemTimeError(e.to_string()))?;

        let rotation_epoch = (now.as_secs() / self.rotation_period_secs) as u32;

        // Compute node shard: blake3(node_pk || epoch)
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.node_public_key);
        hasher.update(&rotation_epoch.to_le_bytes());
        let hash = hasher.finalize();
        let mut node_shard = [0u8; 4];
        node_shard.copy_from_slice(&hash.as_bytes()[0..4]);

        let payload = BeaconPayload {
            mesh_epoch: rotation_epoch,
            node_shard,
            flags: 0,
        };

        // Encrypt the payload using XChaCha20-Poly1305
        let cipher = XChaCha20Poly1305::new(self.group_key.as_ref().into());

        // Construct nonce deterministically from epoch and group_key
        let mut nonce_bytes = [0u8; 24];
        nonce_bytes[0..4].copy_from_slice(&rotation_epoch.to_le_bytes());

        // Derive the rest of nonce from group_key
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.group_key);
        hasher.update(&rotation_epoch.to_le_bytes());
        let hash = hasher.finalize();
        nonce_bytes[4..24].copy_from_slice(&hash.as_bytes()[0..20]);

        let nonce = chacha20poly1305::XNonce::from_slice(&nonce_bytes);

        let plaintext = payload.to_bytes();
        let payload_struct = Payload {
            msg: &plaintext,
            aad: &[],
        };

        let encrypted_payload = cipher
            .encrypt(nonce, payload_struct)
            .map_err(|e| BleBeaconError::CreationFailed(e.to_string()))?;

        Ok(BleBeacon {
            service_uuid: BLE_BEACON_SERVICE_UUID,
            encrypted_payload,
            rotation_epoch,
        })
    }
}

/// Parser for discovered BLE beacons
pub struct BeaconParser {
    group_key: [u8; 32],
    rotation_period_secs: u64,
}

impl BeaconParser {
    /// Create a new beacon parser
    pub fn new(group_key: [u8; 32]) -> Self {
        Self {
            group_key,
            rotation_period_secs: DEFAULT_BEACON_ROTATION_SECS,
        }
    }

    /// Set the beacon rotation period in seconds
    pub fn with_rotation_period(mut self, secs: u64) -> Self {
        self.rotation_period_secs = secs;
        self
    }

    /// Parse and decrypt a beacon if we have the group key
    pub fn parse(&self, beacon_data: &[u8]) -> Result<BeaconPayload, BleBeaconError> {
        decrypt_beacon_with_period(&self.group_key, beacon_data, self.rotation_period_secs)
            .map_err(|_| BleBeaconError::DecryptionFailed)
    }

    /// Check if a beacon is fresh (within acceptable epoch window)
    pub fn is_fresh(&self, rotation_epoch: u32) -> Result<bool, BleBeaconError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| BleBeaconError::SystemTimeError(e.to_string()))?;

        let current_epoch = (now.as_secs() / self.rotation_period_secs) as u32;

        // Accept beacons from current and previous epoch (clock skew tolerance)
        Ok((current_epoch as i32 - rotation_epoch as i32).abs() <= 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beacon_builder_creates_beacon() {
        let group_key = [0x42u8; 32];
        let node_pk = [0xaa; 32];

        let builder = BeaconBuilder::new(group_key, node_pk);
        let beacon = builder.build().expect("Beacon creation should succeed");

        assert_eq!(beacon.service_uuid, BLE_BEACON_SERVICE_UUID);
        assert!(!beacon.encrypted_payload.is_empty());
    }

    #[test]
    fn test_beacon_size_matches_contents() {
        let group_key = [0x42u8; 32];
        let node_pk = [0xaa; 32];

        let builder = BeaconBuilder::new(group_key, node_pk);
        let beacon = builder.build().expect("Beacon creation should succeed");

        let expected_size = 2 + beacon.encrypted_payload.len();
        assert_eq!(beacon.size(), expected_size);
    }

    #[test]
    fn test_beacon_parser_decrypts_valid_beacon() {
        let group_key = [0x42u8; 32];
        let node_pk = [0xaa; 32];

        let builder = BeaconBuilder::new(group_key, node_pk);
        let beacon = builder.build().expect("Beacon creation should succeed");

        let parser = BeaconParser::new(group_key);
        let payload = parser
            .parse(&beacon.encrypted_payload)
            .expect("Parsing should succeed");

        assert_eq!(payload.flags, 0);
        assert_eq!(payload.node_shard.len(), 4);
    }

    #[test]
    fn test_beacon_parser_rejects_wrong_key() {
        let group_key = [0x42u8; 32];
        let wrong_key = [0x99u8; 32];
        let node_pk = [0xaa; 32];

        let builder = BeaconBuilder::new(group_key, node_pk);
        let beacon = builder.build().expect("Beacon creation should succeed");

        let parser = BeaconParser::new(wrong_key);
        let result = parser.parse(&beacon.encrypted_payload);

        assert!(result.is_err(), "Wrong key should fail to decrypt");
    }

    #[test]
    fn test_beacon_parser_detects_fresh_beacons() {
        let group_key = [0x42u8; 32];
        let node_pk = [0xaa; 32];

        let builder = BeaconBuilder::new(group_key, node_pk);
        let beacon = builder.build().expect("Beacon creation should succeed");

        let parser = BeaconParser::new(group_key);
        let is_fresh = parser
            .is_fresh(beacon.rotation_epoch)
            .expect("Should check freshness");

        assert!(is_fresh, "Current epoch beacon should be fresh");
    }

    #[test]
    fn test_beacon_parser_accepts_skewed_epochs() {
        let group_key = [0x42u8; 32];
        let node_pk = [0xaa; 32];

        let builder = BeaconBuilder::new(group_key, node_pk);
        let beacon = builder.build().expect("Beacon creation should succeed");

        let parser = BeaconParser::new(group_key);

        // Test epoch_offset -1 (previous epoch)
        let is_fresh_prev = parser
            .is_fresh(beacon.rotation_epoch.saturating_sub(1))
            .expect("Should check freshness");
        assert!(is_fresh_prev, "Previous epoch should be fresh");

        // Test epoch_offset +1 (next epoch)
        let is_fresh_next = parser
            .is_fresh(beacon.rotation_epoch.saturating_add(1))
            .expect("Should check freshness");
        assert!(is_fresh_next, "Next epoch should be fresh");
    }

    #[test]
    fn test_beacon_parser_rejects_stale_epochs() {
        let group_key = [0x42u8; 32];
        let node_pk = [0xaa; 32];

        let builder = BeaconBuilder::new(group_key, node_pk);
        let beacon = builder.build().expect("Beacon creation should succeed");

        let parser = BeaconParser::new(group_key);

        // Test epoch_offset -2 (two epochs old)
        let is_fresh = parser
            .is_fresh(beacon.rotation_epoch.saturating_sub(2))
            .expect("Should check freshness");
        assert!(!is_fresh, "Stale epoch should not be fresh");
    }

    #[test]
    fn test_beacon_builder_with_custom_rotation_period() {
        let group_key = [0x42u8; 32];
        let node_pk = [0xaa; 32];

        let builder = BeaconBuilder::new(group_key, node_pk)
            .with_rotation_period(3600); // 1 hour

        let beacon = builder.build().expect("Beacon creation should succeed");
        assert!(!beacon.encrypted_payload.is_empty());
    }

    #[test]
    fn test_beacon_parser_with_custom_rotation_period() {
        let group_key = [0x42u8; 32];
        let node_pk = [0xaa; 32];

        let builder = BeaconBuilder::new(group_key, node_pk)
            .with_rotation_period(3600);

        let beacon = builder.build().expect("Beacon creation should succeed");

        let parser = BeaconParser::new(group_key).with_rotation_period(3600);
        let payload = parser
            .parse(&beacon.encrypted_payload)
            .expect("Parsing should succeed");

        assert_eq!(payload.flags, 0);
    }

    #[test]
    fn test_beacon_rotation_different_epochs() {
        let group_key = [0x42u8; 32];
        let node_pk = [0xaa; 32];

        let builder = BeaconBuilder::new(group_key, node_pk);
        let beacon1 = builder.build().expect("First beacon creation");
        let beacon2 = builder.build().expect("Second beacon creation");

        // Beacons might be identical if in same epoch, but should be parseable either way
        let parser = BeaconParser::new(group_key);

        let payload1 = parser
            .parse(&beacon1.encrypted_payload)
            .expect("First beacon should parse");
        let payload2 = parser
            .parse(&beacon2.encrypted_payload)
            .expect("Second beacon should parse");

        assert_eq!(payload1.flags, 0);
        assert_eq!(payload2.flags, 0);
    }

    #[test]
    fn test_beacon_parser_rejects_invalid_format() {
        let group_key = [0x42u8; 32];
        let parser = BeaconParser::new(group_key);

        let invalid_data = vec![0u8; 10]; // Too short
        let result = parser.parse(&invalid_data);

        assert!(result.is_err(), "Invalid format should fail");
    }

    #[test]
    fn test_beacon_service_uuid_constant() {
        assert_eq!(BLE_BEACON_SERVICE_UUID, 0xDF01);
    }

    #[test]
    fn test_default_rotation_period_constant() {
        assert_eq!(DEFAULT_BEACON_ROTATION_SECS, 15 * 60);
    }

    #[test]
    fn test_beacon_builder_default_rotation() {
        let group_key = [0x42u8; 32];
        let node_pk = [0xaa; 32];

        let builder = BeaconBuilder::new(group_key, node_pk);
        assert_eq!(
            builder.rotation_period_secs,
            DEFAULT_BEACON_ROTATION_SECS
        );
    }
}
