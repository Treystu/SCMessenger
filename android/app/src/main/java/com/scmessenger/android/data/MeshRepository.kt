package com.scmessenger.android.data

import android.content.Context
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import timber.log.Timber
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

    // Incoming messages flow for notifications
    private val _incomingMessages = kotlinx.coroutines.flow.MutableSharedFlow<uniffi.api.MessageRecord>(replay = 0)
    val incomingMessages = _incomingMessages.asSharedFlow()

    private val repoScope = kotlinx.coroutines.CoroutineScope(kotlinx.coroutines.Dispatchers.IO + kotlinx.coroutines.SupervisorJob())

    // Core Delegate reference to prevent GC
    private var coreDelegate: uniffi.api.CoreDelegate? = null
    
    // BLE Components
    private var bleScanner: com.scmessenger.android.transport.ble.BleScanner? = null
    private var bleAdvertiser: com.scmessenger.android.transport.ble.BleAdvertiser? = null
    
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
            meshService?.start()
            
            // 2. Obtain Shared IronCore Instance (Singleton)
            ironCore = meshService?.getCore()
            if (ironCore == null) {
                Timber.w("IronCore instance is null after service start!")
            }
            
            // 3. Wire up CoreDelegate (Rust -> Android Events)
            coreDelegate = object : uniffi.api.CoreDelegate {
                override fun onPeerDiscovered(peerId: String) {
                    Timber.d("Core notified discovery: $peerId")
                }
                
                override fun onPeerDisconnected(peerId: String) {
                    Timber.d("Core notified disconnect: $peerId")
                }
                
                override fun onMessageReceived(senderId: String, messageId: String, data: ByteArray) {
                    Timber.i("Message from $senderId: $messageId")
                    try {
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
                        
                        // Emit for notifications
                        repoScope.launch {
                            _incomingMessages.emit(record)
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
        
        // BLE Scanner: Feeds discovered peers to MeshService
        if (bleScanner == null) {
            bleScanner = com.scmessenger.android.transport.ble.BleScanner(
                context,
                onPeerDiscovered = { peerId ->
                    meshService?.onPeerDiscovered(peerId)
                },
                onDataReceived = { peerId, data ->
                    meshService?.onDataReceived(peerId, data)
                }
            )
        }
        bleScanner?.startScanning()
        
        // BLE Advertiser: Broadcasts our presence
        if (bleAdvertiser == null) {
            bleAdvertiser = com.scmessenger.android.transport.ble.BleAdvertiser(context)
        }
        bleAdvertiser?.startAdvertising()
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
            wifiTransportManager = com.scmessenger.android.transport.WifiTransportManager(context) { peerId ->
                meshService?.onPeerDiscovered(peerId)
            }
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
            if (swarmBridge == null) {
                swarmBridge = uniffi.api.SwarmBridge()
            }
            // SwarmBridge starts automatically on creation or we might need to dial/listen
            // Add known peers from Ledger optionally
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize SwarmBridge")
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

    suspend fun sendMessage(peerId: String, content: String) {
        try {
            // 1. Get recipient's public key
            val contact = contactManager?.get(peerId)
                ?: throw IllegalStateException("Contact not found for peer: $peerId")
            
            val publicKey = contact.publicKey
            
            // 2. Encrypt/Prepare message
            val encryptedData = ironCore?.prepareMessage(publicKey, content)
                ?: throw IllegalStateException("Failed to prepare message: IronCore not initialized")
            
            // Convert List<Byte> (or whatever UniFFI returns) to ByteArray
            // UniFFI 'bytes' maps to ByteArray, so encryptedData is ByteArray.
            
            // 3. Send over network (Multiple transports)
            // Attempt BLE
            bleAdvertiser?.sendData(encryptedData)
            
            // Attempt WiFi
            wifiTransportManager?.sendData(peerId, encryptedData)

            // Attempt Swarm (Internet)
            try {
                swarmBridge?.sendMessage(peerId, encryptedData)
            } catch (e: Exception) {
                Timber.w("Failed to send via SwarmBridge: ${e.message}")
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
            
            Timber.i("Message sent (encrypted) to $peerId")
        } catch (e: Exception) {
            Timber.e(e, "Failed to send message")
            throw e
        }
    }

    suspend fun dial(multiaddr: String) {
        try {
            // Attempt Swarm Dial
            swarmBridge?.dial(multiaddr)
            Timber.i("Dialed $multiaddr via SwarmBridge")
        } catch (e: Exception) {
            Timber.e(e, "Failed to dial $multiaddr")
            throw e
        }
    }
    
    // Identity Management
    fun isIdentityInitialized(): Boolean {
        return ironCore?.getIdentityInfo()?.initialized == true
    }
    
    suspend fun createIdentity() {
        try {
            ironCore?.initializeIdentity()
            Timber.i("Identity created successfully")
        } catch (e: Exception) {
            Timber.e(e, "Failed to create identity")
            throw e
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
        return settingsManager?.load() ?: settingsManager?.defaultSettings() 
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
    
    fun overrideRelayMax(max: UInt) {
        autoAdjustEngine?.overrideRelayMaxPerHour(max)
    }
    
    fun clearAdjustmentOverrides() {
        autoAdjustEngine?.clearOverrides()
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
