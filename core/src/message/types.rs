// Message types — the literal point of this app

use serde::{Deserialize, Serialize};

/// What kind of message this is
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Plain text message
    Text,
    /// Delivery/read receipt
    Receipt,
}

/// Delivery status of a message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryStatus {
    /// Message sent (left this device)
    Sent,
    /// Message delivered to recipient's device
    Delivered,
    /// Message read by recipient
    Read,
    /// Delivery failed
    Failed(String),
}

/// A plaintext message before encryption.
///
/// This is what the application layer creates. It gets encrypted into
/// an `Envelope` before hitting the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID (UUID v4)
    pub id: String,
    /// Sender's identity ID (Blake3 hash of Ed25519 public key)
    pub sender_id: String,
    /// Recipient's identity ID
    pub recipient_id: String,
    /// Message type
    pub message_type: MessageType,
    /// Payload bytes (UTF-8 text for Text messages, serialized Receipt for receipts)
    pub payload: Vec<u8>,
    /// Unix timestamp (seconds)
    pub timestamp: u64,
}

/// A delivery/read receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    /// ID of the message this receipt is for
    pub message_id: String,
    /// New delivery status
    pub status: DeliveryStatus,
    /// Unix timestamp of the status change
    pub timestamp: u64,
}

/// An encrypted message envelope — what actually goes on the wire.
///
/// Contains everything a recipient needs to decrypt the message,
/// assuming they have their own private key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// Sender's Ed25519 public key (32 bytes) — so recipient knows who sent it
    pub sender_public_key: Vec<u8>,
    /// Ephemeral X25519 public key (32 bytes) — for ECDH key agreement
    pub ephemeral_public_key: Vec<u8>,
    /// XChaCha20-Poly1305 nonce (24 bytes)
    pub nonce: Vec<u8>,
    /// Encrypted + authenticated ciphertext
    pub ciphertext: Vec<u8>,
}

/// A signed envelope — envelope with an outer Ed25519 signature.
///
/// This structure adds an outer signature layer over the complete envelope,
/// allowing relay nodes or intermediate systems to verify sender identity
/// without requiring decryption. The sender's public key is included for
/// verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedEnvelope {
    /// The serialized Envelope (typically bincode-encoded)
    pub envelope_data: Vec<u8>,
    /// Ed25519 signature over envelope_data (64 bytes)
    pub sender_signature: Vec<u8>,
    /// Sender's Ed25519 public key (32 bytes) — for signature verification
    pub sender_public_key: Vec<u8>,
}

impl Message {
    /// Create a new text message
    pub fn text(sender_id: String, recipient_id: String, text: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            sender_id,
            recipient_id,
            message_type: MessageType::Text,
            payload: text.as_bytes().to_vec(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Create a receipt message.
    /// Returns an error if the receipt cannot be serialized.
    pub fn receipt(
        sender_id: String,
        recipient_id: String,
        receipt: &Receipt,
    ) -> Result<Self, String> {
        let payload = bincode::serialize(receipt)
            .map_err(|e| format!("Failed to serialize receipt: {}", e))?;
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            sender_id,
            recipient_id,
            message_type: MessageType::Receipt,
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Get text content (only valid for Text messages)
    pub fn text_content(&self) -> Option<String> {
        if self.message_type == MessageType::Text {
            String::from_utf8(self.payload.clone()).ok()
        } else {
            None
        }
    }

    /// Check if message is recent (within threshold_secs).
    /// Rejects future-dated messages (timestamp > now).
    pub fn is_recent(&self, threshold_secs: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // Reject messages from the future
        if self.timestamp > now {
            return false;
        }
        (now - self.timestamp) < threshold_secs
    }
}

impl Receipt {
    /// Create a delivery receipt
    pub fn delivered(message_id: String) -> Self {
        Self {
            message_id,
            status: DeliveryStatus::Delivered,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Create a read receipt
    pub fn read(message_id: String) -> Self {
        Self {
            message_id,
            status: DeliveryStatus::Read,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_text_message() {
        let msg = Message::text(
            "sender123".to_string(),
            "recipient456".to_string(),
            "hello world",
        );

        assert_eq!(msg.message_type, MessageType::Text);
        assert_eq!(msg.text_content().unwrap(), "hello world");
        assert_eq!(msg.sender_id, "sender123");
        assert_eq!(msg.recipient_id, "recipient456");
        assert!(!msg.id.is_empty());
        assert!(msg.timestamp > 0);
    }

    #[test]
    fn test_create_receipt() {
        let receipt = Receipt::delivered("msg-id-123".to_string());
        assert_eq!(receipt.message_id, "msg-id-123");
        assert!(matches!(receipt.status, DeliveryStatus::Delivered));
    }

    #[test]
    fn test_receipt_message() {
        let receipt = Receipt::delivered("msg-123".to_string());
        let msg =
            Message::receipt("sender".to_string(), "recipient".to_string(), &receipt).unwrap();

        assert_eq!(msg.message_type, MessageType::Receipt);
        assert!(msg.text_content().is_none());
    }

    #[test]
    fn test_message_recency() {
        let msg = Message::text("a".into(), "b".into(), "test");
        assert!(msg.is_recent(60)); // Should be recent within 60 seconds

        let mut old_msg = Message::text("a".into(), "b".into(), "test");
        old_msg.timestamp = 0; // epoch
        assert!(!old_msg.is_recent(60));

        // Future-dated messages should not be considered recent
        let mut future_msg = Message::text("a".into(), "b".into(), "test");
        future_msg.timestamp = u64::MAX;
        assert!(!future_msg.is_recent(60));
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::text("sender".into(), "recipient".into(), "hello");
        let bytes = bincode::serialize(&msg).unwrap();
        let restored: Message = bincode::deserialize(&bytes).unwrap();

        assert_eq!(msg.id, restored.id);
        assert_eq!(msg.text_content(), restored.text_content());
    }
}
