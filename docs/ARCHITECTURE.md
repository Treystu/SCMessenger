# Architecture

## Design Principles

1. **No middleman.** Messages go directly between devices. No servers relay, store, or inspect them.
2. **Crypto-first.** Every message is encrypted before it touches the network. Identity is a keypair, not an account.
3. **Minimal viable scope.** Phase 0 is the messaging spine: identity, encryption, transport. Everything else comes later.

## Module Map

```
scmessenger-core/
  identity/
    keys.rs       Ed25519 keypair generation, signing, verification
    store.rs      Sled-backed persistent key storage
    mod.rs        IdentityManager (generate/load/sign/verify)

  crypto/
    encrypt.rs    Per-message encryption: X25519 ECDH + XChaCha20-Poly1305
    mod.rs        Re-exports

  message/
    types.rs      Message, Envelope, Receipt, DeliveryStatus
    codec.rs      Bincode encode/decode with size limits (256KB max)
    mod.rs        Re-exports

  store/
    outbox.rs     Per-peer message queue for store-and-forward
    inbox.rs      Message deduplication (50K ID tracking)
    mod.rs        Re-exports

  transport/
    behaviour.rs  Combined libp2p NetworkBehaviour (request-response, gossipsub, kad, mdns, identify)
    swarm.rs      Swarm lifecycle, command/event channels, SwarmHandle API
    mod.rs        Re-exports

  lib.rs          IronCore facade: lifecycle, identity, messaging, delegate
  api.udl         UniFFI interface definition for mobile bindings
```

## Cryptography

| Operation | Algorithm | Purpose |
|---|---|---|
| Identity | Ed25519 | Signing key, identity derivation |
| Identity hash | Blake3 | `identity_id = blake3(ed25519_public_key)` |
| Key exchange | X25519 ECDH | Ephemeral per-message shared secret |
| KDF | Blake3 derive_key | Shared secret -> symmetric key |
| Encryption | XChaCha20-Poly1305 | Authenticated encryption with 24-byte nonce |
| Key conversion | Ed25519 -> X25519 | Birational map via `curve25519-dalek` |

### Encryption Flow

```
Sender:
  1. Convert recipient Ed25519 public key -> X25519 public key
  2. Generate ephemeral X25519 keypair
  3. ECDH: ephemeral_secret x recipient_x25519_public -> shared_secret
  4. KDF: blake3_derive_key(shared_secret) -> symmetric_key
  5. Encrypt: XChaCha20-Poly1305(symmetric_key, random_nonce, plaintext)
  6. Output: Envelope { sender_pub, ephemeral_pub, nonce, ciphertext }

Recipient:
  1. Convert own Ed25519 signing key -> X25519 static secret
  2. ECDH: recipient_secret x ephemeral_public -> shared_secret
  3. KDF: same derivation -> symmetric_key
  4. Decrypt: XChaCha20-Poly1305(symmetric_key, nonce, ciphertext)
```

## Transport Stack

Built on **libp2p 0.53** with:

- **TCP** transport with Noise encryption and Yamux multiplexing
- **mDNS** for LAN peer discovery (zero-config)
- **Kademlia DHT** for WAN peer discovery (future)
- **Request-Response** protocol (`/sc/message/1.0.0`) for direct message delivery
- **Gossipsub** for future group messaging
- **Identify** protocol for peer metadata exchange

### Network vs Crypto Identity

The CLI uses **separate keypairs** for networking and cryptography:
- **Network identity**: libp2p Ed25519 keypair (for peer-to-peer connections)
- **Crypto identity**: SCMessenger Ed25519 keypair (for message encryption/signing)

This separation keeps the crypto identity independent of the transport layer.

## Platform Strategy

| Target | Binding | Crate |
|---|---|---|
| macOS/Linux | Native Rust CLI | `scmessenger-cli` |
| iOS/Android | UniFFI (C/Swift/Kotlin) | `scmessenger-mobile` |
| Browser | wasm-bindgen | `scmessenger-wasm` |

All three targets share `scmessenger-core` for crypto, identity, and message handling.
