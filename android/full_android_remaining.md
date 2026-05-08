# SCMessenger Android Remaining Development & Stabilization Blueprint

This document represents the **Single Source of Truth** for the remaining development, integration, stabilization, and compliance tasks required to prepare the Android application for its Beta and production rollout. All estimates are strictly provided in **Lines of Code (LOC) Magnitudes**, in compliance with repository rules.

---

## 🛠️ 1. Compilation, SDK, & Build Stability

### 1.1 Multi-Target Android Cross-Compilation Setup
*   **Description**: Configure local and CI tooling to compile the Rust core for all target Android architectures (`aarch64-linux-android`, `armv7-linux-androideabi`, `x86_64-linux-android`) and output JNI shared libraries (`libscmessenger_core.so`) into `core/target/android-libs/{arm64-v8a, armeabi-v7a, x86_64}/`.
*   **Affected Files**:
    *   [android/app/build.gradle](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/app/build.gradle)
    *   [android/local.properties](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/local.properties)
*   **Verification**: Ensure `./gradlew assembleDebug` successfully packages JNI `.so` libraries for all targets.
*   **Estimate**: `~150 LOC` change.

### 1.2 UniFFI Kotlin Bindings Gap Analysis & Validation
*   **Description**: Regenerate UniFFI Kotlin bindings using the modern `gen_kotlin` binary and verify the generated package maps exactly to the Kotlin import statements. Detect any missing/renamed methods on the `IronCore` interface and update references in the Android codebase to resolve compilation failures.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/data/MeshRepository.kt)
    *   [core/src/api.udl](file:///c:/Users/kanal/Documents/Github/SCMessenger/core/src/api.udl)
*   **Verification**: Run `./gradlew :app:generateUniFFIBindings` followed by `./gradlew compileDebugKotlin` to ensure clean compilation.
*   **Estimate**: `~250 LOC` change.

### 1.3 Android Unit Test Pipeline Verification
*   **Description**: Establish clean execution of the Kotlin unit tests suite, ensuring no regressions in navigation policies, repository caching, or lifecycle tracking.
*   **Affected Files**:
    *   [android/app/src/test/java/com/scmessenger/android/test/MeshRepositoryTest.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/app/src/test/java/com/scmessenger/android/test/MeshRepositoryTest.kt)
    *   [android/app/src/test/java/com/scmessenger/android/test/MeshForegroundServiceTest.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/app/src/test/java/com/scmessenger/android/test/MeshForegroundServiceTest.kt)
*   **Verification**: Run `./gradlew :app:testDebugUnitTest --quiet` and verify all tests pass.
*   **Estimate**: `~100 LOC` change.

---

## 🔌 2. Core Wiring & Integration

### 2.1 SwarmBridge Integration & Internet Dial Fallback
*   **Description**: Locate and complete `SwarmBridge` initialization in `MeshRepository`. Wire `swarmBridge.dial(peerId)` for `INTERNET` transport, ensuring `SmartTransportRouter` can fall back to routing messages over cellular/WAN via the GCP relay nodes when BLE or local LAN transports are unavailable.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/data/MeshRepository.kt)
    *   [android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/transport/SmartTransportRouter.kt)
*   **Verification**: Check that sending a message over cellular utilizes the `INTERNET` fallback pathway when BLE is disabled.
*   **Estimate**: `~200 LOC` change.

### 2.2 Gossipsub Topic Management & JoinMesh UI Support
*   **Description**: Complete the initialization of `TopicManager` in `MeshRepository` upon service start. Wire `meshRepository.subscribeTopic(topic)` and `publishTopic(topic, payload)` to the gossipsub engine in the Rust core, providing support for the global, localized discovery, and relay channels.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/data/MeshRepository.kt)
    *   [android/app/src/main/java/com/scmessenger/android/ui/join/JoinMeshScreen.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/ui/join/JoinMeshScreen.kt)
*   **Verification**: Ensure users joining specific topic channels are registered in the core topology.
*   **Estimate**: `~180 LOC` change.

### 2.3 Environment-Driven Bootstrap Configuration & Failover
*   **Description**: Integrate `EnvironmentBootstrapSource` to read bootstrap node configurations from both system properties/env variables and remote JSON sources. Prioritize QUIC/UDP endpoints for cellular reliability, and implement automatic failover to alternative bootstrap relays after 3 failed connection attempts.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/data/MeshRepository.kt)
*   **Verification**: Simulate blockages on the primary GCP relay and verify the client transitions to backup bootstrap nodes.
*   **Estimate**: `~120 LOC` change.

### 2.4 MeshVpnService Routing & System Tunnel
*   **Description**: Wire the VPN toggle on the settings screen to control the lifecycle of `MeshVpnService`. Implement the Android system VPN interface configuration (`android.net.VpnService`) to intercept local IP traffic and inject it directly into the local mesh network.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/ui/settings/PowerSettingsScreen.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/ui/settings/PowerSettingsScreen.kt)
*   **Verification**: Verify the VPN permission prompt appears and that a local virtual adapter is configured upon activation.
*   **Estimate**: `~350 LOC` change.

---

## 📡 3. Bluetooth (BLE) & WiFi Aware Transport Completion

### 3.1 BLE Ingress Processing & Frame Fragmentation
*   **Description**: Wire the raw byte stream received by `BleGattServer` directly into the `MeshEventBus` for parser processing. Implement GATT payload fragmentation and reassembly (for messages exceeding the standard MTU size) to ensure complete message deliveries over BLE.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattClient.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/transport/ble/BleGattClient.kt)
    *   [android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/data/MeshRepository.kt)
*   **Verification**: Send messages larger than 512 bytes over a BLE-only connection and confirm successful delivery.
*   **Estimate**: `~220 LOC` change.

### 3.2 BLE Identity Handshake State Machine
*   **Description**: Implement a structured handshake state machine over the GATT service (Initiating → KeyExchange → Established) to share public keys and nicknames, storing them inside the Contact Manager upon establishment.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattClient.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/transport/ble/BleGattClient.kt)
*   **Verification**: Verify that two nearby devices discoverable only via BLE automatically exchange public keys and create contacts.
*   **Estimate**: `~300 LOC` change.

### 3.3 BleQuotaManager & Auto-Adjust Engine Integration
*   **Description**: Hook up the `BleQuotaManager` to monitor active scan and advertising cycles. Feed the current tracking data into `AndroidPlatformBridge.reportBleScanCount()`, allowing the core's `AutoAdjustEngine` to automatically scale down scan intervals under low power states.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/transport/ble/BleScanner.kt)
*   **Verification**: Verify that scanning interval decreases and follows the telemetry recommendations when low battery states are simulated.
*   **Estimate**: `~130 LOC` change.

### 3.4 BLE Scanning & Advertising Error Recovery
*   **Description**: Complete error recovery logic in `BleScanner` to gracefully handle `SCAN_FAILED_ALREADY_STARTED` (error code 1) and advertising failures (ANR-005). Prevent "Address type mismatch" reconnect loops for iOS peers by implementing a cooldown backoff in `BleGattClient`.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/transport/ble/BleScanner.kt)
    *   [android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattClient.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/transport/ble/BleGattClient.kt)
*   **Verification**: Trigger rapid scanning toggle operations and verify that the bluetooth stack remains stable with zero fatal state loops.
*   **Estimate**: `~180 LOC` change.

---

## 🛡️ 4. Polish, Stability, & ANR Burndown (🔴 P0 Critical)

### 4.1 UI Thread Offloading for Core Operations (ANR Elimination)
*   **Description**: Shift all blocking FFI/UniFFI calls (`uniffi.api.*`) and initial bootstrap sequences (which block for ~2s per relay) off the Main thread and onto `Dispatchers.IO`. Fix overlapping outbox flushes to resolve the critical Android ANR Storm.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/data/MeshRepository.kt)
    *   [android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/service/MeshForegroundService.kt)
*   **Verification**: Ensure no `Application Not Responding` prompts trigger during bootstrap on low-end devices.
*   **Estimate**: `~250 LOC` change.

### 4.2 Startup Crash & Order of Initialization Fix
*   **Description**: Fix startup crashes caused by `IronCoreException.NotInitialized` throwing during history/ledger syncs when started prior to full identity loading. Defer non-critical startup queries until `getIdentityInfo()` completes.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/data/MeshRepository.kt)
*   **Verification**: Perform cold restarts of the application and verify no initialization crashes occur.
*   **Estimate**: `~80 LOC` change.

### 4.3 Idempotent Contact Upserts (Contact Duplication Fix)
*   **Description**: Implement idempotent upserts with strict unique constraints on the peer's unique identifier (`peer_id`) in `MeshRepository` and the local storage mapper, stopping peer discovery from generating duplicate contact UI items.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/data/MeshRepository.kt)
*   **Verification**: Discover the same peer multiple times over different transports and verify that only a single contact item is created.
*   **Estimate**: `~80 LOC` change.

### 4.4 Auto-Backup Exclusion for Cryptographic Safety
*   **Description**: Set `android:allowBackup="false"` or configure a strict `backup_rules.xml` file to prevent the Android cloud backup service from restoring old, stale cryptographic identities and local history on clean installs.
*   **Affected Files**:
    *   [android/app/src/main/AndroidManifest.xml](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/app/src/AndroidManifest.xml)
*   **Verification**: Perform a simulated device restore and verify that stale session/ledger states are correctly excluded.
*   **Estimate**: `~20 LOC` change.

### 4.5 Startup Permission Request Debounce
*   **Description**: Eliminate the permission request loop (9+ requests inside 700ms on startup) by implementing a deduplication flag and a 500ms debounce in the initial activity permissions layout.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/ui/screens/OnboardingScreen.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/ui/screens/OnboardingScreen.kt)
*   **Verification**: Verify that permission dialogs appear sequentially with no flickering.
*   **Estimate**: `~50 LOC` change.

---

## 📦 5. Play Store Compliance & API 35 Regression Testing

### 5.1 Android 15 Background BLE Discovery Enforcement
*   **Description**: Ensure that BLE scanning and advertiser operations conform to Android 15 (API 35) background service limitations. Explicitly bind `foregroundServiceType="connectedDevice"` to `MeshForegroundService` in the manifest and ensure that scan filters are applied to all background operations.
*   **Affected Files**:
    *   [android/app/src/main/AndroidManifest.xml](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/app/src/AndroidManifest.xml)
    *   [android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/service/MeshForegroundService.kt)
*   **Verification**: Run on an Android 15 device/emulator, minimize the app, and verify that nearby peers are still discovered.
*   **Estimate**: `~90 LOC` change.

### 5.2 NsdManager Multicast and Security Exception Handling
*   **Description**: Harden local LAN mDNS discovery in `MdnsServiceDiscovery` to handle `SecurityException` blocks on Android 14+ (API 34) when registering listeners without the proper local network permissions.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/data/MeshRepository.kt)
*   **Verification**: Toggle Wi-Fi state rapidly and verify that mDNS registration recovers without throwing unhandled security exceptions.
*   **Estimate**: `~110 LOC` change.

### 5.3 Edge-to-Edge Navigation Bar & System Padding
*   **Description**: Resolve UI clipping issues by ensuring Jetpack Compose layouts use standard system and navigation bars padding (`systemBarsPadding()` or `navigationBarsPadding()`) instead of hardcoded offsets.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/ui/screens/ChatScreen.kt)
    *   [android/app/src/main/java/com/scmessenger/android/ui/screens/ConversationsScreen.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/ui/screens/ConversationsScreen.kt)
*   **Verification**: Inspect inputs on devices running with 3-button navigation and gesture layouts to confirm no overlap with system controls.
*   **Estimate**: `~80 LOC` change.

### 5.4 Firebase Crashlytics & ANR Reporting Integration
*   **Description**: Initialize Crashlytics in the application lifecycle to collect and track unhandled JNI/Kotlin exceptions, ANRs, and native Rust panics. Add custom diagnostic keys (`app_version`, `sdk_version`, `active_transports`) to trace issues.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/MeshApplication.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/app/src/main/java/com/scmessenger/android/MeshApplication.kt)
    *   [android/app/build.gradle](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/app/build.gradle)
*   **Verification**: Trigger a forced test crash and verify that reports appear in the crash console with correct metadata.
*   **Estimate**: `~60 LOC` change.

---

## 🎨 6. Feature Parity & Release UI Blocks

### 6.1 Identity QR Backup & Restore Flow
*   **Description**: Complete the identity management onboarding flow by implementing QR code generation for the encrypted identity backup payload and QR code scanning to restore the profile on a fresh installation.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/ui/screens/OnboardingScreen.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/ui/screens/OnboardingScreen.kt)
    *   [android/app/src/main/java/com/scmessenger/android/ui/screens/SettingsScreen.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/ui/screens/SettingsScreen.kt)
*   **Verification**: Export identity as a QR code, perform a clean reinstall, scan the QR code during onboarding, and verify the identity is restored.
*   **Estimate**: `~300 LOC` change.

### 6.2 Contact Key QR Sharing & Scanning
*   **Description**: Implement QR code sharing for the user's public key on the contacts tab. Update `AddContactScreen` to allow scanning a peer's QR code to automatically prefill fields (`peer_id`, `public_key`, `nickname`).
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/ui/contacts/AddContactScreen.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/ui/contacts/AddContactScreen.kt)
    *   [android/app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/ui/screens/ContactsScreen.kt)
*   **Verification**: Generate contact QR, scan on a second device, and verify fields prefill correctly.
*   **Estimate**: `~250 LOC` change.

### 6.3 Deep Link Invite Pre-filling
*   **Description**: Set up deep link parsing in the main entry activity for `scmessenger://add` and `https://scmessenger.net/add` formats. Ensure that clicking a link pre-fills the `AddContactScreen` form.
*   **Affected Files**:
    *   [android/app/src/main/AndroidManifest.xml](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/app/src/AndroidManifest.xml)
    *   [android/app/src/main/java/com/scmessenger/android/ui/MeshApp.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/ui/MeshApp.kt)
*   **Verification**: Trigger an invitation link via ADB shell and check that the app opens directly to a prefilled contact addition screen.
*   **Estimate**: `~120 LOC` change.

### 6.4 Beta-Gate Anti-Abuse Rate-Limiting Controls
*   **Description**: Wire local rate-limiting (token bucket mechanism) and peer report components to UI actions. Allow users to flag/report peers or block-and-delete contacts, which triggers a cascade history purge and block record addition.
*   **Affected Files**:
    *   [android/app/src/main/java/com/scmessenger/android/ui/contacts/ContactDetailScreen.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/ui/contacts/ContactDetailScreen.kt)
    *   [android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt](file:///c:/Users/kanal/Documents/Github/SCMessenger/android/ui/screens/ChatScreen.kt)
*   **Verification**: Perform block-and-delete on a contact and verify all history is purged and incoming messages from them are rejected.
*   **Estimate**: `~220 LOC` change.
