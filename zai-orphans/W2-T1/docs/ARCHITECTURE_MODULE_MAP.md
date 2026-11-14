# Architecture

SCMessenger is organized as a Cargo workspace with platform-specific apps. The core library is written in Rust and exposed to mobile platforms via UniFFI.

## Core Entry Point

| File | Purpose |
|------|---------|
| `core/src/iron_core.rs` | Main orchestration — initializes subsystems and wires dependencies |
| `core/src/lib.rs` | Crate root, module declarations |

## Crypto

`core/src/crypto/`

| File | Purpose |
|------|---------|
| `ratchet.rs` | Double Ratchet algorithm implementation |
| `session_manager.rs` | Session lifecycle and key management |
| `kani_proofs.rs` | Formal verification proofs via Kani |
| `encrypt.rs` | Encryption/decryption primitives |
| `backup.rs` | Encrypted backup support |

## Identity

`core/src/identity/`

| File | Purpose |
|------|---------|
| `keys.rs` | Ed25519 key generation and management |
| `store.rs` | Persistent identity storage |

## Routing

`core/src/routing/`

| File | Purpose |
|------|---------|
| `engine.rs` | Core routing engine — path selection and forwarding |
| `local.rs` | Local mesh routing (direct neighbors) |
| `neighborhood.rs` | Neighborhood-level routing (2+ hops) |
| `global.rs` | Global routing for distant peers |
| `multipath.rs` | Multi-path forwarding for reliability |
| `reputation.rs` | Peer reputation scoring |
| `adaptive_ttl.rs` | TTL adjustment based on network conditions |

## Transport

`core/src/transport/`

| File | Purpose |
|------|---------|
| `swarm.rs` | Connection swarm management |
| `abstraction.rs` | Transport trait abstraction |
| `escalation.rs` | Automatic transport escalation (BLE → Wi-Fi) |
| `health.rs` | Transport health monitoring |
| `nat.rs` | NAT traversal utilities |
| `wifi_aware.rs` | Wi-Fi Aware transport |
| `discovery.rs` | Peer discovery coordination |
| `circuit_breaker.rs` | Circuit breaker for failing transports |
| `manager.rs` | Transport lifecycle manager |
| `internet.rs` | Internet-based transport fallback |

`core/src/transport/ble/`

| File | Purpose |
|------|---------|
| `gatt.rs` | BLE GATT client/server |
| `l2cap.rs` | BLE L2CAP CoC channels |
| `scanner.rs` | BLE device scanning |
| `beacon.rs` | BLE advertising/beaconing |

## Drift / DTN

`core/src/drift/`

| File | Purpose |
|------|---------|
| `sync.rs` | Delay-tolerant synchronization protocol |
| `sketch.rs` | MinHash sketches for set reconciliation |
| `store.rs` | Drift message store |
| `frame.rs` | Wire frame encoding |
| `envelope.rs` | Message envelope format |
| `relay.rs` | Custody-based relay logic |
| `policy.rs` | Forwarding and retention policies |
| `compress.rs` | Payload compression |
| `rate_limit.rs` | Rate limiting for relay traffic |

## Storage

`core/src/store/`

| File | Purpose |
|------|---------|
| `backend.rs` | Pluggable storage backend trait |
| `relay_custody.rs` | Relay custody storage for DTN |
| `outbox.rs` | Outgoing message queue |
| `dedup.rs` | Message deduplication |
| `blocked.rs` | Blocked-peer list enforcement |
| `inbox.rs` | Incoming message handling |
| `sweeper.rs` | Storage cleanup and expiry |

## FFI / Bridge

| File | Purpose |
|------|---------|
| `core/src/mobile_bridge.rs` | Primary FFI bridge for iOS/Android |
| `core/src/contacts_bridge.rs` | Contacts-specific FFI bridge |
| `core/src/blocked_bridge.rs` | Blocked-list FFI bridge |
| `core/src/api.udl` | UniFFI interface definition |

## CLI

`cli/src/`

| File | Purpose |
|------|---------|
| `main.rs` | CLI entry point |
| `cli.rs` | Interactive command parser |
| `server.rs` | Local HTTP server (Axum) |
| `ble_daemon.rs` | Background BLE daemon |
| `ble_mesh.rs` | BLE mesh visualization |
| `bootstrap.rs` | Network bootstrap logic |
| `config.rs` | Configuration management |

## WASM

`wasm/src/`

| File | Purpose |
|------|---------|
| `lib.rs` | WASM entry point |
| `transport.rs` | Browser transport layer |
| `mesh.rs` | Mesh networking for WASM |
| `daemon_bridge.rs` | Bridge to native daemon |
| `storage.rs` | IndexedDB-backed storage |
| `worker.rs` | Web Worker interface |

## iOS

`ios/SCMessenger/SCMessenger/Transport/`

| File | Purpose |
|------|---------|
| `BLECentralManager.swift` | BLE Central role (scanning, connecting) |
| `BLEPeripheralManager.swift` | BLE Peripheral role (advertising) |
| `BLEL2CAPManager.swift` | L2CAP channel management |
| `MultipeerTransport.swift` | MultipeerConnectivity transport |
| `mDNSServiceDiscovery.swift` | mDNS/Bonjour service discovery |
| `SmartTransportRouter.swift` | Automatic transport selection |
| `MeshBLEConstants.swift` | BLE service/characteristic UUIDs |
| `LocalTransportFallback.swift` | Fallback when no transport available |

## Android

`android/app/src/main/java/com/scmessenger/android/transport/`

| File | Purpose |
|------|---------|
| `ble/BleGattClient.kt` | BLE GATT client |
| `ble/BleGattServer.kt` | BLE GATT server |
| `ble/BleScanner.kt` | BLE device scanning |
| `ble/BleAdvertiser.kt` | BLE advertising |
| `ble/BleL2capManager.kt` | L2CAP channel management |
| `ble/BleBackoffStrategy.kt` | BLE reconnection backoff |
| `ble/BleQuotaManager.kt` | BLE connection quota management |
| `WifiAwareTransport.kt` | Wi-Fi Aware transport |
| `WifiDirectTransport.kt` | Wi-Fi Direct transport |
| `MdnsServiceDiscovery.kt` | mDNS service discovery |
| `SmartTransportRouter.kt` | Automatic transport selection |
| `TransportManager.kt` | Transport lifecycle management |
| `TransportHealthMonitor.kt` | Transport health monitoring |
| `NetworkDetector.kt` | Network state detection |
| `SubnetProbe.kt` | Subnet peer probing |

## Testing

See [`docs/device-testing.md`](docs/device-testing.md) for the physical-device test procedures used to validate BLE, Wi-Fi Aware, Wi-Fi Direct, Apple Multipeer, and DTN sneakernet scenarios.

## Deferred

- **Acoustic transport** — planned for post-v1.0.0
