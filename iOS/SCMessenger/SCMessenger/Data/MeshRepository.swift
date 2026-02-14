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
@Observable
final class MeshRepository {
    private let logger = Logger(subsystem: "com.scmessenger", category: "Repository")
    private let storagePath: String
    
    // MARK: - UniFFI Components (lazy initialization)
    
    private(set) var ironCore: IronCore?
    private(set) var meshService: MeshService?
    private(set) var contactManager: ContactManager?
    private(set) var historyManager: HistoryManager?
    private(set) var ledgerManager: LedgerManager?
    private(set) var settingsManager: MeshSettingsManager?
    private(set) var autoAdjustEngine: AutoAdjustEngine?
    private(set) var swarmBridge: SwarmBridge?
    
    // Platform bridge
    private var platformBridge: IosPlatformBridge?
    
    // MARK: - Published State
    
    var serviceState: ServiceState = .stopped
    var serviceStats: ServiceStats?
    
    // MARK: - Event Streams
    
    let incomingMessages = PassthroughSubject<MessageRecord, Never>()
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
            settingsManager = try MeshSettingsManager(storagePath: storagePath)
            historyManager = try HistoryManager(storagePath: storagePath)
            contactManager = try ContactManager(storagePath: storagePath)
            ledgerManager = try LedgerManager(storagePath: storagePath)
            autoAdjustEngine = AutoAdjustEngine()
            
            // Pre-load data where applicable
            try? ledgerManager?.load()
            
            logger.info("✓ All managers initialized successfully")
        } catch {
            logger.error("Failed to initialize managers: \(error.localizedDescription)")
            throw error
        }
    }
    
    /// Ensure service is initialized (lazy start if needed)
    /// This enables identity operations before full mesh service is running
    private func ensureServiceInitialized() throws {
        // Check if we need to start/restart the service
        if meshService == nil || serviceState != .running {
            logger.info("Lazy starting MeshService for Identity access")
            
            // Clean up existing service if in invalid state
            if meshService != nil && serviceState != .running {
                meshService?.stop()
                meshService = nil
                serviceState = .stopped
            }
            
            // Initialize managers if not already done
            if settingsManager == nil {
                try initialize()
            }
            
            // Create minimal config for lazy start
            // Use saved settings or sensible defaults
            let settings = (try? settingsManager?.load()) ?? MeshSettings(
                relayEnabled: true,
                maxRelayBudget: DefaultSettings.maxRelayBudget,
                batteryFloor: DefaultSettings.batteryFloor,
                autoAdjust: true
            )
            
            let config = MeshServiceConfig(
                discoveryIntervalMs: 30000,
                relayBudgetPerHour: settings.maxRelayBudget,
                batteryFloorPct: settings.batteryFloor
            )
            
            try startMeshService(config: config)
            logger.info("✓ MeshService started lazily")
        }
        
        // Refresh ironCore reference just in case
        if ironCore == nil {
            ironCore = meshService?.getCore()
            logger.info("IronCore reference refreshed: \(ironCore != nil)")
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
            // Create mesh service
            meshService = try MeshService(config: config)
            
            // Initialize IronCore (but don't create identity - that's done separately)
            if ironCore == nil {
                ironCore = meshService?.getCore()
            }
            
            // Configure platform bridge
            platformBridge = IosPlatformBridge()
            platformBridge?.configure(repository: self)
            
            // Start service
            try meshService?.start()
            
            serviceState = .running
            statusEvents.send(.serviceStateChanged(.running))
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
        logger.info("✓ Mesh service stopped")
    }
    
    /// Pause the mesh service (background mode)
    func pauseMeshService() {
        logger.info("Pausing mesh service")
        guard serviceState == .running else {
            logger.warning("Service not running")
            return
        }
        meshService?.pause()
        logger.info("✓ Mesh service paused")
    }
    
    /// Resume the mesh service (foreground mode)
    func resumeMeshService() {
        logger.info("Resuming mesh service")
        meshService?.resume()
        logger.info("✓ Mesh service resumed")
    }
    
    /// Get current service state
    func getServiceState() -> ServiceState {
        return serviceState
    }
    
    // MARK: - Identity Management
    
    /// Get identity information
    func getIdentityInfo() -> IdentityInfo? {
        return ironCore?.getIdentityInfo()
    }
    
    /// Check if identity is initialized
    func isIdentityInitialized() -> Bool {
        do {
            try ensureServiceInitialized()
            return ironCore?.getIdentityInfo()?.initialized == true
        } catch {
            logger.error("Failed to check identity status: \(error.localizedDescription)")
            return false
        }
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
            throw MeshError.contactNotFound("Contact \(peerId) not found")
        }
        
        // Prepare and send message
        let encryptedBytes = try ironCore.prepareMessage(recipientPublicKeyHex: recipientPublicKey, text: content)
        
        // TODO: Actually send via network transport
        // For now, just add to outbox
        logger.info("✓ Message prepared and added to outbox: \(encryptedBytes.count) bytes")
        
        // Record in history
        let messageRecord = MessageRecord(
            id: UUID().uuidString,
            direction: .sent,
            peerId: peerId,
            content: content,
            timestamp: UInt64(Date().timeIntervalSince1970),
            delivered: false
        )
        try? historyManager?.add(record: messageRecord)
    }
    
    /// Handle incoming message (from CoreDelegate callback)
    func onMessageReceived(senderId: String, messageId: String, data: Data) {
        logger.info("Message from \(senderId): \(messageId)")
        
        // RELAY ENFORCEMENT (matches Android pattern exactly)
        // Check if relay/messaging is enabled (bidirectional control)
        // Treat null/missing settings as disabled (fail-safe)
        // Cache settings value to avoid race condition during check
        let currentSettings = try? settingsManager?.load()
        let isRelayEnabled = currentSettings?.relayEnabled == true
        
        if !isRelayEnabled {
            // Silently drop message when relay disabled (matches Android)
            logger.warning("⚠️ Dropped message from \(senderId): relay disabled")
            return
        }
        
        // Process message
        do {
            // Decrypt message (if needed)
            // For now, just record in history
            let content = String(data: data, encoding: .utf8) ?? "[binary]"
            
            let messageRecord = MessageRecord(
                id: messageId,
                direction: .received,
                peerId: senderId,
                content: content,
                timestamp: UInt64(Date().timeIntervalSince1970),
                delivered: true
            )
            
            try? historyManager?.add(record: messageRecord)
            
            // Notify UI
            incomingMessages.send(messageRecord)
            logger.info("✓ Message received and processed from \(senderId)")
        }
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
        // Validate settings constraints
        guard settings.maxRelayBudget > 0 else { return false }
        guard settings.batteryFloor >= 0 && settings.batteryFloor <= 100 else { return false }
        return true
    }
    
    // MARK: - Ledger Management
    
    func recordConnection(multiaddr: String, peerId: String) throws {
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        try ledgerManager.recordConnection(multiaddr: multiaddr, peerId: peerId)
    }
    
    func recordConnectionFailure(multiaddr: String) throws {
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        try ledgerManager.recordFailure(multiaddr: multiaddr)
    }
    
    func getDialableAddresses() throws -> [LedgerEntry] {
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        return try ledgerManager.dialable()
    }
    
    func getAllKnownTopics() throws -> [String] {
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        return try ledgerManager.allTopics()
    }
    
    func getLedgerSummary() throws -> String {
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        return try ledgerManager.summary()
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
        logger.info("✓ Contact removed: \(peerId)")
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
        return try contactManager.count()
    }
    
    // MARK: - Message History
    
    func getMessages(peerId: String) throws -> [MessageRecord] {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        return try historyManager.conversation(peerId: peerId, limit: 100)
    }
    
    func getConversation(peerId: String, limit: UInt32 = 100) throws -> [MessageRecord] {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        return try historyManager.conversation(peerId: peerId, limit: limit)
    }
    
    func getRecentMessages(peerFilter: String? = nil, limit: UInt32 = 50) throws -> [MessageRecord] {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        return try historyManager.recent(peerFilter: peerFilter, limit: limit)
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
        return try historyManager.count()
    }
    
    // MARK: - Platform Reporting
    
    func reportBattery(pct: UInt8, charging: Bool) {
        // Update device profile and report to Rust
        logger.debug("Battery: \(pct)% charging=\(charging)")
        // TODO: Report to autoAdjustEngine
    }
    
    func reportNetwork(wifi: Bool, cellular: Bool) {
        logger.debug("Network: wifi=\(wifi) cellular=\(cellular)")
        // TODO: Report to autoAdjustEngine
    }
    
    func reportMotion(state: MotionState) {
        logger.debug("Motion: \(state)")
        // TODO: Report to autoAdjustEngine
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
        // Handle BLE data packet
    }
    
    func sendBlePacket(peerId: String, data: Data) {
        logger.debug("Send BLE packet to \(peerId): \(data.count) bytes")
        // Send via BLE transport
    }
    
    // MARK: - Auto-Adjustment Engine
    
    func computeAdjustmentProfile(deviceProfile: DeviceProfile) throws -> AdjustmentProfile {
        guard let autoAdjustEngine = autoAdjustEngine else {
            throw MeshError.notInitialized("AutoAdjustEngine not initialized")
        }
        return autoAdjustEngine.computeProfile(deviceProfile: deviceProfile)
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
        autoAdjustEngine.overrideBleInterval(scanMs: scanMs, advertiseMs: advertiseMs)
        logger.info("✓ BLE interval overridden: scan=\(scanMs)ms advertise=\(advertiseMs)ms")
    }
    
    func overrideRelayMax(maxRelayPerHour: UInt32) throws {
        guard let autoAdjustEngine = autoAdjustEngine else {
            throw MeshError.notInitialized("AutoAdjustEngine not initialized")
        }
        autoAdjustEngine.overrideRelayMax(maxRelayPerHour: maxRelayPerHour)
        logger.info("✓ Relay max overridden: \(maxRelayPerHour)/hour")
    }
    
    func clearAdjustmentOverrides() throws {
        guard let autoAdjustEngine = autoAdjustEngine else {
            throw MeshError.notInitialized("AutoAdjustEngine not initialized")
        }
        autoAdjustEngine.clearOverrides()
        logger.info("✓ Adjustment overrides cleared")
    }
    
    // MARK: - Identity Helpers
    
    func getIdentitySnippet() -> String {
        guard let identity = ironCore?.getIdentityInfo(),
              let publicKey = identity.publicKeyHex else {
            return "????????"
        }
        return String(publicKey.prefix(8))
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
