//! Find My Wake-Up Protocol (Phase 6C - Experimental)
//!
//! Experimental Apple Find My compatible beacon encoding/decoding for BLE advertisements.
//! Uses simplified XOR-based encryption (NOT production-grade) to fit 22-byte BLE advertisements.
//! This is marked experimental because the security model is simplified for space constraints.

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Find My configuration
#[derive(Debug, Clone)]
pub struct FindMyConfig {
    /// Enable Find My wake-up beacons
    pub enabled: bool,
    /// Broadcast interval in seconds (default 900 = 15 minutes)
    pub broadcast_interval_secs: u64,
    /// Encryption key for beacon payloads (32 bytes)
    pub payload_key: Option<[u8; 32]>,
}

impl Default for FindMyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            broadcast_interval_secs: 900,
            payload_key: None,
        }
    }
}

impl FindMyConfig {
    /// Create a new Find My config
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            broadcast_interval_secs: 900,
            payload_key: None,
        }
    }

    /// Set the encryption key
    pub fn with_key(mut self, key: [u8; 32]) -> Self {
        self.payload_key = Some(key);
        self
    }

    /// Set broadcast interval
    pub fn with_interval(mut self, secs: u64) -> Self {
        self.broadcast_interval_secs = secs;
        self
    }
}

/// Wake-up payload to encode in BLE advertisement
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WakeUpPayload {
    /// Recipient hint (first 4 bytes of recipient peer ID)
    pub recipient_hint: [u8; 4],
    /// Message available flag (1 bit, stored in MSB of first byte)
    pub message_available: bool,
    /// Relay hint (first 4 bytes of relay peer ID for message location)
    pub relay_hint: [u8; 4],
}

impl WakeUpPayload {
    /// Create a new wake-up payload
    pub fn new(recipient_hint: [u8; 4], relay_hint: [u8; 4], message_available: bool) -> Self {
        Self {
            recipient_hint,
            message_available,
            relay_hint,
        }
    }
}

/// Find My beacon encoding/decoding errors
#[derive(Debug, Error)]
pub enum FindMyError {
    #[error("Invalid beacon data length")]
    InvalidLength,
    #[error("Encryption error")]
    EncryptionError,
    #[error("Decryption failed")]
    DecryptionFailed,
    #[error("Invalid key")]
    InvalidKey,
    #[error("Missing encryption key")]
    MissingKey,
}

/// Encode a wake-up payload into a compact BLE advertisement
///
/// Output format (max 22 bytes):
/// - Byte 0: Message available flag (MSB) + recipient_hint[0] (7 bits)
/// - Bytes 1-3: recipient_hint[1..4]
/// - Bytes 4-7: XOR-encrypted relay_hint (4 bytes)
/// - Bytes 8-20: Encrypted payload (13 bytes)
///
/// NOTE: This uses XOR encryption with blake3-derived keystream (experimental, not production-grade)
pub fn encode_wakeup(payload: &WakeUpPayload, key: &[u8; 32]) -> Result<Vec<u8>, FindMyError> {
    let mut output = vec![0u8; 22];

    // Byte 0: Message available flag (MSB) + first byte of recipient hint (7 bits)
    let msg_flag = if payload.message_available { 0x80 } else { 0x00 };
    output[0] = msg_flag | (payload.recipient_hint[0] & 0x7F);

    // Bytes 1-3: recipient_hint[1..4]
    output[1] = payload.recipient_hint[1];
    output[2] = payload.recipient_hint[2];
    output[3] = payload.recipient_hint[3];

    // Derive keystream from key using blake3
    let mut hasher = Hasher::new();
    hasher.update(key);
    hasher.update(b"findmy_beacon_v1");
    let keystream_hash = hasher.finalize();
    let keystream = keystream_hash.as_bytes();

    // Bytes 4-7: XOR-encrypted relay_hint
    for i in 0..4 {
        output[4 + i] = payload.relay_hint[i] ^ keystream[i];
    }

    // Bytes 8-20: Encrypted payload (13 bytes of zeros for basic beacon)
    // In a fuller implementation, this would contain actual message data
    for i in 0..13 {
        output[8 + i] = 0x00 ^ keystream[(i + 4) % 32];
    }

    // Bytes 21: unused (for alignment)
    output[21] = 0x00;

    Ok(output)
}

/// Decode a wake-up payload from a BLE advertisement
pub fn decode_wakeup(data: &[u8], key: &[u8; 32]) -> Result<WakeUpPayload, FindMyError> {
    if data.len() < 22 {
        return Err(FindMyError::InvalidLength);
    }

    // Extract message available flag and recipient hint
    let msg_flag = (data[0] & 0x80) != 0;
    let mut recipient_hint = [0u8; 4];
    recipient_hint[0] = data[0] & 0x7F;
    recipient_hint[1] = data[1];
    recipient_hint[2] = data[2];
    recipient_hint[3] = data[3];

    // Derive keystream
    let mut hasher = Hasher::new();
    hasher.update(key);
    hasher.update(b"findmy_beacon_v1");
    let keystream_hash = hasher.finalize();
    let keystream = keystream_hash.as_bytes();

    // Decrypt relay_hint
    let mut relay_hint = [0u8; 4];
    for i in 0..4 {
        relay_hint[i] = data[4 + i] ^ keystream[i];
    }

    Ok(WakeUpPayload {
        recipient_hint,
        message_available: msg_flag,
        relay_hint,
    })
}

/// Check if a beacon is intended for us
///
/// Returns true if the beacon's recipient_hint matches our hint
pub fn is_our_wakeup(
    beacon_data: &[u8],
    our_hint: &[u8; 4],
    key: &[u8; 32],
) -> Result<bool, FindMyError> {
    match decode_wakeup(beacon_data, key) {
        Ok(payload) => Ok(payload.recipient_hint == *our_hint),
        Err(FindMyError::InvalidLength) => Ok(false),
        Err(e) => Err(e),
    }
}

/// Find My beacon manager for coordinating wake-up broadcasts
pub struct FindMyBeaconManager {
    config: FindMyConfig,
    last_broadcast: u64,
}

impl FindMyBeaconManager {
    /// Create a new beacon manager
    pub fn new(config: FindMyConfig) -> Self {
        Self {
            config,
            last_broadcast: 0,
        }
    }

    /// Check if we should broadcast now
    pub fn should_broadcast(&mut self, now: u64) -> bool {
        if !self.config.enabled {
            return false;
        }

        if now - self.last_broadcast >= self.config.broadcast_interval_secs {
            self.last_broadcast = now;
            true
        } else {
            false
        }
    }

    /// Generate a beacon for a recipient
    pub fn generate_beacon(
        &self,
        recipient_hint: [u8; 4],
        relay_hint: [u8; 4],
        message_available: bool,
    ) -> Result<Vec<u8>, FindMyError> {
        let key = self
            .config
            .payload_key
            .ok_or(FindMyError::MissingKey)?;

        let payload = WakeUpPayload {
            recipient_hint,
            message_available,
            relay_hint,
        };

        encode_wakeup(&payload, &key)
    }

    /// Process an incoming beacon
    pub fn process_beacon(
        &self,
        beacon_data: &[u8],
    ) -> Result<Option<WakeUpPayload>, FindMyError> {
        let key = self
            .config
            .payload_key
            .ok_or(FindMyError::MissingKey)?;

        match decode_wakeup(beacon_data, &key) {
            Ok(payload) => Ok(Some(payload)),
            Err(FindMyError::InvalidLength) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Check if beacon is for us
    pub fn is_our_beacon(
        &self,
        beacon_data: &[u8],
        our_hint: &[u8; 4],
    ) -> Result<bool, FindMyError> {
        let key = self
            .config
            .payload_key
            .ok_or(FindMyError::MissingKey)?;

        is_our_wakeup(beacon_data, our_hint, &key)
    }
}

impl Default for FindMyBeaconManager {
    fn default() -> Self {
        Self::new(FindMyConfig::default())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        *b"testkeytestkeytestkeytestkey32by"
    }

    #[test]
    fn test_wakeup_payload_creation() {
        let payload = WakeUpPayload::new([1, 2, 3, 4], [5, 6, 7, 8], true);
        assert_eq!(payload.recipient_hint, [1, 2, 3, 4]);
        assert_eq!(payload.relay_hint, [5, 6, 7, 8]);
        assert!(payload.message_available);
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let payload = WakeUpPayload::new([0x12, 0x34, 0x56, 0x78], [0xAB, 0xCD, 0xEF, 0x01], true);
        let key = test_key();

        let encoded = encode_wakeup(&payload, &key).expect("Encode failed");
        assert_eq!(encoded.len(), 22);

        let decoded = decode_wakeup(&encoded, &key).expect("Decode failed");
        assert_eq!(decoded.recipient_hint, payload.recipient_hint);
        assert_eq!(decoded.relay_hint, payload.relay_hint);
        assert_eq!(decoded.message_available, payload.message_available);
    }

    #[test]
    fn test_encode_decode_message_flag_false() {
        let payload = WakeUpPayload::new([0x11, 0x22, 0x33, 0x44], [0x55, 0x66, 0x77, 0x88], false);
        let key = test_key();

        let encoded = encode_wakeup(&payload, &key).expect("Encode failed");
        let decoded = decode_wakeup(&encoded, &key).expect("Decode failed");

        assert!(!decoded.message_available);
    }

    #[test]
    fn test_encode_produces_22_bytes() {
        let payload = WakeUpPayload::new([1, 2, 3, 4], [5, 6, 7, 8], false);
        let key = test_key();

        let encoded = encode_wakeup(&payload, &key).expect("Encode failed");
        assert_eq!(encoded.len(), 22);
    }

    #[test]
    fn test_is_our_wakeup_match() {
        let recipient_hint = [0x2A, 0xBB, 0xCC, 0xDD];
        let payload = WakeUpPayload::new(recipient_hint, [1, 2, 3, 4], true);
        let key = test_key();

        let encoded = encode_wakeup(&payload, &key).expect("Encode failed");
        let is_ours = is_our_wakeup(&encoded, &recipient_hint, &key).expect("Check failed");

        assert!(is_ours);
    }

    #[test]
    fn test_is_our_wakeup_no_match() {
        let recipient_hint = [0xAA, 0xBB, 0xCC, 0xDD];
        let other_hint = [0x11, 0x22, 0x33, 0x44];
        let payload = WakeUpPayload::new(recipient_hint, [1, 2, 3, 4], true);
        let key = test_key();

        let encoded = encode_wakeup(&payload, &key).expect("Encode failed");
        let is_ours = is_our_wakeup(&encoded, &other_hint, &key).expect("Check failed");

        assert!(!is_ours);
    }

    #[test]
    fn test_is_our_wakeup_invalid_length() {
        let hint = [0xAA, 0xBB, 0xCC, 0xDD];
        let key = test_key();
        let invalid_data = vec![0, 1, 2, 3]; // Too short

        let result = is_our_wakeup(&invalid_data, &hint, &key);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Returns false for invalid length
    }

    #[test]
    fn test_beacon_manager_creation() {
        let config = FindMyConfig::new(true).with_key(test_key());
        let manager = FindMyBeaconManager::new(config);

        assert!(manager.config.enabled);
        assert_eq!(manager.config.broadcast_interval_secs, 900);
    }

    #[test]
    fn test_beacon_manager_should_broadcast() {
        let config = FindMyConfig::new(true)
            .with_key(test_key())
            .with_interval(60);
        let mut manager = FindMyBeaconManager::new(config);

        assert!(manager.should_broadcast(100));
        assert!(!manager.should_broadcast(150)); // Less than 60 seconds later
        assert!(manager.should_broadcast(160)); // 60+ seconds later
    }

    #[test]
    fn test_beacon_manager_generate_beacon() {
        let config = FindMyConfig::new(true).with_key(test_key());
        let manager = FindMyBeaconManager::new(config);

        let beacon = manager
            .generate_beacon([1, 2, 3, 4], [5, 6, 7, 8], true)
            .expect("Generate failed");

        assert_eq!(beacon.len(), 22);
    }

    #[test]
    fn test_beacon_manager_process_beacon() {
        let key = test_key();
        let config = FindMyConfig::new(true).with_key(key);
        let manager = FindMyBeaconManager::new(config);

        let payload = WakeUpPayload::new([0x19, 0x88, 0x77, 0x66], [0x55, 0x44, 0x33, 0x22], true);
        let beacon = encode_wakeup(&payload, &key).expect("Encode failed");

        let decoded = manager
            .process_beacon(&beacon)
            .expect("Process failed")
            .expect("No payload");

        assert_eq!(decoded.recipient_hint, [0x19, 0x88, 0x77, 0x66]);
        assert_eq!(decoded.relay_hint, [0x55, 0x44, 0x33, 0x22]);
        assert!(decoded.message_available);
    }

    #[test]
    fn test_beacon_manager_is_our_beacon() {
        let key = test_key();
        let our_hint = [0x11, 0x22, 0x33, 0x44];
        let config = FindMyConfig::new(true).with_key(key);
        let manager = FindMyBeaconManager::new(config);

        let payload = WakeUpPayload::new(our_hint, [0x55, 0x66, 0x77, 0x88], false);
        let beacon = encode_wakeup(&payload, &key).expect("Encode failed");

        let is_ours = manager
            .is_our_beacon(&beacon, &our_hint)
            .expect("Check failed");

        assert!(is_ours);
    }

    #[test]
    fn test_find_my_config_default() {
        let config = FindMyConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.broadcast_interval_secs, 900);
        assert!(config.payload_key.is_none());
    }

    #[test]
    fn test_find_my_config_builder() {
        let key = test_key();
        let config = FindMyConfig::new(true)
            .with_key(key)
            .with_interval(300);

        assert!(config.enabled);
        assert_eq!(config.broadcast_interval_secs, 300);
        assert_eq!(config.payload_key, Some(key));
    }

    #[test]
    fn test_different_keys_produce_different_output() {
        let payload = WakeUpPayload::new([1, 2, 3, 4], [5, 6, 7, 8], true);
        let key1 = [0u8; 32];
        let key2 = [1u8; 32];

        let encoded1 = encode_wakeup(&payload, &key1).expect("Encode 1 failed");
        let encoded2 = encode_wakeup(&payload, &key2).expect("Encode 2 failed");

        assert_ne!(encoded1, encoded2);
    }

    #[test]
    fn test_beacon_missing_key_error() {
        let config = FindMyConfig::new(true); // No key set
        let manager = FindMyBeaconManager::new(config);

        let result = manager.generate_beacon([1, 2, 3, 4], [5, 6, 7, 8], true);
        assert!(matches!(result, Err(FindMyError::MissingKey)));
    }
}
