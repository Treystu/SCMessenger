# SCMessenger Android

Android client app for SCMessenger using Kotlin/Compose with UniFFI integration to Rust core.

## Build Prerequisites

- Rust toolchain
- `cargo-ndk`
- Android Rust targets:
  - `aarch64-linux-android`
  - `armv7-linux-androideabi`
  - `x86_64-linux-android`
  - `i686-linux-android`
- Java 17+
- Android SDK with `ANDROID_HOME` set

## Verify Environment

```bash
cd android
./verify-build-setup.sh
```

Latest local verification summary (2026-02-23):

- Rust/cargo/cargo-ndk: present
- Android Rust targets: present
- UniFFI Kotlin binding generation: pass
- Build preflight passes when `ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk`
- `./gradlew assembleDebug`: pass

## Build

```bash
cd android
ANDROID_HOME="$HOME/Library/Android/sdk" ./gradlew assembleDebug
```

## Runtime Architecture

- Rust core (`scmessenger-core`) provides identity, crypto, transport, and storage.
- UniFFI bridge (`scmessenger-mobile`) exposes `MeshService`, `SwarmBridge`, and managers.
- Android app layers:
  - `MeshRepository` as integration boundary
  - Compose UI + ViewModels
  - BLE/WiFi transport managers
  - foreground service and platform callbacks

## Security Notes

- Identity/signing: Ed25519
- Message encryption path: X25519 ECDH + XChaCha20-Poly1305 (implemented in Rust core)
- No central account requirement in core protocol model

## Known Open Gaps

- WiFi Aware transport needs device-level validation across supported hardware:
  - `android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt`

See `docs/CURRENT_STATE.md` and `REMAINING_WORK_TRACKING.md` for prioritized backlog context.
