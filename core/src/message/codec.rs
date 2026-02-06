// Message codec — serialization with size limits to prevent abuse

use super::types::{Envelope, Message};
use anyhow::{bail, Result};

/// Maximum encoded message size: 256 KB
/// This prevents memory exhaustion from malicious oversized messages.
pub const MAX_MESSAGE_SIZE: usize = 256 * 1024;

/// Maximum text payload: 64 KB
pub const MAX_PAYLOAD_SIZE: usize = 64 * 1024;

/// Serialize a Message to bytes (bincode)
pub fn encode_message(msg: &Message) -> Result<Vec<u8>> {
    if msg.payload.len() > MAX_PAYLOAD_SIZE {
        bail!(
            "Payload too large: {} bytes (max {})",
            msg.payload.len(),
            MAX_PAYLOAD_SIZE
        );
    }

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
    fn test_reject_oversized_decode() {
        let big_bytes = vec![0u8; MAX_MESSAGE_SIZE + 1];
        let result = decode_message(&big_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_envelope_roundtrip() {
        let envelope = Envelope {
            sender_public_key: vec![1u8; 32],
            ephemeral_public_key: vec![2u8; 32],
            nonce: vec![3u8; 24],
            ciphertext: vec![4u8; 100],
        };

        let bytes = encode_envelope(&envelope).unwrap();
        let restored = decode_envelope(&bytes).unwrap();

        assert_eq!(envelope.sender_public_key, restored.sender_public_key);
        assert_eq!(envelope.nonce, restored.nonce);
        assert_eq!(envelope.ciphertext, restored.ciphertext);
    }
}
