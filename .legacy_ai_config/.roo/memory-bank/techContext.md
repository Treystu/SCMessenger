# Technology Context

## Core Stack
- **Language**: Rust (edition 2021)
- **Async Runtime**: tokio
- **Networking**: libp2p (swarm, gossipsub, relay, identify)
- **Cryptography**: Ed25519, X25519, XChaCha20-Poly1305, Blake3

## Platform Bindings
- **UniFFI**: Rust to Kotlin/Swift bindings
- **WASM**: WebAssembly for browser support

## Mobile Platforms
- **Android**: Kotlin + Jetpack Compose, min SDK 26, target SDK 35
- **iOS**: SwiftUI, iOS 16+, XCFramework integration

## Build Tools
- **Rust**: cargo, rustup
- **Android**: Gradle 8.x, Android Gradle Plugin
- **iOS**: Xcode 15+, xcodebuild
- **WASM**: wasm-pack, wasm-bindgen

## Key Dependencies
- `libp2p` - P2P networking
- `tokio` - Async runtime
- `serde` - Serialization
- `thiserror` - Error handling
- `uniffi` - Cross-platform bindings
- `blake3` - Hashing
- `ed25519-dalek` - Signatures
- `x25519-dalek` - Key exchange
- `chacha20poly1305` - Encryption

## Testing
- `cargo test` for Rust unit/integration tests
- Platform-specific test suites for mobile
- Interop matrix for cross-platform validation
