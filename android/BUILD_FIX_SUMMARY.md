# Android Build Fix Summary

## Problem
The Android build was failing with 66 compilation errors, all related to unresolved references to `uniffi.api.*` types.

## Root Cause
The Gradle task `generateUniFFIBindings` had incorrect command-line arguments that prevented it from successfully generating the UniFFI Kotlin bindings from the Rust core library.

## Solution

### 1. Fixed `generateUniFFIBindings` Task
**File:** `android/app/build.gradle`

**Before:**
```gradle
commandLine 'cargo', 'run', '--bin', 'gen_kotlin', '--features', 'gen-bindings', '--', 'generate', 'src/api.udl', '--language', 'kotlin', '--out-dir', 'target/generated-sources/uniffi/kotlin'
```

**After:**
```gradle
commandLine 'cargo', 'run', '--bin', 'gen_kotlin', '--features', 'gen-bindings'
```

**Reason:** The `gen_kotlin` binary doesn't accept command-line arguments. It's hardcoded to:
- Read from `src/api.udl`
- Generate Kotlin bindings
- Output to `target/generated-sources/uniffi/kotlin`

### 2. Verified Bindings Generation
The generated file `core/target/generated-sources/uniffi/kotlin/uniffi/api/api.kt` (230KB) contains all required types:

#### Interfaces (8)
- `IronCore` - Core identity and messaging
- `MeshService` - Service lifecycle
- `ContactManager` - Contact management
- `HistoryManager` - Message history
- `LedgerManager` - Connection tracking
- `MeshSettingsManager` - Settings persistence
- `AutoAdjustEngine` - Battery-aware optimization
- `SwarmBridge` - libp2p network bridge

#### Callback Interfaces (2)
- `CoreDelegate` - Core events
- `PlatformBridge` - Platform state callbacks

#### Enums (5)
- `ServiceState` - STOPPED, STARTING, RUNNING, STOPPING
- `MotionState` - STILL, WALKING, RUNNING, AUTOMOTIVE, UNKNOWN
- `DiscoveryMode` - NORMAL, CAUTIOUS, PARANOID
- `MessageDirection` - SENT, RECEIVED
- `AdjustmentProfile` - MAXIMUM, HIGH, STANDARD, REDUCED, MINIMAL

#### Data Classes (12)
- `IdentityInfo`
- `SignatureResult`
- `MeshServiceConfig`
- `ServiceStats`
- `DeviceProfile`
- `BleAdjustment`
- `RelayAdjustment`
- `MeshSettings`
- `Contact`
- `MessageRecord`
- `HistoryStats`
- `LedgerEntry`

#### Error Type (1)
- `IronCoreError` enum

## Build Process

### Prerequisites
1. Rust toolchain (1.93+)
2. Android NDK 26.1.10909125
3. cargo-ndk: `cargo install cargo-ndk`
4. Android Rust targets:
   ```bash
   rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
   ```

### Build Steps

The Gradle build automatically:

1. **Generates UniFFI bindings** (runs before preBuild)
   ```bash
   cd core
   cargo run --bin gen_kotlin --features gen-bindings
   ```
   Output: `core/target/generated-sources/uniffi/kotlin/uniffi/api/api.kt`

2. **Builds Rust library** for all Android ABIs (runs before preBuild)
   ```bash
   cargo ndk -t aarch64-linux-android build --release -p scmessenger-core
   cargo ndk -t armv7-linux-androideabi build --release -p scmessenger-core
   cargo ndk -t x86_64-linux-android build --release -p scmessenger-core
   cargo ndk -t i686-linux-android build --release -p scmessenger-core
   ```
   Output: `core/target/android-libs/{arm64-v8a,armeabi-v7a,x86_64,x86}/libscmessenger_core.so`

3. **Compiles Kotlin code** with generated bindings in classpath

4. **Packages APK** with JNI libraries

### Manual Build
```bash
cd android
./gradlew assembleDebug
```

### Build Output Locations
- Generated bindings: `core/target/generated-sources/uniffi/kotlin/` (ignored in git)
- JNI libraries: `core/target/android-libs/` (ignored in git)
- APK: `android/app/build/outputs/apk/debug/app-debug.apk`

## Verification

### Check Bindings Generated
```bash
ls -lh core/target/generated-sources/uniffi/kotlin/uniffi/api/api.kt
# Should show ~230KB file
```

### Check JNI Libraries Built
```bash
ls -lh core/target/android-libs/*/libscmessenger_core.so
# Should show 4 .so files (one per ABI)
```

### Verify Types Available
All Android Kotlin code expecting `uniffi.api.*` types should now compile successfully.

## Notes

1. **Generated files are not committed** - They're in `.gitignore` because they're build artifacts generated from `core/src/api.udl`.

2. **Build time** - First build takes 5-10 minutes due to Rust compilation for 4 Android ABIs.

3. **Incremental builds** - Subsequent builds are much faster if Rust code hasn't changed.

4. **CI/CD** - Ensure Rust toolchain, Android NDK, and cargo-ndk are available in build environment.

## Resolution Status

✅ **RESOLVED** - The 66 build errors were all "unresolved reference" errors for `uniffi.api.*` types. With the fixed `generateUniFFIBindings` task, these bindings are now properly generated during the build process, and all types are available to the Kotlin compiler.
