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
import Security

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
    private enum IdentityBackupStore {
        static let service = "com.scmessenger.identity"
        static let account = "identity_backup_v1"
    }
    private let logger = Logger(subsystem: "com.scmessenger", category: "Repository")
    private let storagePath: String

    // MARK: - Bootstrap Nodes for NAT Traversal

    /// Static fallback bootstrap node multiaddrs for NAT traversal and internet roaming.
    /// These are used only if env override and remote fetch both fail/are absent.
    private static let staticBootstrapNodes: [String] = [
        "/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWL6KesqENjgojaLTxJiwXdvgmEkbvh1znyu8FdJQEizmV",
    ]

    /// Resolved bootstrap nodes using the core BootstrapResolver.
    /// Priority: SC_BOOTSTRAP_NODES env var → remote URL → static fallback.
    static let defaultBootstrapNodes: [String] = {
        let config = BootstrapConfig(
            staticNodes: staticBootstrapNodes,
            remoteUrl: nil,  // Set to a bootstrap-list URL when available
            fetchTimeoutSecs: 5,
            envOverrideKey: "SC_BOOTSTRAP_NODES"
        )
        return BootstrapResolver(config: config).resolve()
    }()

    private static let bootstrapRelayPeerIds: Set<String> = Set(
        defaultBootstrapNodes.compactMap { parseBootstrapRelay(from: $0)?.relayPeerId }
    )

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
    private var pendingOutboxRetryTask: Task<Void, Never>?
    private var lastRelayBootstrapDialAt: Date = .distantPast
    private let receiptAwaitSeconds: UInt64 = 8
    private var identitySyncSentPeers: Set<String> = []

    private var pendingOutboxURL: URL {
        URL(fileURLWithPath: storagePath).appendingPathComponent("pending_outbox.json")
    }

    private struct RoutingHints {
        let libp2pPeerId: String?
        let addresses: [String]
    }

    private struct TransportIdentityResolution {
        let canonicalPeerId: String
        let publicKey: String
        let nickname: String?
    }

    private struct PendingOutboundEnvelope: Codable {
        let queueId: String
        let historyRecordId: String
        let peerId: String
        let routePeerId: String?
        let addresses: [String]
        let envelopeBase64: String
        let createdAtEpochSec: UInt64
        let attemptCount: UInt32
        let nextAttemptAtEpochSec: UInt64
    }

    private struct MessageIdentityHints {
        let identityId: String?
        let publicKey: String?
        let nickname: String?
        let libp2pPeerId: String?
        let listeners: [String]
        let externalAddresses: [String]
        let connectionHints: [String]
    }

    private struct DecodedMessagePayload {
        let kind: String
        let text: String
        let hints: MessageIdentityHints?
    }

    private struct DeliveryAttemptResult {
        let acked: Bool
        let routePeerId: String?
    }

    private struct PeerDiscoveryInfo {
        let canonicalPeerId: String
        let publicKey: String?
        let nickname: String?
        let transport: MeshEventBus.TransportType
        let lastSeen: UInt64
    }

    private struct ReplayDiscoveredIdentity {
        var canonicalPeerId: String
        var publicKey: String?
        var nickname: String?
        var routePeerId: String?
        var transport: MeshEventBus.TransportType
    }

    // Device state for auto-adjustment
    private var currentBatteryPct: UInt8 = 100
    private var currentIsCharging: Bool = true
    private var currentMotionState: MotionState = .unknown
    private var lastAppliedPowerSnapshot: String?

    // MARK: - Published State

    var serviceState: ServiceState = .stopped
    var serviceStats: ServiceStats?
    var networkStatus = NetworkStatus()
    private var discoveredPeerMap: [String: PeerDiscoveryInfo] = [:]

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

    var isAutoAdjustEnabled: Bool {
        UserDefaults.standard.object(forKey: "auto_adjust_enabled") as? Bool ?? true
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
        // Use Application Support for internal app state (not user-facing docs).
        let appSupportPath = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        let meshPath = appSupportPath.appendingPathComponent("mesh", isDirectory: true)
        self.storagePath = meshPath.path

        logger.info("MeshRepository initialized with storage: \(self.storagePath)")

        // Create storage directory if needed
        try? FileManager.default.createDirectory(at: meshPath, withIntermediateDirectories: true)

        // Avoid restoring mesh state on reinstall from iCloud backup.
        var values = URLResourceValues()
        values.isExcludedFromBackup = true
        var mutableMeshPath = meshPath
        try? mutableMeshPath.setResourceValues(values)
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
            try ensureLocalIdentityFederation()

            // Wire CoreDelegate: Rust → Swift callbacks
            let coreDelegate = CoreDelegateImpl(meshRepository: self)
            self.coreDelegateImpl = coreDelegate  // store strong reference
            ironCore?.setDelegate(delegate: coreDelegate)
            logger.info("CoreDelegate registered for Rust->Swift callbacks")

            // Ensure identity exists (foundational requirement)
            if !isIdentityInitialized() {
                logger.info("Auto-initializing new identity for first run")
                try? ironCore?.initializeIdentity()
                try? ensureLocalIdentityFederation()
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
                broadcastIdentityBeacon()
                logger.info("Internet transport (Swarm) initiated")
            } else if settings?.internetEnabled == true {
                logger.warning("Postponing Swarm start: Identity not ready")
            }

            serviceState = .running
            statusEvents.send(.serviceStateChanged(.running))

            // Start BLE advertising and scanning
            blePeripheralManager?.startAdvertising()
            bleCentralManager?.startScanning()
            applyPowerAdjustments(reason: "service_started")
            startPendingOutboxRetryLoop()
            Task { await flushPendingOutbox(reason: "service_started") }

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
        pendingOutboxRetryTask?.cancel()
        pendingOutboxRetryTask = nil
        identitySyncSentPeers.removeAll()

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

        try? ensureLocalIdentityFederation()

        let settings = try? settingsManager?.load()
        if settings?.internetEnabled == true {
            do {
                // Configure bootstrap nodes for NAT traversal
                meshService?.setBootstrapNodes(addrs: Self.defaultBootstrapNodes)
                try meshService?.startSwarm(listenAddr: "/ip4/0.0.0.0/tcp/0")
                broadcastIdentityBeacon()
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
            try ensureLocalIdentityFederation()
            logger.info("✓ Identity created successfully")
            broadcastIdentityBeacon()
        } catch {
            logger.error("Failed to create identity: \(error.localizedDescription)")
            throw error
        }
    }

    private func ensureLocalIdentityFederation() throws {
        guard let ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }

        var info = ironCore.getIdentityInfo()
        if !info.initialized {
            let restored = restoreIdentityFromKeychain(ironCore: ironCore)
            if restored {
                info = ironCore.getIdentityInfo()
            }
        }
        if !info.initialized {
            logger.info("Auto-initializing new identity for first run")
            try ironCore.initializeIdentity()
            info = ironCore.getIdentityInfo()
            persistIdentityBackupToKeychain(ironCore: ironCore)
        }

        let nickname = info.nickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        if nickname.isEmpty {
            let defaultNickname = buildDefaultLocalNickname(info: info)
            try ironCore.setNickname(nickname: defaultNickname)
            logger.info("Auto-set local nickname: \(defaultNickname)")
            persistIdentityBackupToKeychain(ironCore: ironCore)
        } else {
            persistIdentityBackupToKeychain(ironCore: ironCore)
        }
    }

    @discardableResult
    private func restoreIdentityFromKeychain(ironCore: IronCore) -> Bool {
        guard let backupPayload = readIdentityBackupFromKeychain() else {
            return false
        }
        do {
            try ironCore.importIdentityBackup(backup: backupPayload)
            logger.info("Restored identity from iOS Keychain backup payload")
            return true
        } catch {
            logger.warning("Identity Keychain restore failed: \(error.localizedDescription, privacy: .public)")
            return false
        }
    }

    private func persistIdentityBackupToKeychain(ironCore: IronCore?) {
        guard let ironCore else { return }
        do {
            let backup = try ironCore.exportIdentityBackup()
            writeIdentityBackupToKeychain(backup)
        } catch {
            logger.warning("Failed to persist identity backup payload: \(error.localizedDescription, privacy: .public)")
        }
    }

    private func readIdentityBackupFromKeychain() -> String? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: IdentityBackupStore.service,
            kSecAttrAccount as String: IdentityBackupStore.account,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne,
        ]
        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        guard status == errSecSuccess, let data = result as? Data else {
            return nil
        }
        return String(data: data, encoding: .utf8)
    }

    private func writeIdentityBackupToKeychain(_ payload: String) {
        guard let data = payload.data(using: .utf8) else { return }
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: IdentityBackupStore.service,
            kSecAttrAccount as String: IdentityBackupStore.account,
        ]
        let attributes: [String: Any] = [
            kSecValueData as String: data,
            kSecAttrAccessible as String: kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly,
        ]
        let updateStatus = SecItemUpdate(query as CFDictionary, attributes as CFDictionary)
        if updateStatus == errSecSuccess {
            return
        }
        if updateStatus == errSecItemNotFound {
            var addQuery = query
            addQuery.merge(attributes) { _, new in new }
            _ = SecItemAdd(addQuery as CFDictionary, nil)
        }
    }

    private func buildDefaultLocalNickname(info: IdentityInfo) -> String {
        let source = info.publicKeyHex ?? info.identityId ?? info.libp2pPeerId ?? "peer"
        let suffix = String(source.suffix(6))
        let normalizedSuffix = suffix.isEmpty ? "peer" : suffix
        return "ios-\(normalizedSuffix)".lowercased()
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
        let routing = parseRoutingHintsFromNotes(contact?.notes)
        let routePeerCandidates = buildRoutePeerCandidates(
            peerId: peerId,
            cachedRoutePeerId: routing.libp2pPeerId,
            notes: contact?.notes
        )
        let preferredRoutePeerId = routePeerCandidates.first

        // Prepare and send message (use trimmed key to handle any stored whitespace)
        let outboundContent = encodeMessageWithIdentityHints(content)
        let prepared = try ironCore.prepareMessageWithId(recipientPublicKeyHex: trimmedKey, text: outboundContent)
        let messageId = prepared.messageId.trimmingCharacters(in: .whitespacesAndNewlines)
        if messageId.isEmpty {
            throw MeshError.notInitialized("Core returned empty message ID")
        }
        let envelopeData = Data(prepared.envelopeData)

        // Record in history FIRST so it's persisted even if bridge fails
        let messageRecord = MessageRecord(
            id: messageId,
            direction: .sent,
            peerId: peerId,
            content: content,
            timestamp: UInt64(Date().timeIntervalSince1970),
            delivered: false
        )
        try? historyManager?.add(record: messageRecord)

        // Notify UI (Unified flow for sent messages)
        messageUpdates.send(messageRecord)

        // 3. Send over core-selected swarm route only.
        // Mobile app passes identity/routing hints; Rust core owns path selection.
        let delivery = await attemptDirectSwarmDelivery(
            routePeerCandidates: routePeerCandidates,
            addresses: routing.addresses,
            envelopeData: envelopeData
        )
        let selectedRoutePeerId = delivery.routePeerId ?? preferredRoutePeerId

        if delivery.acked {
            enqueuePendingOutbound(
                historyRecordId: messageId,
                peerId: peerId,
                routePeerId: selectedRoutePeerId,
                addresses: routing.addresses,
                envelopeData: envelopeData,
                initialAttemptCount: 1,
                initialDelaySec: receiptAwaitSeconds
            )
        } else {
            enqueuePendingOutbound(
                historyRecordId: messageId,
                peerId: peerId,
                routePeerId: selectedRoutePeerId,
                addresses: routing.addresses,
                envelopeData: envelopeData,
                initialAttemptCount: 1,
                initialDelaySec: 0
            )
        }
    }

    /// Handle incoming message (from CoreDelegate callback)
    func onMessageReceived(
        senderId: String,
        senderPublicKeyHex: String,
        messageId: String,
        senderTimestamp: UInt64,
        data: Data
    ) {
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

        let normalizedSenderKey = normalizePublicKey(senderPublicKeyHex)
        let rawContent = String(data: data, encoding: .utf8) ?? "[binary]"
        let decodedPayload = decodeMessageWithIdentityHints(rawContent)
        let hintedIdentity = decodedPayload.hints
        let hintedKey = normalizePublicKey(hintedIdentity?.publicKey)
        let verifiedHints: MessageIdentityHints? = {
            if let hintedKey, let normalizedSenderKey, hintedKey != normalizedSenderKey {
                logger.warning(
                    "Ignoring forged message identity hint for \(senderId): key mismatch (hint=\(hintedKey.prefix(8))..., envelope=\(normalizedSenderKey.prefix(8))...)"
                )
                return nil
            }
            return hintedIdentity
        }()

        var canonicalPeerId = resolveCanonicalPeerId(senderId: senderId, senderPublicKeyHex: senderPublicKeyHex)
        canonicalPeerId = resolveCanonicalPeerIdFromMessageHints(
            resolvedCanonicalPeerId: canonicalPeerId,
            senderId: senderId,
            senderPublicKeyHex: senderPublicKeyHex,
            hintedIdentityId: verifiedHints?.identityId
        )

        let hintedRoutePeerId = verifiedHints?.libp2pPeerId?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty
        let routePeerId: String? = isLibp2pPeerId(senderId) ? senderId : hintedRoutePeerId
        let hintedAddresses = (verifiedHints?.listeners ?? []) +
            (verifiedHints?.externalAddresses ?? []) +
            (verifiedHints?.connectionHints ?? [])
        let hintedDialCandidates = buildDialCandidatesForPeer(
            routePeerId: routePeerId,
            rawAddresses: hintedAddresses,
            includeRelayCircuits: true
        )
        let knownNickname = selectAuthoritativeNickname(
            incoming: verifiedHints?.nickname,
            existing: resolveKnownPeerNickname(
                canonicalPeerId: canonicalPeerId,
                routePeerId: routePeerId,
                publicKey: normalizedSenderKey
            )
        )
        if canonicalPeerId != senderId {
            logger.info("Canonicalized sender \(senderId) -> \(canonicalPeerId) using public key match")
        }
        if isBootstrapRelayPeer(canonicalPeerId) {
            logger.info("Ignoring payload attributed to bootstrap relay peer \(canonicalPeerId)")
            return
        }

        // Auto-upsert contact: senderPublicKeyHex is guaranteed valid (Rust verified it during decrypt)
        let existingContact = try? contactManager?.get(peerId: canonicalPeerId)
        if existingContact == nil {
            if let normalizedSenderKey {
                var routeNotes: String?
                if let routePeerId, !routePeerId.isEmpty {
                    routeNotes = appendRoutingHint(notes: nil, key: "libp2p_peer_id", value: routePeerId)
                }
                routeNotes = upsertRoutingListeners(
                    notes: routeNotes,
                    listeners: normalizeOutboundListenerHints(hintedDialCandidates)
                )
                let autoContact = Contact(
                    peerId: canonicalPeerId,
                    nickname: knownNickname,
                    localNickname: nil,
                    publicKey: normalizedSenderKey,
                    addedAt: UInt64(Date().timeIntervalSince1970),
                    lastSeen: UInt64(Date().timeIntervalSince1970),
                    notes: routeNotes
                )
                do {
                    try contactManager?.add(contact: autoContact)
                    logger.info("Auto-created contact from received message: \(canonicalPeerId.prefix(8)) key: \(normalizedSenderKey.prefix(8))...")
                } catch {
                    logger.warning("Auto-create contact failed for \(canonicalPeerId.prefix(8)): \(error.localizedDescription)")
                }
            }
        } else if let existingContact {
            try? contactManager?.updateLastSeen(peerId: canonicalPeerId)

            if (existingContact.nickname?.isEmpty ?? true), let knownNickname, !knownNickname.isEmpty {
                let updatedContact = Contact(
                    peerId: existingContact.peerId,
                    nickname: knownNickname,
                    localNickname: existingContact.localNickname,
                    publicKey: existingContact.publicKey,
                    addedAt: existingContact.addedAt,
                    lastSeen: existingContact.lastSeen,
                    notes: existingContact.notes
                )
                try? contactManager?.add(contact: updatedContact)
            }

            if let routePeerId, !routePeerId.isEmpty,
               let normalizedSenderKey,
               normalizePublicKey(existingContact.publicKey) == normalizedSenderKey,
               parseRoutingHintsFromNotes(existingContact.notes).libp2pPeerId == nil {
                let updatedNotes = appendRoutingHint(notes: existingContact.notes, key: "libp2p_peer_id", value: routePeerId)
                let updatedNotesWithListeners = upsertRoutingListeners(
                    notes: updatedNotes,
                    listeners: normalizeOutboundListenerHints(hintedDialCandidates)
                )
                let updatedContact = Contact(
                    peerId: existingContact.peerId,
                    nickname: existingContact.nickname,
                    localNickname: existingContact.localNickname,
                    publicKey: existingContact.publicKey,
                    addedAt: existingContact.addedAt,
                    lastSeen: existingContact.lastSeen,
                    notes: updatedNotesWithListeners
                )
                try? contactManager?.add(contact: updatedContact)
            }
        }

        if let normalizedSenderKey {
            upsertFederatedContact(
                canonicalPeerId: canonicalPeerId,
                publicKey: normalizedSenderKey,
                nickname: knownNickname,
                libp2pPeerId: routePeerId,
                listeners: hintedDialCandidates,
                createIfMissing: false
            )
            let discoveredNickname = prepopulateDiscoveryNickname(
                nickname: knownNickname,
                peerId: canonicalPeerId,
                publicKey: normalizedSenderKey
            )
            let discoveryInfo = PeerDiscoveryInfo(
                canonicalPeerId: canonicalPeerId,
                publicKey: normalizedSenderKey,
                nickname: discoveredNickname,
                transport: .internet,
                lastSeen: UInt64(Date().timeIntervalSince1970)
            )
            updateDiscoveredPeer(canonicalPeerId, info: discoveryInfo)
            if let routePeerId, routePeerId != canonicalPeerId {
                updateDiscoveredPeer(routePeerId, info: discoveryInfo)
            }
            let listeners = ((routePeerId.map(getDialHintsForRoutePeer(_:)) ?? []) + hintedDialCandidates)
                .reduce(into: [String]()) { acc, addr in
                    if !acc.contains(addr) { acc.append(addr) }
                }
            MeshEventBus.shared.peerEvents.send(.identityDiscovered(
                peerId: canonicalPeerId,
                publicKey: normalizedSenderKey,
                nickname: discoveredNickname,
                libp2pPeerId: routePeerId,
                listeners: listeners,
                blePeerId: nil
            ))
            annotateIdentityInLedger(
                routePeerId: routePeerId,
                listeners: listeners,
                publicKey: normalizedSenderKey,
                nickname: discoveredNickname
            )
        }

        let messageKind = decodedPayload.kind
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .lowercased()
        if messageKind == "identity_sync" {
            logger.debug("Processed identity sync from \(canonicalPeerId) (route=\(routePeerId ?? "none"))")
            sendDeliveryReceiptAsync(senderPublicKeyHex: senderPublicKeyHex, messageId: messageId)
            return
        }

        // Process message
        let content = decodedPayload.text

        if let existing = try? historyManager?.get(id: messageId),
           existing.direction == .received {
            logger.debug("Duplicate inbound message \(messageId); acknowledging without UI emit")
            sendDeliveryReceiptAsync(senderPublicKeyHex: senderPublicKeyHex, messageId: messageId)
            return
        }

        let fallbackNow = UInt64(Date().timeIntervalSince1970)
        let canonicalTimestamp = senderTimestamp > 0 ? senderTimestamp : fallbackNow
        let messageRecord = MessageRecord(
            id: messageId,
            direction: .received,
            peerId: canonicalPeerId,
            content: content,
            timestamp: canonicalTimestamp,
            delivered: true
        )

        try? historyManager?.add(record: messageRecord)

        // Notify UI
        messageUpdates.send(messageRecord)
        logger.info("Message received and processed from \(canonicalPeerId)")

        // Send delivery receipt ACK back to sender
        sendDeliveryReceiptAsync(senderPublicKeyHex: senderPublicKeyHex, messageId: messageId)
    }

    /// Handle delivery receipt callbacks from CoreDelegate.
    /// Marks local history and removes pending retry entries when IDs match.
    func onDeliveryReceipt(messageId: String, status: String) {
        let normalized = status.lowercased()
        guard normalized == "delivered" || normalized == "read" else { return }
        try? historyManager?.markDelivered(id: messageId)
        removePendingOutbound(historyRecordId: messageId)
        MeshEventBus.shared.messageEvents.send(.delivered(messageId: messageId))
    }

    private func sendDeliveryReceiptAsync(senderPublicKeyHex: String, messageId: String) {
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

    private func sendIdentitySyncIfNeeded(routePeerId: String, knownPublicKey: String? = nil) {
        let normalizedRoute = routePeerId.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalizedRoute.isEmpty,
              isLibp2pPeerId(normalizedRoute),
              !isBootstrapRelayPeer(normalizedRoute) else {
            return
        }
        guard identitySyncSentPeers.insert(normalizedRoute).inserted else {
            return
        }

        Task { @MainActor in
            let extractedPublicKey: String? = {
                guard let ironCore else { return nil }
                return try? ironCore.extractPublicKeyFromPeerId(peerId: normalizedRoute)
            }()
            let recipientPublicKey = normalizePublicKey(knownPublicKey) ??
                normalizePublicKey(extractedPublicKey)
            guard let recipientPublicKey else {
                identitySyncSentPeers.remove(normalizedRoute)
                return
            }

            do {
                let payload = encodeIdentitySyncPayload()
                let prepared = try ironCore?.prepareMessageWithId(
                    recipientPublicKeyHex: recipientPublicKey,
                    text: payload
                )
                guard let prepared else {
                    identitySyncSentPeers.remove(normalizedRoute)
                    return
                }
                try swarmBridge?.sendMessage(peerId: normalizedRoute, data: Data(prepared.envelopeData))
                logger.debug("Identity sync sent to \(normalizedRoute)")
            } catch {
                identitySyncSentPeers.remove(normalizedRoute)
                logger.debug("Failed to send identity sync to \(normalizedRoute): \(error.localizedDescription)")
            }
        }
    }

    /// Resolve incoming sender IDs to a canonical contact ID.
    ///
    /// Canonicalization prefers one stable contact per public key.
    /// Exact sender ID matches still win, then a unique public-key match wins.
    /// Routing hints are used only as fallback when key-based matching is ambiguous.
    private func resolveCanonicalPeerId(senderId: String, senderPublicKeyHex: String) -> String {
        guard let normalizedIncomingKey = normalizePublicKey(senderPublicKeyHex),
              let contacts = try? contactManager?.list() else {
            return senderId
        }

        let exactMatch = contacts.contains {
            $0.peerId == senderId && normalizePublicKey($0.publicKey) == normalizedIncomingKey
        }
        if exactMatch { return senderId }

        let keyedMatches = contacts.filter {
            normalizePublicKey($0.publicKey) == normalizedIncomingKey
        }
        if keyedMatches.count == 1 {
            return keyedMatches[0].peerId
        }
        if keyedMatches.count > 1 {
            logger.warning("Ambiguous canonical sender mapping for key \(normalizedIncomingKey.prefix(8))...; trying route-hint fallback")
        }

        if isLibp2pPeerId(senderId) {
            let linkedIdentityMatches = contacts.filter {
                guard normalizePublicKey($0.publicKey) == normalizedIncomingKey else { return false }
                guard $0.peerId != senderId else { return false }
                guard let notes = $0.notes,
                      let routing = parseRoutingInfo(notes: notes) else { return false }
                return routing.libp2pPeerId == senderId
            }

            if linkedIdentityMatches.count == 1 {
                return linkedIdentityMatches[0].peerId
            }
            if linkedIdentityMatches.count > 1 {
                logger.warning("Ambiguous canonical sender mapping for \(senderId); keeping raw sender ID")
            }
            return senderId
        }

        guard isIdentityId(senderId) else { return senderId }
        let keyedRoutedMatches = contacts.filter {
            guard normalizePublicKey($0.publicKey) == normalizedIncomingKey else { return false }
            guard $0.peerId != senderId else { return false }
            return parseRoutingHintsFromNotes($0.notes).libp2pPeerId != nil || isLibp2pPeerId($0.peerId)
        }
        if keyedRoutedMatches.count == 1 {
            return keyedRoutedMatches[0].peerId
        }
        if keyedRoutedMatches.count > 1 {
            logger.warning("Ambiguous identity sender mapping for \(senderId); keeping raw sender ID")
        }
        return senderId
    }

    private func resolveCanonicalPeerIdFromMessageHints(
        resolvedCanonicalPeerId: String,
        senderId: String,
        senderPublicKeyHex: String,
        hintedIdentityId: String?
    ) -> String {
        guard let normalizedHint = hintedIdentityId?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty,
              isIdentityId(normalizedHint) else {
            return resolvedCanonicalPeerId
        }
        if normalizedHint == resolvedCanonicalPeerId { return resolvedCanonicalPeerId }
        if isBootstrapRelayPeer(normalizedHint) { return resolvedCanonicalPeerId }

        let normalizedSenderKey = normalizePublicKey(senderPublicKeyHex)
        let contacts = (try? contactManager?.list()) ?? []

        if let normalizedSenderKey {
            if let hintedContact = contacts.first(where: { $0.peerId == normalizedHint }),
               normalizePublicKey(hintedContact.publicKey) == normalizedSenderKey {
                return normalizedHint
            }

            let keyMatches = contacts.filter { normalizePublicKey($0.publicKey) == normalizedSenderKey }
            if keyMatches.count == 1 {
                return keyMatches[0].peerId
            }
            if !keyMatches.isEmpty {
                return resolvedCanonicalPeerId
            }
        }

        if resolvedCanonicalPeerId == senderId || isLibp2pPeerId(resolvedCanonicalPeerId) {
            return normalizedHint
        }
        return resolvedCanonicalPeerId
    }

    private func encodeMessageWithIdentityHints(_ content: String) -> String {
        return encodeMeshMessagePayload(content: content, kind: "text")
    }

    private func encodeIdentitySyncPayload() -> String {
        return encodeMeshMessagePayload(content: "", kind: "identity_sync")
    }

    private func encodeMeshMessagePayload(content: String, kind: String) -> String {
        guard let info = ironCore?.getIdentityInfo(),
              let publicKeyHex = normalizePublicKey(info.publicKeyHex) else {
            return kind == "identity_sync" ? "" : content
        }

        let listeners = Array(normalizeOutboundListenerHints(getListeningAddresses()).prefix(3))
        let externalAddresses = Array(normalizeExternalAddressHints(getExternalAddresses()).prefix(3))
        let connectionHints = (listeners + externalAddresses).reduce(into: [String]()) { acc, addr in
            if !acc.contains(addr) { acc.append(addr) }
        }

        let sender: [String: Any] = [
            "identity_id": info.identityId?.trimmingCharacters(in: .whitespacesAndNewlines) ?? "",
            "public_key": publicKeyHex,
            "nickname": String((normalizeNickname(info.nickname) ?? "").prefix(64)),
            "libp2p_peer_id": info.libp2pPeerId?.trimmingCharacters(in: .whitespacesAndNewlines) ?? "",
            "listeners": listeners,
            "external_addresses": externalAddresses,
            "connection_hints": connectionHints
        ]
        let payload: [String: Any] = [
            "schema": "scm.message.identity.v1",
            "kind": kind,
            "text": content,
            "sender": sender
        ]
        guard JSONSerialization.isValidJSONObject(payload),
              let data = try? JSONSerialization.data(withJSONObject: payload),
              let encoded = String(data: data, encoding: .utf8) else {
            return kind == "identity_sync" ? "" : content
        }
        return encoded
    }

    private func decodeMessageWithIdentityHints(_ raw: String) -> DecodedMessagePayload {
        guard let data = raw.data(using: .utf8),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              (json["schema"] as? String) == "scm.message.identity.v1" else {
            return DecodedMessagePayload(kind: "text", text: raw, hints: nil)
        }

        let kind = ((json["kind"] as? String) ?? "text")
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty ?? "text"
        let text = (json["text"] as? String) ?? raw
        let sender = json["sender"] as? [String: Any]
        let hintedLibp2p = (sender?["libp2p_peer_id"] as? String)?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty
        let hints = MessageIdentityHints(
            identityId: (sender?["identity_id"] as? String)?
                .trimmingCharacters(in: .whitespacesAndNewlines)
                .nilIfEmpty,
            publicKey: normalizePublicKey(sender?["public_key"] as? String),
            nickname: normalizeNickname(sender?["nickname"] as? String),
            libp2pPeerId: (hintedLibp2p != nil && isLibp2pPeerId(hintedLibp2p!)) ? hintedLibp2p : nil,
            listeners: parseStringArray(sender?["listeners"]),
            externalAddresses: parseStringArray(sender?["external_addresses"]),
            connectionHints: parseStringArray(sender?["connection_hints"])
        )
        return DecodedMessagePayload(kind: kind, text: text, hints: hints)
    }

    private func parseStringArray(_ value: Any?) -> [String] {
        guard let array = value as? [Any] else { return [] }
        return array
            .compactMap { item in
                if let str = item as? String {
                    let trimmed = str.trimmingCharacters(in: .whitespacesAndNewlines)
                    return trimmed.isEmpty ? nil : trimmed
                }
                return nil
            }
            .reduce(into: [String]()) { acc, item in
                if !acc.contains(item) { acc.append(item) }
            }
    }

    private func normalizePublicKey(_ key: String?) -> String? {
        guard let value = key?.trimmingCharacters(in: .whitespacesAndNewlines),
              value.count == 64 else {
            return nil
        }
        let validHex = value.unicodeScalars.allSatisfy { scalar in
            CharacterSet(charactersIn: "0123456789abcdefABCDEF").contains(scalar)
        }
        guard validHex else { return nil }
        return value.lowercased()
    }

    private func normalizeNickname(_ nickname: String?) -> String? {
        let normalized = nickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        return normalized.isEmpty ? nil : normalized
    }

    private func isSyntheticFallbackNickname(_ nickname: String?) -> Bool {
        guard let normalized = normalizeNickname(nickname)?.lowercased() else { return false }
        // "peer-xxxxxx" is a receiver-side placeholder and should never overwrite
        // a sender-provided nickname.
        return normalized.hasPrefix("peer-")
    }

    private func selectAuthoritativeNickname(incoming: String?, existing: String?) -> String? {
        let incomingNormalized = normalizeNickname(incoming)
        let existingNormalized = normalizeNickname(existing)

        let incomingSynthetic = isSyntheticFallbackNickname(incomingNormalized)
        let existingSynthetic = isSyntheticFallbackNickname(existingNormalized)

        if incomingNormalized == nil && existingSynthetic { return nil }
        if incomingNormalized == nil { return existingNormalized }
        if incomingSynthetic && existingNormalized == nil { return nil }
        if incomingSynthetic && existingSynthetic { return nil }
        if incomingSynthetic { return existingNormalized }
        if existingSynthetic { return incomingNormalized }
        return incomingNormalized
    }

    private func isBlePeerId(_ value: String) -> Bool {
        UUID(uuidString: value.trimmingCharacters(in: .whitespacesAndNewlines)) != nil
    }

    private func selectCanonicalPeerId(incoming: String, existing: String) -> String {
        let incomingId = incoming.trimmingCharacters(in: .whitespacesAndNewlines)
        let existingId = existing.trimmingCharacters(in: .whitespacesAndNewlines)
        if incomingId.isEmpty { return existingId }
        if existingId.isEmpty || existingId == incomingId { return incomingId }

        let incomingIsLibp2p = isLibp2pPeerId(incomingId)
        let existingIsLibp2p = isLibp2pPeerId(existingId)
        let incomingIsIdentity = isIdentityId(incomingId)
        let existingIsIdentity = isIdentityId(existingId)
        let incomingIsBle = isBlePeerId(incomingId)
        let existingIsBle = isBlePeerId(existingId)

        if existingIsIdentity && incomingIsLibp2p { return existingId }
        if incomingIsIdentity && existingIsLibp2p { return incomingId }
        if existingIsBle && !incomingIsBle { return incomingId }
        if !existingIsBle && incomingIsBle { return existingId }
        return incomingId
    }

    private func prepopulateDiscoveryNickname(
        nickname: String?,
        peerId: String,
        publicKey: String?
    ) -> String? {
        let incomingNickname = normalizeNickname(nickname)

        let normalizedKey = normalizePublicKey(publicKey)
        let contacts = (try? contactManager?.list()) ?? []
        let fromContact = contacts.first(where: {
            $0.peerId == peerId ||
            (
                normalizedKey != nil &&
                normalizePublicKey($0.publicKey) == normalizedKey
            )
        })?.nickname
        return selectAuthoritativeNickname(incoming: incomingNickname, existing: fromContact)
    }

    private func resolveKnownPeerNickname(
        canonicalPeerId: String,
        routePeerId: String?,
        publicKey: String?
    ) -> String? {
        let normalizedKey = normalizePublicKey(publicKey)
        let routeCandidate = routePeerId?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty

        let fromDiscovery = discoveredPeerMap.values
            .first(where: { info in
                info.canonicalPeerId == canonicalPeerId ||
                (routeCandidate != nil && info.canonicalPeerId == routeCandidate) ||
                (
                    normalizedKey != nil &&
                    normalizePublicKey(info.publicKey) == normalizedKey
                )
            })?
            .nickname

        let fromLedger = (ledgerManager?.dialableAddresses() ?? [])
            .first(where: { entry in
                entry.peerId == canonicalPeerId ||
                (routeCandidate != nil && entry.peerId == routeCandidate) ||
                (
                    normalizedKey != nil &&
                    normalizePublicKey(entry.publicKey) == normalizedKey
                )
            })?
            .nickname

        let fromContact = ((try? contactManager?.list()) ?? [])
            .first(where: { contact in
                contact.peerId == canonicalPeerId ||
                (routeCandidate != nil && contact.peerId == routeCandidate) ||
                (
                    normalizedKey != nil &&
                    normalizePublicKey(contact.publicKey) == normalizedKey
                )
            })?
            .nickname

        let discoveryOrLedger = selectAuthoritativeNickname(incoming: fromDiscovery, existing: fromLedger)
        return selectAuthoritativeNickname(incoming: discoveryOrLedger, existing: fromContact)
    }

    private func updateDiscoveredPeer(_ key: String, info: PeerDiscoveryInfo) {
        let existing = discoveredPeerMap[key]
        if let existing,
           existing.publicKey != nil,
           info.publicKey == nil,
           info.lastSeen >= existing.lastSeen,
           info.lastSeen - existing.lastSeen < 300 {
            return
        }
        let merged: PeerDiscoveryInfo
        if let existing {
            merged = PeerDiscoveryInfo(
                canonicalPeerId: selectCanonicalPeerId(
                    incoming: info.canonicalPeerId,
                    existing: existing.canonicalPeerId
                ),
                publicKey: info.publicKey ?? existing.publicKey,
                nickname: selectAuthoritativeNickname(incoming: info.nickname, existing: existing.nickname),
                transport: (info.transport == .internet || existing.transport == .internet) ? .internet : info.transport,
                lastSeen: max(info.lastSeen, existing.lastSeen)
            )
        } else {
            merged = info
        }

        let canonicalPeerId = merged.canonicalPeerId.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            ? key.trimmingCharacters(in: .whitespacesAndNewlines)
            : merged.canonicalPeerId.trimmingCharacters(in: .whitespacesAndNewlines)
        let canonicalPublicKey = normalizePublicKey(merged.publicKey)

        discoveredPeerMap[canonicalPeerId] = merged
        discoveredPeerMap = discoveredPeerMap.filter { mapKey, candidate in
            if mapKey == canonicalPeerId { return true }
            let sameCanonicalPeerId = candidate.canonicalPeerId.trimmingCharacters(in: .whitespacesAndNewlines) == canonicalPeerId
            let samePublicKey = canonicalPublicKey != nil &&
                normalizePublicKey(candidate.publicKey) == canonicalPublicKey
            return !(sameCanonicalPeerId || samePublicKey)
        }
    }

    private func pruneDisconnectedPeer(_ peerId: String) {
        let normalizedPeerId = peerId.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalizedPeerId.isEmpty else { return }

        let disconnectedPublicKey = normalizePublicKey(discoveredPeerMap[normalizedPeerId]?.publicKey)
        discoveredPeerMap = discoveredPeerMap.filter { key, info in
            if key == normalizedPeerId { return false }
            if info.canonicalPeerId == normalizedPeerId { return false }
            if let disconnectedPublicKey,
               normalizePublicKey(info.publicKey) == disconnectedPublicKey {
                return false
            }
            return true
        }
    }

    private func annotateIdentityInLedger(
        routePeerId: String?,
        listeners: [String],
        publicKey: String?,
        nickname: String?
    ) {
        guard let routePeerId = routePeerId?.trimmingCharacters(in: .whitespacesAndNewlines),
              !routePeerId.isEmpty,
              isLibp2pPeerId(routePeerId) else {
            return
        }

        let dialHints = buildDialCandidatesForPeer(
            routePeerId: routePeerId,
            rawAddresses: listeners,
            includeRelayCircuits: true
        )
        guard !dialHints.isEmpty else { return }

        let normalizedKey = normalizePublicKey(publicKey)
        let normalizedNickname = nickname?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty

        for multiaddr in dialHints {
            ledgerManager?.annotateIdentity(
                multiaddr: multiaddr,
                peerId: routePeerId,
                publicKey: normalizedKey,
                nickname: normalizedNickname
            )
        }
    }

    func getDialHintsForRoutePeer(_ routePeerId: String) -> [String] {
        let normalizedRoute = routePeerId.trimmingCharacters(in: .whitespacesAndNewlines)
        guard isLibp2pPeerId(normalizedRoute) else { return [] }

        let fromLedger = (ledgerManager?.dialableAddresses() ?? [])
            .filter { $0.peerId == normalizedRoute }
            .map { $0.multiaddr }
        return buildDialCandidatesForPeer(
            routePeerId: normalizedRoute,
            rawAddresses: fromLedger,
            includeRelayCircuits: true
        )
    }

    func replayDiscoveredPeerEvents() {
        guard !discoveredPeerMap.isEmpty else { return }

        var aggregates: [String: ReplayDiscoveredIdentity] = [:]

        for (mapKey, info) in discoveredPeerMap {
            let canonicalPeerId = info.canonicalPeerId.trimmingCharacters(in: .whitespacesAndNewlines)
            guard !canonicalPeerId.isEmpty else { continue }

            let normalizedKey = normalizePublicKey(info.publicKey)
            let aggregateKey = normalizedKey ?? canonicalPeerId
            let routeCandidate: String? = {
                if isLibp2pPeerId(mapKey) { return mapKey }
                if isLibp2pPeerId(canonicalPeerId) { return canonicalPeerId }
                return nil
            }()
            let discoveredNickname = prepopulateDiscoveryNickname(
                nickname: info.nickname,
                peerId: canonicalPeerId,
                publicKey: normalizedKey
            )

            if var existing = aggregates[aggregateKey] {
                if existing.publicKey == nil, let normalizedKey { existing.publicKey = normalizedKey }
                existing.canonicalPeerId = selectCanonicalPeerId(
                    incoming: canonicalPeerId,
                    existing: existing.canonicalPeerId
                )
                existing.nickname = selectAuthoritativeNickname(
                    incoming: discoveredNickname,
                    existing: existing.nickname
                )
                if (existing.routePeerId?.isEmpty ?? true), let routeCandidate { existing.routePeerId = routeCandidate }
                if existing.transport != .internet, info.transport == .internet {
                    existing.transport = info.transport
                }
                aggregates[aggregateKey] = existing
            } else {
                aggregates[aggregateKey] = ReplayDiscoveredIdentity(
                    canonicalPeerId: canonicalPeerId,
                    publicKey: normalizedKey,
                    nickname: discoveredNickname,
                    routePeerId: routeCandidate,
                    transport: info.transport
                )
            }
        }

        for peer in aggregates.values {
            let listeners = peer.routePeerId.map(getDialHintsForRoutePeer(_:)) ?? []
            if let publicKey = peer.publicKey, !publicKey.isEmpty {
                MeshEventBus.shared.peerEvents.send(.identityDiscovered(
                    peerId: peer.canonicalPeerId,
                    publicKey: publicKey,
                    nickname: peer.nickname,
                    libp2pPeerId: peer.routePeerId,
                    listeners: listeners,
                    blePeerId: nil
                ))
            } else {
                MeshEventBus.shared.peerEvents.send(.discovered(peerId: peer.canonicalPeerId))
            }
        }
    }

    private func appendRoutingHint(notes: String?, key: String, value: String?) -> String? {
        guard let value = value?.trimmingCharacters(in: .whitespacesAndNewlines), !value.isEmpty else {
            return notes
        }

        let existing = notes?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let segments = existing.split(whereSeparator: { $0 == ";" || $0 == "\n" }).map { String($0) }
        let alreadyPresent = segments.contains {
            let trimmed = $0.trimmingCharacters(in: .whitespacesAndNewlines)
            guard trimmed.hasPrefix("\(key):") else { return false }
            let current = trimmed.replacingOccurrences(of: "\(key):", with: "")
                .trimmingCharacters(in: .whitespacesAndNewlines)
            return current == value
        }
        if alreadyPresent { return notes }

        let merged = [existing, "\(key):\(value)"]
            .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
            .filter { !$0.isEmpty }
            .joined(separator: ";")
        return merged.isEmpty ? nil : merged
    }

    private func resolveTransportIdentity(libp2pPeerId: String) -> TransportIdentityResolution? {
        guard isLibp2pPeerId(libp2pPeerId) else { return nil }
        let extractedKey: String? = {
            guard let core = ironCore else { return nil }
            return try? core.extractPublicKeyFromPeerId(peerId: libp2pPeerId)
        }()
        guard let extractedKey,
              let normalizedKey = normalizePublicKey(extractedKey) else {
            return nil
        }

        let selfKey = normalizePublicKey(ironCore?.getIdentityInfo().publicKeyHex)
        if selfKey == normalizedKey { return nil }

        guard let contacts = try? contactManager?.list() else {
            return TransportIdentityResolution(
                canonicalPeerId: libp2pPeerId,
                publicKey: normalizedKey,
                nickname: nil
            )
        }

        let keyMatches = contacts.filter { normalizePublicKey($0.publicKey) == normalizedKey }
        if keyMatches.count > 1 {
            logger.warning("Multiple contacts share transport key \(normalizedKey.prefix(8))...; using explicit route match where possible")
        }

        let routeLinked = keyMatches.first {
            $0.peerId == libp2pPeerId || parseRoutingHintsFromNotes($0.notes).libp2pPeerId == libp2pPeerId
        }
        let canonicalContact = routeLinked ?? keyMatches.first

        return TransportIdentityResolution(
            canonicalPeerId: canonicalContact?.peerId ?? libp2pPeerId,
            publicKey: normalizedKey,
            nickname: canonicalContact?.nickname
        )
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
        let routing = parseRoutingHintsFromNotes(contact.notes)
        annotateIdentityInLedger(
            routePeerId: routing.libp2pPeerId,
            listeners: routing.addresses,
            publicKey: contact.publicKey,
            nickname: contact.nickname
        )
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
        let normalizedNickname = nickname?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty
        try contactManager.setNickname(peerId: peerId, nickname: normalizedNickname)
        logger.info("✓ Contact nickname updated: \(peerId)")
    }

    func setLocalNickname(peerId: String, nickname: String?) throws {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        let normalizedNickname = nickname?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty
        try contactManager.setLocalNickname(peerId: peerId, nickname: normalizedNickname)
        logger.info("✓ Local nickname updated: \(peerId)")
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
        removePendingOutbound(historyRecordId: id)
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
        applyPowerAdjustments(reason: "battery_changed")
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
        applyPowerAdjustments(reason: "network_changed")
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
        applyPowerAdjustments(reason: "motion_changed")
    }

    func setAutoAdjustEnabled(_ enabled: Bool) {
        UserDefaults.standard.set(enabled, forKey: "auto_adjust_enabled")
        logger.info("AutoAdjust toggled: \(enabled)")
        applyPowerAdjustments(reason: "settings_toggled")
    }

    // MARK: - Background Operations

    func onEnteringBackground() {
        logger.info("Repository: entering background")
        pauseMeshService()
        do {
            try saveLedger()
        } catch {
            logger.warning("Failed to save ledger on background transition: \(error.localizedDescription)")
        }
    }

    func onEnteringForeground() {
        logger.info("Repository: entering foreground")
        resumeMeshService()
        updateStats()
    }

    func pauseService() {
        pauseMeshService()
    }

    func syncPendingMessages() async throws {
        logger.info("Syncing pending messages")
        // Pending outbox data is managed by Rust core. The best available
        // sync trigger here is to refresh transport connectivity so queued
        // messages can be retried by the core.
        if serviceState != .running {
            try ensureServiceInitialized()
        }

        let contacts = (try? contactManager?.list()) ?? []
        for contact in contacts {
            guard let notes = contact.notes,
                  let routing = parseRoutingInfo(notes: notes),
                  !routing.addresses.isEmpty else {
                continue
            }
            connectToPeer(routing.libp2pPeerId, addresses: routing.addresses)
        }

        await flushPendingOutbox(reason: "sync_pending")

        updateStats()
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
        if serviceState != .running {
            try ensureServiceInitialized()
        }

        bleCentralManager?.startScanning()
        blePeripheralManager?.startAdvertising()
        updateStats()
    }

    func performBulkSync() async throws {
        logger.info("Performing bulk sync")
        try await syncPendingMessages()
        try await quickPeerDiscovery()
        try await updatePeerLedger()
    }

    func cleanupOldMessages() async throws {
        logger.info("Cleaning up old messages")
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }

        // Retention policy: clear conversations for peers not seen in 180 days.
        let staleThresholdSeconds: UInt64 = 180 * 24 * 60 * 60
        let now = UInt64(Date().timeIntervalSince1970)

        for contact in try contactManager.list() {
            guard let lastSeen = contact.lastSeen else { continue }
            if now > lastSeen && (now - lastSeen) > staleThresholdSeconds {
                try? historyManager.removeConversation(peerId: contact.peerId)
            }
        }
    }

    func updatePeerLedger() async throws {
        logger.info("Updating peer ledger")
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        try ledgerManager.save()
    }

    /// Handle libp2p transport identity updates from the Rust core.
    /// Transport peer IDs are route hints, not user/contact identities.
    func handleTransportPeerDiscovered(peerId: String) {
        let selfLibp2p = ironCore?.getIdentityInfo().libp2pPeerId?
            .trimmingCharacters(in: .whitespacesAndNewlines)
        if let selfLibp2p, !selfLibp2p.isEmpty, selfLibp2p == peerId {
            logger.debug("Ignoring self transport discovery: \(peerId)")
            return
        }

        if isBootstrapRelayPeer(peerId) {
            logger.debug("Ignoring bootstrap relay peer discovery event: \(peerId)")
            return
        }

        if let transportIdentity = resolveTransportIdentity(libp2pPeerId: peerId) {
            let discoveredNickname = prepopulateDiscoveryNickname(
                nickname: transportIdentity.nickname,
                peerId: transportIdentity.canonicalPeerId,
                publicKey: transportIdentity.publicKey
            )
            let relayHints = buildDialCandidatesForPeer(
                routePeerId: peerId,
                rawAddresses: [],
                includeRelayCircuits: true
            )
            let discoveryInfo = PeerDiscoveryInfo(
                canonicalPeerId: transportIdentity.canonicalPeerId,
                publicKey: transportIdentity.publicKey,
                nickname: discoveredNickname,
                transport: .internet,
                lastSeen: UInt64(Date().timeIntervalSince1970)
            )
            updateDiscoveredPeer(peerId, info: discoveryInfo)
            if transportIdentity.canonicalPeerId != peerId {
                updateDiscoveredPeer(transportIdentity.canonicalPeerId, info: discoveryInfo)
            }
            MeshEventBus.shared.peerEvents.send(.identityDiscovered(
                peerId: transportIdentity.canonicalPeerId,
                publicKey: transportIdentity.publicKey,
                nickname: discoveredNickname,
                libp2pPeerId: peerId,
                listeners: relayHints,
                blePeerId: nil
            ))
            annotateIdentityInLedger(
                routePeerId: peerId,
                listeners: relayHints,
                publicKey: transportIdentity.publicKey,
                nickname: discoveredNickname
            )
            persistRouteHintsForTransportPeer(
                libp2pPeerId: peerId,
                listeners: relayHints,
                knownPublicKey: transportIdentity.publicKey
            )
            upsertFederatedContact(
                canonicalPeerId: transportIdentity.canonicalPeerId,
                publicKey: transportIdentity.publicKey,
                nickname: transportIdentity.nickname,
                libp2pPeerId: peerId,
                listeners: relayHints,
                createIfMissing: false
            )
            try? contactManager?.updateLastSeen(peerId: transportIdentity.canonicalPeerId)
            try? contactManager?.updateLastSeen(peerId: peerId)
            if !relayHints.isEmpty {
                connectToPeer(peerId, addresses: relayHints)
            }
        } else {
            let discoveryInfo = PeerDiscoveryInfo(
                canonicalPeerId: peerId,
                publicKey: nil,
                nickname: nil,
                transport: .internet,
                lastSeen: UInt64(Date().timeIntervalSince1970)
            )
            updateDiscoveredPeer(peerId, info: discoveryInfo)
            MeshEventBus.shared.peerEvents.send(.discovered(peerId: peerId))
        }
    }

    func handleTransportPeerDisconnected(peerId: String) {
        pruneDisconnectedPeer(peerId)
    }

    func handleTransportPeerIdentified(peerId: String, listenAddrs: [String]) {
        let selfLibp2p = ironCore?.getIdentityInfo().libp2pPeerId?
            .trimmingCharacters(in: .whitespacesAndNewlines)
        if let selfLibp2p, !selfLibp2p.isEmpty, selfLibp2p == peerId {
            logger.debug("Ignoring self transport identity: \(peerId)")
            return
        }

        let dialCandidates = buildDialCandidatesForPeer(
            routePeerId: peerId,
            rawAddresses: listenAddrs,
            includeRelayCircuits: true
        )

        if isBootstrapRelayPeer(peerId) {
            logger.info("Treating bootstrap peer \(peerId) as transport relay only")
        } else {
            let transportIdentity = resolveTransportIdentity(libp2pPeerId: peerId)
            if let transportIdentity {
                let discoveredNickname = prepopulateDiscoveryNickname(
                    nickname: transportIdentity.nickname,
                    peerId: transportIdentity.canonicalPeerId,
                    publicKey: transportIdentity.publicKey
                )
                let discoveryInfo = PeerDiscoveryInfo(
                    canonicalPeerId: transportIdentity.canonicalPeerId,
                    publicKey: transportIdentity.publicKey,
                    nickname: discoveredNickname,
                    transport: .internet,
                    lastSeen: UInt64(Date().timeIntervalSince1970)
                )
                updateDiscoveredPeer(peerId, info: discoveryInfo)
                if transportIdentity.canonicalPeerId != peerId {
                    updateDiscoveredPeer(transportIdentity.canonicalPeerId, info: discoveryInfo)
                }
                MeshEventBus.shared.peerEvents.send(.identityDiscovered(
                    peerId: transportIdentity.canonicalPeerId,
                    publicKey: transportIdentity.publicKey,
                    nickname: discoveredNickname,
                    libp2pPeerId: peerId,
                    listeners: dialCandidates,
                    blePeerId: nil
                ))
                annotateIdentityInLedger(
                    routePeerId: peerId,
                    listeners: dialCandidates,
                    publicKey: transportIdentity.publicKey,
                    nickname: discoveredNickname
                )
                try? contactManager?.updateLastSeen(peerId: transportIdentity.canonicalPeerId)
                try? contactManager?.updateLastSeen(peerId: peerId)
            } else {
                let discoveryInfo = PeerDiscoveryInfo(
                    canonicalPeerId: peerId,
                    publicKey: nil,
                    nickname: nil,
                    transport: .internet,
                    lastSeen: UInt64(Date().timeIntervalSince1970)
                )
                updateDiscoveredPeer(peerId, info: discoveryInfo)
                logger.debug("Transport identity unavailable for \(peerId)")
            }
            MeshEventBus.shared.peerEvents.send(.connected(peerId: peerId))
            persistRouteHintsForTransportPeer(
                libp2pPeerId: peerId,
                listeners: dialCandidates,
                knownPublicKey: transportIdentity?.publicKey
            )
            if let transportIdentity {
                upsertFederatedContact(
                    canonicalPeerId: transportIdentity.canonicalPeerId,
                    publicKey: transportIdentity.publicKey,
                    nickname: transportIdentity.nickname,
                    libp2pPeerId: peerId,
                    listeners: dialCandidates,
                    createIfMissing: false
                )
            }
            sendIdentitySyncIfNeeded(routePeerId: peerId, knownPublicKey: transportIdentity?.publicKey)
        }

        // Identified implies an active session already exists; avoid re-dial loops here.
        Task { await flushPendingOutbox(reason: "peer_identified:\(peerId)") }
        broadcastIdentityBeacon()
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
        let rawNickname = ((info["nickname"] as? String) ?? (info["name"] as? String) ?? "")
            .trimmingCharacters(in: .whitespacesAndNewlines)
        let libp2pPeerId = info["libp2p_peer_id"] as? String
        let listeners = info["listeners"] as? [String]
        let externalAddresses = info["external_addresses"] as? [String]
        let connectionHints = info["connection_hints"] as? [String]

        let idFromInfo = (info["identity_id"] as? String)?.trimmingCharacters(in: .whitespacesAndNewlines)
        let identityId = (idFromInfo?.isEmpty == false) ? (idFromInfo ?? blePeerId) : blePeerId

        let discoveredNickname = prepopulateDiscoveryNickname(
            nickname: rawNickname,
            peerId: identityId,
            publicKey: publicKeyHex
        )
        logger.info(
            "Peer BLE identity read: \(blePeerId.prefix(8)) key: \(publicKeyHex.prefix(8))... identity=\(identityId.prefix(12)) nickname='\((discoveredNickname ?? "").prefix(24))'"
        )
        guard let normalizedKey = normalizePublicKey(publicKeyHex) else {
            let trimmed = publicKeyHex.trimmingCharacters(in: .whitespacesAndNewlines)
            logger.warning("Ignoring BLE identity from \(blePeerId.prefix(8)): invalid key (\(trimmed.count) chars)")
            return
        }
        let selfIdentity = ironCore?.getIdentityInfo()
        let selfKey = normalizePublicKey(selfIdentity?.publicKeyHex)
        let selfIdentityId = selfIdentity?.identityId?
            .trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let selfLibp2pPeerId = selfIdentity?.libp2pPeerId?
            .trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let normalizedLibp2p: String? = {
            guard let raw = libp2pPeerId?.trimmingCharacters(in: .whitespacesAndNewlines),
                  !raw.isEmpty else { return nil }
            return raw
        }()
        if (selfKey != nil && selfKey == normalizedKey) ||
            (!selfIdentityId.isEmpty && selfIdentityId == identityId) ||
            (!selfLibp2pPeerId.isEmpty && selfLibp2pPeerId == normalizedLibp2p) {
            logger.debug("Ignoring self BLE identity beacon from \(blePeerId.prefix(8))")
            return
        }

        // Emit to nearby peers bus — UI will show peer in Nearby section for user to manually add
        let nonEmptyNickname = rawNickname.isEmpty ? nil : rawNickname
        let nonEmptyLibp2p = normalizedLibp2p
        let mergedHints = (listeners ?? []) + (externalAddresses ?? []) + (connectionHints ?? [])
        let dialCandidates = buildDialCandidatesForPeer(
            routePeerId: nonEmptyLibp2p,
            rawAddresses: mergedHints,
            includeRelayCircuits: true
        )
        let discoveryInfo = PeerDiscoveryInfo(
            canonicalPeerId: identityId,
            publicKey: normalizedKey,
            nickname: discoveredNickname,
            transport: .ble,
            lastSeen: UInt64(Date().timeIntervalSince1970)
        )
        updateDiscoveredPeer(identityId, info: discoveryInfo)
        if let nonEmptyLibp2p, nonEmptyLibp2p != identityId {
            updateDiscoveredPeer(nonEmptyLibp2p, info: discoveryInfo)
        }
        MeshEventBus.shared.peerEvents.send(.identityDiscovered(
            peerId: identityId,
            publicKey: normalizedKey,
            nickname: discoveredNickname,
            libp2pPeerId: nonEmptyLibp2p,
            listeners: dialCandidates,
            blePeerId: blePeerId
        ))
        annotateIdentityInLedger(
            routePeerId: nonEmptyLibp2p,
            listeners: dialCandidates,
            publicKey: normalizedKey,
            nickname: discoveredNickname
        )
        logger.info("Emitted identityDiscovered for \(blePeerId.prefix(8)) key: \(normalizedKey.prefix(8))...")
        // Update lastSeen if already a saved contact
        try? contactManager?.updateLastSeen(peerId: blePeerId)
        try? contactManager?.updateLastSeen(peerId: identityId)
        if let libp2pPeerId, !libp2pPeerId.isEmpty {
            try? contactManager?.updateLastSeen(peerId: libp2pPeerId)
        }
        upsertFederatedContact(
            canonicalPeerId: identityId,
            publicKey: normalizedKey,
            nickname: nonEmptyNickname,
            libp2pPeerId: nonEmptyLibp2p,
            listeners: dialCandidates,
            createIfMissing: false
        )

        // Auto-dial discovered peer via Swarm if we have libp2p info
        if let peerId = nonEmptyLibp2p, !dialCandidates.isEmpty {
            logger.info("Auto-dialing discovered peer over Swarm: \(peerId)")
            connectToPeer(peerId, addresses: dialCandidates)
            Task { await flushPendingOutbox(reason: "peer_identity_read") }
        }
    }

    private func parseRoutingInfo(notes: String) -> (libp2pPeerId: String, addresses: [String])? {
        let segments = notes
            .split(whereSeparator: { $0 == ";" || $0 == "\n" })
            .map { String($0) }
        var libp2pPeerId: String?
        var addresses: [String] = []

        for segment in segments {
            let trimmed = segment.trimmingCharacters(in: .whitespacesAndNewlines)
            if trimmed.hasPrefix("libp2p_peer_id:") {
                let value = trimmed.replacingOccurrences(of: "libp2p_peer_id:", with: "")
                    .trimmingCharacters(in: .whitespacesAndNewlines)
                if !value.isEmpty { libp2pPeerId = value }
            } else if trimmed.hasPrefix("listeners:") {
                let value = trimmed.replacingOccurrences(of: "listeners:", with: "")
                    .trimmingCharacters(in: .whitespacesAndNewlines)
                if !value.isEmpty {
                    addresses = value
                        .split(separator: ",")
                        .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
                        .filter { !$0.isEmpty }
                }
            }
        }

        guard let peerId = libp2pPeerId else { return nil }
        return (peerId, addresses)
    }

    private func parseRoutingHintsFromNotes(_ notes: String?) -> RoutingHints {
        guard let notes,
              let parsed = parseRoutingInfo(notes: notes) else {
            return RoutingHints(libp2pPeerId: nil, addresses: [])
        }
        return RoutingHints(libp2pPeerId: parsed.libp2pPeerId, addresses: parsed.addresses)
    }

    private func parseAllRoutingPeerIds(from notes: String?) -> [String] {
        guard let notes, !notes.isEmpty else { return [] }
        var out: [String] = []
        let segments = notes
            .split(whereSeparator: { $0 == ";" || $0 == "\n" })
            .map { String($0) }

        for segment in segments {
            let trimmed = segment.trimmingCharacters(in: .whitespacesAndNewlines)
            guard trimmed.hasPrefix("libp2p_peer_id:") else { continue }
            let value = trimmed.replacingOccurrences(of: "libp2p_peer_id:", with: "")
                .trimmingCharacters(in: .whitespacesAndNewlines)
            if !value.isEmpty, isLibp2pPeerId(value), !out.contains(value) {
                out.append(value)
            }
        }
        return out
    }

    private func buildRoutePeerCandidates(peerId: String, cachedRoutePeerId: String?, notes: String?) -> [String] {
        var candidates: [String] = []
        let notedPeerIds = parseAllRoutingPeerIds(from: notes)
        if let newest = notedPeerIds.last, !newest.isEmpty {
            candidates.append(newest)
        }
        for hint in notedPeerIds.reversed() where !candidates.contains(hint) {
            candidates.append(hint)
        }
        if let cached = cachedRoutePeerId?.trimmingCharacters(in: .whitespacesAndNewlines),
           !cached.isEmpty,
           !candidates.contains(cached) {
            candidates.append(cached)
        }
        if isLibp2pPeerId(peerId), !candidates.contains(peerId) {
            candidates.append(peerId)
        }
        return candidates.filter { isLibp2pPeerId($0) }
    }

    private func isLibp2pPeerId(_ value: String) -> Bool {
        value.hasPrefix("12D3Koo") || value.hasPrefix("Qm")
    }

    private func isIdentityId(_ value: String) -> Bool {
        guard value.count == 64 else { return false }
        return value.unicodeScalars.allSatisfy { scalar in
            CharacterSet(charactersIn: "0123456789abcdefABCDEF").contains(scalar)
        }
    }

    private func startPendingOutboxRetryLoop() {
        guard pendingOutboxRetryTask == nil else { return }
        pendingOutboxRetryTask = Task { [weak self] in
            while !Task.isCancelled {
                await self?.flushPendingOutbox(reason: "periodic")
                try? await Task.sleep(nanoseconds: 5_000_000_000)
            }
        }
    }

    private func attemptDirectSwarmDelivery(
        routePeerCandidates: [String],
        addresses: [String],
        envelopeData: Data
    ) async -> DeliveryAttemptResult {
        guard let swarmBridge else {
            return DeliveryAttemptResult(acked: false, routePeerId: routePeerCandidates.first)
        }
        let sanitizedCandidates = routePeerCandidates
            .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
            .filter { !$0.isEmpty && isLibp2pPeerId($0) && !isBootstrapRelayPeer($0) }
            .reduce(into: [String]()) { acc, peer in
                if !acc.contains(peer) { acc.append(peer) }
            }
        guard !sanitizedCandidates.isEmpty else {
            return DeliveryAttemptResult(acked: false, routePeerId: routePeerCandidates.first)
        }

        primeRelayBootstrapConnections()

        for routePeerId in sanitizedCandidates {
            let dialCandidates = buildDialCandidatesForPeer(
                routePeerId: routePeerId,
                rawAddresses: addresses,
                includeRelayCircuits: true
            )
            if !dialCandidates.isEmpty {
                connectToPeer(routePeerId, addresses: dialCandidates)
                _ = await awaitPeerConnection(peerId: routePeerId)
            }

            do {
                try swarmBridge.sendMessage(peerId: routePeerId, data: envelopeData)
                logger.info("✓ Direct delivery ACK from \(routePeerId)")
                return DeliveryAttemptResult(acked: true, routePeerId: routePeerId)
            } catch {
                logger.warning("Core-routed delivery failed for \(routePeerId): \(error.localizedDescription); retrying via relay-circuit")
            }

            let relayOnly = relayCircuitAddresses(for: routePeerId)
            if !relayOnly.isEmpty {
                connectToPeer(routePeerId, addresses: relayOnly)
                _ = await awaitPeerConnection(peerId: routePeerId)
                try? await Task.sleep(nanoseconds: 250_000_000)
                do {
                    try swarmBridge.sendMessage(peerId: routePeerId, data: envelopeData)
                    logger.info("✓ Delivery ACK from \(routePeerId) after relay-circuit retry")
                    return DeliveryAttemptResult(acked: true, routePeerId: routePeerId)
                } catch {
                    logger.warning("Relay-circuit retry failed for \(routePeerId): \(error.localizedDescription)")
                }
            }
        }
        return DeliveryAttemptResult(acked: false, routePeerId: sanitizedCandidates.first)
    }

    private func awaitPeerConnection(peerId: String, timeoutMs: UInt64 = 1200) async -> Bool {
        guard let swarmBridge else { return false }
        let deadline = Date().addingTimeInterval(Double(timeoutMs) / 1000.0)
        while Date() < deadline {
            if swarmBridge.getPeers().contains(peerId) {
                return true
            }
            try? await Task.sleep(nanoseconds: 100_000_000)
        }
        return false
    }

    private func enqueuePendingOutbound(
        historyRecordId: String,
        peerId: String,
        routePeerId: String?,
        addresses: [String],
        envelopeData: Data,
        initialAttemptCount: UInt32 = 0,
        initialDelaySec: UInt64 = 0
    ) {
        let now = UInt64(Date().timeIntervalSince1970)
        var queue = loadPendingOutbox().filter { $0.historyRecordId != historyRecordId }
        queue.append(
            PendingOutboundEnvelope(
                queueId: UUID().uuidString,
                historyRecordId: historyRecordId,
                peerId: peerId,
                routePeerId: routePeerId,
                addresses: addresses,
                envelopeBase64: envelopeData.base64EncodedString(),
                createdAtEpochSec: now,
                attemptCount: initialAttemptCount,
                nextAttemptAtEpochSec: now + initialDelaySec
            )
        )
        savePendingOutbox(queue)
        Task { await flushPendingOutbox(reason: "enqueue") }
    }

    private func flushPendingOutbox(reason: String) async {
        let now = UInt64(Date().timeIntervalSince1970)
        let queue = loadPendingOutbox()
        if queue.isEmpty { return }

        var nextQueue: [PendingOutboundEnvelope] = []
        nextQueue.reserveCapacity(queue.count)

        for item in queue {
            if item.nextAttemptAtEpochSec > now {
                nextQueue.append(item)
                continue
            }

            if let existing = try? historyManager?.get(id: item.historyRecordId), existing.delivered == true {
                continue
            }

            guard let envelopeData = Data(base64Encoded: item.envelopeBase64) else {
                logger.warning("Dropping corrupt pending envelope \(item.queueId)")
                continue
            }

            let contact = (try? contactManager?.get(peerId: item.peerId)) ?? nil
            let latestRouting = parseRoutingHintsFromNotes(contact?.notes)
            let routePeerCandidates = buildRoutePeerCandidates(
                peerId: item.peerId,
                cachedRoutePeerId: item.routePeerId,
                notes: contact?.notes
            )
            let resolvedRoutePeerId = routePeerCandidates.first
            let resolvedAddresses = buildDialCandidatesForPeer(
                routePeerId: resolvedRoutePeerId,
                rawAddresses: item.addresses + latestRouting.addresses,
                includeRelayCircuits: true
            )

            let delivery = await attemptDirectSwarmDelivery(
                routePeerCandidates: routePeerCandidates,
                addresses: resolvedAddresses,
                envelopeData: envelopeData
            )
            let selectedRoutePeerId = delivery.routePeerId ?? resolvedRoutePeerId

            if delivery.acked {
                nextQueue.append(
                    PendingOutboundEnvelope(
                        queueId: item.queueId,
                        historyRecordId: item.historyRecordId,
                        peerId: item.peerId,
                        routePeerId: selectedRoutePeerId,
                        addresses: resolvedAddresses,
                        envelopeBase64: item.envelopeBase64,
                        createdAtEpochSec: item.createdAtEpochSec,
                        attemptCount: item.attemptCount + 1,
                        nextAttemptAtEpochSec: now + receiptAwaitSeconds
                    )
                )
                continue
            }

            let nextAttemptCount = item.attemptCount + 1
            let shift = Int(min(nextAttemptCount, 6))
            let backoff = UInt64(min(60, 1 << shift))
            nextQueue.append(
                    PendingOutboundEnvelope(
                        queueId: item.queueId,
                        historyRecordId: item.historyRecordId,
                        peerId: item.peerId,
                        routePeerId: selectedRoutePeerId,
                        addresses: resolvedAddresses,
                        envelopeBase64: item.envelopeBase64,
                        createdAtEpochSec: item.createdAtEpochSec,
                    attemptCount: nextAttemptCount,
                    nextAttemptAtEpochSec: now + backoff
                )
            )
        }

        savePendingOutbox(nextQueue)
    }

    private func loadPendingOutbox() -> [PendingOutboundEnvelope] {
        guard let data = try? Data(contentsOf: pendingOutboxURL), !data.isEmpty else {
            return []
        }
        guard let decoded = try? JSONDecoder().decode([PendingOutboundEnvelope].self, from: data) else {
            logger.warning("Failed to parse pending outbox")
            return []
        }
        return decoded
    }

    private func savePendingOutbox(_ queue: [PendingOutboundEnvelope]) {
        do {
            let data = try JSONEncoder().encode(queue)
            try data.write(to: pendingOutboxURL, options: .atomic)
        } catch {
            logger.warning("Failed to persist pending outbox: \(error.localizedDescription)")
        }
    }

    private func removePendingOutbound(historyRecordId: String) {
        guard !historyRecordId.isEmpty else { return }
        let queue = loadPendingOutbox()
        let filtered = queue.filter { $0.historyRecordId != historyRecordId }
        guard filtered.count != queue.count else { return }
        savePendingOutbox(filtered)
    }

    private func prioritizeAddressesForCurrentNetwork(_ addresses: [String]) -> [String] {
        guard addresses.count > 1 else { return addresses }
        let lan = addresses.filter { isSameLanAddress($0) }
        guard !lan.isEmpty else { return addresses }
        return (lan + addresses.filter { !lan.contains($0) })
    }

    private func isSameLanAddress(_ multiaddr: String) -> Bool {
        guard let targetIp = extractIpv4FromMultiaddr(multiaddr),
              let localIp = getLocalIpAddress() else {
            return false
        }
        return sameSubnet24(localIp, targetIp)
    }

    private func extractIpv4FromMultiaddr(_ multiaddr: String) -> String? {
        let marker = "/ip4/"
        guard let markerRange = multiaddr.range(of: marker) else { return nil }
        let tail = multiaddr[markerRange.upperBound...]
        let components = tail.split(separator: "/")
        guard let first = components.first else { return nil }
        return String(first)
    }

    private func sameSubnet24(_ a: String, _ b: String) -> Bool {
        let lhs = a.split(separator: ".")
        let rhs = b.split(separator: ".")
        guard lhs.count == 4, rhs.count == 4 else { return false }
        return lhs[0] == rhs[0] && lhs[1] == rhs[1] && lhs[2] == rhs[2]
    }

    private static func parseBootstrapRelay(from multiaddr: String) -> (transportAddr: String, relayPeerId: String)? {
        guard let range = multiaddr.range(of: "/p2p/", options: .backwards) else {
            return nil
        }
        let transportAddr = String(multiaddr[..<range.lowerBound]).trimmingCharacters(in: .whitespacesAndNewlines)
        let relayPeerId = String(multiaddr[range.upperBound...]).trimmingCharacters(in: .whitespacesAndNewlines)
        guard !transportAddr.isEmpty, !relayPeerId.isEmpty else { return nil }
        return (transportAddr, relayPeerId)
    }

    func isBootstrapRelayPeer(_ peerId: String) -> Bool {
        return Self.bootstrapRelayPeerIds.contains(peerId)
    }

    private func buildDialCandidatesForPeer(
        routePeerId: String?,
        rawAddresses: [String],
        includeRelayCircuits: Bool
    ) -> [String] {
        var deduped: [String] = []
        for addr in rawAddresses.compactMap({ normalizeAddressHint($0) }) where !deduped.contains(addr) {
            deduped.append(addr)
        }
        let prioritized = prioritizeAddressesForCurrentNetwork(deduped)
        let relayCircuits = (includeRelayCircuits && (routePeerId?.isEmpty == false))
            ? relayCircuitAddresses(for: routePeerId!)
            : []
        var merged: [String] = []
        for addr in prioritized + relayCircuits where !merged.contains(addr) {
            merged.append(addr)
        }
        return merged
    }

    private func normalizeOutboundListenerHints(_ raw: [String]) -> [String] {
        var out: [String] = []
        for addr in raw.compactMap({ normalizeAddressHint($0) }) where !out.contains(addr) {
            out.append(addr)
        }
        return out
    }

    private func normalizeExternalAddressHints(_ raw: [String]) -> [String] {
        var out: [String] = []
        for addr in raw.compactMap({ normalizeAddressHint($0) }) where !out.contains(addr) {
            out.append(addr)
        }
        return out
    }

    private func normalizeAddressHint(_ raw: String) -> String? {
        let trimmed = raw.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return nil }

        let replacedWildcard: String
        if trimmed.contains("/ip4/0.0.0.0/"), let localIp = getLocalIpAddress() {
            replacedWildcard = trimmed.replacingOccurrences(of: "/ip4/0.0.0.0/", with: "/ip4/\(localIp)/")
        } else {
            replacedWildcard = trimmed
        }

        let candidate = replacedWildcard.hasPrefix("/")
            ? replacedWildcard
            : (toMultiaddrFromSocketAddress(replacedWildcard) ?? "")
        guard !candidate.isEmpty else { return nil }
        guard isDialableAddress(candidate) else { return nil }
        return candidate
    }

    private func toMultiaddrFromSocketAddress(_ value: String) -> String? {
        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return nil }
        if trimmed.hasPrefix("/") { return trimmed }

        guard let separatorIdx = trimmed.lastIndex(of: ":") else { return nil }
        let host = String(trimmed[..<separatorIdx]).trimmingCharacters(in: CharacterSet(charactersIn: "[]"))
        let portStr = String(trimmed[trimmed.index(after: separatorIdx)...])
        guard let port = Int(portStr), (1...65535).contains(port), !host.isEmpty else { return nil }

        let ipv4Regex = try? NSRegularExpression(pattern: #"^\d{1,3}(\.\d{1,3}){3}$"#)
        let hostRange = NSRange(host.startIndex..<host.endIndex, in: host)
        if host.contains(":") {
            return "/ip6/\(host)/tcp/\(port)"
        }
        if let ipv4Regex, ipv4Regex.firstMatch(in: host, options: [], range: hostRange) != nil {
            return "/ip4/\(host)/tcp/\(port)"
        }
        return "/dns4/\(host)/tcp/\(port)"
    }

    private func isDialableAddress(_ multiaddr: String) -> Bool {
        if multiaddr.contains("/p2p-circuit") { return true }
        guard let ip = extractIpv4FromMultiaddr(multiaddr) else { return true }
        if ip == "0.0.0.0" { return false }
        if ip.hasPrefix("127.") { return false }
        if ip.hasPrefix("169.254.") { return false }
        if isPrivateIPv4(ip) {
            return isSameLanAddress(multiaddr)
        }
        return true
    }

    private func isPrivateIPv4(_ ip: String) -> Bool {
        let octets = ip.split(separator: ".").compactMap { Int($0) }
        guard octets.count == 4 else { return false }
        return octets[0] == 10
            || (octets[0] == 172 && (16...31).contains(octets[1]))
            || (octets[0] == 192 && octets[1] == 168)
    }

    private func relayCircuitAddresses(for targetPeerId: String) -> [String] {
        guard isLibp2pPeerId(targetPeerId) else { return [] }
        return Self.defaultBootstrapNodes.compactMap { bootstrap in
            guard let relay = Self.parseBootstrapRelay(from: bootstrap) else { return nil }
            return "\(relay.transportAddr)/p2p/\(relay.relayPeerId)/p2p-circuit/p2p/\(targetPeerId)"
        }
    }

    private func primeRelayBootstrapConnections() {
        guard let swarmBridge else { return }
        let now = Date()
        guard now.timeIntervalSince(lastRelayBootstrapDialAt) >= 10 else { return }
        lastRelayBootstrapDialAt = now

        for addr in Self.defaultBootstrapNodes {
            do {
                try swarmBridge.dial(multiaddr: addr)
            } catch {
                logger.debug("Relay bootstrap dial skipped for \(addr): \(error.localizedDescription)")
            }
        }
    }

    private func persistRouteHintsForTransportPeer(
        libp2pPeerId: String,
        listeners: [String],
        knownPublicKey: String? = nil
    ) {
        guard !libp2pPeerId.isEmpty else { return }
        guard let contacts = try? contactManager?.list() else { return }
        let normalizedListeners = normalizeOutboundListenerHints(listeners)
        let extractedTransportKey: String? = {
            guard let core = ironCore else { return nil }
            return try? core.extractPublicKeyFromPeerId(peerId: libp2pPeerId)
        }()
        let normalizedTransportKey = knownPublicKey
            ?? normalizePublicKey(extractedTransportKey)

        for contact in contacts {
            let routing = parseRoutingHintsFromNotes(contact.notes)
            let match = contact.peerId == libp2pPeerId
                || routing.libp2pPeerId == libp2pPeerId
                || (
                    normalizedTransportKey != nil &&
                    normalizePublicKey(contact.publicKey) == normalizedTransportKey
                )
            if !match { continue }

            let withPeerId = appendRoutingHint(notes: contact.notes, key: "libp2p_peer_id", value: libp2pPeerId)
            let withListeners = upsertRoutingListeners(notes: withPeerId, listeners: normalizedListeners)
            if withListeners == contact.notes { continue }

            let updated = Contact(
                peerId: contact.peerId,
                nickname: contact.nickname,
                localNickname: contact.localNickname,
                publicKey: contact.publicKey,
                addedAt: contact.addedAt,
                lastSeen: contact.lastSeen,
                notes: withListeners
            )
            try? contactManager?.add(contact: updated)
        }
    }

    private func upsertFederatedContact(
        canonicalPeerId: String,
        publicKey: String,
        nickname: String?,
        libp2pPeerId: String?,
        listeners: [String],
        createIfMissing: Bool = true
    ) {
        let normalizedPeerId = canonicalPeerId.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalizedPeerId.isEmpty else { return }
        guard let normalizedKey = normalizePublicKey(publicKey) else { return }

        let normalizedLibp2p = libp2pPeerId?.trimmingCharacters(in: .whitespacesAndNewlines)
        if let normalizedLibp2p, !normalizedLibp2p.isEmpty, isBootstrapRelayPeer(normalizedLibp2p) {
            return
        }

        let contacts = (try? contactManager?.list()) ?? []
        let existingByKey = contacts.first { normalizePublicKey($0.publicKey) == normalizedKey }
        let existingById = contacts.first { $0.peerId == normalizedPeerId }

        // Auth guard: do not accept federated nickname updates for an existing peerId
        // unless the source key matches the stored key for that identity.
        if let existingById,
           normalizePublicKey(existingById.publicKey) != normalizedKey {
            logger.warning(
                "Rejected federated nickname update for \(normalizedPeerId, privacy: .public): key mismatch"
            )
            return
        }
        let existing = existingByKey ?? existingById
        if existing == nil && !createIfMissing {
            return
        }

        var notes = existing?.notes
        if let normalizedLibp2p, !normalizedLibp2p.isEmpty {
            notes = appendRoutingHint(notes: notes, key: "libp2p_peer_id", value: normalizedLibp2p)
        }
        notes = upsertRoutingListeners(notes: notes, listeners: normalizeOutboundListenerHints(listeners))

        let now = UInt64(Date().timeIntervalSince1970)
        let resolvedPeerId = existing?.peerId ?? normalizedPeerId
        let resolvedPublicKey = existingByKey?.publicKey ?? normalizedKey
        let incomingNickname = normalizeNickname(nickname)
        let resolvedNickname = selectAuthoritativeNickname(
            incoming: incomingNickname,
            existing: existing?.nickname
        )

        let updated = Contact(
            peerId: resolvedPeerId,
            nickname: resolvedNickname,
            localNickname: existing?.localNickname,
            publicKey: resolvedPublicKey,
            addedAt: existing?.addedAt ?? now,
            lastSeen: now,
            notes: notes
        )
        try? contactManager?.add(contact: updated)
        annotateIdentityInLedger(
            routePeerId: normalizedLibp2p,
            listeners: listeners,
            publicKey: resolvedPublicKey,
            nickname: resolvedNickname
        )
    }

    private func upsertRoutingListeners(notes: String?, listeners: [String]) -> String? {
        guard !listeners.isEmpty else { return notes }
        let base = notes?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let filtered = base
            .split(whereSeparator: { $0 == ";" || $0 == "\n" })
            .map { String($0).trimmingCharacters(in: .whitespacesAndNewlines) }
            .filter { !$0.isEmpty && !$0.hasPrefix("listeners:") }
        return (filtered + ["listeners:\(listeners.joined(separator: ","))"]).joined(separator: ";")
    }

    private func broadcastIdentityBeacon() {
        guard let info = ironCore?.getIdentityInfo(),
              let publicKeyHex = info.publicKeyHex else { return }

        // Keep BLE identity beacons compact to avoid platform read failures.
        // Android/iOS both have observed issues when payload exceeds ~512 bytes.
        var listeners = Array(normalizeOutboundListenerHints(getListeningAddresses()).prefix(2))
        var externalAddresses = Array(normalizeExternalAddressHints(getExternalAddresses()).prefix(2))
        let nickname = String((info.nickname ?? "").prefix(32))

        func buildBeacon() -> [String: Any] {
            let connectionHints = Array(Set(listeners + externalAddresses)).sorted()
            return [
                "identity_id": info.identityId ?? "",
                "public_key": publicKeyHex,
                "nickname": nickname,
                "libp2p_peer_id": info.libp2pPeerId ?? "",
                "listeners": listeners,
                "external_addresses": externalAddresses,
                "connection_hints": connectionHints
            ]
        }

        var beacon: [String: Any] = buildBeacon()
        var data = try? JSONSerialization.data(withJSONObject: beacon)
        if let encoded = data, encoded.count > 480 {
            listeners = Array(listeners.prefix(1))
            externalAddresses = Array(externalAddresses.prefix(1))
            beacon = buildBeacon()
            data = try? JSONSerialization.data(withJSONObject: beacon)
        }
        if let encoded = data, encoded.count > 480 {
            listeners = []
            externalAddresses = []
            beacon = buildBeacon()
            data = try? JSONSerialization.data(withJSONObject: beacon)
        }
        if let encoded = data, encoded.count > 480 {
            beacon = [
                "identity_id": info.identityId ?? "",
                "public_key": publicKeyHex,
                "nickname": nickname,
                "libp2p_peer_id": info.libp2pPeerId ?? "",
                "listeners": [],
                "external_addresses": [],
                "connection_hints": []
            ]
            data = try? JSONSerialization.data(withJSONObject: beacon)
        }
        guard let data else {
            logger.error("Failed to serialize identity beacon")
            return
        }
        blePeripheralManager?.setIdentityData(data)
        logger.info("BLE identity beacon set: \(publicKeyHex.prefix(8))... (\(data.count) bytes)")
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
        applyPowerAdjustments(reason: "overrides_cleared")
    }

    private func applyPowerAdjustments(reason: String) {
        guard let meshService = meshService else { return }
        guard let engine = autoAdjustEngine else {
            logger.debug("Power profile skipped (\(reason)): AutoAdjustEngine unavailable")
            return
        }

        guard isAutoAdjustEnabled else {
            if lastAppliedPowerSnapshot != "disabled" {
                logger.info("Power profile skipped (\(reason)): AutoAdjust disabled")
                lastAppliedPowerSnapshot = "disabled"
            }
            return
        }

        let profile = DeviceProfile(
            batteryPct: currentBatteryPct,
            isCharging: currentIsCharging,
            hasWifi: networkStatus.wifi,
            motionState: currentMotionState
        )

        let adjustmentProfile = engine.computeProfile(device: profile)
        let relayAdjustment = engine.computeRelayAdjustment(profile: adjustmentProfile)
        let bleAdjustment = engine.computeBleAdjustment(profile: adjustmentProfile)

        meshService.setRelayBudget(messagesPerHour: relayAdjustment.maxPerHour)
        bleCentralManager?.applyScanSettings(intervalMs: bleAdjustment.scanIntervalMs)

        let snapshot = [
            "p:\(adjustmentProfile)",
            "relay:\(relayAdjustment.maxPerHour)",
            "scan:\(bleAdjustment.scanIntervalMs)",
            "adv:\(bleAdjustment.advertiseIntervalMs)",
            "tx:\(bleAdjustment.txPowerDbm)",
            "bat:\(profile.batteryPct)",
            "chg:\(profile.isCharging)",
            "wifi:\(profile.hasWifi)",
            "motion:\(profile.motionState)"
        ].joined(separator: "|")

        if lastAppliedPowerSnapshot != snapshot {
            logger.info(
                "Power profile applied (\(reason)): profile=\(String(describing: adjustmentProfile)) relay=\(relayAdjustment.maxPerHour)/h scan=\(bleAdjustment.scanIntervalMs)ms adv=\(bleAdjustment.advertiseIntervalMs)ms tx=\(bleAdjustment.txPowerDbm)dBm battery=\(profile.batteryPct)% charging=\(profile.isCharging) wifi=\(profile.hasWifi) motion=\(String(describing: profile.motionState))"
            )
            lastAppliedPowerSnapshot = snapshot
        } else {
            logger.debug("Power profile unchanged (\(reason)): \(snapshot)")
        }
    }

    // MARK: - Identity Export Helpers

    func getPreferredRelay() -> String? {
        return ledgerManager?.getPreferredRelays(limit: 1).first?.peerId
    }

    func connectToPeer(_ peerId: String, addresses: [String]) {
        let dialCandidates = buildDialCandidatesForPeer(
            routePeerId: peerId,
            rawAddresses: addresses,
            includeRelayCircuits: false
        )

        for addr in dialCandidates {
            // Only append /p2p/ component if the peerId is a valid libp2p PeerId format
            // (base58btc multihash, starts with "12D3Koo" or "Qm").
            // Blake3 hex identity_ids (64 hex chars) are NOT valid libp2p PeerIds.
            var finalAddr = addr
            if isLibp2pPeerId(peerId) && !addr.contains("/p2p/") {
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

    func getExternalAddresses() -> [String] {
        return swarmBridge?.getExternalAddresses() ?? []
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
        var listeners = normalizeOutboundListenerHints(getListeningAddresses())
        let externalAddresses = normalizeExternalAddressHints(getExternalAddresses())
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

            listeners = updatedListeners
        }

        let payload: [String: Any] = [
            "identity_id": identity.identityId ?? "",
            "nickname": identity.nickname ?? "",
            "public_key": identity.publicKeyHex ?? "",
            "libp2p_peer_id": identity.libp2pPeerId ?? "",
            "listeners": listeners,
            "external_addresses": externalAddresses,
            "connection_hints": Array(Set(listeners + externalAddresses)),
            "relay": relay
        ]
        guard let data = try? JSONSerialization.data(withJSONObject: payload),
              let json = String(data: data, encoding: .utf8) else {
            return "{}"
        }
        return json
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
        let trimmedNickname = nickname.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedNickname.isEmpty else {
            throw MeshError.invalidInput("Nickname cannot be empty")
        }
        try ironCore.setNickname(nickname: trimmedNickname)
        persistIdentityBackupToKeychain(ironCore: ironCore)
        logger.info("✓ Nickname set to: \(trimmedNickname)")
        broadcastIdentityBeacon()
        identitySyncSentPeers.removeAll()
        let connectedPeers = swarmBridge?.getPeers() ?? []
        for routePeerId in connectedPeers {
            sendIdentitySyncIfNeeded(routePeerId: routePeerId)
        }
    }
}

// MARK: - Error Types

enum MeshError: LocalizedError {
    case notInitialized(String)
    case relayDisabled(String)
    case contactNotFound(String)
    case invalidInput(String)
    case alreadyRunning

    var errorDescription: String? {
        switch self {
        case .notInitialized(let msg): return msg
        case .relayDisabled(let msg): return msg
        case .contactNotFound(let msg): return msg
        case .invalidInput(let msg): return msg
        case .alreadyRunning: return "Service is already running"
        }
    }
}

private extension String {
    var nilIfEmpty: String? {
        isEmpty ? nil : self
    }
}
