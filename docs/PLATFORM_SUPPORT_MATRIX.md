# SCMessenger Platform Support Matrix (v0.2.0-alpha)

Status: Active
Last updated: 2026-03-03
Lock policy: **Lock to Code** (baseline values match current repository build configuration)

## Minimum Supported Versions

### Android

| Attribute | Requirement |
| --- | --- |
| `minSdk` | 26 (Android 8.0) |
| `targetSdk` | 34 (Android 14) |
| `compileSdk` | 34 |
| ABI | `arm64-v8a`, `x86_64` |
| BLE | Required for nearby path |
| WiFi Direct | Supported |
| WiFi Aware | Optional (device/API dependent) |
| Runtime permissions | Bluetooth + location + nearby WiFi + notifications (API-gated) |

Source of truth: `android/build.gradle` and `android/app/build.gradle`.

### iOS

| Attribute | Requirement |
| --- | --- |
| Minimum iOS | 17.0 |
| Architecture | arm64 (device), arm64 simulator toolchain |
| Nearby transports | CoreBluetooth + MultipeerConnectivity |

Source of truth: `iOS/SCMessenger/SCMessenger.xcodeproj` build target settings and runtime verification scripts.

### Web / WASM (Desktop Web Target)

| Attribute | Requirement |
| --- | --- |
| Browser policy | Latest 3 versions (Chrome/Edge/Firefox/Safari) |
| Transport | Internet path (libp2p/WebSocket) |
| Local wireless transports | Not available in browser runtime |
| Build toolchain | `wasm-pack` + `wasm-bindgen` |

### CLI / Relay Nodes

| Attribute | Requirement |
| --- | --- |
| OS families | macOS, Linux, Windows (cross-target checks) |
| Rust | Stable toolchain (workspace currently validates on 1.75+) |
| Relay mode | Supported via `scm relay` |
| Ports | `9001` P2P default (`9000` HTTP status default in relay mode) |

## Role-Mode Support

| Platform | Full Mode | Relay-Only Mode |
| --- | --- | --- |
| Android | Supported | Supported |
| iOS | Supported | Supported |
| WASM/Desktop GUI | Supported | Supported |
| CLI | Supported (`scm start`) | Supported (`scm relay`) |

## Cross-Platform Protocol Surface

Canonical protocol/transport contracts remain in:

- `docs/PROTOCOL.md`
- `core/src/api.udl`

Function-level interoperability and adapter-consumption evidence is tracked in:

- `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md`

## Known Alpha Constraints

1. WASM/Desktop browser target remains internet-path only (no BLE/WiFi nearby stack in-browser).
2. iOS build passes with non-fatal linker warnings tied to simulator SDK/object-version skew in local environments.
3. Function-level adapter parity is currently gap-free in static scan evidence (`docs/INTEROP_MATRIX_V0.2.0_ALPHA.md`); runtime/live network scenarios still require separate device validation.

## Verification Gates (Current Baseline)

1. `cargo check --workspace`
2. `cargo fmt --all -- --check`
3. `cargo clippy --workspace`
4. `cargo clippy --workspace --lib --bins --examples -- -D warnings`
5. `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:compileDebugKotlin :app:lintDebug`
6. `bash ./iOS/verify-test.sh`
7. `cd wasm && wasm-pack build`
