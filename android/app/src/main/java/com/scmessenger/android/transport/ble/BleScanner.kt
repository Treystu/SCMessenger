package com.scmessenger.android.transport.ble

import android.annotation.SuppressLint
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothManager
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanFilter
import android.bluetooth.le.ScanResult
import android.bluetooth.le.ScanSettings
import android.content.Context
import android.os.Handler
import android.os.Looper
import android.os.ParcelUuid
import timber.log.Timber
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap

/**
 * Handles Bluetooth Low Energy scanning for mesh peers.
 *
 * Features:
 * - Duty-cycle management (scan window/interval configurable)
 * - Background vs foreground scan mode switching
 * - Scan result caching to avoid duplicate processing
 * - Configurable scan settings based on AutoAdjustEngine profile
 * - Scans for devices advertising the SCMessenger Service UUID (0xDF01)
 */
class BleScanner(
    private val context: Context,
    private val onPeerDiscovered: (String) -> Unit,
    private val onDataReceived: (String, ByteArray) -> Unit
) {

    private val bluetoothManager = context.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager
    private val bluetoothAdapter = bluetoothManager.adapter
    private val scanner = bluetoothAdapter?.bluetoothLeScanner

    private var isScanning = false
    private var isBackgroundMode = false

    // Duty cycle management
    private var scanWindowMs: Long = 10000L  // 10 seconds
    private var scanIntervalMs: Long = 30000L  // 30 seconds
    private val handler = Handler(Looper.getMainLooper())
    private var dutyCycleRunnable: Runnable? = null

    // Scan result caching to avoid duplicate processing
    private val recentlySeenPeers = ConcurrentHashMap<String, Long>()
    private val peerCacheTimeoutMs = 5000L  // 5 seconds

    // SCMessenger Service UUID: 0xDF01
    // Full UUID: 0000DF01-0000-1000-8000-00805F9B34FB
    companion object {
        val SERVICE_UUID = UUID.fromString("0000DF01-0000-1000-8000-00805F9B34FB")
        val PARCEL_UUID = ParcelUuid(SERVICE_UUID)

        // Scan modes
        // Foreground: continuous scan (window == interval) — no off-window dead time.
        // Android 7+ enforces a scan-restart quota (5 starts in 30s); keeping the
        // scanner running continuously avoids the quota and maximises discovery speed.
        const val DEFAULT_SCAN_WINDOW_MS = 10000L
        const val DEFAULT_SCAN_INTERVAL_MS = 30000L
        const val FOREGROUND_SCAN_WINDOW_MS = 30000L    // continuous: window == interval
        const val FOREGROUND_SCAN_INTERVAL_MS = 30000L  // no pause in foreground
        const val BACKGROUND_SCAN_WINDOW_MS = 5000L
        const val BACKGROUND_SCAN_INTERVAL_MS = 60000L
    }

    private val scanCallback = object : ScanCallback() {
        override fun onScanResult(callbackType: Int, result: ScanResult?) {
            result?.let { scanResult ->
                val device = scanResult.device
                val peerId = device.address

                // Check if we've recently seen this peer
                val now = System.currentTimeMillis()
                val lastSeen = recentlySeenPeers[peerId]
                if (lastSeen != null && (now - lastSeen) < peerCacheTimeoutMs) {
                    // Skip - we've processed this peer recently
                    return
                }

                // Update cache
                recentlySeenPeers[peerId] = now

                // Prune old entries
                pruneOldPeers(now)

                val rssi = scanResult.rssi
                val scanRecord = scanResult.scanRecord

                // Extract Service Data
                val serviceData = scanRecord?.getServiceData(PARCEL_UUID)

                if (serviceData != null) {
                    Timber.v("Discovered peer: $peerId (RSSI: $rssi, Data: ${serviceData.size} bytes)")
                    // Notify discovery
                    onPeerDiscovered(peerId)
                    // Notify data reception
                    onDataReceived(peerId, serviceData)
                } else {
                    // Just discovery (legacy or beacon)
                    Timber.v("Discovered peer (no data): $peerId (RSSI: $rssi)")
                    onPeerDiscovered(peerId)
                }
            }
        }

        override fun onScanFailed(errorCode: Int) {
            Timber.e("BLE Scan failed with error code: $errorCode")
            isScanning = false
        }
    }

    /**
     * Set scan duty cycle parameters.
     */
    fun setScanDutyCycle(windowMs: Long, intervalMs: Long) {
        scanWindowMs = windowMs
        scanIntervalMs = intervalMs
        Timber.d("Scan duty cycle updated: window=${windowMs}ms, interval=${intervalMs}ms")

        // Restart scanning if active
        if (isScanning) {
            stopScanning()
            startScanning()
        }
    }

    /**
     * Switch to background scan mode (lower duty cycle).
     */
    fun setBackgroundMode(background: Boolean) {
        if (isBackgroundMode == background) return

        isBackgroundMode = background

        if (background) {
            setScanDutyCycle(BACKGROUND_SCAN_WINDOW_MS, BACKGROUND_SCAN_INTERVAL_MS)
        } else {
            setScanDutyCycle(FOREGROUND_SCAN_WINDOW_MS, FOREGROUND_SCAN_INTERVAL_MS)
        }

        Timber.i("Scan mode changed: background=$background")
    }

    /**
     * Update scan settings based on AutoAdjust profile.
     */
    fun applyScanSettings(scanIntervalMs: UInt) {
        // Convert AutoAdjust interval to duty cycle
        val window = minOf(scanIntervalMs.toLong(), 20000L)
        val interval = maxOf(scanIntervalMs.toLong(), window + 5000L)

        setScanDutyCycle(window, interval)
    }

    @SuppressLint("MissingPermission")
    fun startScanning() {
        if (scanner == null) {
            Timber.w("Bluetooth Scanner not available")
            return
        }
        if (isScanning) return

        val filters = listOf(
            ScanFilter.Builder()
                .setServiceUuid(ParcelUuid(SERVICE_UUID))
                .build()
        )

        val scanMode = if (isBackgroundMode) {
            ScanSettings.SCAN_MODE_LOW_POWER
        } else {
            ScanSettings.SCAN_MODE_LOW_LATENCY
        }

        val settings = ScanSettings.Builder()
            .setScanMode(scanMode)
            // MATCH_MODE_AGGRESSIVE: trigger reports for even weak signals.
            // CALLBACK_TYPE_ALL_MATCHES: report all matching adverts, not just first.
            // numOfMatches(1): trigger callback on first match per scan cycle for
            // lowest latency peer detection when any peer is nearby.
            .setMatchMode(ScanSettings.MATCH_MODE_AGGRESSIVE)
            .setCallbackType(ScanSettings.CALLBACK_TYPE_ALL_MATCHES)
            .setNumOfMatches(ScanSettings.MATCH_NUM_ONE_ADVERTISEMENT)
            .build()

        try {
            scanner.startScan(filters, settings, scanCallback)
            isScanning = true
            Timber.i("BLE Scanning started (background=$isBackgroundMode)")

            // Start duty cycle if intervals are configured
            if (scanWindowMs < scanIntervalMs) {
                startDutyCycle()
            }
        } catch (e: Exception) {
            Timber.e(e, "Failed to start BLE scan")
        }
    }

    private fun startDutyCycle() {
        // Cancel any existing duty cycle
        stopDutyCycle()

        dutyCycleRunnable = object : Runnable {
            override fun run() {
                if (isScanning) {
                    // Stop scanning for the rest of the interval
                    stopScanningInternal()

                    // Schedule restart after pause
                    handler.postDelayed({
                        if (isScanning) {
                            startScanningInternal()
                        }
                    }, scanIntervalMs - scanWindowMs)
                }

                // Schedule next cycle
                if (isScanning) {
                    handler.postDelayed(this, scanIntervalMs)
                }
            }
        }

        // Start first cycle after scan window
        handler.postDelayed(dutyCycleRunnable!!, scanWindowMs)
        Timber.d("Duty cycle started: ${scanWindowMs}ms scan / ${scanIntervalMs}ms interval")
    }

    private fun stopDutyCycle() {
        dutyCycleRunnable?.let { handler.removeCallbacks(it) }
        dutyCycleRunnable = null
    }

    @SuppressLint("MissingPermission")
    private fun startScanningInternal() {
        if (scanner == null) return

        try {
            val filters = listOf(
                ScanFilter.Builder()
                    .setServiceUuid(ParcelUuid(SERVICE_UUID))
                    .build()
            )

            val scanMode = if (isBackgroundMode) {
                ScanSettings.SCAN_MODE_LOW_POWER
            } else {
                ScanSettings.SCAN_MODE_LOW_LATENCY
            }

            val settings = ScanSettings.Builder()
                .setScanMode(scanMode)
                .setMatchMode(ScanSettings.MATCH_MODE_AGGRESSIVE)
                .setCallbackType(ScanSettings.CALLBACK_TYPE_ALL_MATCHES)
                .setNumOfMatches(ScanSettings.MATCH_NUM_ONE_ADVERTISEMENT)
                .build()

            scanner.startScan(filters, settings, scanCallback)
            Timber.v("BLE scan window started")
        } catch (e: Exception) {
            Timber.e(e, "Failed to restart BLE scan")
        }
    }

    @SuppressLint("MissingPermission")
    private fun stopScanningInternal() {
        if (scanner == null) return

        try {
            scanner.stopScan(scanCallback)
            Timber.v("BLE scan window ended")
        } catch (e: Exception) {
            Timber.e(e, "Failed to stop BLE scan window")
        }
    }

    @SuppressLint("MissingPermission")
    fun stopScanning() {
        if (scanner == null || !isScanning) return

        stopDutyCycle()

        try {
            scanner.stopScan(scanCallback)
            isScanning = false
            Timber.i("BLE Scanning stopped")
        } catch (e: Exception) {
            Timber.e(e, "Failed to stop BLE scan")
        }
    }

    /**
     * Clear the peer cache to allow re-discovery.
     */
    fun clearPeerCache() {
        recentlySeenPeers.clear()
        Timber.d("Peer cache cleared")
    }

    /**
     * Prune old entries from peer cache.
     */
    private fun pruneOldPeers(currentTimeMs: Long) {
        val iterator = recentlySeenPeers.entries.iterator()
        while (iterator.hasNext()) {
            val entry = iterator.next()
            if ((currentTimeMs - entry.value) > peerCacheTimeoutMs) {
                iterator.remove()
            }
        }
    }
}
