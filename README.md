# SCMessenger

**Status**: Active
**Last updated**: 2026-07-11
**Version**: v0.3.4 (alpha, driving to v1.0.0)

[![CI](https://github.com/Treystu/SCMessenger/actions/workflows/ci.yml/badge.svg)](https://github.com/Treystu/SCMessenger/actions/workflows/ci.yml)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](rust-toolchain.toml)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

**Messaging that works when the internet does not.**

SCMessenger is a sovereign, end-to-end encrypted, decentralized messaging
mesh. No servers. No accounts. No phone numbers. Your identity is a keypair
you generate on-device; your messages travel over whatever path physically
exists between you and your peer -- Bluetooth in a crowd, WiFi on a plane,
LAN at home, or a relay across the internet -- racing every available
transport and delivering over the first one that lands.

## Why it exists

Every mainstream messenger dies with its servers: censored, subpoenaed,
rate-limited, or simply offline. SCMessenger assumes the worst-case network
from the start -- no internet, no WiFi, a stranger's phone passing by on
BLE -- and treats the happy path as a bonus. If any radio on your device can
reach any radio on theirs, directly or through intermediate custody, the
message gets through.

## How it works

**Transport ladder, raced in parallel (sub-500ms failover):**

```
BLE  ->  WiFi Aware / WiFi Direct (Android) | Multipeer (iOS)
     ->  mDNS / LAN (TCP + WebSocket + QUIC)
     ->  QUIC / TCP relay
     ->  Internet relay (store-and-forward custody)
```

- **Adaptive ports**: if the default port is firewalled, listeners and
  dialers ladder through 443, 80, 8080, and ephemeral ports -- whatever
  lands traffic on that network is the right port. Last-good routes are
  remembered per network.
- **Relay custody**: offline peers do not lose messages. Relays hold
  encrypted envelopes until receipt confirmation, then release custody.
- **Store**: local-only sled database; nothing leaves the device
  unencrypted.

**Cryptography:**

- Identity: Ed25519 signing keys, generated and held on-device.
- Sessions: X25519 ECDH with a double ratchet; XChaCha20-Poly1305
  authenticated encryption for every message.
- **Post-quantum migration in progress**: hybrid X25519 + ML-KEM-768 key
  agreement (libcrux), versioned wire envelopes, suite negotiation, and a
  PQ-augmented ratchet are implemented and undergoing mandatory adversarial
  review before v1.0.0. Old data always stays decryptable; new sessions
  negotiate the strongest suite both ends support.
- Privacy layer: onion routing and cover traffic modules for
  metadata-resistant delivery.
- Verification: property-based tests (proptest), Kani formal proofs on
  crypto paths, and a standing adversarial-review gate on every change to
  crypto, transport, routing, or privacy code.

## Platforms

| Platform | Client | State |
|---|---|---|
| Windows / Linux / macOS | `scmessenger-cli` headless daemon + local web UI | Active; Windows CLI <-> Android validated end-to-end across LAN/TCP/relay (Phase 1 exit, 2026-07-10) |
| Android | Kotlin / Jetpack Compose app | Active; full transport stack incl. BLE + WiFi Direct |
| iOS | SwiftUI app | Feature-parity codebase (BLE, Multipeer, LAN, relay); bindings regen pending post-PQC |
| Browser | WASM thin client over local JSON-RPC WebSocket | Active |
| Linux desktop (KMP) | Compose Multiplatform | Planned (v1.0 scope) |

One Rust core (`scmessenger-core`) drives all of them via UniFFI bindings
(Android/iOS) and JSON-RPC (browser).

## Quick start

```bash
# Prerequisites: Rust stable (rustup), Git
git clone https://github.com/Treystu/SCMessenger.git
cd SCMessenger

# Build and test the core
cargo build --workspace
cargo test --workspace

# Run a node (HTTP UI on 9000, P2P on 9001, WASM bridge on 9002)
cargo run --release --bin scmessenger-cli -- start

# Run a headless relay node
cargo run --release --bin scmessenger-cli -- relay --listen /ip4/0.0.0.0/tcp/0
```

The daemon's control surfaces bind to localhost only. Linux builds need
`libdbus-1-dev` and `pkg-config` (BLE support).

Platform guides: [Android](docs/platform/ANDROID_SETUP.md) |
[iOS](docs/platform/IOS_SETUP.md) | [WASM](docs/platform/WASM_SETUP.md) |
[CLI](docs/platform/CLI_SETUP.md)

## Workspace layout

```
core/     scmessenger-core: identity, crypto, transport, store, routing, relay, privacy
cli/      headless daemon + embedded web server
mobile/   UniFFI bridge crate (Android/iOS bindings)
wasm/     browser thin-client (JSON-RPC over WebSocket)
android/  Kotlin/Compose app          iOS/      SwiftUI app
docs/     canonical documentation     HANDOFF/  live task backlog
```

## Documentation

- [DOCUMENTATION.md](DOCUMENTATION.md) -- docs hub and navigation
- [Architecture](docs/ARCHITECTURE.md) -- system design
- [Current State](docs/CURRENT_STATE.md) -- verified implementation status
- [v1.0.0 Execution Plan](HANDOFF/V1_0_0_EXECUTION_PLAN.md) -- the road to 1.0
- [Testing Guide](docs/TESTING_GUIDE.md) -- gates and test inventory
- [Protocol](docs/PROTOCOL.md) -- wire contract

## Contributing

Contributions welcome -- see [CONTRIBUTING.md](CONTRIBUTING.md). The short
version: fork, branch, `cargo test --workspace`, `cargo fmt` +
`cargo clippy`, conventional commits, PR. Changes to `core/src/{crypto,
transport,routing,privacy}` require adversarial security review before
merge (see [SECURITY.md](SECURITY.md)).

## Security

Do not open public issues for vulnerabilities. Report privately via GitHub
Security Advisories. Policy: [SECURITY.md](SECURITY.md).

## License

Public domain under [The Unlicense](LICENSE). Take it, fork it, ship it --
sovereignty includes the code.
