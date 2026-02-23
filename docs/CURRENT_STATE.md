# SCMessenger Current State (Verified)

Last verified: **2026-02-23** (local workspace checks on this machine)

## Verified Commands and Results

### Rust Workspace

- `cargo test --workspace`
  - Result: **pass**
  - Totals from suite output:
    - CLI: 17 passed
    - Core unit: 227 passed, 7 ignored
    - Core integration: 52 passed
    - Mobile crate: 4 passed
    - WASM crate (native/non-browser tests): 24 passed
  - Aggregate: **324 passed, 0 failed, 7 ignored**

### CLI Surface

- `cargo run -p scmessenger-cli -- --help`
  - Verified commands: `init`, `identity`, `contact`, `config`, `history`, `start`, `send`, `status`, `stop`, `test`

### Platform Build Readiness Scripts

- `./android/verify-build-setup.sh`
  - Result: **pass** (with `ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk`)
- `./iOS/verify-build-setup.sh`
  - Result: **pass**
  - Confirmed Swift bindings generation and static library build

### Platform App Builds

- Android:
  - `cd android && ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk ./gradlew assembleDebug`
  - Result: **pass**
- iOS:
  - `xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -destination 'platform=iOS Simulator,name=iPhone 17' build`
  - Result: **pass**

### Browser/WASM Runtime Validation

- `wasm-pack --version`
  - Result: **not available** (`wasm-pack` not installed in this environment)
  - Note: browser runtime tests were not executed here

## Implemented Functionality (Repository State)

- Sovereign identity and key management (Ed25519), persisted storage
- Message encryption/signing pipeline (X25519 + XChaCha20-Poly1305 + signatures)
- Store-and-forward queues with persistence
- libp2p swarm transport with discovery, messaging, relay, and NAT reflection
- Interactive CLI with:
  - contact and history management
  - live node mode
  - local control API
  - embedded web landing/dashboard server
- Mobile UniFFI surface (MeshService, SwarmBridge, managers, settings)
- iOS and Android app codebases with active integration to Rust core
- WASM crate with WebSocket/WebRTC abstractions and native stub-path tests

## Known Gaps and Partial Areas

### Product/Feature Gaps

- Topic subscribe/unsubscribe/publish is now wired through Rust bridge on Android and iOS
  - Android: `android/app/src/main/java/com/scmessenger/android/data/TopicManager.kt`
  - iOS: `iOS/SCMessenger/SCMessenger/Data/TopicManager.swift`
- iOS privacy toggles are intentionally disabled in UI pending core toggle APIs
  - `iOS/SCMessenger/SCMessenger/ViewModels/SettingsViewModel.swift`
- Android and iOS QR import/join flows are wired (Google Code Scanner on Android, VisionKit on iOS)
  - Android: `android/app/src/main/java/com/scmessenger/android/ui/join/JoinMeshScreen.kt`
  - Android contacts: `android/app/src/main/java/com/scmessenger/android/ui/contacts/AddContactScreen.kt`
  - iOS join: `iOS/SCMessenger/SCMessenger/Views/Topics/JoinMeshView.swift`
  - iOS contacts: `iOS/SCMessenger/SCMessenger/Views/Contacts/ContactsListView.swift`
- Android and iOS can generate identity QR codes from full identity export payloads (ID, public key, nickname, libp2p peer ID, listeners, relay)
  - Android identity QR: `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityScreen.kt`
  - Android export source: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
  - iOS identity QR: `iOS/SCMessenger/SCMessenger/Views/Settings/SettingsView.swift`
  - iOS export source: `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`
- Android `WifiAwareTransport` compile issue was fixed; runtime behavior still needs field validation across devices/NAT scenarios
  - `android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt`

### Operational/Test Coverage Gaps

- Browser-executed WASM tests are not currently verified in this environment (`wasm-pack` missing)
- Android build verification requires `ANDROID_HOME` to be set in-shell

### Code Quality Gaps

- Several integration tests compile with warnings (unused imports/variables in core integration test files)

## Source of Truth

Use this file plus:

- `README.md` (repo entrypoint)
- `docs/TESTING_GUIDE.md` (test commands and expected outcomes)
- `REMAINING_WORK_TRACKING.md` (active gap backlog)

Treat older status and audit report docs as historical snapshots unless they are explicitly linked from the three files above as current.
