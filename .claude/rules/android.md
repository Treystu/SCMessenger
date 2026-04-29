# Android Development Rules

Re-injected into agent context on every turn.

## Build Environment

- **Gradle:** 8.13, **AGP:** 8.13.2, **Kotlin:** 1.9.20
- **minSdk:** 26, **compileSdk:** 35
- **DI:** Hilt
- **UI:** Jetpack Compose

## Architecture

- `MeshRepository` → ViewModels → Compose UI
- UniFFI-generated bindings in `uniffi.api` package — never modify generated files directly.
- Transport managers: BLE, WiFi (Aware/Direct), foreground service for mesh persistence.

## Rust Cross-Compilation

Required targets (via `cargo-ndk`):
- `aarch64-linux-android` (required)
- `x86_64-linux-android` (required)
- `armv7-linux-androideabi` (full coverage)
- `i686-linux-android` (full coverage)

## Build Commands

```bash
cd android
./gradlew assembleDebug -x lint --quiet
```

## Pre-Merge Checklist

- `./gradlew assembleDebug` succeeds.
- `./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.test.RoleNavigationPolicyTest"` passes.
- No hardcoded strings in UI — all user-facing text in `strings.xml`.
- Foreground service notification channel is configured for Android 14+.
- BLE and WiFi permissions are declared in manifest with runtime request logic.
