# SCMessenger Android

Android client for SCMessenger - a decentralized, peer-to-peer mesh messaging application with end-to-end encryption.

## Architecture

### Rust Core Integration via UniFFI

The Android app uses UniFFI to bridge between Kotlin and the Rust core library (`scmessenger-core`). All cryptography, identity management, and mesh protocol logic runs in Rust for maximum security and performance.

**Key Components:**

- **MeshService** (Rust): Lifecycle management for the mesh network
- **ContactManager** (Rust): Contact storage with sled database
- **HistoryManager** (Rust): Message history persistence
- **LedgerManager** (Rust): Connection tracking for bootstrap
- **AutoAdjustEngine** (Rust): Battery/network-aware parameter tuning
- **PlatformBridge** (Kotlin→Rust): System state callbacks

### Android Components

- **MeshForegroundService**: Keeps mesh network alive in background
- **AndroidPlatformBridge**: Monitors battery, network, lifecycle
- **MeshRepository**: Single source of truth for Rust core access
- **PreferencesRepository**: Android app preferences via DataStore

### UI Stack

- **Jetpack Compose**: Modern declarative UI
- **Material 3**: Design system
- **Hilt**: Dependency injection
- **Navigation Compose**: Screen routing

## Building

### Quick Setup Check

Run the verification script to check if your build environment is ready:

```bash
cd android
./verify-build-setup.sh
```

This will check for all prerequisites and test that bindings generation works.

### Prerequisites

1. **Rust toolchain** (install from https://rustup.rs)
2. **cargo-ndk** for cross-compilation:
   ```bash
   cargo install cargo-ndk
   ```
3. **Android Rust targets**:
   ```bash
   rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
   ```
4. **NDK** version 26.1.10909125 (Android Studio will download this automatically)
5. **Java 17+** (for Android Gradle builds)

Note: You do **NOT** need to install `uniffi-bindgen` separately. The project uses a custom binary (`core/src/bin/gen_kotlin.rs`) that's built as part of the process.

### Build Steps

The Gradle build automatically handles:

1. **Generating Kotlin bindings** from `core/src/api.udl` using `cargo run --bin gen_kotlin`
2. **Building Rust libraries** for all Android ABIs via `cargo-ndk`
3. **Packaging JNI libraries** into the APK

To build:

```bash
cd android
./gradlew assembleDebug
```

Or use Android Studio's build button.

### Troubleshooting

If you encounter "unresolved reference" errors for `uniffi.api.*` types:

1. Verify bindings can be generated:
   ```bash
   cd core
   cargo run --bin gen_kotlin --features gen-bindings
   ```

2. Check the generated file exists:
   ```bash
   ls -lh core/target/generated-sources/uniffi/kotlin/uniffi/api/api.kt
   ```

3. Run the verification script: `./verify-build-setup.sh`

See `BUILD_FIX_SUMMARY.md` for details on the build process.

## Project Structure

```
android/
├── app/
│   ├── src/main/
│   │   ├── java/com/scmessenger/android/
│   │   │   ├── data/           # Repositories
│   │   │   ├── di/             # Hilt modules
│   │   │   ├── service/        # Foreground service, platform bridge
│   │   │   └── ui/             # Compose screens and theme
│   │   ├── res/                # Android resources
│   │   └── AndroidManifest.xml
│   ├── build.gradle            # App-level build with Rust integration
│   └── proguard-rules.pro
├── build.gradle                # Root build configuration
└── settings.gradle
```

## Permissions

SCMessenger requires the following permissions for mesh networking:

- **Bluetooth**: BLE discovery and messaging
- **Location**: WiFi Aware requires fine location (not used for tracking)
- **Notifications**: Message alerts
- **Foreground Service**: Keep mesh alive in background

All permissions are requested at runtime with appropriate rationale.

## Development

### Logging

Uses Timber for logging. All logs tagged with class name automatically.

Debug builds plant a `DebugTree` for logcat output.

### Testing

```bash
./gradlew test           # Unit tests
./gradlew connectedTest  # Instrumentation tests
```

## Security

- **Identity**: Ed25519 keypairs generated and stored in Rust
- **Messages**: AES-256-GCM encryption via Rust cryptography
- **No telemetry**: Fully decentralized, no analytics
- **Backup exclusions**: Sensitive data excluded from Android backup

## License

Same as parent SCMessenger project.
