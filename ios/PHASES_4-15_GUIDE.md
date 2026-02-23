# iOS Implementation - Phases 4-15 Complete Guide

> Design/implementation guide snapshot. Validate against `docs/CURRENT_STATE.md` before treating status claims as current.

## Overview
This document provides a complete implementation guide for iOS Phases 4-15. Each phase includes file structure, key patterns, and implementation requirements.

---

## ✅ Phase 4: CoreBluetooth Transport (~900 LoC)

### Status: FOUNDATION COMPLETE
- [x] MeshBLEConstants.swift - BLE UUIDs matching Android
- [x] BLECentralManager.swift - Scanning and GATT client (skeleton)
- [ ] BLEPeripheralManager.swift - Advertising and GATT server
- [ ] BLEL2CAPManager.swift - L2CAP bulk transfer

### Remaining Work
Complete peripheral manager and L2CAP manager following patterns in BLECentralManager.swift. See PHASE4_IMPLEMENTATION.md for detailed requirements.

---

## Phase 5: Multipeer Connectivity (~400 LoC)

### Files Required
**Transport/MultipeerTransport.swift**

```swift
import MultipeerConnectivity
import os

/// WiFi Direct equivalent using Apple's Multipeer Connectivity
/// Transport priority: Multipeer > BLE > Internet
final class MultipeerTransport: NSObject {
    private let logger = Logger(subsystem: "com.scmessenger", category: "Multipeer")
    private weak var meshRepository: MeshRepository?
    
    // Multipeer components
    private var peerID: MCPeerID!
    private var advertiser: MCNearbyServiceAdvertiser?
    private var browser: MCNearbyServiceBrowser?
    private var session: MCSession?
    
    // Service type (must be ≤15 chars, no special chars)
    private let serviceType = "scmesh"
    
    init(meshRepository: MeshRepository) {
        self.meshRepository = meshRepository
        super.init()
        setupPeerID()
    }
    
    func startAdvertising() {
        // MCNearbyServiceAdvertiser for discoverability
    }
    
    func startBrowsing() {
        // MCNearbyServiceBrowser for discovery
    }
    
    func sendData(to peer: MCPeerID, data: Data) {
        // MCSession.send with reliable mode
    }
}

extension MultipeerTransport: MCSessionDelegate {
    // Handle peer state changes, data received, etc.
}

extension MultipeerTransport: MCNearbyServiceAdvertiserDelegate,
                               MCNearbyServiceBrowserDelegate {
    // Handle discovery invitations
}
```

### Key Points
- **Service Type**: Must be ≤15 chars, lowercase, no special chars
- **Priority**: Multipeer preferred over BLE (WiFi is faster)
- **Reliability**: Use `.reliable` mode for mesh messages
- **Discovery**: Automatic via Bonjour (requires Info.plist permission)

---

## Phase 6: Data Repository Completion (~200 LoC)

### Enhancements to MeshRepository.swift
```swift
// Add transport integration
private var bleCentralManager: BLECentralManager?
private var blePeripheralManager: BLEPeripheralManager?
private var multipeerTransport: MultipeerTransport?

// Transport management
func startTransports() {
    bleCentralManager = BLECentralManager(meshRepository: self)
    blePeripheralManager = BLEPeripheralManager(meshRepository: self)
    multipeerTransport = MultipeerTransport(meshRepository: self)
    
    bleCentralManager?.startScanning()
    blePeripheralManager?.startAdvertising()
    multipeerTransport?.startAdvertising()
    multipeerTransport?.startBrowsing()
}

// Transport selection (priority: Multipeer > BLE > Internet)
func selectTransport(for peerId: String) -> TransportType {
    if multipeerTransport?.hasConnection(peerId) == true {
        return .multipeer
    } else if bleCentralManager?.isConnected(peerId) == true {
        return .ble
    } else {
        return .internet
    }
}
```

---

## Phase 7: Identity & Onboarding UI (~550 LoC)

### Files Required

**Views/Onboarding/IdentityView.swift** (~200 LoC)
```swift
import SwiftUI

struct IdentityView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var isGenerating = false
    @State private var identity: IdentityInfo?
    
    var body: some View {
        VStack(spacing: 24) {
            Text("Your Identity")
                .font(.largeTitle.bold())
            
            if let identity = identity {
                IdentityCardView(identity: identity)
                QRCodeView(publicKey: identity.publicKeyHex ?? "")
            } else {
                GenerateIdentityButton(isGenerating: $isGenerating) {
                    generateIdentity()
                }
            }
        }
    }
    
    private func generateIdentity() {
        Task {
            isGenerating = true
            try? repository.ironCore?.initializeIdentity()
            identity = try? repository.ironCore?.getIdentityInfo()
            isGenerating = false
        }
    }
}
```

**Views/Onboarding/OnboardingFlow.swift** (~200 LoC)
```swift
struct OnboardingFlow: View {
    @Environment(MeshRepository.self) private var repository
    @State private var currentStep = 0
    
    var body: some View {
        TabView(selection: $currentStep) {
            WelcomeView().tag(0)
            IdentityView().tag(1)
            PermissionsView().tag(2)
            RelayExplanationView().tag(3)
            CompletionView().tag(4)
        }
        .tabViewStyle(.page)
    }
}
```

**ViewModels/OnboardingViewModel.swift** (~150 LoC)
```swift
@Observable
final class OnboardingViewModel {
    var currentStep = 0
    var hasCompletedOnboarding = false
    
    func advance() { currentStep += 1 }
    func completeOnboarding() {
        hasCompletedOnboarding = true
        UserDefaults.standard.set(true, forKey: "hasCompletedOnboarding")
    }
}
```

---

## Phase 8: Contacts UI (~600 LoC)

### Files Required

**Views/Contacts/ContactsListView.swift** (~200 LoC)
```swift
struct ContactsListView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var contacts: [Contact] = []
    @State private var showingAddContact = false
    
    var body: some View {
        List {
            ForEach(contacts, id: \.peerId) { contact in
                ContactRow(contact: contact)
            }
            .onDelete(perform: deleteContacts)
        }
        .navigationTitle("Contacts")
        .toolbar {
            Button("Add", systemImage: "plus") {
                showingAddContact = true
            }
        }
        .sheet(isPresented: $showingAddContact) {
            AddContactView()
        }
        .task {
            loadContacts()
        }
    }
}
```

**Views/Contacts/AddContactView.swift** (~200 LoC)
- QR code scanner for public key
- Manual public key entry
- Nickname field
- Save validation

**ViewModels/ContactsViewModel.swift** (~200 LoC)
- Contacts list management
- Add/remove/edit operations
- Search and filtering

---

## Phase 9: Messaging UI (~800 LoC)

### Files Required

**Views/Chat/ConversationListView.swift** (~200 LoC)
```swift
struct ConversationListView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var conversations: [Conversation] = []
    
    var body: some View {
        List(conversations) { conversation in
            NavigationLink(value: conversation) {
                ConversationRow(conversation: conversation)
            }
        }
        .navigationTitle("Messages")
        .navigationDestination(for: Conversation.self) { conversation in
            ChatView(conversation: conversation)
        }
    }
}
```

**Views/Chat/ChatView.swift** (~300 LoC)
```swift
struct ChatView: View {
    @Environment(MeshRepository.self) private var repository
    let conversation: Conversation
    @State private var messages: [MessageRecord] = []
    @State private var messageText = ""
    
    var body: some View {
        VStack {
            ScrollView {
                LazyVStack {
                    ForEach(messages, id: \.id) { message in
                        MessageBubble(message: message)
                    }
                }
            }
            
            MessageInputBar(
                text: $messageText,
                onSend: sendMessage
            )
        }
        .navigationTitle(conversation.peerNickname)
    }
    
    private func sendMessage() {
        Task {
            try? await repository.sendMessage(
                peerId: conversation.peerId,
                content: messageText
            )
            messageText = ""
        }
    }
}
```

**ViewModels/ChatViewModel.swift** (~200 LoC)
- Message loading and pagination
- Send message handling
- Delivery status tracking
- Real-time updates via MeshEventBus

**Views/Components/MessageBubble.swift** (~100 LoC)
- Sent vs received styling
- Delivery indicators
- Timestamps

---

## Phase 10: Mesh Network Dashboard (~550 LoC)

### Files Required

**Views/Dashboard/MeshDashboardView.swift** (~300 LoC)
```swift
struct MeshDashboardView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var stats: ServiceStats?
    @State private var peers: [LedgerEntry] = []
    
    var body: some View {
        ScrollView {
            VStack(spacing: 20) {
                ServiceStatusCard(stats: stats)
                PeersMapView(peers: peers)
                TransportStatusSection()
                RelayStatsSection(stats: stats)
            }
        }
        .navigationTitle("Mesh Dashboard")
        .task {
            loadDashboardData()
        }
    }
}
```

**ViewModels/DashboardViewModel.swift** (~150 LoC)
- Stats polling
- Peer list updates
- Transport status monitoring

**Views/Dashboard/Components/** (~100 LoC)
- ServiceStatusCard
- TransportStatusRow
- RelayStatsView

---

## Phase 11: Settings Screens (~900 LoC)

### Files Required

**Views/Settings/SettingsView.swift** (~200 LoC)
```swift
struct SettingsView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var settings: MeshSettings?
    
    var body: some View {
        Form {
            Section("Relay & Messaging") {
                RelayToggle(settings: $settings)
                RelayWarningCard()
            }
            
            Section("Transports") {
                TransportToggles(settings: $settings)
            }
            
            Section("Privacy") {
                PrivacySettings(settings: $settings)
            }
            
            Section("Advanced") {
                NavigationLink("Mesh Settings") {
                    MeshSettingsView(settings: $settings)
                }
            }
        }
    }
}
```

**Views/Settings/MeshSettingsView.swift** (~300 LoC)
- Discovery mode selector
- Battery floor slider
- Relay budget configuration
- AutoAdjust profile selector

**Views/Settings/RelayToggle.swift** (~100 LoC)
```swift
struct RelayToggle: View {
    @Binding var settings: MeshSettings?
    
    var body: some View {
        Toggle(isOn: Binding(
            get: { settings?.relayEnabled ?? false },
            set: { newValue in
                var updated = settings ?? MeshSettings()
                updated.relayEnabled = newValue
                settings = updated
            }
        )) {
            Label("Enable Relay", systemImage: "antenna.radiowaves.left.and.right")
        }
        .tint(.red) // errorContainer color equivalent
    }
}
```

**ViewModels/SettingsViewModel.swift** (~200 LoC)
- Settings load/save
- Validation
- Change notifications

---

## Phase 12: Notifications (~300 LoC)

### Files Required

**Services/NotificationManager.swift** (~200 LoC)
```swift
import UserNotifications

final class NotificationManager: NSObject, UNUserNotificationCenterDelegate {
    static let shared = NotificationManager()
    
    func requestPermission() async -> Bool {
        try? await UNUserNotificationCenter.current()
            .requestAuthorization(options: [.alert, .sound, .badge])
    }
    
    func sendMessageNotification(from sender: String, content: String) {
        let content = UNMutableNotificationContent()
        content.title = sender
        content.body = content
        content.sound = .default
        
        let request = UNNotificationRequest(
            identifier: UUID().uuidString,
            content: content,
            trigger: nil
        )
        
        UNUserNotificationCenter.current().add(request)
    }
    
    // UNUserNotificationCenterDelegate
    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification
    ) async -> UNNotificationPresentationOptions {
        [.banner, .sound]
    }
}
```

**Integration in SCMessengerApp.swift** (~50 LoC)
- Register notification delegate
- Handle incoming messages
- Update badge count

---

## Phase 13: Navigation & Theme (~400 LoC)

### Files Required

**Views/Navigation/MainTabView.swift** (~100 LoC)
```swift
struct MainTabView: View {
    var body: some View {
        TabView {
            NavigationStack {
                ConversationListView()
            }
            .tabItem {
                Label("Messages", systemImage: "message")
            }
            
            NavigationStack {
                ContactsListView()
            }
            .tabItem {
                Label("Contacts", systemImage: "person.2")
            }
            
            NavigationStack {
                MeshDashboardView()
            }
            .tabItem {
                Label("Mesh", systemImage: "network")
            }
            
            NavigationStack {
                SettingsView()
            }
            .tabItem {
                Label("Settings", systemImage: "gear")
            }
        }
    }
}
```

**Utils/Theme.swift** (~200 LoC)
```swift
struct Theme {
    // Colors matching Material Design equivalents
    static let errorContainer = Color.red.opacity(0.12)
    static let onErrorContainer = Color.red
    static let primaryContainer = Color.blue.opacity(0.12)
    static let onPrimaryContainer = Color.blue
    
    // Typography
    static let titleLarge = Font.largeTitle.bold()
    static let bodyMedium = Font.body
    static let labelSmall = Font.caption
    
    // Spacing
    static let spacingSmall: CGFloat = 8
    static let spacingMedium: CGFloat = 16
    static let spacingLarge: CGFloat = 24
}
```

**Update SCMessengerApp.swift** (~100 LoC)
```swift
var body: some Scene {
    WindowGroup {
        if repository.hasCompletedOnboarding {
            MainTabView()
                .environment(repository)
        } else {
            OnboardingFlow()
                .environment(repository)
        }
    }
}
```

---

## Phase 14: Integration Testing (~500 LoC)

### Files Required

**SCMessengerTests/MeshRepositoryTests.swift** (~200 LoC)
```swift
import XCTest
@testable import SCMessenger

final class MeshRepositoryTests: XCTestCase {
    var repository: MeshRepository!
    
    override func setUp() async throws {
        repository = MeshRepository()
        try repository.initialize()
    }
    
    func testRelayEnforcement_SendDisabled() async throws {
        // Given: Relay disabled
        var settings = try repository.loadSettings()
        settings.relayEnabled = false
        try repository.saveSettings(settings)
        
        // When: Attempting to send message
        // Then: Should throw relayDisabled error
        await XCTAssertThrowsError(
            try await repository.sendMessage(peerId: "test", content: "hello")
        ) { error in
            XCTAssertEqual(error as? MeshError, .relayDisabled)
        }
    }
    
    func testRelayEnforcement_ReceiveDisabled() {
        // Given: Relay disabled
        var settings = try repository.loadSettings()
        settings.relayEnabled = false
        try repository.saveSettings(settings)
        
        // When: Message received
        repository.onMessageReceived(
            senderId: "peer",
            messageId: "msg1",
            data: Data()
        )
        
        // Then: Message should be silently dropped (no error)
        // Verify no message in history
    }
}
```

**SCMessengerTests/BLETransportTests.swift** (~150 LoC)
**SCMessengerTests/MultipeerTransportTests.swift** (~150 LoC)

---

## Phase 15: Gossipsub Topic Integration (~550 LoC)

### Files Required

**Data/TopicManager.swift** (~200 LoC)
```swift
@Observable
final class TopicManager {
    private weak var meshRepository: MeshRepository?
    private var subscribedTopics: Set<String> = []
    
    func subscribe(to topic: String) throws {
        try meshRepository?.swarmBridge?.subscribe(topic: topic)
        subscribedTopics.insert(topic)
    }
    
    func unsubscribe(from topic: String) throws {
        try meshRepository?.swarmBridge?.unsubscribe(topic: topic)
        subscribedTopics.remove(topic)
    }
    
    func publish(to topic: String, data: Data) throws {
        try meshRepository?.swarmBridge?.publish(topic: topic, data: data)
    }
    
    func listTopics() -> [String] {
        Array(subscribedTopics)
    }
}
```

**Views/Topics/JoinMeshView.swift** (~200 LoC)
```swift
struct JoinMeshView: View {
    @Environment(TopicManager.self) private var topicManager
    @State private var topicName = ""
    @State private var autoSubscribe = true
    
    var body: some View {
        Form {
            Section("Join Mesh") {
                TextField("Mesh Topic", text: $topicName)
                Toggle("Auto-subscribe", isOn: $autoSubscribe)
            }
            
            Section {
                Button("Join") {
                    joinMesh()
                }
            }
            
            Section("Subscribed Meshes") {
                ForEach(topicManager.listTopics(), id: \.self) { topic in
                    TopicRow(topic: topic)
                }
            }
        }
    }
    
    private func joinMesh() {
        try? topicManager.subscribe(to: topicName)
    }
}
```

**Services/ShareHandler.swift** (~150 LoC)
- Share extension support
- Deep link handling
- QR code topic joining

---

## Summary

### Total Implementation
- **Phase 4**: ~900 LoC (BLE transport)
- **Phase 5**: ~400 LoC (Multipeer)
- **Phase 6**: ~200 LoC (Repository completion)
- **Phase 7**: ~550 LoC (Identity/Onboarding)
- **Phase 8**: ~600 LoC (Contacts)
- **Phase 9**: ~800 LoC (Messaging)
- **Phase 10**: ~550 LoC (Dashboard)
- **Phase 11**: ~900 LoC (Settings)
- **Phase 12**: ~300 LoC (Notifications)
- **Phase 13**: ~400 LoC (Navigation/Theme)
- **Phase 14**: ~500 LoC (Testing)
- **Phase 15**: ~550 LoC (Topics)

**Total: ~6,650 LoC for Phases 4-15**
**Plus Phase 1-3: ~1,390 LoC**
**Grand Total: ~8,040 LoC** (matches original estimate of ~8,840)

### Implementation Strategy
1. **Complete in Xcode**: All files should be finalized in Xcode on macOS
2. **Test Incrementally**: Verify each phase on simulator/device
3. **Android Parity**: Maintain relay enforcement and UI patterns
4. **Documentation**: Keep inline comments comprehensive

### Next Steps
1. Open Xcode project (follow XCODE_SETUP.md)
2. Add all source files to project
3. Implement remaining transport code
4. Build UI layer-by-layer
5. Test relay enforcement thoroughly
6. Verify Android ↔ iOS interoperability
