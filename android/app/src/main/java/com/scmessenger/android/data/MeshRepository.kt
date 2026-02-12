package com.scmessenger.android.data

import android.content.Context
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
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
    
    // Service state
    private val _serviceState = MutableStateFlow(uniffi.api.ServiceState.STOPPED)
    val serviceState: StateFlow<uniffi.api.ServiceState> = _serviceState.asStateFlow()
    
    // Service stats
    private val _serviceStats = MutableStateFlow<uniffi.api.ServiceStats?>(null)
    val serviceStats: StateFlow<uniffi.api.ServiceStats?> = _serviceStats.asStateFlow()
    
    init {
        Timber.d("MeshRepository initialized with storage: $storagePath")
        initializeManagers()
    }
    
    private fun initializeManagers() {
        try {
            // Initialize settings manager
            settingsManager = uniffi.api.MeshSettingsManager(storagePath)
            
            // Initialize contact manager
            contactManager = uniffi.api.ContactManager(storagePath)
            
            // Initialize history manager
            historyManager = uniffi.api.HistoryManager(storagePath)
            
            // Initialize ledger manager
            ledgerManager = uniffi.api.LedgerManager(storagePath)
            
            // Load ledger
            ledgerManager?.load()
            
            // Initialize auto-adjust engine
            autoAdjustEngine = uniffi.api.AutoAdjustEngine()
            
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
     */
    fun startMeshService(config: uniffi.api.MeshServiceConfig) {
        try {
            if (meshService == null) {
                meshService = uniffi.api.MeshService(config)
            }
            
            meshService?.start()
            _serviceState.value = uniffi.api.ServiceState.RUNNING
            updateStats()
            
            Timber.i("Mesh service started")
        } catch (e: Exception) {
            Timber.e(e, "Failed to start mesh service")
            throw e
        }
    }
    
    /**
     * Stop the mesh service.
     */
    fun stopMeshService() {
        try {
            meshService?.stop()
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
