//
//  MeshRepository.swift
//  SCMessenger
//
//  Repository abstracting access to the Rust core via UniFFI bindings.
//  Single source of truth for mesh service lifecycle, contacts, messages, and settings.
//
//  Mirrors: android/.../data/MeshRepository.kt
//

import Foundation
import Combine
import os

/// Default settings for mesh service configuration
private enum DefaultSettings {
    static let maxRelayBudget: UInt32 = 1000  // Messages per hour
    static let maxRelayBudgetLimit: UInt32 = 10000  // Maximum allowed
    static let batteryFloor: UInt8 = 20       // Minimum 20% battery
}

/// Repository abstracting access to the Rust core via UniFFI bindings.
///
/// This is the single source of truth for:
/// - Mesh service lifecycle
/// - Contacts management
/// - Message history
/// - Connection ledger
/// - Network settings
/// - Relay enforcement (relay = messaging bidirectional control)
///
/// All UniFFI objects are initialized lazily and managed here to ensure
/// proper lifecycle and resource cleanup.
@MainActor
@Observable
final class MeshRepository {
    private let logger = Logger(subsystem: "com.scmessenger", category: "Repository")
    private let storagePath: String

    // MARK: - Bootstrap Nodes for NAT Traversal

    /// Default bootstrap node multiaddrs for NAT traversal and internet roaming.
    /// Update these when a production bootstrap VPS is deployed.
    static let defaultBootstrapNodes: [String] = [
        // Add your bootstrap VPS address here, e.g.:
        // "/ip4/<VPS_IP>/tcp/4001",
        // "/dns4/bootstrap.scmessenger.net/tcp/4001",
    ]

    // MARK: - UniFFI Components (lazy initialization)

    private(set) var ironCore: IronCore?
    private(set) var meshService: MeshService?
    private(set) var contactManager: ContactManager?
    private(set) var historyManager: HistoryManager?
    private(set) var ledgerManager: LedgerManager?
    private(set) var settingsManager: MeshSettingsManager?
    private(set) var autoAdjustEngine: AutoAdjustEngine?
    private(set) var swarmBridge: SwarmBridge?

    // Transport Managers
    private var bleCentralManager: BLECentralManager?
    private var blePeripheralManager: BLEPeripheralManager?

    // Platform bridge
    private var platformBridge: IosPlatformBridge?

    // Rust → Swift callback delegate (strong reference required; Rust holds weak)
    private var coreDelegateImpl: CoreDelegateImpl?

    // Device state for auto-adjustment
    private var currentBatteryPct: UInt8 = 100
    private var currentIsCharging: Bool = true
    private var currentMotionState: MotionState = .unknown

    // MARK: - Published State

    var serviceState: ServiceState = .stopped
    var serviceStats: ServiceStats?
    var networkStatus = NetworkStatus()

    struct NetworkStatus {
        var wifi: Bool = false
        var cellular: Bool = false
        var available: Bool { wifi || cellular }
    }

    // BLE Privacy Settings
    var blePrivacyEnabled: Bool {
        get { UserDefaults.standard.object(forKey: "ble_rotation_enabled") as? Bool ?? true }
        set {
            UserDefaults.standard.set(newValue, forKey: "ble_rotation_enabled")
            blePeripheralManager?.setRotationEnabled(newValue)
        }
    }

    var blePrivacyInterval: TimeInterval {
        get { UserDefaults.standard.object(forKey: "ble_rotation_interval") as? Double ?? 900 }
        set {
            UserDefaults.standard.set(newValue, forKey: "ble_rotation_interval")
            blePeripheralManager?.setRotationInterval(newValue)
        }
    }

    // MARK: - Event Streams
    
    /// Stream of ALL message updates (sent and received)
    let messageUpdates = PassthroughSubject<MessageRecord, Never>()

    /// Legacy alias for backward compatibility (filtered to received only if needed, but for now direct alias)
    var incomingMessages: PassthroughSubject<MessageRecord, Never> { messageUpdates }

    let peerEvents = PassthroughSubject<PeerEvent, Never>()
    let statusEvents = PassthroughSubject<StatusEvent, Never>()

    enum PeerEvent {
        case discovered(peerId: String)
        case connected(peerId: String)
        case disconnected(peerId: String)
    }

    enum StatusEvent {
        case serviceStateChanged(ServiceState)
        case statsUpdated(ServiceStats)
    }

    // MARK: - Initialization

    init() {
        // Use app's documents directory for storage
        let documentsPath = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
        self.storagePath = documentsPath.appendingPathComponent("mesh").path

        logger.info("MeshRepository initialized with storage: \(self.storagePath)")

        // Create storage directory if needed
        try? FileManager.default.createDirectory(atPath: storagePath, withIntermediateDirectories: true)
    }

    /// Initialize all managers
    func initialize() throws {
        logger.info("Initializing managers")

        do {
            // Initialize data managers
            settingsManager = MeshSettingsManager(storagePath: storagePath)

            // Ensure settings exist (load or create defaults)
            if (try? settingsManager?.load()) == nil {
                logger.info("No settings found, applying defaults")
                if let defaults = settingsManager?.defaultSettings() {
                    try? settingsManager?.save(settings: defaults)
                }
            }
            historyManager = try HistoryManager(storagePath: storagePath)
            contactManager = try ContactManager(storagePath: storagePath)
            ledgerManager = LedgerManager(storagePath: storagePath)
            autoAdjustEngine = AutoAdjustEngine()

            // Initialize transport managers
            bleCentralManager = BLECentralManager(meshRepository: self)
            blePeripheralManager = BLEPeripheralManager(meshRepository: self)

            // Pre-load data where applicable
            try? ledgerManager?.load()

            logger.info("✓ All managers initialized successfully")
        } catch {
            logger.error("Failed to initialize managers: \(error.localizedDescription)")
            throw error
        }
    }

    /// Public start method called from App entry point
    func start() {
        logger.info("Application requested repository start")
        do {
            try ensureServiceInitialized()

            // Apply saved BLE settings now that managers are initialized
            blePeripheralManager?.setRotationEnabled(blePrivacyEnabled)
            blePeripheralManager?.setRotationInterval(blePrivacyInterval)
        } catch {
            logger.error("Failed to start repository: \(error.localizedDescription)")
        }
    }

    /// Ensure service is initialized (lazy start if needed)
    /// This enables identity operations before full mesh service is running
    private func ensureServiceInitialized() throws {
        // Initialize when service is not running (mirrors Android: state != RUNNING)
        // This properly handles all non-running states including .stopped, .starting, .stopping, .paused
        if meshService == nil || serviceState != .running {
            logger.info("Lazy starting MeshService for Identity access")

            // Clean up existing service if stopped but not nil
            if meshService != nil {
                meshService?.stop()
                meshService = nil
                ironCore = nil
            }

            // Initialize managers if not already done
            if settingsManager == nil {
                try initialize()
            }

            // Create minimal config for lazy start
            // Use saved settings or defaults from settings manager
            guard let settingsManager = settingsManager else {
                throw MeshError.notInitialized("SettingsManager not initialized for lazy start")
            }

            let settings = (try? settingsManager.load()) ?? settingsManager.defaultSettings()

            let config = MeshServiceConfig(
                discoveryIntervalMs: 30000,
                batteryFloorPct: settings.batteryFloor
            )

            try startMeshService(config: config)
            logger.info("✓ MeshService started lazily")
        }

        // Verify ironCore is available after initialization
        if ironCore == nil {
            logger.error("⚠️ IronCore is nil despite service running - attempting refresh")
            ironCore = meshService?.getCore()
            if ironCore == nil {
                throw MeshError.notInitialized("Failed to obtain IronCore from running service")
            }
        }
    }

    // MARK: - Service Lifecycle

    /// Start the mesh service with configuration
    func startMeshService(config: MeshServiceConfig) throws {
        logger.info("Starting mesh service")

        guard serviceState == .stopped else {
            logger.warning("Service already started or starting")
            return
        }

        serviceState = .starting
        statusEvents.send(.serviceStateChanged(.starting))

        do {
            // Create mesh service with persistent storage
            meshService = MeshService.withStorage(config: config, storagePath: storagePath)

            // Start service first — IronCore is created during start()
            try meshService?.start()

            // Configure platform bridge (Swift -> Rust callbacks)
            platformBridge = IosPlatformBridge()
            platformBridge?.configure(repository: self)
            meshService?.setPlatformBridge(bridge: platformBridge)

            // Now obtain IronCore (only available after start())
            ironCore = meshService?.getCore()
            if ironCore == nil {
                throw MeshError.notInitialized("Failed to obtain IronCore from MeshService")
            }

            // Wire CoreDelegate: Rust → Swift callbacks
            let coreDelegate = CoreDelegateImpl(meshRepository: self)
            self.coreDelegateImpl = coreDelegate  // store strong reference
            ironCore?.setDelegate(delegate: coreDelegate)
            logger.info("CoreDelegate registered for Rust->Swift callbacks")

            // Ensure identity exists (foundational requirement)
            if !isIdentityInitialized() {
                logger.info("Auto-initializing new identity for first run")
                try? ironCore?.initializeIdentity()
            }

            // Broadcast BLE identity beacon so nearby peers can read our public key
            broadcastIdentityBeacon()

            // Obtain the SwarmBridge from MeshService (managed by Rust)
            swarmBridge = meshService?.getSwarmBridge()

            // Initialize internet transport if enabled (only if identity is ready)
            let settings = try? settingsManager?.load()
            if settings?.internetEnabled == true && isIdentityInitialized() {
                // Configure bootstrap nodes for NAT traversal
                meshService?.setBootstrapNodes(addrs: Self.defaultBootstrapNodes)
                // Listen on random port
                try? meshService?.startSwarm(listenAddr: "/ip4/0.0.0.0/tcp/0")
                logger.info("Internet transport (Swarm) initiated")
            } else if settings?.internetEnabled == true {
                logger.warning("Postponing Swarm start: Identity not ready")
            }

            serviceState = .running
            statusEvents.send(.serviceStateChanged(.running))

            // Start BLE advertising and scanning
            blePeripheralManager?.startAdvertising()
            bleCentralManager?.startScanning()

            logger.info("✓ Mesh service started successfully")
        } catch {
            serviceState = .stopped
            statusEvents.send(.serviceStateChanged(.stopped))
            logger.error("Failed to start mesh service: \(error.localizedDescription)")
            throw error
        }
    }

    /// Stop the mesh service
    func stopMeshService() {
        logger.info("Stopping mesh service")

        guard serviceState == .running else {
            logger.warning("Service not running")
            return
        }

        serviceState = .stopping
        statusEvents.send(.serviceStateChanged(.stopping))

        meshService?.stop()

        serviceState = .stopped
        statusEvents.send(.serviceStateChanged(.stopped))

        bleCentralManager?.stopScanning()
        blePeripheralManager?.stopAdvertising()

        logger.info("✓ Mesh service stopped")
    }

    /// Pause the mesh service (background mode)
    func pauseMeshService() {
        logger.info("Pausing mesh service")
        guard serviceState == .running else {
            logger.warning("Service not running (current state: \(self.serviceState))")
            return
        }
        meshService?.pause()
        // Note: pause() is an internal operation that reduces activity
        // The external serviceState remains .running (no .paused state exists)
        logger.info("✓ Mesh service paused")
    }

    /// Resume the mesh service (foreground mode)
    func resumeMeshService() {
        logger.info("Resuming mesh service")
        guard serviceState == .running else {
            logger.warning("Cannot resume - service not in running state (current: \(self.serviceState))")
            return
        }
        meshService?.resume()
        logger.info("✓ Mesh service resumed")
    }

    /// Get current service state
    func getServiceState() -> ServiceState {
        return serviceState
    }

    /// Initialize internet transport (Swarm) if enabled and identity is ready
    func initializeAndStartSwarm() {
        guard isIdentityInitialized() else {
            logger.warning("Postponing Swarm start: Identity not ready")
            return
        }

        let settings = try? settingsManager?.load()
        if settings?.internetEnabled == true {
            do {
                // Configure bootstrap nodes for NAT traversal
                meshService?.setBootstrapNodes(addrs: Self.defaultBootstrapNodes)
                try meshService?.startSwarm(listenAddr: "/ip4/0.0.0.0/tcp/0")
                logger.info("✓ Internet transport (Swarm) started manually")
            } catch {
                logger.error("Failed to start swarm: \(error.localizedDescription)")
            }
        }
    }

    // MARK: - Identity Management

    /// Get identity information
    func getIdentityInfo() -> IdentityInfo? {
        return ironCore?.getIdentityInfo()
    }

    /// Check if identity is initialized.
    ///
    /// Intentionally lightweight — reads current ironCore state only.
    /// Do NOT call ensureServiceInitialized() here; this function is called
    /// from inside startMeshService() and a recursive ensureServiceInitialized()
    /// would destroy the service being started (nulling meshService/ironCore mid-flight).
    func isIdentityInitialized() -> Bool {
        return ironCore?.getIdentityInfo().initialized == true
    }

    /// Create a new identity (first-time setup)
    func createIdentity() throws {
        logger.info("Creating identity")

        do {
            try ensureServiceInitialized()

            guard let ironCore = ironCore else {
                logger.error("IronCore is nil after ensureServiceInitialized! Cannot create identity.")
                throw MeshError.notInitialized("Mesh service initialization failed")
            }

            logger.info("Calling ironCore.initializeIdentity()...")
            try ironCore.initializeIdentity()
            logger.info("✓ Identity created successfully")
            broadcastIdentityBeacon()
        } catch {
            logger.error("Failed to create identity: \(error.localizedDescription)")
            throw error
        }
    }

    // MARK: - Messaging (with Relay Enforcement)

    /// Send a message to a peer
    /// CRITICAL: Enforces relay = messaging coupling
    func sendMessage(peerId: String, content: String) async throws {
        logger.info("Send message to \(peerId)")

        // RELAY ENFORCEMENT (matches Android pattern exactly)
        // Check if relay/messaging is enabled (bidirectional control)
        // Treat null/missing settings as disabled (fail-safe)
        // Cache settings value to avoid race condition during check
        let currentSettings = try? settingsManager?.load()
        let isRelayEnabled = currentSettings?.relayEnabled == true

        if !isRelayEnabled {
            let errorMsg = "Cannot send message: Relay is disabled. Enable relay in Settings to send and receive messages."
            logger.error("\(errorMsg)")
            throw MeshError.relayDisabled(errorMsg)
        }

        // Proceed with sending message
        guard let ironCore = ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }

        // Get recipient's public key
        let contact = try? contactManager?.get(peerId: peerId)
        guard let recipientPublicKey = contact?.publicKey else {
            throw MeshError.contactNotFound("Contact \(peerId) not found or has no public key")
        }

        // Pre-validate public key format to provide descriptive errors
        let trimmedKey = recipientPublicKey.trimmingCharacters(in: .whitespacesAndNewlines)
        if trimmedKey.isEmpty {
            logger.error("❌ Contact \(peerId) has an empty public key")
            throw MeshError.contactNotFound("Contact \(peerId) has no public key. Please re-add this contact with a valid public key.")
        }
        if trimmedKey.count != 64 {
            logger.error("❌ Contact \(peerId) has invalid public key length: \(trimmedKey.count) chars (expected 64)")
            throw MeshError.contactNotFound("Contact \(peerId) has an invalid public key (wrong length: \(trimmedKey.count), expected 64 hex characters). Please re-add this contact.")
        }
        let hexChars = CharacterSet(charactersIn: "0123456789abcdefABCDEF")
        if !trimmedKey.unicodeScalars.allSatisfy({ hexChars.contains($0) }) {
            logger.error("❌ Contact \(peerId) has non-hex characters in public key")
            throw MeshError.contactNotFound("Contact \(peerId) has an invalid public key (non-hex characters found). Please re-add this contact.")
        }

        logger.debug("Preparing message for \(peerId) with key: \(trimmedKey.prefix(8))...")

        // Prepare and send message (use trimmed key to handle any stored whitespace)
        let encryptedBytes = try ironCore.prepareMessage(recipientPublicKeyHex: trimmedKey, text: content)

        // Record in history FIRST so it's persisted even if bridge fails
        let messageRecord = MessageRecord(
            id: UUID().uuidString,
            direction: .sent,
            peerId: peerId,
            content: content,
            timestamp: UInt64(Date().timeIntervalSince1970),
            delivered: false
        )
        try? historyManager?.add(record: messageRecord)

        // Notify UI (Unified flow for sent messages)
        messageUpdates.send(messageRecord)

        // 3. Send over network (Multiple transports)
        // Attempt BLE delivery (Best effort, broadcast to all connected peers)
        bleCentralManager?.broadcastData(Data(encryptedBytes))
        blePeripheralManager?.broadcastDataToCentrals(Data(encryptedBytes))

        // Send via SwarmBridge (Network delivery) — broadcast to all connected peers.
        // The message is encrypted for the specific recipient; only they can decrypt it.
        if let swarmBridge = swarmBridge {
            do {
                try swarmBridge.sendToAllPeers(data: Data(encryptedBytes))
                logger.info("✓ Message broadcast to peers: \(encryptedBytes.count) bytes")
            } catch {
                logger.warning("SwarmBridge delivery queued (no peers connected): \(error.localizedDescription)")
            }
        } else {
            logger.warning("SwarmBridge not initialized, message saved locally for later delivery.")
        }
    }

    /// Handle incoming message (from CoreDelegate callback)
    func onMessageReceived(senderId: String, senderPublicKeyHex: String, messageId: String, data: Data) {
        logger.info("Message from \(senderId): \(messageId)")

        // RELAY ENFORCEMENT (matches Android pattern exactly)
        // Check if relay/messaging is enabled (bidirectional control)
        // Treat null/missing settings as disabled (fail-safe)
        // Cache settings value to avoid race condition during check
        let currentSettings = try? settingsManager?.load()
        let isRelayEnabled = currentSettings?.relayEnabled == true

        if !isRelayEnabled {
            // Silently drop message when relay disabled (matches Android)
            logger.warning("Dropped message from \(senderId): relay disabled")
            return
        }

        let trimmedKey = senderPublicKeyHex.trimmingCharacters(in: .whitespacesAndNewlines)
        let canonicalPeerId = resolveCanonicalPeerId(senderId: senderId, senderPublicKeyHex: trimmedKey)
        if canonicalPeerId != senderId {
            logger.info("Canonicalized sender \(senderId) -> \(canonicalPeerId) using public key match")
        }

        // Auto-upsert contact: senderPublicKeyHex is guaranteed valid (Rust verified it during decrypt)
        let existingContact = try? contactManager?.get(peerId: canonicalPeerId)
        if existingContact == nil {
            if trimmedKey.count == 64 {
                let autoContact = Contact(
                    peerId: canonicalPeerId,
                    nickname: nil,
                    publicKey: trimmedKey,
                    addedAt: UInt64(Date().timeIntervalSince1970),
                    lastSeen: UInt64(Date().timeIntervalSince1970),
                    notes: nil
                )
                do {
                    try contactManager?.add(contact: autoContact)
                    logger.info("Auto-created contact from received message: \(canonicalPeerId.prefix(8)) key: \(trimmedKey.prefix(8))...")
                } catch {
                    logger.warning("Auto-create contact failed for \(canonicalPeerId.prefix(8)): \(error.localizedDescription)")
                }
            }
        } else {
            try? contactManager?.updateLastSeen(peerId: canonicalPeerId)
        }

        // Process message
        let content = String(data: data, encoding: .utf8) ?? "[binary]"

        let messageRecord = MessageRecord(
            id: messageId,
            direction: .received,
            peerId: canonicalPeerId,
            content: content,
            timestamp: UInt64(Date().timeIntervalSince1970),
            delivered: true
        )

        try? historyManager?.add(record: messageRecord)

        // Notify UI
        messageUpdates.send(messageRecord)
        logger.info("Message received and processed from \(canonicalPeerId)")

        // Send delivery receipt ACK back to sender
        Task {
            do {
                let receiptBytes = try ironCore?.prepareReceipt(recipientPublicKeyHex: senderPublicKeyHex, messageId: messageId)
                if let receiptBytes = receiptBytes {
                    try swarmBridge?.sendToAllPeers(data: receiptBytes)
                    logger.debug("Delivery receipt broadcast for \(messageId)")
                }
            } catch {
                logger.debug("Failed to send delivery receipt for \(messageId): \(error)")
            }
        }
    }

    /// Resolve incoming sender IDs to an existing contact using public-key identity.
    ///
    /// This prevents duplicate conversations when transport IDs differ
    /// (e.g. libp2p peer ID vs identity ID) but cryptographic identity is the same.
    private func resolveCanonicalPeerId(senderId: String, senderPublicKeyHex: String) -> String {
        guard let normalizedIncomingKey = normalizePublicKey(senderPublicKeyHex),
              let contacts = try? contactManager?.list() else {
            return senderId
        }

        let keyMatches = contacts.filter { normalizePublicKey($0.publicKey) == normalizedIncomingKey }
        guard !keyMatches.isEmpty else { return senderId }

        let best = keyMatches.max { lhs, rhs in
            contactRank(lhs, senderId: senderId) < contactRank(rhs, senderId: senderId)
        }
        return best?.peerId ?? senderId
    }

    private func normalizePublicKey(_ key: String?) -> String? {
        guard let value = key?.trimmingCharacters(in: .whitespacesAndNewlines),
              value.count == 64 else {
            return nil
        }
        return value.lowercased()
    }

    private func contactRank(_ contact: Contact, senderId: String) -> Int {
        var score = 0
        if contact.peerId == senderId { score += 4 }
        if let nickname = contact.nickname, !nickname.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty { score += 8 }
        if contact.peerId.hasPrefix("12D3Koo") || contact.peerId.hasPrefix("Qm") { score += 1 }
        return score
    }

    // MARK: - Settings Management

    func loadSettings() throws -> MeshSettings {
        guard let settingsManager = settingsManager else {
            throw MeshError.notInitialized("SettingsManager not initialized")
        }
        return try settingsManager.load()
    }

    func saveSettings(_ settings: MeshSettings) throws {
        guard let settingsManager = settingsManager else {
            throw MeshError.notInitialized("SettingsManager not initialized")
        }
        try settingsManager.save(settings: settings)
        logger.info("✓ Settings saved (relay: \(settings.relayEnabled))")
    }

    func validateSettings(_ settings: MeshSettings) -> Bool {
        // Delegate to Rust-side validation via UniFFI for consistency with Android
        guard let settingsManager = settingsManager else {
            logger.error("SettingsManager not initialized; cannot validate settings")
            return false
        }

        do {
            try settingsManager.validate(settings: settings)
            return true
        } catch {
            logger.warning("Settings validation failed: \(String(describing: error))")
            return false
        }
    }

    // MARK: - Ledger Management

    func recordConnection(multiaddr: String, peerId: String) throws {
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        ledgerManager.recordConnection(multiaddr: multiaddr, peerId: peerId)
    }

    func recordConnectionFailure(multiaddr: String) throws {
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        ledgerManager.recordFailure(multiaddr: multiaddr)
    }

    func getDialableAddresses() throws -> [LedgerEntry] {
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        return ledgerManager.dialableAddresses()
    }

    func getAllKnownTopics() throws -> [String] {
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        return ledgerManager.allKnownTopics()
    }

    func getLedgerSummary() throws -> String {
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        return ledgerManager.summary()
    }

    func saveLedger() throws {
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        try ledgerManager.save()
    }

    // MARK: - Contacts Management

    func getContacts() throws -> [Contact] {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        return try contactManager.list()
    }

    func getContact(peerId: String) throws -> Contact? {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        return try contactManager.get(peerId: peerId)
    }

    func addContact(_ contact: Contact) throws {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        try contactManager.add(contact: contact)
        logger.info("✓ Contact added: \(contact.peerId)")
    }

    func removeContact(peerId: String) throws {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        try contactManager.remove(peerId: peerId)
        try? historyManager?.removeConversation(peerId: peerId)
        logger.info("✓ Contact removed: \(peerId) and their message history")
    }

    func searchContacts(query: String) throws -> [Contact] {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        return try contactManager.search(query: query)
    }

    func setContactNickname(peerId: String, nickname: String?) throws {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        try contactManager.setNickname(peerId: peerId, nickname: nickname)
        logger.info("✓ Contact nickname updated: \(peerId)")
    }

    func getContactCount() throws -> UInt32 {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        return contactManager.count()
    }

    // MARK: - Message History

    func getConversation(peerId: String, limit: UInt32 = 100) throws -> [MessageRecord] {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        return try historyManager.conversation(peerId: peerId, limit: limit)
    }

    func getRecentMessages(peerIdFilter: String? = nil, limit: UInt32 = 50) throws -> [MessageRecord] {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        return try historyManager.recent(peerFilter: peerIdFilter, limit: limit)
    }

    func getMessage(id: String) throws -> MessageRecord? {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        return try historyManager.get(id: id)
    }

    func addMessage(record: MessageRecord) throws {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        try historyManager.add(record: record)
    }

    func searchMessages(query: String, limit: UInt32 = 50) throws -> [MessageRecord] {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        return try historyManager.search(query: query, limit: limit)
    }

    func markMessageDelivered(id: String) throws {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        try historyManager.markDelivered(id: id)
    }

    func clearHistory() throws {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        try historyManager.clear()
        logger.info("✓ Message history cleared")
    }

    func clearConversation(peerId: String) throws {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        try historyManager.clearConversation(peerId: peerId)
        logger.info("✓ Conversation cleared for peer: \(peerId)")
    }

    func getHistoryStats() throws -> HistoryStats? {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        return try historyManager.stats()
    }

    func getMessageCount() throws -> UInt32 {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        return historyManager.count()
    }

    // MARK: - Platform Reporting

    func reportBattery(pct: UInt8, charging: Bool) {
        logger.debug("Battery: \(pct)% charging=\(charging)")
        currentBatteryPct = pct
        currentIsCharging = charging

        // 1. Report to Rust MeshService
        let profile = DeviceProfile(
            batteryPct: pct,
            isCharging: charging,
            hasWifi: networkStatus.wifi,
            motionState: currentMotionState
        )
        meshService?.updateDeviceState(profile: profile)

        // 2. Auto-adjust local transport and relay budgets
        if let engine = autoAdjustEngine {
            let adjProfile = engine.computeProfile(device: profile)
            let relayAdj = engine.computeRelayAdjustment(profile: adjProfile)

            // Apply new budget to MeshService
            meshService?.setRelayBudget(messagesPerHour: relayAdj.maxPerHour)

            // Adjust BLE intervals if needed
            let bleAdj = engine.computeBleAdjustment(profile: adjProfile)
            bleCentralManager?.applyScanSettings(intervalMs: bleAdj.scanIntervalMs)
            logger.info("Auto-adjusted relay budget: \(relayAdj.maxPerHour)/hr")
        }
    }

    func reportNetwork(wifi: Bool, cellular: Bool) {
        logger.debug("Network: wifi=\(wifi) cellular=\(cellular)")
        networkStatus.wifi = wifi
        networkStatus.cellular = cellular

        // Report to Rust
        let profile = DeviceProfile(
            batteryPct: currentBatteryPct,
            isCharging: currentIsCharging,
            hasWifi: wifi,
            motionState: currentMotionState
        )
        meshService?.updateDeviceState(profile: profile)
    }

    func reportMotion(state: MotionState) {
        logger.debug("Motion: \(state)")
        currentMotionState = state

        // Report to Rust
        let profile = DeviceProfile(
            batteryPct: currentBatteryPct,
            isCharging: currentIsCharging,
            hasWifi: networkStatus.wifi,
            motionState: state
        )
        meshService?.updateDeviceState(profile: profile)
    }

    // MARK: - Background Operations

    func onEnteringBackground() {
        logger.info("Repository: entering background")
        // Reduce activity, save state
    }

    func onEnteringForeground() {
        logger.info("Repository: entering foreground")
        // Resume full activity
    }

    func pauseService() {
        logger.info("Pausing service")
        // Pause but don't stop (for background expiration)
    }

    func syncPendingMessages() async throws {
        logger.info("Syncing pending messages")
        // Sync outbox with network
    }

    func updateStats() {
        logger.info("Updating stats")
        if let service = meshService {
            serviceStats = service.getStats()
            if let stats = serviceStats {
                statusEvents.send(.statsUpdated(stats))
            }
        }
    }

    func quickPeerDiscovery() async throws {
        logger.info("Quick peer discovery")
        // Brief scan for nearby peers
    }

    func performBulkSync() async throws {
        logger.info("Performing bulk sync")
        // Full sync with all peers
    }

    func cleanupOldMessages() async throws {
        logger.info("Cleaning up old messages")
        // Remove old messages based on retention policy
    }

    func updatePeerLedger() async throws {
        logger.info("Updating peer ledger")
        // Update connection statistics
    }

    // MARK: - BLE Transport Integration

    func onBleDataReceived(peerId: String, data: Data) {
        logger.debug("BLE data from \(peerId): \(data.count) bytes")
        // Forward to MeshService
        meshService?.onDataReceived(peerId: peerId, data: data)
    }

    func sendBlePacket(peerId: String, data: Data) {
        logger.debug("Send BLE packet to \(peerId): \(data.count) bytes")

        // Direct packet to appropriate manager based on UUID match
        // Note: peerId here is likely the UUID from the transport layer if Rust is treating it as a handle

        // Try Central role first (we are client, sending to peripheral)
        if let uuid = UUID(uuidString: peerId) {
            bleCentralManager?.sendData(to: uuid, data: data)
        } else {
            logger.warning("sendBlePacket: Invalid UUID string \(peerId)")
        }

        // Also try Peripheral role (we are server, pushing notification to central)
        blePeripheralManager?.sendDataToConnectedCentral(peerId: peerId, data: data)
    }

    /// Called when BLE central reads the identity GATT characteristic from a peer.
    /// Automatically creates a contact if none exists for this peer.
    func onPeerIdentityRead(blePeerId: String, info: [String: Any]) {
        guard let publicKeyHex = info["public_key"] as? String else { return }
        let nickname = info["nickname"] as? String
        let libp2pPeerId = info["libp2p_peer_id"] as? String
        let listeners = info["listeners"] as? [String]

        let idFromInfo = (info["identity_id"] as? String)?.trimmingCharacters(in: .whitespacesAndNewlines)
        let identityId = (idFromInfo?.isEmpty == false) ? idFromInfo! : blePeerId

        logger.info("Peer BLE identity read: \(blePeerId.prefix(8)) key: \(publicKeyHex.prefix(8))...")
        let trimmedKey = publicKeyHex.trimmingCharacters(in: .whitespacesAndNewlines)
        guard trimmedKey.count == 64 else {
            logger.warning("Ignoring BLE identity from \(blePeerId.prefix(8)): wrong key length \(trimmedKey.count)")
            return
        }

        // Store libp2p info in contact notes for recovery
        var notes: String?
        if let peerId = libp2pPeerId, !peerId.isEmpty {
            let addrs = (listeners ?? []).joined(separator: ",")
            notes = "libp2p_peer_id:\(peerId);listeners:\(addrs)"
        }

        // Emit to nearby peers bus — UI will show peer in Nearby section for user to manually add
        let nonEmptyNickname = (nickname?.isEmpty == false) ? nickname : nil
        let nonEmptyLibp2p = (libp2pPeerId?.isEmpty == false) ? libp2pPeerId : nil
        MeshEventBus.shared.peerEvents.send(.identityDiscovered(
            peerId: identityId,
            publicKey: trimmedKey,
            nickname: nonEmptyNickname,
            libp2pPeerId: nonEmptyLibp2p,
            listeners: listeners ?? [],
            blePeerId: blePeerId
        ))
        logger.info("Emitted identityDiscovered for \(blePeerId.prefix(8)) key: \(trimmedKey.prefix(8))...")
        // Update lastSeen if already a saved contact
        if (try? contactManager?.get(peerId: blePeerId)) != nil {
            try? contactManager?.updateLastSeen(peerId: blePeerId)
        }

        // Auto-dial discovered peer via Swarm if we have libp2p info
        if let peerId = libp2pPeerId, let addrs = listeners, !peerId.isEmpty, !addrs.isEmpty {
            logger.info("Auto-dialing discovered peer over Swarm: \(peerId)")
            connectToPeer(peerId, addresses: addrs)
        }
    }

    private func broadcastIdentityBeacon() {
        guard let info = ironCore?.getIdentityInfo(),
              let publicKeyHex = info.publicKeyHex else { return }

        let listeners = getListeningAddresses()
        let beacon: [String: Any] = [
            "identity_id": info.identityId ?? "",
            "public_key": publicKeyHex,
            "nickname": info.nickname ?? "",
            "libp2p_peer_id": info.libp2pPeerId ?? "",
            "listeners": listeners
        ]
        guard let data = try? JSONSerialization.data(withJSONObject: beacon) else {
            logger.error("Failed to serialize identity beacon")
            return
        }
        blePeripheralManager?.setIdentityData(data)
        logger.info("BLE identity beacon set: \(publicKeyHex.prefix(8))... (includes libp2p)")
    }

    // MARK: - Auto-Adjustment Engine

    func computeAdjustmentProfile(deviceProfile: DeviceProfile) throws -> AdjustmentProfile {
        guard let autoAdjustEngine = autoAdjustEngine else {
            throw MeshError.notInitialized("AutoAdjustEngine not initialized")
        }
        return autoAdjustEngine.computeProfile(device: deviceProfile)
    }

    func computeBleAdjustment(profile: AdjustmentProfile) throws -> BleAdjustment {
        guard let autoAdjustEngine = autoAdjustEngine else {
            throw MeshError.notInitialized("AutoAdjustEngine not initialized")
        }
        return autoAdjustEngine.computeBleAdjustment(profile: profile)
    }

    func computeRelayAdjustment(profile: AdjustmentProfile) throws -> RelayAdjustment {
        guard let autoAdjustEngine = autoAdjustEngine else {
            throw MeshError.notInitialized("AutoAdjustEngine not initialized")
        }
        return autoAdjustEngine.computeRelayAdjustment(profile: profile)
    }

    func overrideBleInterval(scanMs: UInt32, advertiseMs: UInt32) throws {
        guard let autoAdjustEngine = autoAdjustEngine else {
            throw MeshError.notInitialized("AutoAdjustEngine not initialized")
        }
        // Note: Only scan interval override is supported in new API
        autoAdjustEngine.overrideBleScanInterval(intervalMs: scanMs)
        logger.info("✓ BLE interval overridden: scan=\(scanMs)ms advertise=\(advertiseMs)ms")
    }

    func overrideRelayMax(maxRelayPerHour: UInt32) throws {
        guard let autoAdjustEngine = autoAdjustEngine else {
            throw MeshError.notInitialized("AutoAdjustEngine not initialized")
        }
        autoAdjustEngine.overrideRelayMaxPerHour(max: maxRelayPerHour)
        logger.info("✓ Relay max overridden: \(maxRelayPerHour)/hour")
    }

    func clearAdjustmentOverrides() throws {
        guard let autoAdjustEngine = autoAdjustEngine else {
            throw MeshError.notInitialized("AutoAdjustEngine not initialized")
        }
        autoAdjustEngine.clearOverrides()
        logger.info("✓ Adjustment overrides cleared")
    }

    // MARK: - Identity Export Helpers

    func getPreferredRelay() -> String? {
        return ledgerManager?.getPreferredRelays(limit: 1).first?.peerId
    }

    func connectToPeer(_ peerId: String, addresses: [String]) {
        for addr in addresses {
            // Only append /p2p/ component if the peerId is a valid libp2p PeerId format
            // (base58btc multihash, starts with "12D3Koo" or "Qm").
            // Blake3 hex identity_ids (64 hex chars) are NOT valid libp2p PeerIds.
            var finalAddr = addr
            let isLibp2pPeerId = peerId.hasPrefix("12D3Koo") || peerId.hasPrefix("Qm")
            if isLibp2pPeerId && !addr.contains("/p2p/") {
                finalAddr = "\(addr)/p2p/\(peerId)"
            }
            do {
                try swarmBridge?.dial(multiaddr: finalAddr)
                logger.info("Dialing \(finalAddr)")
            } catch {
                logger.error("Failed to dial \(finalAddr): \(error.localizedDescription)")
            }
        }
    }

    func getListeningAddresses() -> [String] {
        return swarmBridge?.getListeners() ?? []
    }

    func getTopics() -> [String] {
        return swarmBridge?.getTopics() ?? []
    }

    func subscribeTopic(_ topic: String) throws {
        guard let swarmBridge = swarmBridge else {
            throw MeshError.notInitialized("SwarmBridge not initialized")
        }
        try swarmBridge.subscribeTopic(topic: topic)
    }

    func unsubscribeTopic(_ topic: String) throws {
        guard let swarmBridge = swarmBridge else {
            throw MeshError.notInitialized("SwarmBridge not initialized")
        }
        try swarmBridge.unsubscribeTopic(topic: topic)
    }

    func publishTopic(_ topic: String, data: Data) throws {
        guard let swarmBridge = swarmBridge else {
            throw MeshError.notInitialized("SwarmBridge not initialized")
        }
        try swarmBridge.publishTopic(topic: topic, data: data)
    }

    func getLocalIpAddress() -> String? {
        var address: String?
        var ifaddr: UnsafeMutablePointer<ifaddrs>?
        if getifaddrs(&ifaddr) == 0 {
            var ptr = ifaddr
            while ptr != nil {
                defer { ptr = ptr?.pointee.ifa_next }

                let interface = ptr?.pointee
                let addrFamily = interface?.ifa_addr.pointee.sa_family

                if addrFamily == UInt8(AF_INET) { // IPv4 only for now
                    if let namePtr = interface?.ifa_name,
                       let name = String(validatingUTF8: namePtr),
                       name == "en0" { // Default WiFi interface on iOS
                        var hostname = [CChar](repeating: 0, count: Int(NI_MAXHOST))
                        getnameinfo(interface?.ifa_addr, socklen_t(interface?.ifa_addr.pointee.sa_len ?? 0),
                                   &hostname, socklen_t(hostname.count),
                                   nil, socklen_t(0), NI_NUMERICHOST)
                        address = String(cString: hostname)
                    }
                }
            }
            freeifaddrs(ifaddr)
        }
        return address
    }

    // MARK: - Identity Helpers

    func getFullIdentityInfo() -> IdentityInfo? {
        return ironCore?.getIdentityInfo()
    }

    func getIdentityExportString() -> String {
        guard let identity = getFullIdentityInfo() else { return "{}" }
        var listeners = getListeningAddresses()
        let relay = getPreferredRelay() ?? "None"
        let localIp = getLocalIpAddress()

        // Replace 0.0.0.0 with actual LAN IP
        if let localIp = localIp {
            var updatedListeners = [String]()
            for addr in listeners {
                if addr.contains("0.0.0.0") {
                    updatedListeners.append(addr.replacingOccurrences(of: "0.0.0.0", with: localIp))
                } else {
                    updatedListeners.append(addr)
                }
            }

            // If empty, suggest standard port
            if updatedListeners.isEmpty {
                updatedListeners.append("/ip4/\(localIp)/tcp/9001 (Potential)")
            }
            listeners = updatedListeners
        }

        let listenersJson = "[\"\(listeners.joined(separator: "\",\""))\"]"

        let libp2pId = identity.libp2pPeerId ?? ""
        return """
        {
          "identity_id": "\(identity.identityId ?? "")",
          "nickname": "\(identity.nickname ?? "")",
          "public_key": "\(identity.publicKeyHex ?? "")",
          "libp2p_peer_id": "\(libp2pId)",
          "listeners": \(listeners.isEmpty ? "[]" : listenersJson),
          "relay": "\(relay)"
        }
        """
    }

    func getIdentitySnippet() -> String {
        guard let identity = ironCore?.getIdentityInfo(),
              let publicKey = identity.publicKeyHex else {
            return "????????"
        }
        return String(publicKey.prefix(8))
    }

    func getIdentityDisplay() -> String {
        if let nick = ironCore?.getIdentityInfo().nickname {
            return nick
        }
        return getIdentitySnippet()
    }

    func getNickname() -> String? {
        return ironCore?.getIdentityInfo().nickname
    }

    func setNickname(_ nickname: String) throws {
        guard let ironCore = ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }
        try ironCore.setNickname(nickname: nickname)
        logger.info("✓ Nickname set to: \(nickname)")
        broadcastIdentityBeacon()
    }
}

// MARK: - Error Types

enum MeshError: LocalizedError {
    case notInitialized(String)
    case relayDisabled(String)
    case contactNotFound(String)
    case alreadyRunning

    var errorDescription: String? {
        switch self {
        case .notInitialized(let msg): return msg
        case .relayDisabled(let msg): return msg
        case .contactNotFound(let msg): return msg
        case .alreadyRunning: return "Service is already running"
        }
    }
}
