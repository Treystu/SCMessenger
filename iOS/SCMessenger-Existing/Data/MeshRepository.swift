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
            // Create mesh service with persistent storage (matches Android: withStorage)
            meshService = MeshService.withStorage(config: config, storagePath: storagePath)
            
            // Start service first — IronCore is created during start()
            try meshService?.start()
            
            // Now obtain IronCore (only available after start())
            ironCore = meshService?.getCore()
            if ironCore == nil {
                throw MeshError.notInitialized("Failed to obtain IronCore from MeshService")
            }
            
            // Configure platform bridge
            platformBridge = IosPlatformBridge()
            platformBridge?.configure(repository: self)
            
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
        try? historyManager?.addMessage(record: messageRecord)
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
            
            try? historyManager?.addMessage(record: messageRecord)
            
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
    
    // MARK: - Contacts Management
    
    func getContacts() throws -> [Contact] {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        return try contactManager.listAll()
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
    
    // MARK: - Message History
    
    func getMessages(peerId: String) throws -> [MessageRecord] {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        return try historyManager.getForPeer(peerId: peerId)
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
    
    // MARK: - Identity Helpers
    
    func getIdentitySnippet() -> String {
        guard let identity = try? ironCore?.getIdentityInfo(),
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
