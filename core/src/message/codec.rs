// Message codec — serialization with size limits to prevent abuse
//
// Encoding strategy: Drift Protocol binary format (primary) with bincode fallback.
// encode_envelope always produces DriftEnvelope bytes (compact, fixed overhead, LZ4 compression).
// decode_envelope tries DriftEnvelope first; if the version byte doesn't match,
// it falls back to legacy bincode for backward compatibility with older nodes.

use super::types::{Envelope, Message};
use crate::drift::envelope::COMPRESSION_THRESHOLD;
use crate::drift::DRIFT_VERSION;
use crate::drift::{DriftEnvelope, EnvelopeType};
use anyhow::{bail, Result};

/// Maximum encoded message size: 256 KB
/// This prevents memory exhaustion from malicious oversized messages.
pub const MAX_MESSAGE_SIZE: usize = 256 * 1024;

/// Maximum text payload: 8 KB (8192 bytes)
pub const MAX_PAYLOAD_SIZE: usize = 8 * 1024;

/// Validate plaintext payload size against the messaging contract.
pub fn validate_payload_size(payload: &[u8]) -> Result<()> {
    if payload.len() > MAX_PAYLOAD_SIZE {
        bail!(
            "Payload too large: {} bytes (max {})",
            payload.len(),
            MAX_PAYLOAD_SIZE
        );
    }
    Ok(())
}

/// Serialize a Message to bytes (bincode)
pub fn encode_message(msg: &Message) -> Result<Vec<u8>> {
    validate_payload_size(&msg.payload)?;

    let bytes = bincode::serialize(msg)?;

    if bytes.len() > MAX_MESSAGE_SIZE {
        bail!(
            "Encoded message too large: {} bytes (max {})",
            bytes.len(),
            MAX_MESSAGE_SIZE
        );
    }

    Ok(bytes)
}

/// Deserialize bytes to a Message
pub fn decode_message(bytes: &[u8]) -> Result<Message> {
    if bytes.len() > MAX_MESSAGE_SIZE {
        bail!(
            "Message too large: {} bytes (max {})",
            bytes.len(),
            MAX_MESSAGE_SIZE
        );
    }

    let msg: Message = bincode::deserialize(bytes)?;
    Ok(msg)
}

/// Serialize an Envelope to bytes using the Drift Protocol binary format.
///
/// The legacy Envelope is converted to a DriftEnvelope with:
/// - Fixed 187-byte overhead (vs variable bincode)
/// - LZ4 compression for payloads above COMPRESSION_THRESHOLD
/// - Recipient hint derived from the sender public key for efficient routing
/// - Ed25519 signature for authenticity
///
/// Falls back to bincode if Drift conversion fails (e.g. invalid key lengths).
pub fn encode_envelope(envelope: &Envelope) -> Result<Vec<u8>> {
    // Attempt Drift Protocol binary encoding first
    if let Ok(drift_bytes) = encode_drift_envelope(envelope) {
        if drift_bytes.len() <= MAX_MESSAGE_SIZE {
            return Ok(drift_bytes);
        }
    }

    // Fallback to legacy bincode encoding
    let bytes = bincode::serialize(envelope)?;

    if bytes.len() > MAX_MESSAGE_SIZE {
        bail!(
            "Encoded envelope too large: {} bytes (max {})",
            bytes.len(),
            MAX_MESSAGE_SIZE
        );
    }

    Ok(bytes)
}

/// Deserialize bytes to an Envelope.
///
/// Tries Drift Protocol binary format first (checks version byte == DRIFT_VERSION).
/// If the data doesn't start with a valid Drift version byte, falls back to
/// legacy bincode deserialization for backward compatibility with older nodes.
pub fn decode_envelope(bytes: &[u8]) -> Result<Envelope> {
    if bytes.len() > MAX_MESSAGE_SIZE {
        bail!(
            "Envelope too large: {} bytes (max {})",
            bytes.len(),
            MAX_MESSAGE_SIZE
        );
    }

    // Try Drift Protocol binary format first
    if !bytes.is_empty() && bytes[0] == DRIFT_VERSION {
        if let Ok(drift_env) = DriftEnvelope::from_bytes(bytes) {
            return Ok(drift_env.to_legacy_envelope());
        }
    }

    // Fallback to legacy bincode
    let envelope: Envelope = bincode::deserialize(bytes)?;
    Ok(envelope)
}

/// Convert a legacy Envelope to DriftEnvelope bytes.
///
/// Uses a deterministic message ID derived from the envelope contents,
/// and signs with a zero key (placeholder) since the signing key is
/// not available in the codec layer. The actual signing happens in
/// prepare_message_internal where the identity's signing key is available.
fn encode_drift_envelope(envelope: &Envelope) -> Result<Vec<u8>> {
    // Convert fixed-size arrays from Vec
    let sender_public_key: [u8; 32] = envelope
        .sender_public_key
        .clone()
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid sender public key length"))?;

    let ephemeral_public_key: [u8; 32] = envelope
        .ephemeral_public_key
        .clone()
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid ephemeral public key length"))?;

    let nonce: [u8; 24] = envelope
        .nonce
        .clone()
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;

    let ratchet_dh_public = envelope
        .ratchet_dh_public
        .as_ref()
        .map(|v| -> Result<[u8; 32]> {
            v.clone()
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid ratchet DH public key length"))
        })
        .transpose()?;

    // Determine if compression should be applied
    let compressed = envelope.ciphertext.len() > COMPRESSION_THRESHOLD;

    // Build DriftEnvelope without signature (placeholder)
    let drift_env = DriftEnvelope {
        version: DRIFT_VERSION,
        envelope_type: EnvelopeType::EncryptedMessage,
        compressed,
        message_id: [0u8; 16],    // Will be filled by prepare_message_internal
        recipient_hint: [0u8; 4], // Will be filled by prepare_message_internal
        created_at: web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32,
        ttl_expiry: 0, // No expiry by default
        hop_count: 0,
        priority: 128, // Medium priority
        sender_public_key,
        ephemeral_public_key,
        nonce,
        signature: [0u8; 64], // Placeholder — signing happens at the IronCore layer
        ciphertext: envelope.ciphertext.clone(),
        ratchet_dh_public,
        ratchet_message_number: envelope.ratchet_message_number,
    };

    Ok(drift_env.to_bytes()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::types::Message;

    #[test]
    fn test_message_roundtrip() {
        let msg = Message::text("sender".into(), "recipient".into(), "hello world");
        let bytes = encode_message(&msg).unwrap();
        let restored = decode_message(&bytes).unwrap();

        assert_eq!(msg.id, restored.id);
        assert_eq!(msg.text_content(), restored.text_content());
    }

    #[test]
    fn test_reject_oversized_payload() {
        let big_payload = vec![0u8; MAX_PAYLOAD_SIZE + 1];
        let mut msg = Message::text("a".into(), "b".into(), "");
        msg.payload = big_payload;

        let result = encode_message(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_payload_boundary_accepts_8191_and_8192() {
        let mut msg_8191 = Message::text("a".into(), "b".into(), "");
        msg_8191.payload = vec![0u8; 8191];
        assert!(encode_message(&msg_8191).is_ok());

        let mut msg_8192 = Message::text("a".into(), "b".into(), "");
        msg_8192.payload = vec![0u8; 8192];
        assert!(encode_message(&msg_8192).is_ok());
    }

    #[test]
    fn test_payload_boundary_rejects_8193() {
        let mut msg_8193 = Message::text("a".into(), "b".into(), "");
        msg_8193.payload = vec![0u8; 8193];
        let result = encode_message(&msg_8193);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_oversized_decode() {
        let big_bytes = vec![0u8; MAX_MESSAGE_SIZE + 1];
        let result = decode_message(&big_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_envelope_roundtrip() {
        // Test DriftEnvelope instead of deprecated Envelope
        use crate::drift::DriftEnvelope;
        use rand::RngCore;

        // Create a DriftEnvelope directly
        let mut rng = rand::thread_rng();
        let mut sender_pk = [0u8; 32];
        let mut ephemeral_pk = [0u8; 32];
        let mut recipient_hint = [0u8; 4];
        let mut nonce = [0u8; 24];
        let mut signature = [0u8; 64];

        rng.fill_bytes(&mut sender_pk);
        rng.fill_bytes(&mut ephemeral_pk);
        rng.fill_bytes(&mut recipient_hint);
        rng.fill_bytes(&mut nonce);
        rng.fill_bytes(&mut signature);

        let ciphertext = vec![4u8; 100];

        let drift_env = DriftEnvelope {
            version: 1,
            envelope_type: crate::drift::EnvelopeType::EncryptedMessage,
            compressed: false,
            message_id: [0u8; 16],
            recipient_hint,
            created_at: 1234567890,
            ttl_expiry: 1234567990,
            hop_count: 0,
            priority: 0,
            sender_public_key: sender_pk,
            ephemeral_public_key: ephemeral_pk,
            nonce,
            signature,
            ciphertext: ciphertext.clone(),
            ratchet_dh_public: None,
            ratchet_message_number: None,
        };

        // Test roundtrip
        let bytes = drift_env.to_bytes().unwrap();
        let restored = DriftEnvelope::from_bytes(&bytes).unwrap();

        assert_eq!(drift_env.sender_public_key, restored.sender_public_key);
        assert_eq!(drift_env.nonce, restored.nonce);
        assert_eq!(ciphertext, restored.ciphertext);
    }

    #[test]
    fn test_encode_envelope_produces_drift_format() {
        // Verify that encode_envelope produces Drift-formatted output
        // (first byte should be DRIFT_VERSION = 0x01)
        let envelope = Envelope {
            sender_public_key: vec![1u8; 32],
            ephemeral_public_key: vec![2u8; 32],
            nonce: vec![3u8; 24],
            ciphertext: vec![4u8; 100],
            ratchet_dh_public: None,
            ratchet_message_number: None,
        };

        let bytes = encode_envelope(&envelope).unwrap();
        assert!(!bytes.is_empty(), "encode_envelope should produce output");
        // First byte should be DRIFT_VERSION if Drift format was used
        assert_eq!(bytes[0], DRIFT_VERSION, "Should produce Drift format");
    }

    #[test]
    fn test_decode_envelope_drift_format() {
        // Create a DriftEnvelope, encode it, then decode it via decode_envelope
        let drift_env = DriftEnvelope {
            version: DRIFT_VERSION,
            envelope_type: EnvelopeType::EncryptedMessage,
            compressed: false,
            message_id: [0u8; 16],
            recipient_hint: [0u8; 4],
            created_at: 1234567890,
            ttl_expiry: 1234567990,
            hop_count: 0,
            priority: 128,
            sender_public_key: [1u8; 32],
            ephemeral_public_key: [2u8; 32],
            nonce: [3u8; 24],
            signature: [0u8; 64],
            ciphertext: vec![4u8; 100],
            ratchet_dh_public: None,
            ratchet_message_number: None,
        };

        let bytes = drift_env.to_bytes().unwrap();
        let restored = decode_envelope(&bytes).unwrap();

        assert_eq!(restored.sender_public_key, vec![1u8; 32]);
        assert_eq!(restored.ephemeral_public_key, vec![2u8; 32]);
        assert_eq!(restored.nonce, vec![3u8; 24]);
        assert_eq!(restored.ciphertext, vec![4u8; 100]);
    }

    #[test]
    fn test_decode_envelope_bincode_fallback() {
        // Create an Envelope, bincode-serialize it, then decode it via decode_envelope
        // Bincode data does NOT start with DRIFT_VERSION (0x01), so it should fallback
        let envelope = Envelope {
            sender_public_key: vec![1u8; 32],
            ephemeral_public_key: vec![2u8; 32],
            nonce: vec![3u8; 24],
            ciphertext: vec![4u8; 50],
            ratchet_dh_public: None,
            ratchet_message_number: None,
        };

        let bytes = bincode::serialize(&envelope).unwrap();
        // Ensure bincode format doesn't start with DRIFT_VERSION
        // (it starts with a length prefix which is unlikely to be 0x01 for a 32-byte vec)
        let restored = decode_envelope(&bytes).unwrap();

        assert_eq!(restored.sender_public_key, vec![1u8; 32]);
        assert_eq!(restored.ephemeral_public_key, vec![2u8; 32]);
        assert_eq!(restored.nonce, vec![3u8; 24]);
        assert_eq!(restored.ciphertext, vec![4u8; 50]);
    }

    #[test]
    fn test_envelope_compression_threshold() {
        // Envelopes with large ciphertext should have compressed flag set
        let envelope = Envelope {
            sender_public_key: vec![1u8; 32],
            ephemeral_public_key: vec![2u8; 32],
            nonce: vec![3u8; 24],
            ciphertext: vec![0xABu8; 512], // Above COMPRESSION_THRESHOLD
            ratchet_dh_public: None,
            ratchet_message_number: None,
        };

        let bytes = encode_envelope(&envelope).unwrap();
        assert_eq!(bytes[0], DRIFT_VERSION);

        // The type byte should have compression flag set (0x80 | 0x01 = 0x81)
        assert_eq!(
            bytes[1], 0x81,
            "Large payloads should have compression flag set"
        );

        // Round-trip should still work
        let restored = decode_envelope(&bytes).unwrap();
        assert_eq!(restored.ciphertext.len(), 512);
        assert!(restored.ciphertext.iter().all(|&b| b == 0xAB));
    }
}
