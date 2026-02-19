package com.scmessenger.android.data

import android.content.Context
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import timber.log.Timber
import kotlinx.coroutines.flow.filter
import java.io.File

/**
 * Repository abstracting access to the Rust core via UniFFI bindings.
 *
 * This is the single source of truth for:
 * - Mesh service lifecycle
 * - Contacts management
 * - Message history
 * - Connection ledger
 * - Network settings
 *
 * All UniFFI objects are initialized lazily and managed here to ensure
 * proper lifecycle and resource cleanup.
 */
class MeshRepository(private val context: Context) {

    private val storagePath: String = context.filesDir.absolutePath

    // Mesh service instance (lazy init)
    private var meshService: uniffi.api.MeshService? = null

    // Managers (lazy init)
    private var contactManager: uniffi.api.ContactManager? = null
    private var historyManager: uniffi.api.HistoryManager? = null
    private var ledgerManager: uniffi.api.LedgerManager? = null
    private var settingsManager: uniffi.api.MeshSettingsManager? = null
    private var autoAdjustEngine: uniffi.api.AutoAdjustEngine? = null

    // Core & Network (lazy init)
    private var ironCore: uniffi.api.IronCore? = null
    // Swarm Bridge (Internet/Libp2p)
    private var swarmBridge: uniffi.api.SwarmBridge? = null

    // Wifi Transport
    private var wifiTransportManager: com.scmessenger.android.transport.WifiTransportManager? = null

    // Service state
    private val _serviceState = MutableStateFlow(uniffi.api.ServiceState.STOPPED)
    val serviceState: StateFlow<uniffi.api.ServiceState> = _serviceState.asStateFlow()

    // Service stats
    private val _serviceStats = MutableStateFlow<uniffi.api.ServiceStats?>(null)
    val serviceStats: StateFlow<uniffi.api.ServiceStats?> = _serviceStats.asStateFlow()

    // Message updates flow (both sent and received) used for UI updates
    private val _messageUpdates = kotlinx.coroutines.flow.MutableSharedFlow<uniffi.api.MessageRecord>(replay = 0)
    val messageUpdates = _messageUpdates.asSharedFlow()

    // Compatibility for notifications (incoming only)
    val incomingMessages = messageUpdates.filter { it.direction == uniffi.api.MessageDirection.RECEIVED }

    private val repoScope = kotlinx.coroutines.CoroutineScope(kotlinx.coroutines.Dispatchers.IO + kotlinx.coroutines.SupervisorJob())

    // Core Delegate reference to prevent GC
    private var coreDelegate: uniffi.api.CoreDelegate? = null

    // BLE Components
    private var bleScanner: com.scmessenger.android.transport.ble.BleScanner? = null
    private var bleAdvertiser: com.scmessenger.android.transport.ble.BleAdvertiser? = null
    private var bleGattServer: com.scmessenger.android.transport.ble.BleGattServer? = null
    private var bleGattClient: com.scmessenger.android.transport.ble.BleGattClient? = null

    init {
        Timber.d("MeshRepository initialized with storage: $storagePath")
        initializeManagers()
    }

    private fun initializeManagers() {
        try {
            // Initialize Data Managers
            settingsManager = uniffi.api.MeshSettingsManager(storagePath)
            historyManager = uniffi.api.HistoryManager(storagePath)
            contactManager = uniffi.api.ContactManager(storagePath)
            ledgerManager = uniffi.api.LedgerManager(storagePath)
            autoAdjustEngine = uniffi.api.AutoAdjustEngine()

            // Pre-load data where applicable
            ledgerManager?.load()

            Timber.i("All managers initialized successfully")
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize managers")
        }
    }

    // ========================================================================
    // MESH SERVICE LIFECYCLE
    // ========================================================================

    /**
     * Start the mesh service with the given configuration.
     * This initializes the Rust core, starts BLE transport, and wires up events.
     */
    fun startMeshService(config: uniffi.api.MeshServiceConfig) {
        try {
            Timber.d("Starting MeshService...")
            if (meshService == null) {
                // Use the 'withStorage' constructor to ensure DB path is correct
                meshService = uniffi.api.MeshService.withStorage(config, storagePath)
            }

            // 1. Start the Rust Core Service
            if (meshService?.getState() == uniffi.api.ServiceState.RUNNING) {
                Timber.d("MeshService is already running")
            } else {
                meshService?.start()
            }

            // 2. Obtain Shared IronCore Instance (Singleton)
            ironCore = meshService?.getCore()
            if (ironCore == null) {
                Timber.w("IronCore instance is null after service start!")
            }

            // 3. Wire up CoreDelegate (Rust -> Android Events)
            coreDelegate = object : uniffi.api.CoreDelegate {
                override fun onPeerDiscovered(peerId: String) {
                    Timber.d("Core notified discovery: $peerId")
                    repoScope.launch {
                        com.scmessenger.android.service.MeshEventBus.emitPeerEvent(
                            com.scmessenger.android.service.PeerEvent.Discovered(
                                peerId,
                                com.scmessenger.android.service.TransportType.INTERNET
                            )
                        )
                    }
                }

                override fun onPeerDisconnected(peerId: String) {
                    Timber.d("Core notified disconnect: $peerId")
                    repoScope.launch {
                        com.scmessenger.android.service.MeshEventBus.emitPeerEvent(
                            com.scmessenger.android.service.PeerEvent.Disconnected(peerId)
                        )
                    }
                }

                override fun onMessageReceived(senderId: String, senderPublicKeyHex: String, messageId: String, data: ByteArray) {
                    Timber.i("Message from $senderId: $messageId")
                    try {
                        // Check if relay/messaging is enabled (bidirectional control)
                        // Treat null/missing settings as disabled (fail-safe)
                        // Cache settings value to avoid race condition during check
                        val currentSettings = settingsManager?.load()
                        val isRelayEnabled = currentSettings?.relayEnabled == true

                        if (!isRelayEnabled) {
                            Timber.w("Dropping received message - mesh participation is disabled or settings unavailable")
                            return
                        }

                        // Auto-upsert contact: senderPublicKeyHex is guaranteed valid Ed25519 key
                        // (Rust only fires this callback after successful decryption)
                        val existingContact = try { contactManager?.get(senderId) } catch (e: Exception) { null }
                        if (existingContact == null && senderPublicKeyHex.trim().length == 64) {
                            val autoContact = uniffi.api.Contact(
                                peerId = senderId,
                                nickname = null,
                                publicKey = senderPublicKeyHex.trim(),
                                addedAt = (System.currentTimeMillis() / 1000).toULong(),
                                lastSeen = (System.currentTimeMillis() / 1000).toULong(),
                                notes = null
                            )
                            try {
                                contactManager?.add(autoContact)
                                Timber.i("Auto-created contact from received message: ${senderId.take(8)} key: ${senderPublicKeyHex.take(8)}...")
                            } catch (e: Exception) {
                                Timber.w("Auto-create contact failed for ${senderId.take(8)}: ${e.message}")
                            }
                        } else if (existingContact != null) {
                            try { contactManager?.updateLastSeen(senderId) } catch (e: Exception) {
                                Timber.d("updateLastSeen failed: ${e.message}")
                            }
                        }

                        val content = data.toString(Charsets.UTF_8)
                        val record = uniffi.api.MessageRecord(
                            id = messageId,
                            direction = uniffi.api.MessageDirection.RECEIVED,
                            peerId = senderId,
                            content = content,
                            timestamp = (System.currentTimeMillis() / 1000).toULong(),
                            delivered = true
                        )
                        historyManager?.add(record)

                        // Emit for notifications and UI updates
                        repoScope.launch {
                            _messageUpdates.emit(record)
                        }

                        // Send delivery receipt ACK back to sender via SwarmBridge.
                        // senderPublicKeyHex is the sender's Ed25519 public key hex —
                        // prepareReceipt() encrypts an ACK envelope addressed to that key.
                        repoScope.launch {
                            try {
                                val receiptBytes = ironCore?.prepareReceipt(senderPublicKeyHex, messageId)
                                if (receiptBytes != null) {
                                    // Broadcast delivery receipt ACK back to all peers.
                                    // The receipt is encrypted for the specific recipient; only they can decrypt it.
                                    swarmBridge?.sendToAllPeers(receiptBytes)
                                    Timber.d("Delivery receipt broadcast for $messageId to $senderId")
                                }
                            } catch (e: Exception) {
                                Timber.d("Failed to send delivery receipt for $messageId: ${e.message}")
                            }
                        }
                    } catch (e: Exception) {
                        Timber.e(e, "Failed to process received message")
                    }
                }

                override fun onReceiptReceived(messageId: String, status: String) {
                     Timber.d("Receipt for $messageId: $status")
                     historyManager?.markDelivered(messageId)
                }
            }
            ironCore?.setDelegate(coreDelegate)

            // 4. Start Android Transports (BLE & WiFi & Swarm)
            initializeAndStartBle()
            initializeAndStartWifi()
            initializeAndStartSwarm()

            // 5. Update State
            _serviceState.value = uniffi.api.ServiceState.RUNNING
            updateStats()

            Timber.i("Mesh service started successfully")
        } catch (e: Exception) {
            Timber.e(e, "Failed to start mesh service")
            // Propagate error to UI/Service?
            _serviceState.value = uniffi.api.ServiceState.STOPPED
        }
    }

    private fun initializeAndStartBle() {
        val settings = loadSettings()
        if (!settings.bleEnabled) {
            Timber.d("BLE disabled in settings")
            return
        }

        // BLE Scanner: Feeds discovered peers to MeshService and handles GATT connections
        if (bleScanner == null) {
            bleScanner = com.scmessenger.android.transport.ble.BleScanner(
                context,
                onPeerDiscovered = { peerId ->
                    meshService?.onPeerDiscovered(peerId)
                    // Connect via GATT client to read identity if needed
                    bleGattClient?.connect(peerId)
                },
                onDataReceived = { peerId, data ->
                    meshService?.onDataReceived(peerId, data)
                }
            )
        }
        bleScanner?.startScanning()

        // BLE GATT Client: Manages connections to discovered peers
        if (bleGattClient == null) {
            bleGattClient = com.scmessenger.android.transport.ble.BleGattClient(
                context,
                onIdentityReceived = { peerId, data ->
                    onPeerIdentityRead(peerId, data)
                },
                onDataReceived = { peerId, data ->
                    meshService?.onDataReceived(peerId, data)
                }
            )
        }

        // BLE Advertiser: Broadcasts our presence
        if (bleAdvertiser == null) {
            bleAdvertiser = com.scmessenger.android.transport.ble.BleAdvertiser(context)
        }
        bleAdvertiser?.startAdvertising()

        // BLE GATT Server: Serves our identity and accepts writes from nearby peers
        if (bleGattServer == null) {
            bleGattServer = com.scmessenger.android.transport.ble.BleGattServer(
                context,
                onDataReceived = { peerId, data ->
                    meshService?.onDataReceived(peerId, data)
                }
            )
        }
        bleGattServer?.start()

        // Set identity beacon on BLE GATT server so nearby peers can read our Ed25519 public key
        val identity = ironCore?.getIdentityInfo()
        val publicKeyHex = identity?.publicKeyHex
        if (!publicKeyHex.isNullOrEmpty()) {
            try {
                val beaconJson = org.json.JSONObject()
                    .put("public_key", publicKeyHex)
                    .put("nickname", identity.nickname ?: "")
                    .put("libp2p_peer_id", identity.libp2pPeerId ?: "")
                    .put("listeners", org.json.JSONArray(getListeningAddresses()))
                    .toString()
                    .toByteArray(Charsets.UTF_8)
                bleGattServer?.setIdentityData(beaconJson)
                Timber.i("BLE GATT identity beacon set: ${publicKeyHex.take(8)}... (includes libp2p)")
            } catch (e: Exception) {
                Timber.w("Failed to set BLE GATT identity beacon: ${e.message}")
            }
        }
    }

    /**
     * Called when BLE identity beacon is read from a peer.
     * Extracts identity info and attempts to dial the peer via libp2p if possible.
     */
    private fun onPeerIdentityRead(blePeerId: String, data: ByteArray) {
        try {
            val json = org.json.JSONObject(data.toString(Charsets.UTF_8))
            val publicKeyHex = json.getString("public_key")
            val nickname = json.optString("nickname")
            val libp2pPeerId = json.optString("libp2p_peer_id")
            val listeners = json.optJSONArray("listeners")

            Timber.i("Peer identity read from $blePeerId: ${publicKeyHex.take(8)}...")

            // Auto-create/update contact
            val existing = try { contactManager?.get(blePeerId) } catch (e: Exception) { null }
            if (existing == null) {
                val notes = if (!libp2pPeerId.isNullOrEmpty()) "libp2p_peer_id: $libp2pPeerId" else null
                val contact = uniffi.api.Contact(
                    peerId = blePeerId,
                    nickname = if (nickname.isNullOrEmpty()) null else nickname,
                    publicKey = publicKeyHex,
                    addedAt = (System.currentTimeMillis() / 1000).toULong(),
                    lastSeen = (System.currentTimeMillis() / 1000).toULong(),
                    notes = notes
                )
                contactManager?.add(contact)
                Timber.d("Created contact for BLE peer: $blePeerId")
            } else {
                contactManager?.updateLastSeen(blePeerId)
                // Update notes if libp2p PeerId is newly discovered
                if (!libp2pPeerId.isNullOrEmpty() && existing.notes?.contains(libp2pPeerId) != true) {
                    val newNotes = "libp2p_peer_id: $libp2pPeerId"
                    contactManager?.setNickname(blePeerId, existing.nickname) // Set notes via metadata?
                    // Wait, UniFFI Contact interface doesn't have setNotes.
                    // But we can re-add or ignore.
                }
            }

            // Attempt to dial via Swarm if we have libp2p info
            if (!libp2pPeerId.isNullOrEmpty() && listeners != null) {
                for (i in 0 until listeners.length()) {
                    val addr = listeners.getString(i)
                    connectToPeer(libp2pPeerId, listOf(addr))
                }
            }
        } catch (e: Exception) {
            Timber.w("Failed to parse peer identity read: ${e.message}")
        }
    }

    private fun initializeAndStartWifi() {
        val settings = loadSettings()
        if (!settings.wifiAwareEnabled && !settings.wifiDirectEnabled) {
            Timber.d("WiFi Transports disabled in settings")
            // Note: WifiTransportManager manages both. Need granular control?
            // For now, if either is enabled, we start it, let it handle internals?
            // But WifiTransportManager likely starts both.
            // Assuming strict check:
             return
        }

        if (wifiTransportManager == null) {
            wifiTransportManager = com.scmessenger.android.transport.WifiTransportManager(
                context,
                onPeerDiscovered = { peerId ->
                    meshService?.onPeerDiscovered(peerId)
                },
                onDataReceived = { peerId, data ->
                    meshService?.onDataReceived(peerId, data)
                }
            )
        }
        wifiTransportManager?.initialize()
        wifiTransportManager?.startDiscovery()
    }

    private fun initializeAndStartSwarm() {
        val settings = loadSettings()
        if (!settings.internetEnabled) {
            Timber.d("Swarm/Internet disabled in settings")
            return
        }

        try {
            // Ensure identity is initialized locally before starting Swarm
            // (IdentityKeys are needed for libp2p PeerId)
            if (isIdentityInitialized() == false) {
                 Timber.i("Auto-initializing identity for first run...")
                 ironCore?.initializeIdentity()
            }

            if (isIdentityInitialized() == true) {
                // Initiate swarm in Rust core
                meshService?.startSwarm("/ip4/0.0.0.0/tcp/0")

                // Obtain the SwarmBridge managed by Rust MeshService
                swarmBridge = meshService?.getSwarmBridge()

                Timber.i("✓ Internet transport (Swarm) initiated and bridge wired")
            } else {
                Timber.w("Postponing Swarm start: Identity not ready")
            }
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize Swarm transport")
        }
    }

    fun setPlatformBridge(bridge: uniffi.api.PlatformBridge) {
        meshService?.setPlatformBridge(bridge)
    }

    /**
     * Stop the mesh service and all transports.
     */
    fun stopMeshService() {
        try {
            // Stop BLE
            bleScanner?.stopScanning()
            bleAdvertiser?.stopAdvertising()
            bleGattServer?.stop()
            bleGattClient?.cleanup()

            // Stop WiFi
            wifiTransportManager?.stopDiscovery()

            // Stop Swarm
            swarmBridge?.shutdown()

            // Stop Rust Core
            meshService?.stop()

            // Clear State
            _serviceState.value = uniffi.api.ServiceState.STOPPED
            _serviceStats.value = null

            Timber.i("Mesh service stopped")
        } catch (e: Exception) {
            Timber.e(e, "Failed to stop mesh service")
        }
    }

    /**
     * Pause the mesh service (reduced activity).
     */
    fun pauseMeshService() {
        meshService?.pause()
        Timber.d("Mesh service paused")
    }

    /**
     * Resume the mesh service (full activity).
     */
    fun resumeMeshService() {
        meshService?.resume()
        Timber.d("Mesh service resumed")
    }

    /**
     * Get current service state.
     */
    fun getServiceState(): uniffi.api.ServiceState {
        return meshService?.getState() ?: uniffi.api.ServiceState.STOPPED
    }

    /**
     * Update and emit service stats.
     */
    private fun updateStats() {
        try {
            _serviceStats.value = meshService?.getStats()
        } catch (e: Exception) {
            Timber.e(e, "Failed to get service stats")
        }
    }

    // ========================================================================
    // CONTACTS
    // ========================================================================

    fun addContact(contact: uniffi.api.Contact) {
        contactManager?.add(contact)
        Timber.d("Contact added: ${contact.peerId}")
    }

    fun getContact(peerId: String): uniffi.api.Contact? {
        return contactManager?.get(peerId)
    }

    fun removeContact(peerId: String) {
        contactManager?.remove(peerId)
        Timber.d("Contact removed: $peerId")
    }

    fun listContacts(): List<uniffi.api.Contact> {
        return contactManager?.list() ?: emptyList()
    }

    fun searchContacts(query: String): List<uniffi.api.Contact> {
        return contactManager?.search(query) ?: emptyList()
    }

    fun setContactNickname(peerId: String, nickname: String?) {
        contactManager?.setNickname(peerId, nickname)
        Timber.d("Contact nickname updated: $peerId -> $nickname")
    }

    fun getContactCount(): UInt {
        return contactManager?.count() ?: 0u
    }

    // ========================================================================
    // MESSAGE HISTORY
    // ========================================================================

    fun getIdentityInfo(): uniffi.api.IdentityInfo? {
        return ironCore?.getIdentityInfo()
    }

    fun setNickname(nickname: String) {
        ironCore?.setNickname(nickname)
        Timber.i("Nickname set to: $nickname")
    }

    suspend fun sendMessage(peerId: String, content: String) {
        kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
            try {
                // Check if relay/messaging is enabled (bidirectional control)
                // Treat null/missing settings as disabled (fail-safe)
                // Cache settings value to avoid race condition during check
                val currentSettings = settingsManager?.load()
                val isRelayEnabled = currentSettings?.relayEnabled == true

                if (!isRelayEnabled) {
                    throw IllegalStateException("Cannot send messages: mesh participation is disabled. Enable mesh participation in settings to send and receive messages.")
                }

                // 1. Get recipient's public key
                val contact = contactManager?.get(peerId)
                    ?: throw IllegalStateException("Contact not found for peer: $peerId")

                val publicKey = contact.publicKey.trim()

                // Pre-validate public key to provide descriptive errors
                if (publicKey.isEmpty()) {
                    throw IllegalStateException("Contact $peerId has no public key. Please re-add this contact with a valid public key.")
                }
                if (publicKey.length != 64) {
                    throw IllegalStateException("Contact $peerId has invalid public key (length: ${publicKey.length}, expected 64 hex chars). Please re-add this contact.")
                }
                if (!publicKey.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' }) {
                    throw IllegalStateException("Contact $peerId has invalid public key (non-hex characters). Please re-add this contact.")
                }

                Timber.d("Preparing message for $peerId with key: ${publicKey.take(8)}...")

                // 2. Encrypt/Prepare message (use trimmed key)
                val encryptedData = ironCore?.prepareMessage(publicKey, content)
                    ?: throw IllegalStateException("Failed to prepare message: IronCore not initialized")

                // Convert List<Byte> (or whatever UniFFI returns) to ByteArray
                // UniFFI 'bytes' maps to ByteArray, so encryptedData is ByteArray.

                // 3. Send over network (Multiple transports)
                // Attempt BLE (GATT)
                // Try Central role (push to peripheral)
                bleGattClient?.sendData(peerId, encryptedData)
                // Try Peripheral role (push to central)
                bleGattServer?.sendData(peerId, encryptedData)

                // Attempt WiFi
                wifiTransportManager?.sendData(peerId, encryptedData)

                // Attempt Swarm (Internet) — broadcast to all connected peers.
                try {
                    swarmBridge?.sendToAllPeers(encryptedData)
                } catch (e: Exception) {
                    Timber.w("SwarmBridge delivery queued (no peers connected): ${e.message}")
                }

                // Note: In a real mesh, we would route via MeshService/Libp2p which manages transports.
                // Since we moved Transport logic to Kotlin for Phases 4/5, we call them here.
                // Ideally, we get a confirmation callback.

                // 4. Save to history
                val record = uniffi.api.MessageRecord(
                    id = java.util.UUID.randomUUID().toString(),
                    peerId = peerId,
                    direction = uniffi.api.MessageDirection.SENT,
                    content = content,
                    timestamp = (System.currentTimeMillis() / 1000).toULong(),
                    delivered = false // Will be updated on receipt
                )
                historyManager?.add(record)

                // Emit for UI updates (e.g., chat list)
                repoScope.launch {
                    _messageUpdates.emit(record)
                }

                Timber.i("Message sent (encrypted) to $peerId")
            } catch (e: Exception) {
                Timber.e(e, "Failed to send message")
                throw e
            }
        }
    }

    suspend fun dial(multiaddr: String) {
        kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
            try {
                // Attempt Swarm Dial
                swarmBridge?.dial(multiaddr)
                Timber.i("Dialed $multiaddr via SwarmBridge")
            } catch (e: Exception) {
                Timber.e(e, "Failed to dial $multiaddr")
                throw e
            }
        }
    }

    suspend fun dialPeer(multiaddr: String) = dial(multiaddr)

    // Identity Management
    fun isIdentityInitialized(): Boolean {
        ensureServiceInitialized()
        return ironCore?.getIdentityInfo()?.initialized == true
    }

    suspend fun createIdentity() {
        kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
            try {
                ensureServiceInitialized()
                if (ironCore == null) {
                    Timber.e("IronCore is null after ensureServiceInitialized! Cannot create identity.")
                    throw IllegalStateException("Mesh service initialization failed")
                }
                Timber.d("Calling ironCore.initializeIdentity()...")
                ironCore?.initializeIdentity()
                Timber.i("Identity created successfully")
            } catch (e: Exception) {
                Timber.e(e, "Failed to create identity")
                throw e
            }
        }
    }

    // Helper to ensure service is initialized lazily
    private fun ensureServiceInitialized() {
        if (meshService == null || meshService?.getState() != uniffi.api.ServiceState.RUNNING) {
            Timber.d("Lazy starting MeshService for Identity access...")
            try {
                // If service not running, launch it properly
                val settings = loadSettings()
                val config = uniffi.api.MeshServiceConfig(
                    discoveryIntervalMs = 30000u,
                    batteryFloorPct = settings.batteryFloor
                )
                startMeshService(config)

                Timber.d("MeshService started lazily")
            } catch (e: Exception) {
                Timber.e(e, "Failed to start MeshService lazily")
            }
        }

        // Refresh ironCore reference just in case
        if (ironCore == null) {
            ironCore = meshService?.getCore()
            Timber.d("IronCore reference refreshed: ${ironCore != null}")
        }
    }

    // Keep legacy addMessage for receiving side or manual adds
    fun addMessage(record: uniffi.api.MessageRecord) {
        historyManager?.add(record)
    }

    fun getMessage(id: String): uniffi.api.MessageRecord? {
        return historyManager?.get(id)
    }

    fun getRecentMessages(peerFilter: String? = null, limit: UInt = 50u): List<uniffi.api.MessageRecord> {
        return historyManager?.recent(peerFilter, limit) ?: emptyList()
    }

    fun getConversation(peerId: String, limit: UInt = 100u): List<uniffi.api.MessageRecord> {
        return historyManager?.conversation(peerId, limit) ?: emptyList()
    }

    fun searchMessages(query: String, limit: UInt = 50u): List<uniffi.api.MessageRecord> {
        return historyManager?.search(query, limit) ?: emptyList()
    }

    fun markMessageDelivered(id: String) {
        historyManager?.markDelivered(id)
    }

    fun clearHistory() {
        historyManager?.clear()
        Timber.i("Message history cleared")
    }

    fun clearConversation(peerId: String) {
        historyManager?.clearConversation(peerId)
        Timber.i("Conversation cleared: $peerId")
    }

    fun getHistoryStats(): uniffi.api.HistoryStats? {
        return historyManager?.stats()
    }

    fun getMessageCount(): UInt {
        return historyManager?.count() ?: 0u
    }

    // ========================================================================
    // LEDGER
    // ========================================================================

    fun recordConnection(multiaddr: String, peerId: String) {
        ledgerManager?.recordConnection(multiaddr, peerId)
    }

    fun recordConnectionFailure(multiaddr: String) {
        ledgerManager?.recordFailure(multiaddr)
    }

    fun getDialableAddresses(): List<uniffi.api.LedgerEntry> {
        return ledgerManager?.dialableAddresses() ?: emptyList()
    }

    fun getAllKnownTopics(): List<String> {
        return ledgerManager?.allKnownTopics() ?: emptyList()
    }

    fun getLedgerSummary(): String {
        return ledgerManager?.summary() ?: "Ledger not available"
    }

    fun saveLedger() {
        ledgerManager?.save()
    }

    // ========================================================================
    // SETTINGS
    // ========================================================================

    fun loadSettings(): uniffi.api.MeshSettings {
        val loaded = try {
            settingsManager?.load()
        } catch (e: Exception) {
            Timber.w("Settings load failed (likely first run), using defaults: ${e.message}")
            null
        }

        return loaded ?: settingsManager?.defaultSettings()
            ?: uniffi.api.MeshSettings(
                relayEnabled = true,
                maxRelayBudget = 200u,
                batteryFloor = 20u,
                bleEnabled = true,
                wifiAwareEnabled = true,
                wifiDirectEnabled = true,
                internetEnabled = true,
                discoveryMode = uniffi.api.DiscoveryMode.NORMAL,
                onionRouting = false
            )
    }

    fun saveSettings(settings: uniffi.api.MeshSettings) {
        settingsManager?.save(settings)
        Timber.i("Settings saved")
    }

    fun validateSettings(settings: uniffi.api.MeshSettings): Boolean {
        return try {
            settingsManager?.validate(settings)
            true
        } catch (e: Exception) {
            Timber.w(e, "Settings validation failed")
            false
        }
    }

    // ========================================================================
    // AUTO-ADJUST ENGINE
    // ========================================================================

    fun computeAdjustmentProfile(deviceProfile: uniffi.api.DeviceProfile): uniffi.api.AdjustmentProfile {
        return autoAdjustEngine?.computeProfile(deviceProfile)
            ?: uniffi.api.AdjustmentProfile.STANDARD
    }

    fun computeBleAdjustment(profile: uniffi.api.AdjustmentProfile): uniffi.api.BleAdjustment {
        return autoAdjustEngine?.computeBleAdjustment(profile)
            ?: uniffi.api.BleAdjustment(
                scanIntervalMs = 2000u,
                advertiseIntervalMs = 500u,
                txPowerDbm = -4
            )
    }

    fun computeRelayAdjustment(profile: uniffi.api.AdjustmentProfile): uniffi.api.RelayAdjustment {
        return autoAdjustEngine?.computeRelayAdjustment(profile)
            ?: uniffi.api.RelayAdjustment(
                maxPerHour = 200u,
                priorityThreshold = 100u,
                maxPayloadBytes = 16384u
            )
    }

    fun overrideBleInterval(intervalMs: UInt) {
        autoAdjustEngine?.overrideBleScanInterval(intervalMs)
    }

    fun setRelayBudget(messagesPerHour: UInt) {
        meshService?.setRelayBudget(messagesPerHour)
    }

    fun updateDeviceState(profile: uniffi.api.DeviceProfile) {
        meshService?.updateDeviceState(profile)
    }

    fun overrideRelayMax(max: UInt) {
        autoAdjustEngine?.overrideRelayMaxPerHour(max)
    }

    fun clearAdjustmentOverrides() {
        autoAdjustEngine?.clearOverrides()
    }

    /**
     * Connect to a peer using provided addresses.
     */
    fun connectToPeer(peerId: String, addresses: List<String>) {
        addresses.forEach { addr ->
            try {
                // Only append /p2p/ component if peerId is a valid libp2p PeerId format
                // (base58btc multihash, starts with "12D3Koo" or "Qm").
                // Blake3 hex identity_ids (64 hex chars) are NOT valid libp2p PeerIds.
                val isLibp2pPeerId = peerId.startsWith("12D3Koo") || peerId.startsWith("Qm")
                val finalAddr = if (isLibp2pPeerId && !addr.contains("/p2p/")) {
                    "$addr/p2p/$peerId"
                } else {
                    addr
                }
                swarmBridge?.dial(finalAddr)
                Timber.d("Dialing $finalAddr")
            } catch (e: Exception) {
                Timber.e(e, "Failed to dial $addr")
            }
        }
    }

    // ========================================================================
    // IDENTITY EXPORT HELPERS
    // ========================================================================    // MARK: - Identity Helpers

    fun getPreferredRelay(): String? {
        val relays = ledgerManager?.getPreferredRelays(1u)
        return relays?.firstOrNull()?.peerId
    }

    fun getListeningAddresses(): List<String> {
        return swarmBridge?.getListeners() ?: emptyList()
    }

    fun getLocalIpAddress(): String? {
        try {
            val interfaces = java.net.NetworkInterface.getNetworkInterfaces()
            while (interfaces.hasMoreElements()) {
                val networkInterface = interfaces.nextElement()
                val addresses = networkInterface.inetAddresses
                while (addresses.hasMoreElements()) {
                    val address = addresses.nextElement()
                    if (!address.isLoopbackAddress && address is java.net.Inet4Address) {
                        return address.hostAddress
                    }
                }
            }
        } catch (e: Exception) {
            timber.log.Timber.e(e, "Failed to get local IP")
        }
        return null
    }

    // ========================================================================
    // OBSERVABLES FOR UI (NEW)
    // ========================================================================

    /**
     * Observe incoming messages from MeshEventBus.
     */
    fun observeIncomingMessages(): kotlinx.coroutines.flow.Flow<com.scmessenger.android.service.MessageEvent> {
        return com.scmessenger.android.service.MeshEventBus.messageEvents
    }

    /**
     * Observe peer events from MeshEventBus.
     */
    fun observePeers(): kotlinx.coroutines.flow.Flow<List<String>> {
        return kotlinx.coroutines.flow.flow {
            com.scmessenger.android.service.MeshEventBus.peerEvents.collect { event ->
                // Convert peer events to peer list
                kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
                    val peers = ledgerManager?.dialableAddresses()?.mapNotNull { it.peerId }?.distinct() ?: emptyList()
                    emit(peers)
                }
            }
        }
    }

    /**
     * Observe network stats with periodic refresh.
     */
    fun observeNetworkStats(): kotlinx.coroutines.flow.Flow<uniffi.api.ServiceStats> {
        return kotlinx.coroutines.flow.flow {
            while (true) {
                kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
                    val stats = meshService?.getStats()
                    if (stats != null) {
                        emit(stats)
                    }
                }
                kotlinx.coroutines.delay(2000) // Refresh every 2 seconds
            }
        }
    }

    // ========================================================================
    // CLEANUP
    // ========================================================================

    fun cleanup() {
        try {
            stopMeshService()
            saveLedger()

            // Clear references
            meshService = null
            contactManager = null
            historyManager = null
            ledgerManager = null
            settingsManager = null
            autoAdjustEngine = null

            Timber.i("MeshRepository cleaned up")
        } catch (e: Exception) {
            Timber.e(e, "Error during cleanup")
        }
    }
}
