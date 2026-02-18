# Architecture

## Design Principles

1. **No middleman.** Every node IS the network. No third-party relays, no external infrastructure.
2. **Crypto-first.** Every message is encrypted before it touches the network. Identity is a keypair, not an account. Key material zeroized on drop.
3. **Relay = Messaging.** You cannot message without relaying. You cannot relay without messaging. Single toggle. This IS the incentive model.
4. **Internet is a transport, not a dependency.** BLE, WiFi Direct, WiFi Aware, and physical proximity are equal transports.

## Module Map

```
scmessenger-core/ (~29K LoC, 71 files, ~638 tests)

  identity/
    keys.rs       Ed25519 keypair generation, signing, verification, Zeroize-on-Drop
    store.rs      Sled-backed persistent key storage
    mod.rs        IdentityManager (generate/load/sign/verify)

  crypto/
    encrypt.rs    X25519 ECDH + XChaCha20-Poly1305, AAD-bound sender auth, envelope signatures
    mod.rs        Re-exports

  message/
    types.rs      Message, Envelope, SignedEnvelope, Receipt, DeliveryStatus
    codec.rs      Bincode encode/decode with size limits (256KB max)
    mod.rs        Re-exports

  store/
    outbox.rs     Per-peer message queue with quotas (10K total, 1K/peer)
    inbox.rs      Message dedup (50K IDs) + storage quotas (10K total, 1K/sender) with eviction
    mod.rs        Re-exports

  transport/
    abstraction.rs  TransportTrait, TransportEvent, TransportType, TransportCapabilities
    behaviour.rs    Combined libp2p NetworkBehaviour
    swarm.rs        Swarm lifecycle, command/event channels, SwarmHandle API
    manager.rs      Transport multiplexer, reconnection with exponential backoff, SendResult
    discovery.rs    DiscoveryMode (Open/Manual/Dark/Silent)
    escalation.rs   Transport escalation protocol (BLE→WiFi→Internet)
    internet.rs     Internet relay mode
    nat.rs          NAT traversal helpers
    wifi_aware.rs   WiFi Aware transport (Android)
    ble/            BLE transport (beacon, GATT, L2CAP, scanner)
    mod.rs          Re-exports

  drift/
    envelope.rs   DriftEnvelope (compact binary, 154 bytes overhead)
    frame.rs      DriftFrame (transport framing with CRC32)
    compress.rs   LZ4 compress/decompress
    sketch.rs     IBLT set reconciliation (Invertible Bloom Lookup Table; deterministic, one round-trip)
    sync.rs       SyncProtocol (handshake, sketch exchange, transfer)
    store.rs      CRDT MeshStore with priority-based eviction
    relay.rs      RelayService (receive→store→forward)
    policy.rs     RelayPolicy (auto-adjust, battery awareness)
    mod.rs        Module root

  routing/
    local.rs        Layer 1 — Local cell topology (full adjacency)
    neighborhood.rs Layer 2 — Neighborhood gossip (summarized awareness)
    global.rs       Layer 3 — Global route table (internet-connected nodes)
    engine.rs       Routing decision engine (layer cascade, multi-path, reputation)
    mod.rs          Module root

  relay/
    server.rs         RelayServer (accept connections, store-and-forward)
    client.rs         RelayClient (connect to known relays, push/pull sync)
    protocol.rs       Relay wire protocol (handshake, auth, sync)
    peer_exchange.rs  Exchange known relay addresses
    bootstrap.rs      Bootstrap protocol (QR code, invite link, BLE discovery)
    invite.rs         Invite system (friend introduces friend)
    findmy.rs         Find My beacon integration (emergency backhaul)
    mod.rs            Module root

  privacy/
    onion.rs    OnionEnvelope (layer, peel, construct)
    circuit.rs  Circuit construction (select N hops, build onion)
    cover.rs    Cover traffic generation
    padding.rs  Message padding to fixed sizes
    timing.rs   Randomized relay delays
    mod.rs      Module root

  mobile/
    service.rs       Mobile service lifecycle
    auto_adjust.rs   Smart auto-adjust (battery, charging, motion)
    ios_strategy.rs  iOS composite background strategy
    settings.rs      MeshSettings (serializable config)
    mod.rs           Module root

  platform/
    service.rs       Platform service management
    auto_adjust.rs   Platform-specific auto-adjust
    settings.rs      Platform settings
    mod.rs           Module root

  wasm_support/
    mesh.rs       Full mesh participation while tab open
    transport.rs  WebRTC/WebSocket transport
    storage.rs    OPFS-backed message store
    mod.rs        Module root

  lib.rs          IronCore facade: lifecycle, identity, messaging, delegate (~19K LoC)
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
