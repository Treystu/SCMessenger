# SCMessenger Platform Support Matrix (v0.1.2-alpha)

> **Last verified:** 2026-02-25

## Minimum Supported Versions

### Android

| Attribute           | Requirement                          |
|---------------------|--------------------------------------|
| **Minimum API**     | API 29 (Android 10)                  |
| **Target API**      | API 34 (Android 14)                  |
| **ABI**             | `arm64-v8a`, `x86_64`               |
| **Bluetooth LE**    | Required for nearby peer discovery   |
| **WiFi Aware**      | Optional (API 29+, device-dependent) |
| **Location perm**   | Required for BLE scanning            |

**Rationale:** Android 10 (API 29) provides WiFi Aware API, BLE 5.0 support,
and scoped storage — all required by SCMessenger transports. As of 2026,
Android 10+ covers >95% of active devices (source: Google Play Console).

### iOS

| Attribute           | Requirement                          |
|---------------------|--------------------------------------|
| **Minimum iOS**     | iOS 15.0                             |
| **Devices**         | iPhone 8 and later                   |
| **Bluetooth LE**    | Required for nearby peer discovery   |
| **Architecture**    | arm64 only                           |

**Rationale:** iOS 15 introduces `Actor` concurrency and modern Swift
concurrency features used throughout the app. The `Observable` macro
requires iOS 17+ but is conditionally used. iPhone 8+ ensures BLE 5.0
hardware. iOS 15+ covers >97% of active iPhones.

### Web (WASM)

| Attribute           | Requirement                            |
|---------------------|----------------------------------------|
| **Chrome**          | Latest 3 versions (currently 120+)     |
| **Firefox**         | Latest 3 versions (currently 121+)     |
| **Safari**          | Latest 3 versions (currently 17.0+)    |
| **Edge**            | Latest 3 versions (Chromium-based)     |
| **WebSocket**       | Required for libp2p transport          |
| **WebRTC**          | Optional (future direct P2P)           |
| **IndexedDB**       | Required for persistence (beta target) |

**Rationale:** "Latest 3 versions" policy covers >98% of desktop and mobile
browser users while keeping maintenance burden minimal. WebSocket transport
is the primary connection method; WebRTC is planned for direct browser-to-browser.

## CLI / Relay Nodes

| Attribute           | Requirement                          |
|---------------------|--------------------------------------|
| **OS**              | Linux (x86_64, aarch64), macOS (arm64, x86_64) |
| **Rust toolchain**  | 1.75+ (stable)                       |
| **Network**         | TCP port 9001 (default, configurable)|
| **Docker**          | Optional (images provided)           |

## Transport Protocol Compatibility

All platforms use the same libp2p protocol identifiers:

- `/sc/message/1.0.0` — Direct messaging
- `/sc/address-reflection/1.0.0` — NAT discovery
- `/sc/relay/1.0.0` — Mesh relay
- `/sc/ledger-exchange/1.0.0` — Peer list sharing
- `/sc/id/1.0.0` — Identity/metadata exchange

## Known Limitations (Alpha)

1. **Android WiFi Aware** — Device-dependent; not available on all Android 10+ devices.
   Validated on Pixel 6 series. Does not affect core messaging (BLE + Internet always available).

2. **iOS Background** — BLE advertising pauses when app is deep-backgrounded by iOS.
   Foreground/fetch modes keep internet transport active.

3. **WASM** — No BLE or WiFi transport. Internet-only via WebSocket.
   IndexedDB persistence is a beta target.

4. **Cellular NAT** — Some carrier-grade NATs block direct P2P. Relay fallback
   ensures delivery in all cases.

## Validation Plan

### Per-Release Gate

1. Build and install on minimum-supported devices for each platform.
2. Verify identity creation, contact exchange, and message send/receive.
3. Verify relay fallback when direct connection is unavailable.
4. Verify upgrade persistence (identity, contacts, history survive update).

### Automated CI Checks

- `cargo test --workspace` — Core unit + integration tests
- `cargo clippy --workspace` — Lint-free
- iOS simulator build — `xcodebuild` against `generic/platform=iOS Simulator`
- Android build — `./gradlew assembleDebug`
- WASM build — `cargo check --target wasm32-unknown-unknown`
