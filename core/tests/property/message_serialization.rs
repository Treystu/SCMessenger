// Property-based tests for message serialization round-trip
// Validates: Requirements 3.5, 13.1

use proptest::prelude::*;
use scmessenger_core::message::types::{
    DeliveryStatus, Envelope, Message, MessageType, Receipt, SignedEnvelope,
};

// Strategy for generating arbitrary MessageType
fn arb_message_type() -> impl Strategy<Value = MessageType> {
    prop_oneof![
        Just(MessageType::Text),
        Just(MessageType::Receipt),
        Just(MessageType::OnionRelay),
    ]
}

// Strategy for generating arbitrary DeliveryStatus
fn arb_delivery_status() -> impl Strategy<Value = DeliveryStatus> {
    prop_oneof![
        Just(DeliveryStatus::Sent),
        Just(DeliveryStatus::Delivered),
        Just(DeliveryStatus::Read),
        Just(DeliveryStatus::Failed),
    ]
}

// Strategy for generating arbitrary Message
fn arb_message() -> impl Strategy<Value = Message> {
    (
        any::<String>(),                             // id
        any::<String>(),                             // sender_id
        any::<String>(),                             // recipient_id
        arb_message_type(),                          // message_type
        prop::collection::vec(any::<u8>(), 0..1024), // payload (0-1024 bytes)
        any::<u64>(),                                // timestamp
    )
        .prop_map(
            |(id, sender_id, recipient_id, message_type, payload, timestamp)| Message {
                id,
                sender_id,
                recipient_id,
                message_type,
                payload,
                timestamp,
            },
        )
}

// Strategy for generating arbitrary Receipt
fn arb_receipt() -> impl Strategy<Value = Receipt> {
    (
        any::<String>(),       // message_id
        arb_delivery_status(), // status
        any::<u64>(),          // timestamp
    )
        .prop_map(|(message_id, status, timestamp)| Receipt {
            message_id,
            status,
            timestamp,
        })
}

// Strategy for generating arbitrary Envelope
fn arb_envelope() -> impl Strategy<Value = Envelope> {
    (
        prop::collection::vec(any::<u8>(), 32..33), // sender_public_key (32 bytes)
        prop::collection::vec(any::<u8>(), 32..33), // ephemeral_public_key (32 bytes)
        prop::collection::vec(any::<u8>(), 24..25), // nonce (24 bytes)
        prop::collection::vec(any::<u8>(), 0..2048), // ciphertext (0-2048 bytes)
        prop::option::of(prop::collection::vec(any::<u8>(), 32..33)), // ratchet_dh_public
        prop::option::of(any::<u32>()),             // ratchet_message_number
    )
        .prop_map(
            |(
                sender_public_key,
                ephemeral_public_key,
                nonce,
                ciphertext,
                ratchet_dh_public,
                ratchet_message_number,
            )| Envelope {
                sender_public_key,
                ephemeral_public_key,
                nonce,
                ciphertext,
                ratchet_dh_public,
                ratchet_message_number,
            },
        )
}

// Strategy for generating arbitrary SignedEnvelope
fn arb_signed_envelope() -> impl Strategy<Value = SignedEnvelope> {
    (
        arb_envelope(),
        prop::collection::vec(any::<u8>(), 64..65), // signature (64 bytes)
    )
        .prop_map(|(envelope, signature)| SignedEnvelope {
            envelope,
            signature,
        })
}

proptest! {
    /// Property 1: Message serialization round-trip consistency
    /// Validates: Requirements 3.5, 13.1
    /// Property: serialize(deserialize(serialize(msg))) == serialize(msg)
    #[test]
    fn test_message_serialization_roundtrip(msg in arb_message()) {
        // Serialize
        let encoded = bincode::serialize(&msg).expect("serialization should succeed");

        // Deserialize
        let decoded: Message = bincode::deserialize(&encoded).expect("deserialization should succeed");

        // Re-serialize
        let re_encoded = bincode::serialize(&decoded).expect("re-serialization should succeed");

        // Property: Bytes should be identical
        prop_assert_eq!(encoded, re_encoded, "Round-trip serialization should produce identical bytes");

        // Additional checks: Fields should match
        prop_assert_eq!(msg.id, decoded.id);
        prop_assert_eq!(msg.sender_id, decoded.sender_id);
        prop_assert_eq!(msg.recipient_id, decoded.recipient_id);
        prop_assert_eq!(msg.message_type, decoded.message_type);
        prop_assert_eq!(msg.payload, decoded.payload);
        prop_assert_eq!(msg.timestamp, decoded.timestamp);
    }

    /// Property 2: Receipt serialization round-trip consistency
    /// Validates: Requirements 13.1
    /// Property: serialize(deserialize(serialize(receipt))) == serialize(receipt)
    #[test]
    fn test_receipt_serialization_roundtrip(receipt in arb_receipt()) {
        // Serialize
        let encoded = bincode::serialize(&receipt).expect("serialization should succeed");

        // Deserialize
        let decoded: Receipt = bincode::deserialize(&encoded).expect("deserialization should succeed");

        // Re-serialize
        let re_encoded = bincode::serialize(&decoded).expect("re-serialization should succeed");

        // Property: Bytes should be identical
        prop_assert_eq!(encoded, re_encoded, "Round-trip serialization should produce identical bytes");

        // Additional checks: Fields should match
        prop_assert_eq!(receipt.message_id, decoded.message_id);
        prop_assert_eq!(receipt.timestamp, decoded.timestamp);
    }

    /// Property 3: Envelope serialization round-trip consistency
    /// Validates: Requirements 13.1
    /// Property: serialize(deserialize(serialize(envelope))) == serialize(envelope)
    #[test]
    fn test_envelope_serialization_roundtrip(envelope in arb_envelope()) {
        // Serialize
        let encoded = bincode::serialize(&envelope).expect("serialization should succeed");

        // Deserialize
        let decoded: Envelope = bincode::deserialize(&encoded).expect("deserialization should succeed");

        // Re-serialize
        let re_encoded = bincode::serialize(&decoded).expect("re-serialization should succeed");

        // Property: Bytes should be identical
        prop_assert_eq!(encoded, re_encoded, "Round-trip serialization should produce identical bytes");

        // Additional checks: Fields should match
        prop_assert_eq!(envelope.sender_public_key, decoded.sender_public_key);
        prop_assert_eq!(envelope.ephemeral_public_key, decoded.ephemeral_public_key);
        prop_assert_eq!(envelope.nonce, decoded.nonce);
        prop_assert_eq!(envelope.ciphertext, decoded.ciphertext);
        prop_assert_eq!(envelope.ratchet_dh_public, decoded.ratchet_dh_public);
        prop_assert_eq!(envelope.ratchet_message_number, decoded.ratchet_message_number);
    }

    /// Property 4: SignedEnvelope serialization round-trip consistency
    /// Validates: Requirements 13.1
    /// Property: serialize(deserialize(serialize(signed_envelope))) == serialize(signed_envelope)
    #[test]
    fn test_signed_envelope_serialization_roundtrip(signed_envelope in arb_signed_envelope()) {
        // Serialize
        let encoded = bincode::serialize(&signed_envelope).expect("serialization should succeed");

        // Deserialize
        let decoded: SignedEnvelope = bincode::deserialize(&encoded).expect("deserialization should succeed");

        // Re-serialize
        let re_encoded = bincode::serialize(&decoded).expect("re-serialization should succeed");

        // Property: Bytes should be identical
        prop_assert_eq!(encoded, re_encoded, "Round-trip serialization should produce identical bytes");

        // Additional checks: Signature should match
        prop_assert_eq!(signed_envelope.signature, decoded.signature);
    }

    /// Property 5: Empty payload serialization
    /// Validates: Requirements 13.8 (edge case: empty collections)
    #[test]
    fn test_empty_payload_serialization(
        id in any::<String>(),
        sender_id in any::<String>(),
        recipient_id in any::<String>(),
        message_type in arb_message_type(),
        timestamp in any::<u64>(),
    ) {
        let msg = Message {
            id,
            sender_id,
            recipient_id,
            message_type,
            payload: vec![], // Empty payload
            timestamp,
        };

        let encoded = bincode::serialize(&msg).expect("serialization should succeed");
        let decoded: Message = bincode::deserialize(&encoded).expect("deserialization should succeed");

        prop_assert!(decoded.payload.is_empty(), "Empty payload should remain empty");
    }

    /// Property 6: Maximum size payload serialization
    /// Validates: Requirements 13.8 (edge case: maximum sizes)
    #[test]
    fn test_max_payload_serialization(payload in prop::collection::vec(any::<u8>(), 65536..65537)) {
        let msg = Message {
            id: "test".to_string(),
            sender_id: "sender".to_string(),
            recipient_id: "recipient".to_string(),
            message_type: MessageType::Text,
            payload,
            timestamp: 0,
        };

        let encoded = bincode::serialize(&msg).expect("serialization should succeed");
        let decoded: Message = bincode::deserialize(&encoded).expect("deserialization should succeed");

        prop_assert_eq!(msg.payload.len(), decoded.payload.len(), "Large payload size should be preserved");
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_property_test_strategies_compile() {
        // Smoke test to ensure strategies compile
        let _ = arb_message();
        let _ = arb_receipt();
        let _ = arb_envelope();
        let _ = arb_signed_envelope();
    }
}
