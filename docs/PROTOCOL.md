# Protocol Specification

## Envelope Format

All messages on the wire are serialized as **bincode-encoded `Envelope` structs**:

```rust
struct Envelope {
    sender_public_key: Vec<u8>,      // 32 bytes, Ed25519 public key
    ephemeral_public_key: Vec<u8>,   // 32 bytes, X25519 ephemeral public key
    nonce: Vec<u8>,                  // 24 bytes, XChaCha20-Poly1305 nonce
    ciphertext: Vec<u8>,             // Encrypted + authenticated payload
}
```

Maximum encoded envelope size: **256 KB**.

## Encryption Scheme

**Per-message ephemeral ECDH + authenticated encryption:**

1. Sender generates an ephemeral X25519 keypair for each message
2. Recipient's Ed25519 public key is converted to X25519 via birational map
3. ECDH produces a shared secret: `ephemeral_secret * recipient_x25519_public`
4. Blake3 `derive_key` with context `"iron-core v2 message encryption 2026-02-05"` produces the 256-bit symmetric key
5. XChaCha20-Poly1305 encrypts the plaintext with a random 24-byte nonce

**Properties:**
- Forward secrecy per message (ephemeral keys)
- Authenticated encryption (Poly1305 MAC)
- Sender identification via `sender_public_key` in envelope

## Signing Scheme

Ed25519 signatures over raw message bytes. Used for:
- Identity verification
- Future: signed receipts, key attestations

## Message Types

The plaintext inside an envelope is a bincode-encoded `Message`:

```rust
struct Message {
    id: String,              // UUID v4
    sender_id: String,       // Blake3 hash of sender's Ed25519 public key
    recipient_id: String,    // Blake3 hash or hex public key of recipient
    message_type: MessageType,
    payload: Vec<u8>,        // UTF-8 text for Text, bincode Receipt for Receipt
    timestamp: u64,          // Unix seconds
}

enum MessageType {
    Text,
    Receipt,
}

struct Receipt {
    message_id: String,
    status: DeliveryStatus,  // Sent | Delivered | Read | Failed
    timestamp: u64,
}
```

Maximum payload size: **64 KB**.

## Protocol IDs

| Protocol | ID | Purpose |
|---|---|---|
| Message delivery | `/sc/message/1.0.0` | Request-response direct messaging |
| Peer identification | `/sc/id/1.0.0` | libp2p identify protocol |

## Message Deduplication

The inbox tracks up to 50,000 seen message IDs. Messages with previously-seen IDs are rejected. This prevents replay attacks and duplicate delivery from retransmission.

## Store-and-Forward

The outbox queues up to 1,000 messages per peer and 10,000 total. When a peer comes online (discovered via mDNS or Kademlia), queued messages can be delivered.
