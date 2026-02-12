package com.scmessenger.android.transport.ble

import android.annotation.SuppressLint
import android.bluetooth.BluetoothManager
import android.bluetooth.le.AdvertiseCallback
import android.bluetooth.le.AdvertiseData
import android.bluetooth.le.AdvertiseSettings
import android.content.Context
import android.os.ParcelUuid
import timber.log.Timber

/**
 * Handles Bluetooth Low Energy advertising to announce presence to the mesh.
 */
class BleAdvertiser(private val context: Context) {

    private val bluetoothManager = context.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager
    private val bluetoothAdapter = bluetoothManager.adapter
    private val advertiser = bluetoothAdapter?.bluetoothLeAdvertiser

    private var isAdvertising = false

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

    @SuppressLint("MissingPermission")
    fun startAdvertising() {
        if (advertiser == null) {
            Timber.w("Bluetooth Advertiser not available")
            return
        }
        if (isAdvertising) return

        val settings = AdvertiseSettings.Builder()
            .setAdvertiseMode(AdvertiseSettings.ADVERTISE_MODE_BALANCED)
            .setConnectable(true)
            .setTimeout(0)
            .setTxPowerLevel(AdvertiseSettings.ADVERTISE_TX_POWER_MEDIUM)
            .build()

        val data = AdvertiseData.Builder()
            .setIncludeDeviceName(false)
            .addServiceUuid(ParcelUuid(BleScanner.SERVICE_UUID))
            // We could add Service Data here (e.g. simplified Peer ID)
            // .addServiceData(...)
            .build()

        try {
            advertiser.startAdvertising(settings, data, advertiseCallback)
        } catch (e: Exception) {
            Timber.e(e, "Failed to start BLE advertising")
        }
    }

    @SuppressLint("MissingPermission")
    fun stopAdvertising() {
        if (advertiser == null || !isAdvertising) return

        try {
            advertiser.stopAdvertising(advertiseCallback)
            isAdvertising = false
            Timber.i("BLE Advertising stopped")
        } catch (e: Exception) {
            Timber.e(e, "Failed to stop BLE advertising")
        }
    }
    @SuppressLint("MissingPermission")
    fun sendData(data: ByteArray) {
        if (data.size > 24) {
            Timber.w("BLE data too large for legacy advertising (${data.size} bytes). Truncating or ignoring.")
            // Real implementation requires GATT or Extended Advertising
        }
        
        // Update advertising data with payload
        val packerUuid = ParcelUuid(BleScanner.SERVICE_UUID)
        val advertiseData = AdvertiseData.Builder()
            .setIncludeDeviceName(false)
            .addServiceUuid(packerUuid)
            .addServiceData(packerUuid, data.take(24).toByteArray())
            .build()

        if (isAdvertising) {
             stopAdvertising()
        }
        
        // Restart with new data (simplified)
        try {
            val settings = AdvertiseSettings.Builder()
                .setAdvertiseMode(AdvertiseSettings.ADVERTISE_MODE_LOW_LATENCY)
                .setConnectable(true)
                .setTxPowerLevel(AdvertiseSettings.ADVERTISE_TX_POWER_HIGH)
                .build()
                
            advertiser?.startAdvertising(settings, advertiseData, advertiseCallback)
            isAdvertising = true
            Timber.d("BLE Advertising updated with data payload")
        } catch (e: Exception) {
             Timber.e(e, "Failed to send data via BLE advertising")
        }
    }
}
