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
import com.scmessenger.android.data.MeshRepository
import dagger.hilt.android.qualifiers.ApplicationContext
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
    
    private val connectivityManager by lazy {
        context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
    }
    
    // Current state
    private var currentBatteryPct: UByte = 100u
    private var isCharging: Boolean = false
    private var hasWifi: Boolean = false
    private var hasCellular: Boolean = false
    
    /**
     * Initialize system monitoring.
     */
    fun initialize() {
        Timber.d("AndroidPlatformBridge initializing")
        
        registerBatteryMonitor()
        registerNetworkMonitor()
        
        // Initial state update
        updateBatteryState()
        updateNetworkState()
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
    // PLATFORMBRIDGE INTERFACE IMPLEMENTATION
    // ========================================================================
    
    override fun onBatteryChanged(batteryPct: UByte, isCharging: Boolean) {
        // Compute adjustment profile based on current device state
        val deviceProfile = uniffi.api.DeviceProfile(
            batteryPct = batteryPct,
            isCharging = isCharging,
            hasWifi = hasWifi,
            hasCellular = hasCellular,
            motion = uniffi.api.MotionState.UNKNOWN,
            isBackground = false  // We'll update this from activity lifecycle
        )
        
        val profile = meshRepository.computeAdjustmentProfile(deviceProfile)
        
        // TODO: Apply adjustments to running mesh service
        Timber.d("Adjustment profile: $profile for battery $batteryPct%, charging=$isCharging")
    }
    
    override fun onNetworkChanged(hasWifi: Boolean, hasCellular: Boolean) {
        // Recompute adjustment based on new network state
        val deviceProfile = uniffi.api.DeviceProfile(
            batteryPct = currentBatteryPct,
            isCharging = isCharging,
            hasWifi = hasWifi,
            hasCellular = hasCellular,
            motion = uniffi.api.MotionState.UNKNOWN,
            isBackground = false
        )
        
        val profile = meshRepository.computeAdjustmentProfile(deviceProfile)
        
        // TODO: Apply adjustments to running mesh service
        Timber.d("Adjustment profile: $profile for network wifi=$hasWifi, cellular=$hasCellular")
    }
    
    override fun onMotionChanged(motion: uniffi.api.MotionState) {
        // TODO: Integrate with Activity Recognition API
        Timber.d("Motion changed: $motion")
    }
    
    override fun onBleDataReceived(peerId: String, data: List<UByte>) {
        // BLE data is typically handled by the transport layer in Rust
        // This callback is for Android-specific BLE discovery
        Timber.d("BLE data received from $peerId: ${data.size} bytes")
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
}
