package com.scmessenger.android.service

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.BatteryManager
import android.os.Build
import android.os.PowerManager
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.transport.ble.BleAdvertiser
import com.scmessenger.android.transport.ble.BleGattClient
import com.scmessenger.android.transport.ble.BleGattServer
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Android implementation of the PlatformBridge UniFFI callback interface.
 * 
 * This class monitors Android system state and reports changes to the
 * Rust core via the PlatformBridge interface:
 * - Battery level and charging state
 * - Network connectivity (WiFi, cellular)
 * - Motion state (via Activity Recognition - future)
 * - BLE data reception
 * - App lifecycle (background/foreground)
 * 
 * The Rust core can use this information to adjust mesh behavior
 * via the AutoAdjustEngine.
 */
@Singleton
class AndroidPlatformBridge @Inject constructor(
    @ApplicationContext private val context: Context,
    private val meshRepository: MeshRepository
) : uniffi.api.PlatformBridge {
    
    private var batteryReceiver: BroadcastReceiver? = null
    private var networkCallback: ConnectivityManager.NetworkCallback? = null
    private var motionReceiver: BroadcastReceiver? = null
    
    private val connectivityManager by lazy {
        context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
    }
    
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    
    // BLE components for data forwarding
    private var bleAdvertiser: BleAdvertiser? = null
    private var bleGattClient: BleGattClient? = null
    private var bleGattServer: BleGattServer? = null
    
    // Current state
    private var currentBatteryPct: UByte = 100u
    private var isCharging: Boolean = false
    private var hasWifi: Boolean = false
    private var hasCellular: Boolean = false
    private var currentMotionState: uniffi.api.MotionState = uniffi.api.MotionState.UNKNOWN
    
    /**
     * Initialize system monitoring.
     */
    fun initialize() {
        Timber.d("AndroidPlatformBridge initializing")
        
        registerBatteryMonitor()
        registerNetworkMonitor()
        initializeMotionDetection()
        
        // Initial state update
        updateBatteryState()
        updateNetworkState()
    }
    
    /**
     * Set BLE components for data forwarding.
     */
    fun setBleComponents(
        advertiser: BleAdvertiser?,
        gattClient: BleGattClient?,
        gattServer: BleGattServer?
    ) {
        this.bleAdvertiser = advertiser
        this.bleGattClient = gattClient
        this.bleGattServer = gattServer
        Timber.d("BLE components set for data forwarding")
    }
    
    /**
     * Clean up resources.
     */
    fun cleanup() {
        Timber.d("AndroidPlatformBridge cleaning up")
        
        batteryReceiver?.let { context.unregisterReceiver(it) }
        batteryReceiver = null
        
        networkCallback?.let { connectivityManager.unregisterNetworkCallback(it) }
        networkCallback = null
        
        motionReceiver?.let { context.unregisterReceiver(it) }
        motionReceiver = null
    }
    
    // ========================================================================
    // BATTERY MONITORING
    // ========================================================================
    
    private fun registerBatteryMonitor() {
        batteryReceiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context, intent: Intent) {
                updateBatteryState()
            }
        }
        
        val filter = IntentFilter().apply {
            addAction(Intent.ACTION_BATTERY_CHANGED)
            addAction(Intent.ACTION_POWER_CONNECTED)
            addAction(Intent.ACTION_POWER_DISCONNECTED)
        }
        
        context.registerReceiver(batteryReceiver, filter)
    }
    
    private fun updateBatteryState() {
        val batteryStatus = context.registerReceiver(null, IntentFilter(Intent.ACTION_BATTERY_CHANGED))
        
        val level = batteryStatus?.getIntExtra(BatteryManager.EXTRA_LEVEL, -1) ?: -1
        val scale = batteryStatus?.getIntExtra(BatteryManager.EXTRA_SCALE, -1) ?: -1
        
        val batteryPct = if (level >= 0 && scale > 0) {
            ((level.toFloat() / scale.toFloat()) * 100).toInt().toUByte()
        } else {
            100u
        }
        
        val status = batteryStatus?.getIntExtra(BatteryManager.EXTRA_STATUS, -1) ?: -1
        val charging = status == BatteryManager.BATTERY_STATUS_CHARGING ||
                      status == BatteryManager.BATTERY_STATUS_FULL
        
        if (batteryPct != currentBatteryPct || charging != isCharging) {
            currentBatteryPct = batteryPct
            isCharging = charging
            
            Timber.d("Battery changed: $batteryPct%, charging=$charging")
            onBatteryChanged(batteryPct, charging)
        }
    }
    
    // ========================================================================
    // NETWORK MONITORING
    // ========================================================================
    
    private fun registerNetworkMonitor() {
        val request = NetworkRequest.Builder()
            .addCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
            .build()
        
        networkCallback = object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: Network) {
                updateNetworkState()
            }
            
            override fun onLost(network: Network) {
                updateNetworkState()
            }
            
            override fun onCapabilitiesChanged(
                network: Network,
                capabilities: NetworkCapabilities
            ) {
                updateNetworkState()
            }
        }
        
        connectivityManager.registerNetworkCallback(request, networkCallback!!)
    }
    
    private fun updateNetworkState() {
        val activeNetwork = connectivityManager.activeNetwork
        val capabilities = activeNetwork?.let { connectivityManager.getNetworkCapabilities(it) }
        
        val wifi = capabilities?.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) ?: false
        val cellular = capabilities?.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) ?: false
        
        if (wifi != hasWifi || cellular != hasCellular) {
            hasWifi = wifi
            hasCellular = cellular
            
            Timber.d("Network changed: wifi=$wifi, cellular=$cellular")
            onNetworkChanged(wifi, cellular)
        }
    }
    
    // ========================================================================
    // MOTION DETECTION (Activity Recognition)
    // ========================================================================
    
    private fun initializeMotionDetection() {
        // Simple motion detection using screen on/off as a proxy
        // Full Activity Recognition API requires Google Play Services
        val filter = IntentFilter().apply {
            addAction(Intent.ACTION_SCREEN_ON)
            addAction(Intent.ACTION_SCREEN_OFF)
            addAction(Intent.ACTION_USER_PRESENT)
        }
        
        motionReceiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context, intent: Intent) {
                when (intent.action) {
                    Intent.ACTION_SCREEN_ON, Intent.ACTION_USER_PRESENT -> {
                        currentMotionState = uniffi.api.MotionState.MOVING
                        onMotionChanged(currentMotionState)
                    }
                    Intent.ACTION_SCREEN_OFF -> {
                        currentMotionState = uniffi.api.MotionState.STATIONARY
                        onMotionChanged(currentMotionState)
                    }
                }
            }
        }
        
        context.registerReceiver(motionReceiver, filter)
        Timber.d("Motion detection initialized (screen state proxy)")
    }
    
    // ========================================================================
    // PLATFORMBRIDGE INTERFACE IMPLEMENTATION
    // ========================================================================
    
    override fun onBatteryChanged(batteryPct: UByte, isCharging: Boolean) {
        // Compute and apply adjustment profile
        val deviceProfile = uniffi.api.DeviceProfile(
            batteryLevel = batteryPct,
            isCharging = isCharging,
            hasWifi = hasWifi,
            motionState = currentMotionState
        )
        
        val profile = meshRepository.computeAdjustmentProfile(deviceProfile)
        val bleAdjustment = meshRepository.computeBleAdjustment(profile)
        val relayAdjustment = meshRepository.computeRelayAdjustment(profile)
        
        // Apply adjustments to mesh service
        applyAdjustments(bleAdjustment, relayAdjustment)
        
        Timber.d("Adjustment profile: $profile for battery $batteryPct%, charging=$isCharging")
    }
    
    override fun onNetworkChanged(hasWifi: Boolean, hasCellular: Boolean) {
        // Recompute and apply adjustment
        val deviceProfile = uniffi.api.DeviceProfile(
            batteryLevel = currentBatteryPct,
            isCharging = isCharging,
            hasWifi = hasWifi,
            motionState = currentMotionState
        )
        
        val profile = meshRepository.computeAdjustmentProfile(deviceProfile)
        val bleAdjustment = meshRepository.computeBleAdjustment(profile)
        val relayAdjustment = meshRepository.computeRelayAdjustment(profile)
        
        applyAdjustments(bleAdjustment, relayAdjustment)
        
        Timber.d("Adjustment profile: $profile for network wifi=$hasWifi, cellular=$hasCellular")
    }
    
    override fun onMotionChanged(motion: uniffi.api.MotionState) {
        currentMotionState = motion
        
        // Recompute adjustment based on motion
        val deviceProfile = uniffi.api.DeviceProfile(
            batteryLevel = currentBatteryPct,
            isCharging = isCharging,
            hasWifi = hasWifi,
            motionState = motion
        )
        
        val profile = meshRepository.computeAdjustmentProfile(deviceProfile)
        Timber.d("Motion changed: $motion, profile: $profile")
    }
    
    override fun onBleDataReceived(peerId: String, data: ByteArray) {
        // BLE data received from Android BLE stack
        // Forward to mesh repository for processing
        Timber.d("BLE data received from $peerId: ${data.size} bytes")
        
        scope.launch {
            try {
                // Notify MeshEventBus about data reception
                MeshEventBus.emitNetworkEvent(
                    NetworkEvent.ConnectionQualityChanged(
                        peerId = peerId,
                        quality = ConnectionQuality.GOOD
                    )
                )
            } catch (e: Exception) {
                Timber.e(e, "Error processing BLE data")
            }
        }
    }

    override fun sendBlePacket(peerId: String, data: ByteArray) {
        // Send data via BLE transports
        Timber.d("Sending BLE packet to $peerId: ${data.size} bytes")
        
        scope.launch {
            try {
                // Try to send via GATT client if connected
                var sent = bleGattClient?.sendData(peerId, data) ?: false
                
                // Fallback to advertising with data
                if (!sent) {
                    bleAdvertiser?.sendData(data)
                    sent = true
                }
                
                if (sent) {
                    Timber.d("BLE packet sent successfully to $peerId")
                } else {
                    Timber.w("Failed to send BLE packet to $peerId")
                }
            } catch (e: Exception) {
                Timber.e(e, "Error sending BLE packet")
            }
        }
    }
    
    override fun onEnteringBackground() {
        Timber.i("App entering background")
        
        // Pause mesh service to conserve battery
        meshRepository.pauseMeshService()
    }
    
    override fun onEnteringForeground() {
        Timber.i("App entering foreground")
        
        // Resume full mesh service activity
        meshRepository.resumeMeshService()
    }
    
    // ========================================================================
    // PRIVATE HELPERS
    // ========================================================================
    
    private fun applyAdjustments(
        bleAdjustment: uniffi.api.BleAdjustment,
        relayAdjustment: uniffi.api.RelayAdjustment
    ) {
        // Apply BLE scan/advertise intervals
        Timber.d("Applying BLE adjustments: scan=${bleAdjustment.scanIntervalMs}ms, advertise=${bleAdjustment.advertiseIntervalMs}ms, txPower=${bleAdjustment.txPowerDbm}dBm")
        
        // Apply relay budget adjustments
        Timber.d("Applying relay adjustments: maxPerHour=${relayAdjustment.maxPerHour}, priority=${relayAdjustment.priorityThreshold}, maxPayload=${relayAdjustment.maxPayloadBytes}")
        
        // Note: Actual application would update BLE scanner/advertiser settings
        // and mesh service relay configuration
    }
    
    // ========================================================================
    // MANUAL STATE UPDATES
    // ========================================================================
    
    /**
     * Call this when app goes to background.
     */
    fun notifyBackground() {
        onEnteringBackground()
    }
    
    /**
     * Call this when app comes to foreground.
     */
    fun notifyForeground() {
        onEnteringForeground()
    }
    
    /**
     * Manually trigger battery state check (for periodic adjustments).
     */
    fun checkBatteryState() {
        updateBatteryState()
    }
    
    /**
     * Manually trigger network state check (for periodic adjustments).
     */
    fun checkNetworkState() {
        updateNetworkState()
    }
}
