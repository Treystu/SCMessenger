# Dependency necessity audit

Scope: direct production dependencies declared in Cargo, Android Gradle, Compose/KMP, the log visualizer, and iOS system frameworks. Scores rate necessity to the currently implemented product: 10 means removal breaks a core user capability; 1 means no demonstrated production use. Transitive dependencies are not scored independently because they are selected by these direct dependencies.

## Rust workspace

| Dependency set | Score | Functions supported | Lean recommendation |
| --- | ---: | --- | --- |
| `tokio`, `async-trait`, `futures` | 10 | Async service lifecycle, transport tasks, streams | Retain. |
| `libp2p` | 10 | Peer discovery, relay, gossipsub, Kademlia, NAT traversal, encrypted transport | Retain; audit feature flags per target. `websocket` and `websocket-websys` are 4 until browser relay support ships. |
| `serde`, `serde_json`, `bincode`, `uuid`, `hex`, `bs58` | 10 | Protocol encoding, persisted records, IDs, key/peer representation | Retain. |
| `ed25519-dalek`, `x25519-dalek`, `curve25519-dalek`, `chacha20poly1305`, `sha2`, `blake3`, `zeroize` | 10 | Identity, key agreement, authenticated encryption, hashing, secret erasure | Retain; security review required before changes. |
| `libcrux-ml-kem`, `ml-dsa` | 7 | Post-quantum key encapsulation and signatures | Retain only if the negotiated protocol actively uses both; otherwise feature-gate until enabled. |
| `pbkdf2`, `argon2` | 9 | Backup-passphrase key derivation and compatibility | Retain while both backup formats are supported; consolidate only after a migration plan. |
| `sled`, `parking_lot`, `dirs`, `camino` | 9 | Durable stores, synchronization, platform storage paths | Retain. `camino` is optional generator support. |
| `lz4_flex`, `crc32fast` | 8 | Compact/integrity-checked backup payloads | Retain if backup format requires them. |
| `uniffi`, `uniffi_bindgen` | 10 | iOS/Android/Rust binding generation | Retain; build-only generator pieces should remain optional. |
| `anyhow`, `thiserror` | 9 | Boundary errors and typed core errors | Retain. |
| `tracing`, `tracing-subscriber`, `tracing-appender` | 8 | Operational diagnostics and CLI logs | Retain; subscriber/appender belong in app binaries, not the core library. |
| `web-time` | 7 | WASM-compatible timing | Keep target-gated. |
| `rustc-hash` | 5 | Faster internal hash maps | Benchmark against standard maps; removal candidate if no measured win. |
| `rand` | 10 | Key material, nonce and randomized protocol state | Retain. |
| `clap`, `colored`, `chrono`, `lazy_static` | 7 | CLI parsing, display, timestamps, static state | CLI-only. Replace `lazy_static` with `OnceLock` if its source usage permits. |
| `warp`, `axum`, `tower`, `tower-http`, `hyper`, `hyper-util`, `http-body-util`, `tokio-stream`, `futures-util` | 3 | CLI HTTP/dashboard paths | Highest Rust reduction opportunity: choose one HTTP stack after mapping active routes, preferably Axum's current stack. |
| `btleplug` | 6 | Desktop/CLI Bluetooth operations | Keep target-gated and verify active CLI use. |
| `wasm-bindgen`, `wasm-bindgen-futures`, `js-sys`, `web-sys`, `serde-wasm-bindgen`, `tracing-wasm`, `getrandom`, `console_error_panic_hook`, `gloo-timers` | 9 | Browser bindings, browser timing/RNG, diagnostics | Retain in the WASM member only. |
| `mockall`, `tokio-test`, `tempfile`, `proptest`, `wasm-bindgen-test` | 10 | Test-only mocks, property tests, temporary stores, WASM tests | Retain as development-only dependencies. |

## Android, shared desktop, web tooling, and iOS

| Dependency set | Score | Functions supported | Lean recommendation |
| --- | ---: | --- | --- |
| Kotlin, Coroutines, AndroidX Core/Lifecycle/Activity/Work/AppCompat | 10 | Android UI, lifecycle, background work | Retain. |
| Compose BOM, UI, Material, Material3, Navigation | 10 | Android user interface and navigation | Retain; `material` is 5 if every screen is Material3. |
| Hilt, Hilt compiler, Hilt navigation | 8 | Android dependency injection | Retain while Android uses Hilt; do not add Koin to the Android app layer. |
| Koin core/android | 6 | Shared KMP/desktop dependency injection | Keep only if shared modules use it; otherwise standardize on one DI boundary. |
| ZXing, Google Code Scanner | 8 | QR mesh/contact scanning | Retain. |
| DataStore, Accompanist permissions, splashscreen | 8 | Preferences, runtime permissions, launch UX | Retain. |
| JNA and UniFFI runtime | 10 | Android-to-Rust bridge | Retain. |
| Timber | 6 | Android logging | Consider routing through one structured logging facade. |
| Compose Desktop, Kotlin Multiplatform | 9 | Desktop/shared UI | Retain for supported desktop product. |
| Chokidar, Express, WS | 7 | Log visualizer server and live updates | Retain only if the visualizer is a supported tool. |
| D3, D3 Sankey, Lucide React | 6 | Log visualizer charts and icons | Retain for visualizer; defer loading chart packages if bundle size matters. |
| SwiftUI, Foundation, Combine, UIKit, Security, OSLog | 10 | iOS UI, state, secure backup, diagnostics | System frameworks; retain. |
| CoreBluetooth, Network, MultipeerConnectivity, CoreMotion, BackgroundTasks | 9 | iOS mesh transport and background behavior | Retain. |
| Vision, VisionKit, CoreImage, UserNotifications | 8 | QR scanning/generation and notification features | Retain when those features remain in product scope. |

## Reduction queue

1. Inventory active CLI HTTP routes, then remove either the Warp or Axum stack. This is the only direct dependency cluster scored below 5 with meaningful transitive weight.
2. Verify whether browser WebSocket transport is live; if not, disable `libp2p` WebSocket features for native builds.
3. Confirm whether the post-quantum primitives are negotiated in release builds before making them mandatory in every target.
4. Check Android screen imports before removing Compose Material 2, Timber, or Accompanist.
5. Run `cargo tree -e features` and platform artifact-size measurements before each removal; functionality and security take priority over a smaller lockfile.
