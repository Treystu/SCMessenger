package com.scmessenger.android.transport.ble

import android.annotation.SuppressLint
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothManager
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanFilter
import android.bluetooth.le.ScanResult
import android.bluetooth.le.ScanSettings
import android.content.Context
import android.os.ParcelUuid
import timber.log.Timber
import java.util.UUID

/**
 * Handles Bluetooth Low Energy scanning for mesh peers.
 * Scans for devices advertising the SCMessenger Service UUID (0xDF01).
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

    // SCMessenger Service UUID: 0xDF01
    // Full UUID: 0000DF01-0000-1000-8000-00805F9B34FB
    companion object {
        val SERVICE_UUID = UUID.fromString("0000DF01-0000-1000-8000-00805F9B34FB")
        val PARCEL_UUID = ParcelUuid(SERVICE_UUID)
    }

    private val scanCallback = object : ScanCallback() {
        override fun onScanResult(callbackType: Int, result: ScanResult?) {
            result?.let { scanResult ->
                val device = scanResult.device
                val rssi = scanResult.rssi
                val scanRecord = scanResult.scanRecord

                // Extract Service Data
                val serviceData = scanRecord?.getServiceData(PARCEL_UUID)
                
                // Use device address as default peer ID if not encoded in payload yet
                val peerId = device.address

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

        val settings = ScanSettings.Builder()
            .setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY) // High duty for discovery
            .build()

        try {
            scanner.startScan(filters, settings, scanCallback)
            isScanning = true
            Timber.i("BLE Scanning started")
        } catch (e: Exception) {
            Timber.e(e, "Failed to start BLE scan")
        }
    }

    @SuppressLint("MissingPermission")
    fun stopScanning() {
        if (scanner == null || !isScanning) return

        try {
            scanner.stopScan(scanCallback)
            isScanning = false
            Timber.i("BLE Scanning stopped")
        } catch (e: Exception) {
            Timber.e(e, "Failed to stop BLE scan")
        }
    }
}
