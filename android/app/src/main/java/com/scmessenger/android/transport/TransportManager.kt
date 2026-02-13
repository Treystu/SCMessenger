package com.scmessenger.android.transport

import android.content.Context
import com.scmessenger.android.service.TransportType
import com.scmessenger.android.transport.ble.*
import timber.log.Timber
import java.util.concurrent.ConcurrentHashMap
import kotlinx.coroutines.*

/**
 * Orchestrates all local transports for SCMessenger.
 * 
 * Responsibilities:
 * - Manages BLE, WiFi Aware, and WiFi Direct transports
 * - Implements transport priority: WiFi Aware > WiFi Direct > BLE
 * - Auto-escalation when BLE discovers a peer (try higher-throughput transports)
 * - Reports available transports to Rust AutoAdjustEngine
 * - Coordinates peer discovery across all transports
 * 
 * Follows the escalation logic from core/src/transport/escalation.rs:
 * 1. BLE for initial discovery (low power, always available)
 * 2. WiFi Aware for medium-range, high-throughput (if available)
 * 3. WiFi Direct for established connections (if available)
 * 4. Internet/libp2p for long-range (handled by SwarmBridge)
 */
class TransportManager(
    private val context: Context,
    private val onPeerDiscovered: (peerId: String, transport: TransportType) -> Unit,
    private val onDataReceived: (peerId: String, data: ByteArray, transport: TransportType) -> Unit
) {
    
    // BLE components
    private var bleScanner: BleScanner? = null
    private var bleAdvertiser: BleAdvertiser? = null
    private var bleGattServer: BleGattServer? = null
    private var bleGattClient: BleGattClient? = null
    private var bleL2capManager: BleL2capManager? = null
    
    // WiFi components
    private var wifiAware: WifiAwareTransport? = null
    private var wifiDirect: WifiDirectTransport? = null
    
    // Track active transports
    private val activeTransports = ConcurrentHashMap<TransportType, Boolean>()
    
    // Track which transport to use for each peer (highest priority available)
    private val peerTransports = ConcurrentHashMap<String, TransportType>()
    
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    
    private var isRunning = false
    
    /**
     * Initialize all transports based on settings.
     */
    fun initialize(
        bleEnabled: Boolean = true,
        wifiAwareEnabled: Boolean = true,
        wifiDirectEnabled: Boolean = true
    ) {
        Timber.i("Initializing TransportManager (BLE=$bleEnabled, Aware=$wifiAwareEnabled, Direct=$wifiDirectEnabled)")
        
        // Initialize BLE components
        if (bleEnabled) {
            initializeBle()
        }
        
        // Initialize WiFi Aware
        if (wifiAwareEnabled) {
            initializeWifiAware()
        }
        
        // Initialize WiFi Direct
        if (wifiDirectEnabled) {
            initializeWifiDirect()
        }
    }
    
    /**
     * Start all enabled transports.
     */
    fun startAll() {
        if (isRunning) {
            Timber.w("TransportManager already running")
            return
        }
        
        isRunning = true
        
        // Start BLE
        bleScanner?.startScanning()
        bleAdvertiser?.startAdvertising()
        bleGattServer?.start()
        bleL2capManager?.startListening()
        
        // Start WiFi Aware
        wifiAware?.start()
        
        // Start WiFi Direct
        wifiDirect?.start()
        
        Timber.i("All transports started")
    }
    
    /**
     * Stop all transports.
     */
    fun stopAll() {
        if (!isRunning) {
            return
        }
        
        isRunning = false
        
        // Stop BLE
        bleScanner?.stopScanning()
        bleAdvertiser?.stopAdvertising()
        bleGattServer?.stop()
        bleL2capManager?.stopListening()
        
        // Stop WiFi
        wifiAware?.stop()
        wifiDirect?.stop()
        
        activeTransports.clear()
        peerTransports.clear()
        
        Timber.i("All transports stopped")
    }
    
    /**
     * Send data to a peer using the best available transport.
     */
    fun sendData(peerId: String, data: ByteArray): Boolean {
        // Use cached transport if available
        val preferredTransport = peerTransports[peerId]
        
        if (preferredTransport != null) {
            val success = sendViaTransport(peerId, data, preferredTransport)
            if (success) return true
        }
        
        // Try transports in priority order: WiFi Aware > WiFi Direct > BLE
        if (activeTransports[TransportType.WIFI_AWARE] == true) {
            if (sendViaTransport(peerId, data, TransportType.WIFI_AWARE)) {
                peerTransports[peerId] = TransportType.WIFI_AWARE
                return true
            }
        }
        
        if (activeTransports[TransportType.WIFI_DIRECT] == true) {
            if (sendViaTransport(peerId, data, TransportType.WIFI_DIRECT)) {
                peerTransports[peerId] = TransportType.WIFI_DIRECT
                return true
            }
        }
        
        if (activeTransports[TransportType.BLE] == true) {
            if (sendViaTransport(peerId, data, TransportType.BLE)) {
                peerTransports[peerId] = TransportType.BLE
                return true
            }
        }
        
        Timber.w("Failed to send data to $peerId via any transport")
        return false
    }
    
    private fun sendViaTransport(peerId: String, data: ByteArray, transport: TransportType): Boolean {
        return when (transport) {
            TransportType.BLE -> {
                // Try L2CAP first, then GATT client, then advertiser
                // Each returns Boolean, so we evaluate success not just nullability
                bleL2capManager?.sendData(peerId, data)?.takeIf { it }
                    ?: bleGattClient?.sendData(peerId, data)?.takeIf { it }
                    ?: bleAdvertiser?.sendData(data)?.takeIf { it }
                    ?: false
            }
            TransportType.WIFI_AWARE -> {
                wifiAware?.sendData(peerId, data) ?: false
            }
            TransportType.WIFI_DIRECT -> {
                wifiDirect?.sendData(peerId, data) ?: false
            }
            TransportType.INTERNET -> {
                // Handled by SwarmBridge, not here
                false
            }
        }
    }
    
    /**
     * Attempt to escalate a peer connection to a higher-throughput transport.
     * Called when a peer is discovered on BLE.
     */
    fun attemptEscalation(peerId: String) {
        scope.launch {
            Timber.d("Attempting transport escalation for $peerId")
            
            // Try WiFi Aware first
            if (wifiAware?.isAvailable() == true) {
                Timber.d("Escalating $peerId to WiFi Aware")
                // WiFi Aware discovery is automatic, just mark as preferred
                activeTransports[TransportType.WIFI_AWARE] = true
            }
            
            // Try WiFi Direct as fallback
            if (activeTransports[TransportType.WIFI_DIRECT] != true) {
                Timber.d("Escalating $peerId to WiFi Direct")
                // WiFi Direct will auto-connect when service is discovered
                activeTransports[TransportType.WIFI_DIRECT] = true
            }
        }
    }
    
    /**
     * Get list of currently active transports.
     */
    fun getActiveTransports(): List<TransportType> {
        return activeTransports.filter { it.value }.keys.toList()
    }
    
    /**
     * Get available transports for AutoAdjustEngine.
     */
    fun getAvailableTransports(): List<String> {
        val available = mutableListOf<String>()
        
        if (bleScanner != null || bleAdvertiser != null) {
            available.add("BLE")
        }
        
        if (wifiAware?.isAvailable() == true) {
            available.add("WiFiAware")
        }
        
        if (wifiDirect != null) {
            available.add("WiFiDirect")
        }
        
        return available
    }
    
    private fun initializeBle() {
        try {
            // Scanner
            bleScanner = BleScanner(
                context,
                onPeerDiscovered = { peerId ->
                    Timber.d("BLE peer discovered: $peerId")
                    activeTransports[TransportType.BLE] = true
                    onPeerDiscovered(peerId, TransportType.BLE)
                    
                    // Attempt escalation to higher transports
                    attemptEscalation(peerId)
                },
                onDataReceived = { peerId, data ->
                    onDataReceived(peerId, data, TransportType.BLE)
                }
            )
            
            // Advertiser
            bleAdvertiser = BleAdvertiser(context)
            
            // GATT Server
            bleGattServer = BleGattServer(context) { peerId, data ->
                onDataReceived(peerId, data, TransportType.BLE)
            }
            
            // GATT Client
            bleGattClient = BleGattClient(
                context,
                onIdentityReceived = { address, identity ->
                    Timber.d("Identity received from $address: ${identity.size} bytes")
                },
                onDataReceived = { address, data ->
                    onDataReceived(address, data, TransportType.BLE)
                }
            )
            
            // L2CAP Manager (Android 10+)
            bleL2capManager = BleL2capManager(context) { peerId, data ->
                onDataReceived(peerId, data, TransportType.BLE)
            }
            
            Timber.d("BLE transports initialized")
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize BLE transports")
        }
    }
    
    private fun initializeWifiAware() {
        try {
            wifiAware = WifiAwareTransport(
                context,
                onPeerDiscovered = { peerId ->
                    Timber.d("WiFi Aware peer discovered: $peerId")
                    activeTransports[TransportType.WIFI_AWARE] = true
                    peerTransports[peerId] = TransportType.WIFI_AWARE
                    onPeerDiscovered(peerId, TransportType.WIFI_AWARE)
                },
                onDataReceived = { peerId, data ->
                    onDataReceived(peerId, data, TransportType.WIFI_AWARE)
                }
            )
            
            if (wifiAware?.isAvailable() == true) {
                Timber.d("WiFi Aware initialized and available")
            } else {
                Timber.d("WiFi Aware not available on this device")
                wifiAware = null
            }
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize WiFi Aware")
            wifiAware = null
        }
    }
    
    private fun initializeWifiDirect() {
        try {
            wifiDirect = WifiDirectTransport(
                context,
                onPeerDiscovered = { peerId ->
                    Timber.d("WiFi Direct peer discovered: $peerId")
                    activeTransports[TransportType.WIFI_DIRECT] = true
                    peerTransports[peerId] = TransportType.WIFI_DIRECT
                    onPeerDiscovered(peerId, TransportType.WIFI_DIRECT)
                },
                onDataReceived = { peerId, data ->
                    onDataReceived(peerId, data, TransportType.WIFI_DIRECT)
                }
            )
            
            Timber.d("WiFi Direct initialized")
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize WiFi Direct")
            wifiDirect = null
        }
    }
    
    /**
     * Enable a specific transport at runtime.
     */
    fun enableTransport(transport: TransportType) {
        when (transport) {
            TransportType.BLE -> {
                bleScanner?.startScanning()
                bleAdvertiser?.startAdvertising()
            }
            TransportType.WIFI_AWARE -> {
                wifiAware?.start()
            }
            TransportType.WIFI_DIRECT -> {
                wifiDirect?.start()
            }
            TransportType.INTERNET -> {
                // Handled separately by SwarmBridge
            }
        }
    }
    
    /**
     * Disable a specific transport at runtime.
     */
    fun disableTransport(transport: TransportType) {
        when (transport) {
            TransportType.BLE -> {
                bleScanner?.stopScanning()
                bleAdvertiser?.stopAdvertising()
            }
            TransportType.WIFI_AWARE -> {
                wifiAware?.stop()
            }
            TransportType.WIFI_DIRECT -> {
                wifiDirect?.stop()
            }
            TransportType.INTERNET -> {
                // Handled separately by SwarmBridge
            }
        }
        
        activeTransports.remove(transport)
    }
    
    /**
     * Cleanup all resources.
     */
    fun cleanup() {
        stopAll()
        
        bleGattClient?.cleanup()
        bleL2capManager?.shutdown()
        wifiAware?.cleanup()
        wifiDirect?.cleanup()
        
        scope.cancel()
        
        Timber.i("TransportManager cleaned up")
    }
}
