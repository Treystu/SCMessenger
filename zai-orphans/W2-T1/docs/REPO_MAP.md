# SCMessenger Repository Map

This document provides a dense, navigational reference mapping the entire SCMessenger repository. It outlines the purpose of each code area, identifies key entry-point files, and describes how components interface with the Rust core.

---

## 1. Rust Core (core/)
Unified cryptographic, persistence, transport routing, and relay service engine for sovereign communications.
- **Connection to Core**: This is the core library itself. Other platforms connect via UniFFI JNI/Swift bindings, JSON-RPC, or direct cargo dependencies.
- **Key Modules & Entry Points**:
  - **Identity**: [keys.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/core/src/identity/keys.rs) - Manages Ed25519 identity keypairs, backup generation, and cryptographic signatures.
  - **Crypto**: [ratchet.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/core/src/crypto/ratchet.rs) - Implements XChaCha20-Poly1305 message encryption and X25519 double-ratchet session progression.
  - **Crypto Session Manager**: [session_manager.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/core/src/crypto/session_manager.rs) - Manages the lifecycle of encrypted sessions and cryptographic key exchanges.
  - **Transport Swarm**: [swarm.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/core/src/transport/swarm.rs) - The libp2p network event loop driving connection state, packet routing, and multi-transport mesh behaviors.
  - **Multiport Ladder**: [multiport.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/core/src/transport/multiport.rs) - Handles automatic port binding, traversal, and local service socket selection.
  - **Routing Engine**: [engine.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/core/src/routing/engine.rs) - Determines optimal message paths using multi-path hop metrics, TTL adjustments, and latency detection.
  - **Onion Routing**: [onion.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/core/src/privacy/onion.rs) - Encapsulates payloads inside onion-routed packets and executes multi-hop payload peeling.
  - **Cover Traffic**: [cover.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/core/src/privacy/cover.rs) - Schedules background cover (dummy) traffic to obscure active metadata pathways.
  - **Store Backend**: [backend.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/core/src/store/backend.rs) - Abstracted engine managing the sled transactional key-value database.
  - **Contacts Storage**: [contacts.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/core/src/store/contacts.rs) - Manages database queries for peer details, seniority, blocklists, and discovery states.
  - **Protocol Framing**: [envelope.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/core/src/drift/envelope.rs) - Governs binary framing, metadata signing, serialization, and DTN-style store-and-carry policies.
  - **Relay Client**: [client.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/core/src/relay/client.rs) - Interacts with bootstrap nodes to retrieve queued offline message envelopes from relays.
  - **Relay Server**: [server.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/core/src/relay/server.rs) - Manages hosted mailboxes for transient store-and-forward peer buffering.
  - **WASM JSON-RPC Bridge**: [rpc.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/core/src/wasm_support/rpc.rs) - Serializes and deserializes WebSocket JSON-RPC command interfaces for standard browser nodes.
  - **Core Entry API**: [iron_core.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/core/src/iron_core.rs) - The primary system coordinator wrapping the database, swarm, and session states under thread-safe locks.
  - **FFI Boundary**: [api.udl](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/core/src/api.udl) - UniFFI interface definition language configuration for multi-platform bindings.

## 2. Command Line Interface (cli/)
Cross-platform terminal utility, background daemon, and HTTP/WebSocket management interface.
- **Connection to Core**: Compiles directly against `scmessenger-core`, initializing an [IronCore](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/core/src/iron_core.rs) node in memory.
- **Key Modules & Entry Points**:
  - **Daemon Runner**: [main.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/main.rs) - Parses input commands (start/stop/send/list) and runs the persistent background process.
  - **Web/RPC Server**: [server.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/server.rs) - Spawns local HTTP and WebSocket servers (defaulting to port 9002) for external JSON-RPC requests.
  - **Transport Bridge**: [transport_bridge.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/transport_bridge.rs) - Connects OS-native network controllers to the core libp2p network manager.
  - **BLE Daemon**: [ble_daemon.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/ble_daemon.rs) - Integrates native Windows/Linux Bluetooth scanning libraries into the CLI runtime.

## 3. WebAssembly Layer (wasm/)
Browser-target modules compiling the core to JS/WASM for direct browser execution.
- **Connection to Core**: Compiles `scmessenger-core` with specific features targeting the `wasm32-unknown-unknown` environment.
- **Key Modules & Entry Points**:
  - **JS Export Surface**: [lib.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/wasm/src/lib.rs) - Exposes `wasm-bindgen` classes, methods, and configurations to frontend Web applications.
  - **WebSocket Bridge**: [daemon_bridge.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/wasm/src/daemon_bridge.rs) - Sends JSON-RPC payloads over WebSocket connections when running as a thin client.
  - **Browser Transport**: [transport.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/wasm/src/transport.rs) - Bridges libp2p routing to WebRTC and WebSocket connection channels inside the browser.
  - **Worker Manager**: [worker.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/wasm/src/worker.rs) - Spawns a background Web Worker thread to isolate memory and network processing from the main UI thread.

## 4. Desktop Bridge (desktop_bridge/)
Integrations with Linux desktop environments including system tray, XDG dirs, and DBus services.
- **Connection to Core**: Standalone crate compiling to a native library; uses UniFFI to bridge system hooks to other wrappers.
- **Key Modules & Entry Points**:
  - **Scaffolding Root**: [lib.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/desktop_bridge/src/lib.rs) - Initiates UniFFI scaffolding and determines XDG directory paths.
  - **Desktop Service Manager**: [desktop_bridge.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/desktop_bridge/src/desktop_bridge.rs) - Coordinates notification overlays, tray icons, and energy configurations.
  - **BLE DBus Hook**: [ble.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/desktop_bridge/src/ble.rs) - Leverages zbus on Linux to run low-level Bluetooth advertising and scanning.

## 5. Headless Client (headless/)
Minimal testing harness for verifying WASM engine behavior and notifications.
- **Connection to Core**: Direct script importing of the packaged JS/WASM node compiled in `wasm/pkg`.
- **Key Modules & Entry Points**:
  - **Client Controller**: [main.js](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/headless/main.js) - Sets up web-page UI listeners, initializes the WASM mesh, and drives peer connections.
  - **Page Layout**: [index.html](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/headless/index.html) - Structural framework display for logging mesh state changes and notifications.

## 6. Android App (android/app/src)
Native Android mobile client constructed with Jetpack Compose.
- **Connection to Core**: Uses Kotlin classes generated from `mobile/src/lib.rs` and `core/src/api.udl` via UniFFI, compiling JNI libraries directly into the APK.
- **Key Modules & Entry Points**:
  - **Mesh Coordinator**: [MeshRepository.kt](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt) - Manages background locks, thread execution, and Rust FFI callback mapping.
  - **BLE Transport Client**: [BleGattClient.kt](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattClient.kt) - Coordinates peripheral GATT read/write pipelines for Bluetooth transport.
  - **BLE Transport Server**: [BleGattServer.kt](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt) - Manages advertising and processing inbound GATT requests.
  - **Platform Bridge Callback**: [AndroidPlatformBridge.kt](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt) - Dispatches Android device telemetry, battery logs, and state signals into the Rust core.
  - **Foreground Service**: [MeshForegroundService.kt](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt) - Keeps background connections active via Android system service notifications.
  - **Contacts View Model**: [ContactsViewModel.kt](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ContactsViewModel.kt) - Maps database peer models to Kotlin UI flows.
  - **Conversations Screen**: [ConversationsScreen.kt](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/android/app/src/main/java/com/scmessenger/android/ui/screens/ConversationsScreen.kt) - Lists recent messages, chat history previews, and peer discovery statuses.

## 7. iOS App (iOS/SCMessenger)
Native iOS mobile client application built with Swift and SwiftUI.
- **Connection to Core**: Imports `SCMessengerCore.xcframework` static framework containing UniFFI Swift generated bridge files.
- **Key Modules & Entry Points**:
  - **Mesh Coordinator**: [MeshRepository.swift](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift) - Lifecycle manager wrapping the Rust UniFFI interface.
  - **BLE Central Manager**: [BLECentralManager.swift](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Transport/BLECentralManager.swift) - Orchestrates BLE peripheral scanning and client connection states on iOS.
  - **BLE Peripheral Manager**: [BLEPeripheralManager.swift](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Transport/BLEPeripheralManager.swift) - Advertises mesh availability and streams incoming payload fragments.
  - **Platform Bridge Callback**: [IosPlatformBridge.swift](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Services/IosPlatformBridge.swift) - Maps iOS hardware states (network pathing, battery, motion) to Rust.
  - **Contacts View Model**: [ContactsViewModel.swift](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/ViewModels/ContactsViewModel.swift) - Formats SwiftUI models for screen bindings.

## 8. Mobile Bridge Wrapper (mobile/)
Standalone library layer exporting Rust core structures for mobile compilers.
- **Connection to Core**: Direct Rust dependency importing the `scmessenger-core` library.
- **Key Modules & Entry Points**:
  - **FFI Exporter**: [lib.rs](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/mobile/src/lib.rs) - Defines integration test suites and re-exports symbols targeted by the UniFFI compiler build script.

## 9. Shared KMP Library (shared/)
Kotlin Multiplatform shared UI and state wrapper.
- **Connection to Core**: Currently provides stubs for multi-platform UI components.
- **Key Modules & Entry Points**:
  - **Compose UI Root**: [SharedApp.kt](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/shared/src/commonMain/kotlin/com/scmessenger/shared/SharedApp.kt) - Houses shared Compose logic for multiplatform Kotlin runtimes.

## 10. Cloud Mesh Testbed (cloud/)
Simulations, profiles, Docker containers, and Terraform provisioning engines.
- **Connection to Core**: Manages virtual clusters of `scmessenger-cli` instances using WireGuard networks.
- **Key Modules & Entry Points**:
  - **Orchestration API**: [main.py](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/cloud/orchestrator/main.py) - Spawns workers, routes test scripts, and registers client topologies.
  - **CLI Docker Config**: [Dockerfile.cli](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/cloud/mesh/Dockerfile.cli) - Packages the Rust daemon build for virtual node deployment.
  - **Cloud Infrastructure**: [main.tf](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/cloud/terraform/main.tf) - Terraform scripts provisioning worker clusters in GCP.

## 11. AWS Infrastructure (infra/)
AWS configuration files and scripts for running bootstrap nodes and global relays.
- **Connection to Core**: Sets up remote cloud infrastructure running the `scmessenger-cli` daemon in headless relay mode.
- **Key Modules & Entry Points**:
  - **EC2 Installer**: [provision-relay.sh](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/infra/aws/provision-relay.sh) - Automates the configuration, installation, and deployment of SCMessenger relay daemons.
  - **Relay IAM Policy**: [iam-policy-scmessenger-relay.json](file:///C:/Users/SCM/Documents/GitHub/SCMessenger/infra/aws/iam-policy-scmessenger-relay.json) - Restricts relay nodes' access to external cloud assets.
