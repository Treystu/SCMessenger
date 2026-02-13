# SCMessenger iOS App

iOS implementation of SCMessenger — the world's first truly sovereign messenger.

## Prerequisites

### Required Software

- **macOS 14+** (Sonoma or later)
- **Xcode 15.2+** with iOS 16+ SDK
- **Rust toolchain** (stable channel)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### iOS Rust Targets

Install the required Rust targets for iOS development:

```bash
rustup target add aarch64-apple-ios        # Device (ARM64)
rustup target add aarch64-apple-ios-sim    # Simulator (Apple Silicon)
rustup target add x86_64-apple-ios         # Simulator (Intel, optional)
```

### Verify Build Setup

Run the verification script to check all prerequisites:

```bash
./ios/verify-build-setup.sh
```

This script checks:
- ✓ Rust toolchain installed
- ✓ iOS targets added
- ✓ Xcode CLI tools installed
- ✓ Project structure complete
- ✓ gen_swift binary can generate bindings
- ✓ Static library compiles

## Build Scripts

### copy-bindings.sh

Generates UniFFI Swift bindings and copies them to the iOS project:

```bash
./ios/copy-bindings.sh
```

**Outputs:**
- `ios/SCMessenger/Generated/api.swift` (~4200 lines)
- `ios/SCMessenger/Generated/apiFFI.h`
- `ios/SCMessenger/Generated/apiFFI.modulemap`

### build-rust.sh

Xcode build phase script that compiles the Rust library for the appropriate target:
- Automatically selects target: `aarch64-apple-ios` (device) or `aarch64-apple-ios-sim` (simulator)
- Handles Debug/Release builds
- Copies `libscmessenger_mobile.a` to Xcode's build directory

**Usage:** Integrated as Xcode "Run Script" build phase (runs automatically during Xcode builds)

## Architecture

```
┌─────────────────────────────────────────┐
│         SwiftUI Layer                   │
│  (Views, Navigation, Theme)             │
└──────────────┬──────────────────────────┘
               │ @Observable ViewModels
┌──────────────▼──────────────────────────┐
│       ViewModel Layer                   │
│  (State Management, Business Logic)     │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│       Repository Layer                  │
│  MeshRepository (single source of truth)│
└──────────────┬──────────────────────────┘
               │ UniFFI boundary
┌──────────────▼──────────────────────────┐
│      UniFFI Generated Swift             │
│  libscmessenger_mobile.a (static lib)   │
└──────────────┬──────────────────────────┘
               │ FFI calls
┌──────────────▼──────────────────────────┐
│          Rust Core                      │
│  Identity, Crypto, Transport, Relay     │
└─────────────────────────────────────────┘
```

## Core Principles

### 1. Relay = Messaging
Single toggle. You relay for others or you don't message. No free riders.

### 2. Every node IS the network
The iPhone IS a relay when it has internet. CoreBluetooth keeps it relaying even offline.

### 3. Internet is a transport, not a dependency
BLE, Multipeer Connectivity, and physical proximity are equal transports. Internet is just faster when available.

### 4. You ARE your keypair
No Apple ID, no phone number, no email. Ed25519 keypair generated locally, stored in app's sandboxed sled database.

### 5. Mass market UX
Grandma-friendly SwiftUI with sensible defaults. Power users get Settings → Mesh Settings → Privacy Settings.

## iOS-Specific Features

### Background Modes (Info.plist)
- `bluetooth-central` — BLE scanning in background
- `bluetooth-peripheral` — BLE advertising in background
- `fetch` — Background fetch for mesh sync
- `processing` — BGProcessingTask for bulk operations

### Transport Stack
1. **Multipeer Connectivity** (WiFi Direct equivalent) — Primary
2. **CoreBluetooth** (BLE) — Fallback for close-range mesh
3. **Internet** (libp2p TCP/QUIC) — When available, fastest

### Background Strategy
iOS has no persistent foreground service like Android. Uses:
- `BGTaskScheduler` for periodic wakeups
- CoreBluetooth background modes for mesh keepalive
- Location services for optional background triggers

Implementation: `core/src/mobile/ios_strategy.rs` (521 lines, 22 tests)

## Project Status

### Phase 1: UniFFI Swift Bindings ✅ COMPLETE
- gen_swift binary created
- 4200 lines of Swift bindings generated
- All 11 interfaces, 11 structs, 6 enums verified

### Phase 2: Xcode Project Scaffolding (In Progress)
- [x] `verify-build-setup.sh` — Build prerequisites verification
- [x] `build-rust.sh` — Xcode build phase script
- [x] `copy-bindings.sh` — Bindings generation script
- [ ] Xcode project creation
- [ ] Info.plist configuration
- [ ] Initial app structure

### Upcoming Phases
- Phase 3: Background Service & Lifecycle (~650 LoC)
- Phase 4: CoreBluetooth Transport (~900 LoC)
- Phase 5: Multipeer Connectivity (~400 LoC)
- Phase 6: Data Repository Layer (~600 LoC)
- Phase 7-13: UI Layers (~4900 LoC)
- Phase 14: Integration Testing (~500 LoC)
- Phase 15: Gossipsub Topic Integration (~550 LoC)

**Total Estimated:** ~8,840 LoC

## Android ↔ iOS Parity

| Android | iOS | Notes |
|---------|-----|-------|
| Kotlin + Jetpack Compose | Swift + SwiftUI | Declarative UI on both |
| Hilt DI | @Observable + @Environment | No Hilt needed |
| Foreground Service | BGTaskScheduler + BLE background | Different lifecycle model |
| BLE API | CoreBluetooth | Same purpose, different API |
| WiFi Aware | Multipeer Connectivity | Apple's mesh equivalent |
| WiFi Direct | Multipeer Connectivity | Same framework |
| cargo-ndk | Native Xcode build phase | Different build integration |
| DataStore | UserDefaults/@AppStorage | Native persistence |
| Timber | os.Logger + Rust tracing | Logging |

## Documentation

- **Full iOS Plan:** `iOS/iosdesign.md` (4523 lines, 15 phases)
- **iOS Strategy (Rust):** `core/src/mobile/ios_strategy.rs`
- **UniFFI Spec:** `core/src/api.udl` (373 lines, shared with Android)

## Build & Run

### First Time Setup
1. Verify prerequisites: `./ios/verify-build-setup.sh`
2. Copy bindings: `./ios/copy-bindings.sh`
3. Open `ios/SCMessenger.xcodeproj` in Xcode
4. Select target device/simulator
5. Build and run (⌘R)

### Subsequent Builds
Just build in Xcode. The `build-rust.sh` script runs automatically as a build phase.

### Clean Build
If you encounter issues:
```bash
cd mobile
cargo clean
cd ../ios
# In Xcode: Product → Clean Build Folder (Shift+⌘K)
```

## License

Same as repository root
