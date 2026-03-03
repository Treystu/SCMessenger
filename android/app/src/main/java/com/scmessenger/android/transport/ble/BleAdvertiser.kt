package com.scmessenger.android.transport.ble

import android.annotation.SuppressLint
import android.Manifest
import android.content.pm.PackageManager
import android.bluetooth.BluetoothManager
import android.bluetooth.le.AdvertiseCallback
import android.bluetooth.le.AdvertiseData
import android.bluetooth.le.AdvertiseSettings
import android.content.Context
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.os.ParcelUuid
import androidx.core.content.ContextCompat
import timber.log.Timber

/**
 * Handles Bluetooth Low Energy advertising to announce presence to the mesh.
 *
 * Features:
 * - Rotation interval support from AutoAdjustEngine
 * - Proper service data encoding with peer identity
 * - Configurable advertise settings based on AutoAdjust profile
 * - Handles large payloads via GATT (delegates to GATT server)
 */
class BleAdvertiser(private val context: Context) {

    private val bluetoothManager = context.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager
    private val bluetoothAdapter = bluetoothManager.adapter
    private val advertiser = bluetoothAdapter?.bluetoothLeAdvertiser

    private var isAdvertising = false
    private var currentIdentityData: ByteArray? = null

    // Rotation management
    private var rotationIntervalMs: Long = 0L  // 0 = no rotation
    private val handler = Handler(Looper.getMainLooper())
    private var rotationRunnable: Runnable? = null

    // Advertise settings
    private var txPowerLevel = AdvertiseSettings.ADVERTISE_TX_POWER_MEDIUM
    private var advertiseMode = AdvertiseSettings.ADVERTISE_MODE_BALANCED

    private val advertiseCallback = object : AdvertiseCallback() {
        override fun onStartSuccess(settingsInEffect: AdvertiseSettings?) {
            Timber.i("BLE Advertising started successfully")
            isAdvertising = true
        }

        override fun onStartFailure(errorCode: Int) {
            Timber.e("BLE Advertising failed with error: $errorCode")
            isAdvertising = false
        }
    }

    /**
     * Set identity data to advertise (e.g., truncated peer ID or beacon).
     */
    fun setIdentityData(data: ByteArray) {
        val previous = currentIdentityData
        if (previous?.contentEquals(data) == true) {
            return
        }
        currentIdentityData = data
        Timber.d("Identity data set: ${data.size} bytes")

        // Refresh advertising payload only when service-data visibility can change.
        // Large identity payloads are served via GATT, so restarting advertising
        // for every GATT-only update creates unnecessary churn/disconnect noise.
        val previousWasAdvertisable = (previous?.size ?: 0) <= 24
        val currentIsAdvertisable = data.size <= 24
        val requiresAdvertiseRefresh = previousWasAdvertisable || currentIsAdvertisable

        if (isAdvertising && requiresAdvertiseRefresh) {
            stopAdvertising()
            startAdvertising()
        }
    }

    /**
     * Alias for setIdentityData to match naming conventions.
     */
    fun updateIdentityBeacon(data: ByteArray) {
        setIdentityData(data)
    }

    /**
     * Set rotation interval for identity beacon rotation.
     */
    fun setRotationInterval(intervalMs: Long) {
        rotationIntervalMs = intervalMs
        Timber.d("Rotation interval set: ${intervalMs}ms")

        if (isAdvertising && intervalMs > 0) {
            startRotation()
        }
    }

    /**
     * Apply advertise settings based on AutoAdjust profile.
     */
    fun applyAdvertiseSettings(intervalMs: UInt, txPowerDbm: Byte) {
        // Map interval to advertise mode
        advertiseMode = when {
            intervalMs < 500u -> AdvertiseSettings.ADVERTISE_MODE_LOW_LATENCY
            intervalMs < 1500u -> AdvertiseSettings.ADVERTISE_MODE_BALANCED
            else -> AdvertiseSettings.ADVERTISE_MODE_LOW_POWER
        }

        // Map tx power
        txPowerLevel = when {
            txPowerDbm >= 0 -> AdvertiseSettings.ADVERTISE_TX_POWER_HIGH
            txPowerDbm >= -10 -> AdvertiseSettings.ADVERTISE_TX_POWER_MEDIUM
            txPowerDbm >= -20 -> AdvertiseSettings.ADVERTISE_TX_POWER_LOW
            else -> AdvertiseSettings.ADVERTISE_TX_POWER_ULTRA_LOW
        }

        Timber.d("Advertise settings updated: mode=$advertiseMode, txPower=$txPowerLevel")

        // Restart advertising if active
        if (isAdvertising) {
            stopAdvertising()
            startAdvertising()
        }
    }

    @SuppressLint("MissingPermission")
    fun startAdvertising() {
        if (advertiser == null) {
            Timber.w("Bluetooth Advertiser not available")
            return
        }
        if (!hasAdvertisePermission()) {
            Timber.w("BLUETOOTH_ADVERTISE permission missing; cannot start BLE advertising")
            return
        }
        if (isAdvertising) return

        val settings = AdvertiseSettings.Builder()
            .setAdvertiseMode(advertiseMode)
            .setConnectable(true)
            .setTimeout(0)
            .setTxPowerLevel(txPowerLevel)
            .build()

        val dataBuilder = AdvertiseData.Builder()
            .setIncludeDeviceName(false)
            .addServiceUuid(ParcelUuid(BleScanner.SERVICE_UUID))

        // Add identity data if available
        currentIdentityData?.let { data ->
            if (data.size <= 24) {
                dataBuilder.addServiceData(ParcelUuid(BleScanner.SERVICE_UUID), data)
            } else {
                Timber.w("Identity data too large for advertising (${data.size} bytes), using GATT")
            }
        }

        val data = dataBuilder.build()

        try {
            advertiser.startAdvertising(settings, data, advertiseCallback)

            // Start rotation if configured
            if (rotationIntervalMs > 0) {
                startRotation()
            }
        } catch (e: SecurityException) {
            Timber.e(e, "Missing permission while starting BLE advertising")
        } catch (e: Exception) {
            Timber.e(e, "Failed to start BLE advertising")
        }
    }

    private fun startRotation() {
        // Cancel any existing rotation
        stopRotation()

        rotationRunnable = object : Runnable {
            override fun run() {
                if (isAdvertising && rotationRunnable != null) {
                    // Rotate: restart advertising without calling full stop/start cycle
                    // This prevents creating new rotation runnables
                    try {
                        if (!hasAdvertisePermission()) {
                            Timber.w("BLUETOOTH_ADVERTISE permission missing; skipping beacon rotation")
                            return
                        }
                        advertiser?.stopAdvertising(advertiseCallback)
                        isAdvertising = false

                        // Restart immediately
                        val settings = AdvertiseSettings.Builder()
                            .setAdvertiseMode(advertiseMode)
                            .setConnectable(true)
                            .setTimeout(0)
                            .setTxPowerLevel(txPowerLevel)
                            .build()

                        val dataBuilder = AdvertiseData.Builder()
                            .setIncludeDeviceName(false)
                            .addServiceUuid(ParcelUuid(BleScanner.SERVICE_UUID))

                        currentIdentityData?.let { data ->
                            if (data.size <= 24) {
                                dataBuilder.addServiceData(ParcelUuid(BleScanner.SERVICE_UUID), data)
                            }
                        }

                        advertiser?.startAdvertising(settings, dataBuilder.build(), advertiseCallback)
                        Timber.d("Beacon rotated")
                    } catch (e: SecurityException) {
                        Timber.e(e, "Missing permission while rotating beacon")
                    } catch (e: Exception) {
                        Timber.e(e, "Failed to rotate beacon")
                    }

                    // Schedule next rotation only if we're still the active runnable
                    if (rotationRunnable == this) {
                        handler.postDelayed(this, rotationIntervalMs)
                    }
                }
            }
        }

        val runnable = rotationRunnable ?: return
        handler.postDelayed(runnable, rotationIntervalMs)
        Timber.d("Beacon rotation started: ${rotationIntervalMs}ms interval")
    }

    private fun stopRotation() {
        rotationRunnable?.let { handler.removeCallbacks(it) }
        rotationRunnable = null
    }

    @SuppressLint("MissingPermission")
    fun stopAdvertising() {
        if (advertiser == null || !isAdvertising) return
        if (!hasAdvertisePermission()) {
            Timber.w("BLUETOOTH_ADVERTISE permission missing; cannot stop BLE advertising cleanly")
            isAdvertising = false
            stopRotation()
            return
        }

        try {
            advertiser.stopAdvertising(advertiseCallback)
            isAdvertising = false
            stopRotation()
            Timber.i("BLE Advertising stopped")
        } catch (e: SecurityException) {
            Timber.e(e, "Missing permission while stopping BLE advertising")
        } catch (e: Exception) {
            Timber.e(e, "Failed to stop BLE advertising")
        }
    }

    /**
     * Send data via advertising.
     * For large payloads, this should delegate to GATT server.
     */
    @SuppressLint("MissingPermission")
    fun sendData(data: ByteArray): Boolean {
        if (!hasAdvertisePermission()) {
            Timber.w("BLUETOOTH_ADVERTISE permission missing; cannot send data via BLE advertising")
            return false
        }
        if (data.size > 24) {
            Timber.w("BLE data too large for legacy advertising (${data.size} bytes). Use GATT server.")
            return false
        }

        // Update advertising data with payload
        val packerUuid = ParcelUuid(BleScanner.SERVICE_UUID)
        val advertiseData = AdvertiseData.Builder()
            .setIncludeDeviceName(false)
            .addServiceUuid(packerUuid)
            .addServiceData(packerUuid, data)
            .build()

        if (isAdvertising) {
             stopAdvertising()
        }

        // Restart with new data using configured settings
        try {
            val settings = AdvertiseSettings.Builder()
                .setAdvertiseMode(advertiseMode)
                .setConnectable(true)
                .setTxPowerLevel(txPowerLevel)
                .build()

            advertiser?.startAdvertising(settings, advertiseData, advertiseCallback)
            Timber.d("BLE Advertising updated with data payload")
            return true
        } catch (e: SecurityException) {
            Timber.e(e, "Missing permission while sending data via BLE advertising")
            return false
        } catch (e: Exception) {
             Timber.e(e, "Failed to send data via BLE advertising")
             return false
        }
    }

    private fun hasAdvertisePermission(): Boolean {
        return Build.VERSION.SDK_INT < Build.VERSION_CODES.S ||
            ContextCompat.checkSelfPermission(
                context,
                Manifest.permission.BLUETOOTH_ADVERTISE
            ) == PackageManager.PERMISSION_GRANTED
    }
}
