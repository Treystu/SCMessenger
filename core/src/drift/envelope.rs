/// Drift Envelope — compact binary format for mesh relay
///
/// Fixed overhead: 186 bytes (18 + 14 + 152 + 2)
/// Format: Header(18) + Routing(14) + Crypto(152) + Payload(2+N)
///
/// Layout (little-endian, no padding):
/// [1]  version
/// [1]  envelope_type
/// [16] message_id
/// [4]  recipient_hint
/// [4]  created_at (LE u32)
/// [4]  ttl_expiry (LE u32)
/// [1]  hop_count
/// [1]  priority
/// [32] sender_public_key
/// [32] ephemeral_public_key
/// [24] nonce
/// [64] signature
/// [2]  ciphertext_len (LE u16)
/// [N]  ciphertext

use super::{DriftError, DRIFT_VERSION};

/// Drift Protocol Envelope — compact binary format for mesh relay
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriftEnvelope {
    // Header (18 bytes)
    /// Protocol version (0x01)
    pub version: u8,
    /// Envelope type (encrypted message, receipt, sync, etc.)
    pub envelope_type: EnvelopeType,
    /// Message ID as raw bytes (16 bytes, UUID)
    pub message_id: [u8; 16],

    // Routing header (14 bytes)
    /// First 4 bytes of blake3(recipient_pk) for efficient routing
    pub recipient_hint: [u8; 4],
    /// Unix timestamp (seconds) when envelope was created
    pub created_at: u32,
    /// TTL expiry (0 = never expires), Unix timestamp
    pub ttl_expiry: u32,
    /// Hop count for relay tracking
    pub hop_count: u8,
    /// Message priority (0-255, higher = more important)
    pub priority: u8,

    // Crypto header (152 bytes)
    /// Sender's Ed25519 public key (32 bytes)
    pub sender_public_key: [u8; 32],
    /// Ephemeral X25519 public key for ECDH (32 bytes)
    pub ephemeral_public_key: [u8; 32],
    /// XChaCha20-Poly1305 nonce (24 bytes)
    pub nonce: [u8; 24],
    /// Ed25519 signature over everything except signature itself (64 bytes)
    pub signature: [u8; 64],

    // Payload (variable length)
    /// Encrypted + authenticated ciphertext
    pub ciphertext: Vec<u8>,
}

/// Message type enumeration for Drift Envelopes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EnvelopeType {
    /// Encrypted message (0x01)
    EncryptedMessage = 0x01,
    /// Delivery receipt (0x02)
    DeliveryReceipt = 0x02,
    /// Sync request (0x03)
    SyncRequest = 0x03,
    /// Sync response (0x04)
    SyncResponse = 0x04,
    /// Peer announcement (0x05)
    PeerAnnouncement = 0x05,
    /// Route advertisement (0x06)
    RouteAdvertisement = 0x06,
}

impl EnvelopeType {
    /// Convert from u8 to EnvelopeType
    pub fn from_u8(value: u8) -> Result<Self, DriftError> {
        match value {
            0x01 => Ok(EnvelopeType::EncryptedMessage),
            0x02 => Ok(EnvelopeType::DeliveryReceipt),
            0x03 => Ok(EnvelopeType::SyncRequest),
            0x04 => Ok(EnvelopeType::SyncResponse),
            0x05 => Ok(EnvelopeType::PeerAnnouncement),
            0x06 => Ok(EnvelopeType::RouteAdvertisement),
            other => Err(DriftError::InvalidEnvelopeType(other)),
        }
    }

    /// Convert to u8
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl DriftEnvelope {
    /// Fixed overhead size: 18 + 14 + 152 + 2 = 186 bytes
    pub const FIXED_OVERHEAD: usize = 186;

    /// Maximum ciphertext size (2^16 - 1 bytes due to u16 length field)
    pub const MAX_CIPHERTEXT: usize = 65535;

    /// Serialize envelope to bytes (little-endian)
    ///
    /// Returns `Err(CiphertextTooLarge)` if ciphertext exceeds MAX_CIPHERTEXT.
    pub fn to_bytes(&self) -> Result<Vec<u8>, DriftError> {
        if self.ciphertext.len() > Self::MAX_CIPHERTEXT {
            return Err(DriftError::CiphertextTooLarge(self.ciphertext.len()));
        }

        let mut buf = Vec::with_capacity(Self::FIXED_OVERHEAD + self.ciphertext.len());

        // Header (18 bytes)
        buf.push(self.version);
        buf.push(self.envelope_type.as_u8());
        buf.extend_from_slice(&self.message_id);

        // Routing header (14 bytes)
        buf.extend_from_slice(&self.recipient_hint);
        buf.extend_from_slice(&self.created_at.to_le_bytes());
        buf.extend_from_slice(&self.ttl_expiry.to_le_bytes());
        buf.push(self.hop_count);
        buf.push(self.priority);

        // Crypto header (120 bytes)
        buf.extend_from_slice(&self.sender_public_key);
        buf.extend_from_slice(&self.ephemeral_public_key);
        buf.extend_from_slice(&self.nonce);
        buf.extend_from_slice(&self.signature);

        // Payload (2 + N bytes)
        buf.extend_from_slice(&(self.ciphertext.len() as u16).to_le_bytes());
        buf.extend_from_slice(&self.ciphertext);

        Ok(buf)
    }

    /// Deserialize envelope from bytes
    ///
    /// Returns error if:
    /// - Buffer too short
    /// - Invalid version
    /// - Invalid envelope type
    pub fn from_bytes(data: &[u8]) -> Result<Self, DriftError> {
        if data.len() < Self::FIXED_OVERHEAD {
            return Err(DriftError::BufferTooShort {
                need: Self::FIXED_OVERHEAD,
                got: data.len(),
            });
        }

        let mut offset = 0;

        // Header (18 bytes)
        let version = data[offset];
        offset += 1;

        if version != DRIFT_VERSION {
            return Err(DriftError::InvalidVersion(version));
        }

        let envelope_type = EnvelopeType::from_u8(data[offset])?;
        offset += 1;

        let mut message_id = [0u8; 16];
        message_id.copy_from_slice(&data[offset..offset + 16]);
        offset += 16;

        // Routing header (14 bytes)
        let mut recipient_hint = [0u8; 4];
        recipient_hint.copy_from_slice(&data[offset..offset + 4]);
        offset += 4;

        let created_at = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        let ttl_expiry = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        let hop_count = data[offset];
        offset += 1;

        let priority = data[offset];
        offset += 1;

        // Crypto header (120 bytes)
        let mut sender_public_key = [0u8; 32];
        sender_public_key.copy_from_slice(&data[offset..offset + 32]);
        offset += 32;

        let mut ephemeral_public_key = [0u8; 32];
        ephemeral_public_key.copy_from_slice(&data[offset..offset + 32]);
        offset += 32;

        let mut nonce = [0u8; 24];
        nonce.copy_from_slice(&data[offset..offset + 24]);
        offset += 24;

        let mut signature = [0u8; 64];
        signature.copy_from_slice(&data[offset..offset + 64]);
        offset += 64;

        // Payload (2 + N bytes)
        let ciphertext_len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;

        if data.len() < offset + ciphertext_len {
            return Err(DriftError::BufferTooShort {
                need: offset + ciphertext_len,
                got: data.len(),
            });
        }

        let ciphertext = data[offset..offset + ciphertext_len].to_vec();

        Ok(DriftEnvelope {
            version,
            envelope_type,
            message_id,
            recipient_hint,
            created_at,
            ttl_expiry,
            hop_count,
            priority,
            sender_public_key,
            ephemeral_public_key,
            nonce,
            signature,
            ciphertext,
        })
    }

    /// Get the recipient hint from a public key using blake3
    pub fn hint_from_public_key(public_key: &[u8; 32]) -> [u8; 4] {
        let hash = blake3::hash(public_key);
        let mut hint = [0u8; 4];
        hint.copy_from_slice(&hash.as_bytes()[..4]);
        hint
    }

    /// Increment hop count (for relay forwarding)
    ///
    /// Uses saturating arithmetic to prevent overflow.
    pub fn increment_hop(&mut self) {
        self.hop_count = self.hop_count.saturating_add(1);
    }

    /// Check if message has expired
    ///
    /// Returns false if ttl_expiry is 0 (never expires).
    pub fn is_expired(&self) -> bool {
        if self.ttl_expiry == 0 {
            return false;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        now > self.ttl_expiry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_envelope() -> DriftEnvelope {
        DriftEnvelope {
            version: DRIFT_VERSION,
            envelope_type: EnvelopeType::EncryptedMessage,
            message_id: [1u8; 16],
            recipient_hint: [2u8; 4],
            created_at: 1234567890,
            ttl_expiry: 1234567900,
            hop_count: 5,
            priority: 10,
            sender_public_key: [3u8; 32],
            ephemeral_public_key: [4u8; 32],
            nonce: [5u8; 24],
            signature: [6u8; 64],
            ciphertext: b"test payload".to_vec(),
        }
    }

    #[test]
    fn test_envelope_type_conversion() {
        assert_eq!(EnvelopeType::EncryptedMessage.as_u8(), 0x01);
        assert_eq!(EnvelopeType::DeliveryReceipt.as_u8(), 0x02);
        assert_eq!(EnvelopeType::SyncRequest.as_u8(), 0x03);
        assert_eq!(EnvelopeType::SyncResponse.as_u8(), 0x04);
        assert_eq!(EnvelopeType::PeerAnnouncement.as_u8(), 0x05);
        assert_eq!(EnvelopeType::RouteAdvertisement.as_u8(), 0x06);

        assert_eq!(
            EnvelopeType::from_u8(0x01).unwrap(),
            EnvelopeType::EncryptedMessage
        );
        assert_eq!(
            EnvelopeType::from_u8(0x02).unwrap(),
            EnvelopeType::DeliveryReceipt
        );
        assert!(EnvelopeType::from_u8(0x99).is_err());
    }

    #[test]
    fn test_envelope_serialize_deserialize() {
        let original = make_test_envelope();
        let bytes = original.to_bytes().unwrap();

        assert_eq!(
            bytes.len(),
            DriftEnvelope::FIXED_OVERHEAD + original.ciphertext.len()
        );

        let restored = DriftEnvelope::from_bytes(&bytes).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_envelope_empty_ciphertext() {
        let mut env = make_test_envelope();
        env.ciphertext = vec![];

        let bytes = env.to_bytes().unwrap();
        assert_eq!(bytes.len(), DriftEnvelope::FIXED_OVERHEAD);

        let restored = DriftEnvelope::from_bytes(&bytes).unwrap();
        assert_eq!(restored.ciphertext.len(), 0);
    }

    #[test]
    fn test_envelope_large_ciphertext() {
        let mut env = make_test_envelope();
        env.ciphertext = vec![0xAB; 10000];

        let bytes = env.to_bytes().unwrap();
        assert_eq!(bytes.len(), DriftEnvelope::FIXED_OVERHEAD + 10000);

        let restored = DriftEnvelope::from_bytes(&bytes).unwrap();
        assert_eq!(restored.ciphertext.len(), 10000);
        assert!(restored.ciphertext.iter().all(|&b| b == 0xAB));
    }

    #[test]
    fn test_envelope_max_ciphertext() {
        let mut env = make_test_envelope();
        env.ciphertext = vec![0x42; DriftEnvelope::MAX_CIPHERTEXT];

        let bytes = env.to_bytes().unwrap();
        let restored = DriftEnvelope::from_bytes(&bytes).unwrap();
        assert_eq!(restored.ciphertext.len(), DriftEnvelope::MAX_CIPHERTEXT);
    }

    #[test]
    fn test_envelope_ciphertext_too_large() {
        let mut env = make_test_envelope();
        env.ciphertext = vec![0x42; DriftEnvelope::MAX_CIPHERTEXT + 1];

        let result = env.to_bytes();
        assert!(matches!(result, Err(DriftError::CiphertextTooLarge(_))));
    }

    #[test]
    fn test_hint_from_public_key_deterministic() {
        let pk = [42u8; 32];
        let hint1 = DriftEnvelope::hint_from_public_key(&pk);
        let hint2 = DriftEnvelope::hint_from_public_key(&pk);

        assert_eq!(hint1, hint2);
    }

    #[test]
    fn test_hint_from_public_key_different_keys() {
        let pk1 = [1u8; 32];
        let pk2 = [2u8; 32];

        let hint1 = DriftEnvelope::hint_from_public_key(&pk1);
        let hint2 = DriftEnvelope::hint_from_public_key(&pk2);

        assert_ne!(hint1, hint2);
    }

    #[test]
    fn test_increment_hop() {
        let mut env = make_test_envelope();
        env.hop_count = 5;

        env.increment_hop();
        assert_eq!(env.hop_count, 6);

        env.hop_count = 255;
        env.increment_hop();
        assert_eq!(env.hop_count, 255); // saturating
    }

    #[test]
    fn test_is_expired_never_expires() {
        let mut env = make_test_envelope();
        env.ttl_expiry = 0;

        assert!(!env.is_expired());
    }

    #[test]
    fn test_is_expired_in_future() {
        let mut env = make_test_envelope();
        env.ttl_expiry = u32::MAX;

        assert!(!env.is_expired());
    }

    #[test]
    fn test_is_expired_in_past() {
        let mut env = make_test_envelope();
        env.ttl_expiry = 1;

        assert!(env.is_expired());
    }

    #[test]
    fn test_buffer_too_short() {
        let data = [0u8; 100];
        let result = DriftEnvelope::from_bytes(&data);

        match result {
            Err(DriftError::BufferTooShort { need, got }) => {
                assert_eq!(need, DriftEnvelope::FIXED_OVERHEAD);
                assert_eq!(got, 100);
            }
            other => panic!("Expected BufferTooShort, got {:?}", other),
        }
    }

    #[test]
    fn test_invalid_version() {
        let mut data = vec![0u8; DriftEnvelope::FIXED_OVERHEAD + 5];
        data[0] = 0xFF; // Invalid version

        let result = DriftEnvelope::from_bytes(&data);
        assert!(matches!(result, Err(DriftError::InvalidVersion(0xFF))));
    }

    #[test]
    fn test_invalid_envelope_type() {
        let mut data = vec![0u8; DriftEnvelope::FIXED_OVERHEAD + 5];
        data[0] = DRIFT_VERSION;
        data[1] = 0x99; // Invalid type

        let result = DriftEnvelope::from_bytes(&data);
        assert!(matches!(
            result,
            Err(DriftError::InvalidEnvelopeType(0x99))
        ));
    }

    #[test]
    fn test_ciphertext_length_exceeds_buffer() {
        let mut data = vec![0u8; DriftEnvelope::FIXED_OVERHEAD];
        data[0] = DRIFT_VERSION;
        data[1] = 0x01; // EncryptedMessage

        // Set ciphertext length to 1000 but don't provide that much data
        let offset = DriftEnvelope::FIXED_OVERHEAD - 2;
        data[offset] = 232; // (1000 & 0xFF)
        data[offset + 1] = 3; // (1000 >> 8)

        let result = DriftEnvelope::from_bytes(&data);
        assert!(matches!(result, Err(DriftError::BufferTooShort { .. })));
    }

    #[test]
    fn test_little_endian_timestamps() {
        let mut env = make_test_envelope();
        env.created_at = 0x12345678;
        env.ttl_expiry = 0xABCDEF00;

        let bytes = env.to_bytes().unwrap();

        // Check that timestamps are stored little-endian
        let created_at_offset = 18 + 4; // After version, type, message_id, recipient_hint
        assert_eq!(bytes[created_at_offset], 0x78);
        assert_eq!(bytes[created_at_offset + 1], 0x56);
        assert_eq!(bytes[created_at_offset + 2], 0x34);
        assert_eq!(bytes[created_at_offset + 3], 0x12);

        let restored = DriftEnvelope::from_bytes(&bytes).unwrap();
        assert_eq!(restored.created_at, 0x12345678);
        assert_eq!(restored.ttl_expiry, 0xABCDEF00);
    }
}
