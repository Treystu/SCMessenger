// Message codec — serialization with size limits to prevent abuse

use super::types::{Envelope, Message};
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

/// Serialize an Envelope to bytes
pub fn encode_envelope(envelope: &Envelope) -> Result<Vec<u8>> {
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

/// Deserialize bytes to an Envelope
pub fn decode_envelope(bytes: &[u8]) -> Result<Envelope> {
    if bytes.len() > MAX_MESSAGE_SIZE {
        bail!(
            "Envelope too large: {} bytes (max {})",
            bytes.len(),
            MAX_MESSAGE_SIZE
        );
    }

    let envelope: Envelope = bincode::deserialize(bytes)?;
    Ok(envelope)
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
        assert!(encode_message(&msg_8193).is_err());
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
}
