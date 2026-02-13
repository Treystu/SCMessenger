SCMessenger iOS App — Master Comprehensive Guide
Table of Contents
Philosophy & Parity Goals
Architecture Overview
Prerequisites & Toolchain
Phase 1: UniFFI Swift Bindings (Rust Side)
Phase 2: Xcode Project Scaffolding
Phase 3: Background Service & Lifecycle
Phase 4: CoreBluetooth Transport Bridge
Phase 5: Multipeer Connectivity (WiFi Direct/Aware Equivalent)
Phase 6: Data Repository Layer
Phase 7: Identity & Onboarding UI
Phase 8: Contacts UI
Phase 9: Messaging UI
Phase 10: Mesh Network Dashboard
Phase 11: Settings Screens
Phase 12: Notifications
Phase 13: Navigation & Theme
Phase 14: Integration Testing
Phase 15: Gossipsub Topic Integration
Complete File Manifest
Build & Run Checklist
Model Selection Per Phase
1. Philosophy & Parity Goals
SCMessenger is the world's first truly sovereign messenger. The iOS app must uphold every core principle:

Relay = Messaging. Single toggle. You relay for others or you don't message. No free riders. The iOS toggle maps to MeshSettings.relay_enabled — when relay is off, messaging is off.
Every node IS the network. The iPhone IS a relay when it has internet. CoreBluetooth central+peripheral keeps it relaying even offline.
Internet is a transport, not a dependency. BLE, Multipeer Connectivity, and physical proximity are equal transports. libp2p TCP/QUIC is used when internet is available.
You ARE your keypair. No Apple ID dependency, no phone number, no email. Ed25519 keypair generated locally in Rust, stored in the app's sandboxed sled database.
Mass market UX. Grandma-friendly. SwiftUI with sensible defaults. Power users get Settings → Mesh Settings → Privacy Settings → Power Settings.
Android ↔ iOS Parity Matrix
Android Component	iOS Equivalent	Notes
Kotlin + Jetpack Compose	Swift + SwiftUI	Declarative UI on both
Hilt DI	Swift @Observable + manual DI via @Environment	No Hilt equivalent needed; Swift's property wrappers suffice
Foreground Service	BGTaskScheduler + BLE background modes	iOS has no persistent foreground service; use background modes
BLE (Android BLE API)	CoreBluetooth (CBCentralManager/CBPeripheralManager)	Already modeled in core/src/mobile/ios_strategy.rs
WiFi Aware	Multipeer Connectivity Framework	Apple's equivalent; different API but same mesh purpose
WiFi Direct	Multipeer Connectivity Framework	MCSession handles both WiFi and peer-to-peer
cargo-ndk (cross-compile)	cargo-lipo / universal binary	Targets: aarch64-apple-ios, aarch64-apple-ios-sim
JNA (UniFFI loading)	Direct static linking (.a)	iOS links libscmessenger_mobile.a directly
Gradle tasks	Xcode Build Phases (Run Script)	Shell scripts in Xcode build phases
DataStore (preferences)	UserDefaults / @AppStorage	Native iOS persistence
Timber (logging)	os.Logger / tracing via Rust	Rust tracing for core, os.Logger for Swift
BootReceiver	Not applicable	iOS doesn't auto-start apps on boot
MeshVpnService	NEPacketTunnelProvider (if needed)	Only if VPN-like functionality is required
2. Architecture Overview
Code
┌─────────────────────────────────────────┐
│         SwiftUI Layer                   │
│  (Views, Navigation, Theme)             │
│  ConversationsView, ContactsView,       │
│  DashboardView, SettingsView, ChatView  │
└──────────────┬──────────────────────────┘
               │ @Observable ViewModels
┌──────────────▼──────────────────────────┐
│       ViewModel Layer                   │
│  (State Management, Business Logic)     │
│  ChatVM, ContactsVM, DashboardVM,       │
│  SettingsVM, MeshServiceVM, IdentityVM  │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│       Repository Layer                  │
│  MeshRepository (single source of truth)│
│  - IronCore (crypto/identity)           │
│  - MeshService (lifecycle)              │
│  - ContactManager (contacts CRUD)       │
│  - HistoryManager (message history)     │
│  - LedgerManager (connection ledger)    │
│  - MeshSettingsManager (settings)       │
│  - AutoAdjustEngine (power tuning)      │
│  - SwarmBridge (network operations)     │
└──────────────┬──────────────────────────┘
               │ UniFFI boundary (Swift ↔ Rust)
┌──────────────▼──────────────────────────┐
│      UniFFI Generated Swift             │
│  (from api.udl → Swift bindings)        │
│  libscmessenger_mobile.a (static lib)   │
└──────────────┬──────────────────────────┘
               │ FFI calls
┌──────────────▼──────────────────────────┐
│          Rust Core                      │
│  Identity (Ed25519), Crypto (XChaCha20) │
│  Message, Store, Transport (libp2p)     │
│  Drift Protocol, Routing, Relay         │
│  Privacy (onion, cover, padding)        │
└─────────────────────────────────────────┘
iOS-Specific Transport Stack
Code
┌──────────────────────────────────────────┐
│         TransportManager.swift           │
│  (Orchestrates all transports)           │
├──────────────────────────────────────────┤
│  CoreBluetoothTransport                  │
│  ├─ CBCentralManager (scanning/connect)  │
│  ├─ CBPeripheralManager (advertising)    │
│  ├─ GATT Services (data exchange)        │
│  └─ L2CAP Channels (bulk transfer)       │
├──────────────────────────────────────────┤
│  MultipeerTransport                      │
│  ├─ MCNearbyServiceAdvertiser            │
│  ├─ MCNearbyServiceBrowser               │
│  └─ MCSession (data/stream)              │
├──────────────────────────────────────────┤
│  InternetTransport                       │
│  └─ SwarmBridge (libp2p via UniFFI)      │
└──────────────────────────────────────────┘

Priority: Multipeer > BLE > Internet
(Multipeer uses WiFi Direct when available, falls back to BLE)
3. Prerequisites & Toolchain
Required Software
macOS 14+ (Sonoma) — required for latest Xcode
Xcode 15.2+ — Swift 5.9+, iOS 16+ SDK
Rust toolchain — rustup, stable channel
iOS targets:
bash
rustup target add aarch64-apple-ios        # Device (ARM64)
rustup target add aarch64-apple-ios-sim    # Simulator (Apple Silicon)
rustup target add x86_64-apple-ios         # Simulator (Intel, optional)
gen_swift Binary
Mirror core/src/bin/gen_kotlin.rs for Swift. Create core/src/bin/gen_swift.rs:

Rust
// core/src/bin/gen_swift.rs
#[cfg(feature = "gen-bindings")]
fn main() {
    use camino::Utf8Path;
    use uniffi::SwiftBindingGenerator;

    let udl_file = Utf8Path::new("src/api.udl");
    let out_dir = Utf8Path::new("target/generated-sources/uniffi/swift");

    uniffi_bindgen::generate_bindings(
        udl_file,
        None,
        SwiftBindingGenerator,
        Some(out_dir),
        None,
        None,
        false,
    )
    .unwrap();
}

#[cfg(not(feature = "gen-bindings"))]
fn main() {}
~20 LoC. Add to core/Cargo.toml under [[bin]]:

TOML
[[bin]]
name = "gen_swift"
path = "src/bin/gen_swift.rs"
required-features = ["gen-bindings"]
Run it:

bash
cd core
cargo run --bin gen_swift --features gen-bindings
# Outputs to: core/target/generated-sources/uniffi/swift/
# Generates: api.swift (Swift bindings) + apiFFI.h (C header) + apiFFI.modulemap
Build the Static Library
bash
cd mobile

# Device build
cargo build --release --target aarch64-apple-ios

# Simulator build (Apple Silicon)
cargo build --release --target aarch64-apple-ios-sim

# Create XCFramework (combines device + simulator)
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libscmessenger_mobile.a \
  -headers core/target/generated-sources/uniffi/swift/ \
  -library target/aarch64-apple-ios-sim/release/libscmessenger_mobile.a \
  -headers core/target/generated-sources/uniffi/swift/ \
  -output SCMessengerCore.xcframework
Verify Build Script (ios/verify-build-setup.sh)
Mirror android/verify-build-setup.sh — checks:

Rust toolchain installed
iOS targets added (aarch64-apple-ios, aarch64-apple-ios-sim)
Xcode CLI tools installed (xcode-select -p)
gen_swift binary can generate bindings from api.udl
Static library compiles for at least one target
~120 LoC.

Phase 1: UniFFI Swift Bindings (Rust Side)
Goal: Expose all 11 UniFFI interfaces to Swift, matching Android parity exactly.

LoC: ~40 (Rust-side additions)

What Already Exists (Shared With Android)
The entire Rust side is already done. The api.udl (373 lines) defines everything both platforms consume:

UDL Interface	Swift Type Generated	Purpose
IronCore	IronCore class	Lifecycle, identity, crypto, messaging
CoreDelegate	CoreDelegate protocol	Peer/message event callbacks
MeshService	MeshService class	Mobile service lifecycle
PlatformBridge	PlatformBridge protocol	Battery, network, BLE callbacks
AutoAdjustEngine	AutoAdjustEngine class	Power-aware parameter tuning
MeshSettingsManager	MeshSettingsManager class	Settings CRUD
ContactManager	ContactManager class	Contact CRUD
HistoryManager	HistoryManager class	Message history
LedgerManager	LedgerManager class	Connection tracking
SwarmBridge	SwarmBridge class	Network operations (send, dial, subscribe)
UniFFI Dictionaries → Swift Structs
UDL Dictionary	Swift Struct	Fields
IdentityInfo	IdentityInfo	identityId: String?, publicKeyHex: String?, initialized: Bool
SignatureResult	SignatureResult	signature: Data, publicKeyHex: String
MeshServiceConfig	MeshServiceConfig	discoveryIntervalMs: UInt32, relayBudgetPerHour: UInt32, batteryFloorPct: UInt8
ServiceStats	ServiceStats	peersDiscovered: UInt32, messagesRelayed: UInt32, bytesTransferred: UInt64, uptimeSecs: UInt64
DeviceProfile	DeviceProfile	batteryPct: UInt8, isCharging: Bool, hasWifi: Bool, motionState: MotionState
BleAdjustment	BleAdjustment	scanIntervalMs: UInt32, advertiseIntervalMs: UInt32, txPowerDbm: Int8
RelayAdjustment	RelayAdjustment	maxPerHour: UInt32, priorityThreshold: UInt8, maxPayloadBytes: UInt32
MeshSettings	MeshSettings	relayEnabled: Bool, maxRelayBudget: UInt32, batteryFloor: UInt8, bleEnabled: Bool, wifiAwareEnabled: Bool, wifiDirectEnabled: Bool, internetEnabled: Bool, discoveryMode: DiscoveryMode, onionRouting: Bool
Contact	Contact	peerId: String, nickname: String?, publicKey: String, addedAt: UInt64, lastSeen: UInt64?, notes: String?
MessageRecord	MessageRecord	id: String, direction: MessageDirection, peerId: String, content: String, timestamp: UInt64, delivered: Bool
HistoryStats	HistoryStats	totalMessages: UInt32, sentCount: UInt32, receivedCount: UInt32, undeliveredCount: UInt32
LedgerEntry	LedgerEntry	multiaddr: String, peerId: String?, successCount: UInt32, failureCount: UInt32, lastSeen: UInt64?, topics: [String]
UniFFI Enums → Swift Enums
UDL Enum	Swift Enum	Variants
IronCoreError	IronCoreError (Error)	.notInitialized, .alreadyRunning, .storageError, .cryptoError, .networkError, .invalidInput, .internal
ServiceState	ServiceState	.stopped, .starting, .running, .stopping
MotionState	MotionState	.still, .walking, .running, .automotive, .unknown
AdjustmentProfile	AdjustmentProfile	.maximum, .high, .standard, .reduced, .minimal
DiscoveryMode	DiscoveryMode	.normal, .cautious, .paranoid
MessageDirection	MessageDirection	.sent, .received
Steps (Execution Order)
Create core/src/bin/gen_swift.rs (20 LoC) — mirrors gen_kotlin.rs with SwiftBindingGenerator
Add [[bin]] entry to core/Cargo.toml (3 LoC)
Run: cargo run --bin gen_swift --features gen-bindings
Verify output: core/target/generated-sources/uniffi/swift/api.swift, apiFFI.h, apiFFI.modulemap
Verify all 11 interfaces appear in generated api.swift
Phase 2: Xcode Project Scaffolding
Goal: Create a buildable Xcode project that links the Rust static library and UniFFI Swift bindings.

LoC: ~500 (project config, scripts, resources)

Project Structure
Code
ios/
├── SCMessenger.xcodeproj/
│   └── project.pbxproj
├── SCMessenger/
│   ├── SCMessengerApp.swift              # @main entry point
│   ├── Info.plist                        # Permissions, background modes
│   ├── Assets.xcassets/                  # App icons, colors
│   ├── Bridging-Header.h                # #import "apiFFI.h"
│   ├── Generated/
│   │   └── api.swift                    # UniFFI generated (copied from core/target/)
│   ├── Models/                          # App-level models
│   ├── ViewModels/                      # @Observable ViewModels
│   ├── Views/                           # SwiftUI views
│   │   ├── Conversations/
│   │   ├── Contacts/
│   │   ├── Dashboard/
│   │   ├── Settings/
│   │   ├── Chat/
│   │   ├── Identity/
│   │   ├── Onboarding/
│   │   └── Components/
│   ├── Services/                        # Background, platform bridge
│   ├── Transport/                       # BLE, Multipeer
│   ├── Data/                            # Repository, persistence
│   └── Utils/                           # Helpers
├── SCMessengerTests/
│   └── *.swift
├── build-rust.sh                        # Xcode build phase script
├── copy-bindings.sh                     # Copy generated Swift bindings
└── verify-build-setup.sh               # Prerequisites check
Info.plist — Required Keys
XML
<!-- Background Modes -->
<key>UIBackgroundModes</key>
<array>
    <string>bluetooth-central</string>       <!-- BLE scanning in background -->
    <string>bluetooth-peripheral</string>    <!-- BLE advertising in background -->
    <string>fetch</string>                   <!-- Background fetch -->
    <string>processing</string>              <!-- BGProcessingTask -->
</array>

<!-- Bluetooth -->
<key>NSBluetoothAlwaysUsageDescription</key>
<string>SCMessenger uses Bluetooth to discover and communicate with nearby mesh nodes.</string>

<!-- Local Network (Multipeer Connectivity) -->
<key>NSLocalNetworkUsageDescription</key>
<string>SCMessenger uses the local network to find nearby mesh nodes via WiFi.</string>
<key>NSBonjourServices</key>
<array>
    <string>_scmesh._tcp</string>           <!-- Bonjour service type for Multipeer -->
</array>

<!-- Location (optional, for background keepalive) -->
<key>NSLocationWhenInUseUsageDescription</key>
<string>Location helps SCMessenger optimize mesh routing for nearby peers.</string>

<!-- Camera (QR code scanning for contact exchange) -->
<key>NSCameraUsageDescription</key>
<string>SCMessenger uses the camera to scan QR codes for adding contacts.</string>

<!-- Notifications -->
<key>NSUserNotificationsUsageDescription</key>
<string>SCMessenger sends notifications when new messages arrive.</string>
build-rust.sh — Xcode Build Phase Script
bash
#!/bin/bash
# Called by Xcode "Run Script" build phase
set -e

cd "${SRCROOT}/../mobile"

if [ "$PLATFORM_NAME" = "iphonesimulator" ]; then
    if [ "$(uname -m)" = "arm64" ]; then
        RUST_TARGET="aarch64-apple-ios-sim"
    else
        RUST_TARGET="x86_64-apple-ios"
    fi
else
    RUST_TARGET="aarch64-apple-ios"
fi

if [ "$CONFIGURATION" = "Release" ]; then
    RUST_PROFILE="--release"
    RUST_DIR="release"
else
    RUST_PROFILE=""
    RUST_DIR="debug"
fi

cargo build $RUST_PROFILE --target "$RUST_TARGET"

# Copy library to Xcode's expected location
mkdir -p "${CONFIGURATION_BUILD_DIR}"
cp "target/${RUST_TARGET}/${RUST_DIR}/libscmessenger_mobile.a" \
   "${CONFIGURATION_BUILD_DIR}/libscmessenger_mobile.a"
~40 LoC.

copy-bindings.sh
bash
#!/bin/bash
set -e
cd "$(dirname "$0")/.."
cargo run --bin gen_swift --features gen-bindings --manifest-path core/Cargo.toml
cp core/target/generated-sources/uniffi/swift/api.swift ios/SCMessenger/Generated/api.swift
cp core/target/generated-sources/uniffi/swift/apiFFI.h ios/SCMessenger/Generated/apiFFI.h
echo "Swift bindings copied successfully"
~10 LoC.

Bridging-Header.h
objc
// SCMessenger-Bridging-Header.h
#import "apiFFI.h"
1 LoC.

Xcode Project Settings
Setting	Value
Deployment Target	iOS 16.0
Swift Language Version	5.9
Library Search Paths	$(CONFIGURATION_BUILD_DIR)
Other Linker Flags	-lscmessenger_mobile
Bridging Header	SCMessenger/Bridging-Header.h
Architectures	arm64
Build Active Architecture Only	Yes (Debug), No (Release)
Steps (Execution Order)
Create Xcode project via File → New → Project → App (SwiftUI, Swift, no Core Data, no tests initially — add test target separately)
Add build-rust.sh as Run Script build phase (before "Compile Sources")
Add copy-bindings.sh as a pre-build script (or run manually before first build)
Add Bridging-Header.h, set in Build Settings
Add api.swift to the project (Generated/ group)
Configure Info.plist with all permission keys and background modes
Add libscmessenger_mobile.a to "Link Binary With Libraries" build phase
Set Library Search Paths
Build and verify it compiles (empty app with Rust linked)
Phase 3: Background Service & Lifecycle
Goal: Keep the mesh alive when the app is backgrounded. This is the most iOS-specific phase — no equivalent to Android's foreground service exists. Must use iOS background modes strategically.

LoC: ~650

iOS Background Strategy (Already in Rust)
The Rust core already has core/src/mobile/ios_strategy.rs (521 lines, 22 tests) which models:

BackgroundMode enum: BluetoothCentral, BluetoothPeripheral, Location, BackgroundFetch, BackgroundProcessing
IosBackgroundConfig with validation
IosBackgroundStrategy orchestrator
CoreBluetoothState tracking
The Swift side must implement the actual iOS APIs that this Rust strategy models.

Files to Create
Services/MeshBackgroundService.swift (~200 LoC)
Swift
import BackgroundTasks
import os

/// Manages all iOS background execution strategies
/// iOS equivalent of Android's MeshForegroundService
@Observable
final class MeshBackgroundService {
    private let logger = Logger(subsystem: "com.scmessenger", category: "Background")
    private let meshRepository: MeshRepository

    // BGTask identifiers
    static let refreshTaskId = "com.scmessenger.mesh.refresh"
    static let processingTaskId = "com.scmessenger.mesh.processing"

    init(meshRepository: MeshRepository) {
        self.meshRepository = meshRepository
    }

    /// Register background tasks — call from app init
    func registerBackgroundTasks() {
        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: Self.refreshTaskId,
            using: nil
        ) { task in
            self.handleBackgroundRefresh(task as! BGAppRefreshTask)
        }

        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: Self.processingTaskId,
            using: nil
        ) { task in
            self.handleBackgroundProcessing(task as! BGProcessingTask)
        }
    }

    /// Schedule next background fetch
    func scheduleBackgroundRefresh() {
        let request = BGAppRefreshTaskRequest(identifier: Self.refreshTaskId)
        request.earliestBeginDate = Date(timeIntervalSinceNow: 15 * 60) // 15 min
        try? BGTaskScheduler.shared.submit(request)
    }

    /// Schedule background processing (longer tasks)
    func scheduleBackgroundProcessing() {
        let request = BGProcessingTaskRequest(identifier: Self.processingTaskId)
        request.requiresNetworkConnectivity = false // mesh works offline
        request.requiresExternalPower = false
        request.earliestBeginDate = Date(timeIntervalSinceNow: 60 * 60) // 1 hour
        try? BGTaskScheduler.shared.submit(request)
    }

    /// Handle background refresh wakeup
    private func handleBackgroundRefresh(_ task: BGAppRefreshTask) {
        scheduleBackgroundRefresh() // Schedule next one

        task.expirationHandler = {
            self.meshRepository.pauseService()
        }

        // Sync messages, relay pending, update peer discovery
        Task {
            do {
                try meshRepository.syncPendingMessages()
                meshRepository.updateStats()
                task.setTaskCompleted(success: true)
            } catch {
                task.setTaskCompleted(success: false)
            }
        }
    }

    /// Handle background processing (bulk operations)
    private func handleBackgroundProcessing(_ task: BGProcessingTask) {
        scheduleBackgroundProcessing()

        task.expirationHandler = {
            self.meshRepository.pauseService()
        }

        Task {
            do {
                try meshRepository.performBulkSync()
                task.setTaskCompleted(success: true)
            } catch {
                task.setTaskCompleted(success: false)
            }
        }
    }

    /// Called when app enters background
    func onEnteringBackground() {
        meshRepository.onEnteringBackground()
        scheduleBackgroundRefresh()
        scheduleBackgroundProcessing()
    }

    /// Called when app enters foreground
    func onEnteringForeground() {
        meshRepository.onEnteringForeground()
    }
}
Services/IosPlatformBridge.swift (~200 LoC)
Implements the PlatformBridge UniFFI callback interface — the iOS equivalent of Android's AndroidPlatformBridge.kt.

Swift
import UIKit
import CoreMotion
import Network
import os

/// Implements Rust PlatformBridge callback interface for iOS
/// Mirrors: android/.../service/AndroidPlatformBridge.kt
final class IosPlatformBridge: PlatformBridge {
    private let logger = Logger(subsystem: "com.scmessenger", category: "Platform")
    private let motionManager = CMMotionActivityManager()
    private let pathMonitor = NWPathMonitor()
    private var meshRepository: MeshRepository?

    func configure(repository: MeshRepository) {
        self.meshRepository = repository
        startBatteryMonitoring()
        startNetworkMonitoring()
        startMotionMonitoring()
    }

    // MARK: - PlatformBridge protocol (called FROM Rust)

    func onBatteryChanged(batteryPct: UInt8, isCharging: Bool) {
        logger.debug("Battery: \(batteryPct)% charging=\(isCharging)")
    }

    func onNetworkChanged(hasWifi: Bool, hasCellular: Bool) {
        logger.debug("Network: wifi=\(hasWifi) cellular=\(hasCellular)")
    }

    func onMotionChanged(motion: MotionState) {
        logger.debug("Motion: \(String(describing: motion))")
    }

    func onBleDataReceived(peerId: String, data: Data) {
        // Forward BLE data received to mesh event system
        meshRepository?.onBleDataReceived(peerId: peerId, data: data)
    }

    func onEnteringBackground() {
        logger.info("App entering background")
    }

    func onEnteringForeground() {
        logger.info("App entering foreground")
    }

    func sendBlePacket(peerId: String, data: Data) {
        // Called by Rust when it needs to send BLE data
        meshRepository?.sendBlePacket(peerId: peerId, data: data)
    }

    // MARK: - iOS System Monitoring

    private func startBatteryMonitoring() {
        UIDevice.current.isBatteryMonitoringEnabled = true
        NotificationCenter.default.addObserver(
            forName: UIDevice.batteryLevelDidChangeNotification,
            object: nil, queue: .main
        ) { [weak self] _ in
            self?.reportBatteryState()
        }
        reportBatteryState()
    }

    private func reportBatteryState() {
        let level = UInt8(max(0, UIDevice.current.batteryLevel) * 100)
        let charging = UIDevice.current.batteryState == .charging
                    || UIDevice.current.batteryState == .full
        meshRepository?.reportBattery(pct: level, charging: charging)
    }

    private func startNetworkMonitoring() {
        pathMonitor.pathUpdateHandler = { [weak self] path in
            let hasWifi = path.usesInterfaceType(.wifi)
            let hasCellular = path.usesInterfaceType(.cellular)
            self?.meshRepository?.reportNetwork(wifi: hasWifi, cellular: hasCellular)
        }
        pathMonitor.start(queue: DispatchQueue.global(qos: .utility))
    }

    private func startMotionMonitoring() {
        guard CMMotionActivityManager.isActivityAvailable() else { return }
        motionManager.startActivityUpdates(to: .main) { [weak self] activity in
            guard let activity = activity else { return }
            let state: MotionState
            if activity.automotive { state = .automotive }
            else if activity.running { state = .running }
            else if activity.walking { state = .walking }
            else if activity.stationary { state = .still }
            else { state = .unknown }
            self?.meshRepository?.reportMotion(state: state)
        }
    }

    deinit {
        pathMonitor.cancel()
        motionManager.stopActivityUpdates()
        UIDevice.current.isBatteryMonitoringEnabled = false
    }
}
Services/MeshEventBus.swift (~100 LoC)
iOS equivalent of Android's MeshEventBus.kt. Uses Combine publishers.

Swift
import Combine

/// Central event dispatch for mesh network events
/// Mirrors: android/.../service/MeshEventBus.kt
final class MeshEventBus {
    static let shared = MeshEventBus()

    // Event streams (equivalent to Android SharedFlow)
    let peerEvents = PassthroughSubject<PeerEvent, Never>()
    let messageEvents = PassthroughSubject<MessageEvent, Never>()
    let statusEvents = PassthroughSubject<StatusEvent, Never>()
    let networkEvents = PassthroughSubject<NetworkEvent, Never>()

    enum PeerEvent {
        case discovered(peerId: String)
        case connected(peerId: String)
        case disconnected(peerId: String)
    }

    enum MessageEvent {
        case received(senderId: String, messageId: String, data: Data)
        case sent(messageId: String)
        case delivered(messageId: String)
        case failed(messageId: String, error: String)
    }

    enum StatusEvent {
        case serviceStateChanged(ServiceState)
        case statsUpdated(ServiceStats)
    }

    enum NetworkEvent {
        case transportEnabled(TransportType)
        case transportDisabled(TransportType)
        case batteryChanged(pct: UInt8, charging: Bool)
    }

    enum TransportType {
        case ble, multipeer, internet
    }
}
Services/CoreDelegateImpl.swift (~80 LoC)
Implements the CoreDelegate callback protocol from UniFFI.

Swift
import os

/// Implements Rust CoreDelegate callback interface
/// Receives events FROM Rust core and publishes to MeshEventBus
final class CoreDelegateImpl: CoreDelegate {
    private let logger = Logger(subsystem: "com.scmessenger", category: "CoreDelegate")
    private let eventBus = MeshEventBus.shared

    func onPeerDiscovered(peerId: String) {
        logger.info("Peer discovered: \(peerId)")
        eventBus.peerEvents.send(.discovered(peerId: peerId))
    }

    func onPeerDisconnected(peerId: String) {
        logger.info("Peer disconnected: \(peerId)")
        eventBus.peerEvents.send(.disconnected(peerId: peerId))
    }

    func onMessageReceived(senderId: String, messageId: String, data: Data) {
        logger.info("Message received: \(messageId) from \(senderId)")
        eventBus.messageEvents.send(.received(
            senderId: senderId,
            messageId: messageId,
            data: data
        ))
    }

    func onReceiptReceived(messageId: String, status: String) {
        logger.info("Receipt: \(messageId) status=\(status)")
        if status == "delivered" {
            eventBus.messageEvents.send(.delivered(messageId: messageId))
        }
    }
}
Steps (Execution Order)
Create MeshEventBus.swift — foundation for all event flow
Create CoreDelegateImpl.swift — wires Rust callbacks to event bus
Create IosPlatformBridge.swift — battery, network, motion monitoring
Create MeshBackgroundService.swift — BGTaskScheduler registration and handling
Wire into SCMessengerApp.swift:
Swift
@main
struct SCMessengerApp: App {
    @State private var meshRepository = MeshRepository()
    @State private var backgroundService: MeshBackgroundService

    init() {
        let repo = MeshRepository()
        meshRepository = repo
        backgroundService = MeshBackgroundService(meshRepository: repo)
        backgroundService.registerBackgroundTasks()
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environment(meshRepository)
                .onReceive(NotificationCenter.default.publisher(
                    for: UIApplication.didEnterBackgroundNotification
                )) { _ in backgroundService.onEnteringBackground() }
                .onReceive(NotificationCenter.default.publisher(
                    for: UIApplication.willEnterForegroundNotification
                )) { _ in backgroundService.onEnteringForeground() }
        }
    }
}
Phase 4: CoreBluetooth Transport Bridge
Goal: Full BLE mesh transport using CoreBluetooth. iOS equivalent of Android's BleScanner, BleAdvertiser, BleGattServer, BleGattClient, BleL2capManager.

LoC: ~900

iOS BLE vs Android BLE Mapping
Android	iOS	Purpose
BleScanner.kt (~300 LoC)	BLECentralManager.swift	Discover peers via BLE scanning
BleAdvertiser.kt (~250 LoC)	BLEPeripheralManager.swift	Advertise self to peers
BleGattServer.kt (~200 LoC)	Part of BLEPeripheralManager.swift	Serve GATT characteristics
BleGattClient.kt (~250 LoC)	Part of BLECentralManager.swift	Read/write GATT characteristics
BleL2capManager.kt (~150 LoC)	BLEL2CAPManager.swift	Bulk data over L2CAP channels
Service UUID & Characteristics
Shared between Android and iOS — must be identical for interop:

Swift
struct MeshBLEConstants {
    static let serviceUUID = CBUUID(string: "6E400001-B5A3-F393-E0A9-E50E24DCCA9E") // SCMesh Service
    static let txCharUUID  = CBUUID(string: "6E400002-B5A3-F393-E0A9-E50E24DCCA9E") // Write (phone→peer)
    static let rxCharUUID  = CBUUID(string: "6E400003-B5A3-F393-E0A9-E50E24DCCA9E") // Notify (peer→phone)
    static let idCharUUID  = CBUUID(string: "6E400004-B5A3-F393-E0A9-E50E24DCCA9E") // Identity beacon
    static let l2capPSM: CBL2CAPPSM = 0x1001
}
Files to Create
Transport/BLECentralManager.swift (~300 LoC)
Swift
import CoreBluetooth
import os

/// Scans for and connects to BLE mesh peers
/// Mirrors: android/.../transport/ble/BleScanner.kt + BleGattClient.kt
final class BLECentralManager: NSObject, CBCentralManagerDelegate, CBPeripheralDelegate {
    private let logger = Logger(subsystem: "com.scmessenger", category: "BLE-Central")
    private var centralManager: CBCentralManager!
    private var discoveredPeripherals: [UUID: CBPeripheral] = [:]
    private var connectedPeripherals: [UUID: CBPeripheral] = [:]
    private var peerCache: [UUID: Date] = [:] // Dedup cache (5s window)
    private let meshRepository: MeshRepository

    // AutoAdjust parameters
    private var scanInterval: TimeInterval = 10.0
    private var scanWindow: TimeInterval = 30.0
    private var isBackgroundMode = false
    private var scanTimer: Timer?

    // Write queue (mirrors Android BleGattClient.writeInProgress pattern)
    private var writeInProgress: [UUID: Bool] = [:]
    private var pendingWrites: [UUID: [Data]] = [:]

    init(meshRepository: MeshRepository) {
        self.meshRepository = meshRepository
        super.init()
        centralManager = CBCentralManager(delegate: self, queue: .global(qos: .utility),
            options: [CBCentralManagerOptionRestoreIdentifierKey: "com.scmessenger.central"])
    }

    func startScanning() { /* Start duty-cycled scanning */ }
    func stopScanning() { /* Stop scanning, disconnect all */ }
    func setBackgroundMode(_ background: Bool) { /* Adjust scan parameters */ }
    func applyScanSettings(intervalMs: UInt32) { /* From AutoAdjust engine */ }
    func sendData(to peripheralId: UUID, data: Data) { /* Queue-managed write */ }

    // CBCentralManagerDelegate
    func centralManagerDidUpdateState(_ central: CBCentralManager) { /* Handle power on/off */ }
    func centralManager(_ central: CBCentralManager, didDiscover peripheral: CBPeripheral, ...) { /* Cache + connect */ }
    func centralManager(_ central: CBCentralManager, didConnect peripheral: CBPeripheral) { /* Discover services */ }
    func centralManager(_ central: CBCentralManager, willRestoreState dict: [String: Any]) { /* State restoration */ }

    // CBPeripheralDelegate
    func peripheral(_ peripheral: CBPeripheral, didDiscoverServices error: Error?) { /* Find mesh service */ }
    func peripheral(_ peripheral: CBPeripheral, didDiscoverCharacteristicsFor service: CBService, ...) { /* Subscribe to RX */ }
    func peripheral(_ peripheral: CBPeripheral, didUpdateValueFor characteristic: CBCharacteristic, ...) { /* Data received */ }
    func peripheral(_ peripheral: CBPeripheral, didWriteValueFor characteristic: CBCharacteristic, ...) { /* Dequeue next write */ }
}
Transport/BLEPeripheralManager.swift (~300 LoC)
Swift
import CoreBluetooth
import os

/// Advertises self and serves GATT characteristics
/// Mirrors: android/.../transport/ble/BleAdvertiser.kt + BleGattServer.kt
final class BLEPeripheralManager: NSObject, CBPeripheralManagerDelegate {
    private let logger = Logger(subsystem: "com.scmessenger", category: "BLE-Peripheral")
    private var peripheralManager: CBPeripheralManager!
    private var meshService: CBMutableService?
    private var txCharacteristic: CBMutableCharacteristic?
    private var rxCharacteristic: CBMutableCharacteristic?
    private var subscribedCentrals: [CBCentral] = []
    private let meshRepository: MeshRepository

    // Rotation for privacy (mirrors Android BleAdvertiser.setRotationInterval)
    private var rotationInterval: TimeInterval = 900 // 15 min
    private var rotationTimer: Timer?
    private var identityData: Data?

    init(meshRepository: MeshRepository) {
        self.meshRepository = meshRepository
        super.init()
        peripheralManager = CBPeripheralManager(delegate: self, queue: .global(qos: .utility),
            options: [CBPeripheralManagerOptionRestoreIdentifierKey: "com.scmessenger.peripheral"])
    }

    func startAdvertising() { /* Build service, add characteristics, start advertising */ }
    func stopAdvertising() { /* Remove services, stop advertising */ }
    func setIdentityData(_ data: Data) { /* Update identity characteristic (≤24 bytes for advertising) */ }
    func setRotationInterval(_ interval: TimeInterval) { /* Privacy rotation */ }
    func applyAdvertiseSettings(intervalMs: UInt32, txPowerDbm: Int8) { /* From AutoAdjust */ }
    func sendNotification(to central: CBCentral, data: Data) { /* Send via RX characteristic */ }

    // CBPeripheralManagerDelegate
    func peripheralManagerDidUpdateState(_ peripheral: CBPeripheralManager) { /* Handle power on/off */ }
    func peripheralManager(_ peripheral: CBPeripheralManager, didReceiveWrite requests: [CBATTRequest]) { /* Handle incoming writes */ }
    func peripheralManager(_ peripheral: CBPeripheralManager, central: CBCentral, didSubscribeTo characteristic: CBCharacteristic) { /* Track subscriber */ }
    func peripheralManager(_ peripheral: CBPeripheralManager, willRestoreState dict: [String: Any]) { /* State restoration */ }
}
Transport/BLEL2CAPManager.swift (~150 LoC)
Swift
import CoreBluetooth
import os

/// L2CAP channel management for bulk data transfer
/// Mirrors: android/.../transport/ble/BleL2capManager.kt
final class BLEL2CAPManager: NSObject {
    private let logger = Logger(subsystem: "com.scmessenger", category: "BLE-L2CAP")

    func openChannel(to peripheral: CBPeripheral, psm: CBL2CAPPSM) { /* Connect L2CAP */ }
    func publishChannel(psm: CBL2CAPPSM) { /* Publish L2CAP PSM */ }
    func sendData(_ data: Data, on channel: CBL2CAPChannel) { /* Stream data */ }
    func closeChannel(_ channel: CBL2CAPChannel) { /* Cleanup */ }
}
Key iOS-Specific BLE Details
State Restoration: iOS kills and restores BLE apps. Must use CBCentralManagerOptionRestoreIdentifierKey and willRestoreState delegate. Android has no equivalent — this is critical for iOS background BLE.
Background Scanning: iOS background BLE scanning is limited — cannot specify service UUIDs in scan filter while backgrounded the same way. Must handle CBCentralManagerScanOptionAllowDuplicatesKey carefully.
MTU: iOS negotiates MTU automatically (up to 512 bytes on modern devices). No manual negotiation like Android.
Advertising Payload: Limited to 28 bytes in background (vs 31 foreground). Identity data must be ≤24 bytes (same constraint as Android's BleAdvertiser.sendData()).
Write Queue: Must mirror Android's BleGattClient.writeInProgress pattern — never overlap writes. iOS will silently drop concurrent writes.
Steps (Execution Order)
Define MeshBLEConstants.swift — shared UUIDs/PSMs matching Android
Create BLECentralManager.swift — scanning, connecting, GATT client, write queue
Create BLEPeripheralManager.swift — advertising, GATT server, identity data
Create BLEL2CAPManager.swift — L2CAP channels for bulk transfer
Wire to TransportManager.swift (created in Phase 5)
Wire to IosPlatformBridge for sendBlePacket() and onBleDataReceived()
Test: verify two iOS devices discover each other via BLE
Phase 5: Multipeer Connectivity
Goal: WiFi-based peer discovery and data exchange. iOS equivalent of Android's WiFi Aware + WiFi Direct combined. Multipeer Connectivity handles both automatically.

LoC: ~400

Android ↔ iOS WiFi Mapping
Android	iOS	Notes
WifiAwareTransport.kt (~350 LoC)	MultipeerTransport.swift	MCNearbyServiceBrowser handles discovery
WifiDirectTransport.kt (~250 LoC)	Same MultipeerTransport.swift	MCSession handles connection
WifiTransportManager.kt (~200 LoC)	Part of TransportManager.swift	Orchestration
Files to Create
Transport/MultipeerTransport.swift (~250 LoC)
Swift
import MultipeerConnectivity
import os

/// WiFi-based peer discovery and messaging via Multipeer Connectivity
/// Mirrors: android/.../transport/WifiAwareTransport.kt + WifiDirectTransport.kt
final class MultipeerTransport: NSObject, MCSessionDelegate, MCNearbyServiceAdvertiserDelegate, MCNearbyServiceBrowserDelegate {
    private let logger = Logger(subsystem: "com.scmessenger", category: "Multipeer")
    private let serviceType = "scmesh" // Must match Bonjour service in Info.plist
    private var peerID: MCPeerID!
    private var session: MCSession!
    private var advertiser: MCNearbyServiceAdvertiser?
    private var browser: MCNearbyServiceBrowser?
    private let meshRepository: MeshRepository
    private var connectedPeers: [MCPeerID: String] = [:] // MCPeerID → mesh peer_id

    init(meshRepository: MeshRepository) {
        self.meshRepository = meshRepository
        super.init()
        // Use truncated public key as display name for MCPeerID
        let displayName = meshRepository.getIdentitySnippet()
        peerID = MCPeerID(displayName: displayName)
        session = MCSession(peer: peerID, securityIdentity: nil, encryptionPreference: .required)
        session.delegate = self
    }

    func start() {
        startAdvertising()
        startBrowsing()
    }

    func stop() {
        advertiser?.stopAdvertisingPeer()
        browser?.stopBrowsingForPeers()
        session.disconnect()
    }

    func sendData(_ data: Data, to peerIds: [String]) throws {
        let targets = session.connectedPeers.filter { peer in
            connectedPeers[peer].map { peerIds.contains($0) } ?? false
        }
        guard !targets.isEmpty else { throw TransportError.noPeersAvailable }
        try session.send(data, toPeers: targets, with: .reliable)
    }

    // MCSessionDelegate
    func session(_ session: MCSession, peer peerID: MCPeerID, didChange state: MCSessionState) { /* Track connect/disconnect */ }
    func session(_ session: MCSession, didReceive data: Data, fromPeer peerID: MCPeerID) { /* Forward to mesh */ }
    func session(_ session: MCSession, didReceive stream: InputStream, withName streamName: String, fromPeer peerID: MCPeerID) { /* Stream handling */ }
    // ... resource received callbacks

    // MCNearbyServiceAdvertiserDelegate
    func advertiser(_ advertiser: MCNearbyServiceAdvertiser, didReceiveInvitationFromPeer peerID: MCPeerID, ...) {
        // Auto-accept invitations (mesh is open by default, encryption handled at Drift layer)
        invitationHandler(true, session)
    }

    // MCNearbyServiceBrowserDelegate
    func browser(_ browser: MCNearbyServiceBrowser, foundPeer peerID: MCPeerID, withDiscoveryInfo info: [String: String]?) {
        // Auto-invite discovered peers
        browser.invitePeer(peerID, to: session, withContext: nil, timeout: 30)
    }
}
Transport/TransportManager.swift (~150 LoC)
Swift
import os

/// Orchestrates all transport layers with priority escalation
/// Mirrors: android/.../transport/TransportManager.kt
/// Priority: Multipeer > BLE > Internet (SwarmBridge)
final class TransportManager {
    private let logger = Logger(subsystem: "com.scmessenger", category: "Transport")
    let bleCentral: BLECentralManager
    let blePeripheral: BLEPeripheralManager
    let bleL2CAP: BLEL2CAPManager
    let multipeer: MultipeerTransport
    private let meshRepository: MeshRepository

    init(meshRepository: MeshRepository) {
        self.meshRepository = meshRepository
        self.bleCentral = BLECentralManager(meshRepository: meshRepository)
        self.blePeripheral = BLEPeripheralManager(meshRepository: meshRepository)
        self.bleL2CAP = BLEL2CAPManager()
        self.multipeer = MultipeerTransport(meshRepository: meshRepository)
    }

    func startAll() {
        bleCentral.startScanning()
        blePeripheral.startAdvertising()
        multipeer.start()
        logger.info("All transports started")
    }

    func stopAll() {
        bleCentral.stopScanning()
        blePeripheral.stopAdvertising()
        multipeer.stop()
        logger.info("All transports stopped")
    }

    func setBackgroundMode(_ background: Bool) {
        bleCentral.setBackgroundMode(background)
        // Multipeer continues in background if BLE is available
    }

    /// Send data via best available transport (escalation)
    func sendData(_ data: Data, to peerId: String) {
        // 1. Try Multipeer (fastest, WiFi bandwidth)
        if let _ = try? multipeer.sendData(data, to: [peerId]) { return }
        // 2. Try BLE GATT
        // 3. Fall back to SwarmBridge (internet)
        try? meshRepository.swarmBridge?.sendMessage(peerId: peerId, data: Array(data))
    }

    /// Apply AutoAdjust settings
    func applyAdjustments(ble: BleAdjustment) {
        bleCentral.applyScanSettings(intervalMs: ble.scanIntervalMs)
        blePeripheral.applyAdvertiseSettings(
            intervalMs: ble.advertiseIntervalMs,
            txPowerDbm: ble.txPowerDbm
        )
    }
}
Steps (Execution Order)
Create MultipeerTransport.swift — browse, advertise, session management
Create TransportManager.swift — orchestrate BLE + Multipeer + Internet
Wire TransportManager into MeshRepository
Wire IosPlatformBridge.sendBlePacket() → TransportManager.bleCentral.sendData()
Test: two iOS devices discover each other via Multipeer Connectivity
Test: data sent via Multipeer arrives at other device
Phase 6: Data Repository Layer
Goal: Single source of truth wrapping all UniFFI managers. iOS equivalent of Android's MeshRepository.kt + PreferencesRepository.kt.

LoC: ~600

Files to Create
Data/MeshRepository.swift (~500 LoC)
Direct mirror of Android's MeshRepository.kt (644 LoC). Every method maps 1:1 to the same UniFFI call.

Swift
import Foundation
import Combine
import os

/// Single source of truth for all mesh operations
/// Mirrors: android/.../data/MeshRepository.kt
@Observable
final class MeshRepository {
    private let logger = Logger(subsystem: "com.scmessenger", category: "Repository")

    // UniFFI managers (initialized with app's storage path)
    private(set) var ironCore: IronCore
    private(set) var meshService: MeshService
    private(set) var contactManager: ContactManager?
    private(set) var historyManager: HistoryManager?
    private(set) var ledgerManager: LedgerManager?
    private(set) var settingsManager: MeshSettingsManager?
    private(set) var autoAdjustEngine: AutoAdjustEngine
    private(set) var swarmBridge: SwarmBridge?

    // Platform bridge
    private let platformBridge = IosPlatformBridge()
    private let coreDelegate = CoreDelegateImpl()

    // Transport
    private(set) var transportManager: TransportManager?

    // Observable state
    var serviceState: ServiceState = .stopped
    var serviceStats: ServiceStats = ServiceStats(peersDiscovered: 0, messagesRelayed: 0, bytesTransferred: 0, uptimeSecs: 0)
    var currentSettings: MeshSettings?

    private let storagePath: String

    init() {
        let documentsPath = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
        self.storagePath = documentsPath.appendingPathComponent("scmessenger").path

        // Create storage directory
        try? FileManager.default.createDirectory(atPath: storagePath, withIntermediateDirectories: true)

        // Initialize UniFFI objects
        self.ironCore = IronCore.withStorage(storagePath: storagePath)
        let config = MeshServiceConfig(discoveryIntervalMs: 30000, relayBudgetPerHour: 100, batteryFloorPct: 15)
        self.meshService = MeshService.withStorage(config: config, storagePath: storagePath)
        self.autoAdjustEngine = AutoAdjustEngine()

        // Initialize managers
        do {
            self.contactManager = try ContactManager(storagePath: storagePath)
            self.historyManager = try HistoryManager(storagePath: storagePath)
            self.settingsManager = MeshSettingsManager(storagePath: storagePath)
            self.ledgerManager = LedgerManager(storagePath: storagePath)
        } catch {
            logger.error("Failed to initialize managers: \(error)")
        }

        // Wire delegates
        ironCore.setDelegate(delegate: coreDelegate)
        meshService.setPlatformBridge(bridge: platformBridge)
        platformBridge.configure(repository: self)

        // Load settings
        currentSettings = try? settingsManager?.load()
    }

    // MARK: - Lifecycle (mirrors Android MeshRepository)

    func startMeshService() throws {
        try meshService.start()
        try ironCore.initializeIdentity()
        try ironCore.start()
        swarmBridge = SwarmBridge()
        transportManager = TransportManager(meshRepository: self)
        transportManager?.startAll()
        serviceState = meshService.getState()
    }

    func stopMeshService() {
        transportManager?.stopAll()
        transportManager = nil
        swarmBridge?.shutdown()
        swarmBridge = nil
        meshService.stop()
        ironCore.stop()
        serviceState = meshService.getState()
    }

    func pauseService() { meshService.pause() }
    func resumeService() { meshService.resume() }

    // MARK: - Identity

    func getIdentityInfo() -> IdentityInfo { ironCore.getIdentityInfo() }
    func getIdentitySnippet() -> String {
        let info = getIdentityInfo()
        return String((info.publicKeyHex ?? "unknown").prefix(8))
    }

    // MARK: - Messaging

    func sendMessage(recipientPubKey: String, text: String) throws {
        let encrypted = try ironCore.prepareMessage(recipientPublicKeyHex: recipientPubKey, text: text)
        try swarmBridge?.sendMessage(peerId: recipientPubKey, data: encrypted)

        // Save to history
        let record = MessageRecord(
            id: UUID().uuidString,
            direction: .sent,
            peerId: recipientPubKey,
            content: text,
            timestamp: UInt64(Date().timeIntervalSince1970),
            delivered: false
        )
        try? historyManager?.add(record: record)
    }

    func getRecentMessages(peerFilter: String?, limit: UInt32) throws -> [MessageRecord] {
        try historyManager?.recent(peerFilter: peerFilter, limit: limit) ?? []
    }

    func getConversation(peerId: String, limit: UInt32) throws -> [MessageRecord] {
        try historyManager?.conversation(peerId: peerId, limit: limit) ?? []
    }

    func searchMessages(query: String, limit: UInt32) throws -> [MessageRecord] {
        try historyManager?.search(query: query, limit: limit) ?? []
    }

    // MARK: - Contacts

    func addContact(_ contact: Contact) throws { try contactManager?.add(contact: contact) }
    func removeContact(peerId: String) throws { try contactManager?.remove(peerId: peerId) }
    func listContacts() throws -> [Contact] { try contactManager?.list() ?? [] }
    func searchContacts(query: String) throws -> [Contact] { try contactManager?.search(query: query) ?? [] }
    func getContact(peerId: String) throws -> Contact? { try contactManager?.get(peerId: peerId) }
    func setNickname(peerId: String, nickname: String?) throws { try contactManager?.setNickname(peerId: peerId, nickname: nickname) }

    // MARK: - Settings

    func loadSettings() throws -> MeshSettings {
        let settings = try settingsManager?.load() ?? settingsManager!.defaultSettings()
        currentSettings = settings
        return settings
    }

    func saveSettings(_ settings: MeshSettings) throws {
        try settingsManager?.validate(settings: settings)
        try settingsManager?.save(settings: settings)
        currentSettings = settings
    }

    // MARK: - AutoAdjust

    func computeAdjustmentProfile(profile: DeviceProfile) -> AdjustmentProfile {
        autoAdjustEngine.computeProfile(device: profile)
    }

    func computeBleAdjustment(profile: AdjustmentProfile) -> BleAdjustment {
        autoAdjustEngine.computeBleAdjustment(profile: profile)
    }

    func computeRelayAdjustment(profile: AdjustmentProfile) -> RelayAdjustment {
        autoAdjustEngine.computeRelayAdjustment(profile: profile)
    }

    // MARK: - Stats

    func updateStats() {
        serviceStats = meshService.getStats()
        serviceState = meshService.getState()
    }

    func getHistoryStats() throws -> HistoryStats {
        try historyManager?.stats() ?? HistoryStats(totalMessages: 0, sentCount: 0, receivedCount: 0, undeliveredCount: 0)
    }

    // MARK: - Ledger

    func getLedgerSummary() -> String { ledgerManager?.summary() ?? "No data" }
    func getDialableAddresses() -> [LedgerEntry] { ledgerManager?.dialableAddresses() ?? [] }

    // MARK: - Platform Bridge forwarding

    func reportBattery(pct: UInt8, charging: Bool) {
        meshService.onDataReceived(peerId: "system", data: Data()) // Trigger stats update
    }

    func reportNetwork(wifi: Bool, cellular: Bool) { /* forward to auto-adjust */ }
    func reportMotion(state: MotionState) { /* forward to auto-adjust */ }
    func onBleDataReceived(peerId: String, data: Data) {
        meshService.onDataReceived(peerId: peerId, data: Array(data))
    }

    func sendBlePacket(peerId: String, data: Data) {
        transportManager?.sendData(data, to: peerId)
    }

    // MARK: - Background

    func onEnteringBackground() { transportManager?.setBackgroundMode(true) }
    func onEnteringForeground() { transportManager?.setBackgroundMode(false) }
    func syncPendingMessages() throws { /* Iterate outbox, try sending */ }
    func performBulkSync() throws { /* Full ledger sync */ }
}
