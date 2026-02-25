package com.scmessenger.android.transport.ble

import android.bluetooth.*
import android.content.Context
import timber.log.Timber
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap
import kotlinx.coroutines.*

/**
 * GATT client for connecting to discovered SCMessenger peripherals.
 *
 * Responsibilities:
 * - Connects to discovered SCMessenger BLE peripherals
 * - Reads identity beacon to get peer's public key
 * - Initiates sync handshake for Drift protocol
 * - Writes encrypted message frames
 * - Manages connection lifecycle and retry logic
 * - Handles reliable write with chunking for >MTU payloads
 * - Maintains connection pool (max 5 concurrent)
 */
class BleGattClient(
    private val context: Context,
    private val onIdentityReceived: (deviceAddress: String, identity: ByteArray) -> Unit,
    private val onDataReceived: (deviceAddress: String, data: ByteArray) -> Unit
) {
    companion object {
        // Initial identity beacon can arrive before the peer publishes final nickname.
        // Re-read shortly after connect to surface nickname promptly in Nearby UI.
        private val IDENTITY_REFRESH_DELAYS_MS = listOf(900L, 2200L)
    }

    private val bluetoothManager = context.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager

    // Active GATT connections (max 5)
    private val activeConnections = ConcurrentHashMap<String, BluetoothGatt>()
    private val maxConnections = 5

    // Connection state tracking
    private val connectionStates = ConcurrentHashMap<String, ConnectionState>()

    // Write queue for handling async writeCharacteristic
    private val pendingWrites = ConcurrentHashMap<String, MutableList<ByteArray>>()

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    /**
     * Connect to a discovered peripheral.
     * Returns true if connection initiated, false if rejected (pool full, already connected).
     */
    fun connect(deviceAddress: String): Boolean {
        // Check connection pool limit
        if (activeConnections.size >= maxConnections) {
            Timber.w("Connection pool full ($maxConnections), cannot connect to $deviceAddress")
            return false
        }

        // Check if already connected
        if (activeConnections.containsKey(deviceAddress)) {
            Timber.d("Already connected to $deviceAddress")
            return true
        }

        val adapter = bluetoothManager?.adapter
        if (adapter == null) {
            Timber.e("Bluetooth adapter not available")
            return false
        }

        return try {
            val device = adapter.getRemoteDevice(deviceAddress)
            connectionStates[deviceAddress] = ConnectionState.CONNECTING

            val gatt = device.connectGatt(
                context,
                false, // autoConnect = false for faster connection
                gattCallback,
                BluetoothDevice.TRANSPORT_LE
            )

            activeConnections[deviceAddress] = gatt
            Timber.d("Connecting to $deviceAddress")
            true
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception connecting to $deviceAddress")
            connectionStates.remove(deviceAddress)
            false
        } catch (e: Exception) {
            Timber.e(e, "Failed to connect to $deviceAddress")
            connectionStates.remove(deviceAddress)
            false
        }
    }

    /**
     * Disconnect from a peripheral.
     */
    fun disconnect(deviceAddress: String) {
        val gatt = activeConnections.remove(deviceAddress) ?: return

        try {
            gatt.disconnect()
            gatt.close()
            connectionStates.remove(deviceAddress)
            Timber.d("Disconnected from $deviceAddress")
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception disconnecting from $deviceAddress")
        } catch (e: Exception) {
            Timber.e(e, "Failed to disconnect from $deviceAddress")
        }
    }

    /**
     * Send data to a connected peripheral.
     * Handles fragmentation if data exceeds MTU.
     */
    fun sendData(deviceAddress: String, data: ByteArray): Boolean {
        val gatt = activeConnections[deviceAddress] ?: run {
            Timber.w("Not connected to $deviceAddress")
            return false
        }

        val state = connectionStates[deviceAddress]
        if (state != ConnectionState.CONNECTED) {
            Timber.w("Cannot send data - not in CONNECTED state: $state")
            return false
        }

        return try {
            val service = gatt.getService(BleGattServer.SERVICE_UUID)
            val characteristic = service?.getCharacteristic(BleGattServer.MESSAGE_CHAR_UUID)

            if (characteristic == null) {
                Timber.e("Message characteristic not found on $deviceAddress")
                return false
            }

            // Handle MTU fragmentation
            val mtu = 512 // Assumed negotiated MTU
            if (data.size > mtu - 3) {
                sendFragmented(gatt, characteristic, data, mtu)
            } else {
                characteristic.value = data
                gatt.writeCharacteristic(characteristic)
                true
            }
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception sending data")
            false
        } catch (e: Exception) {
            Timber.e(e, "Failed to send data to $deviceAddress")
            false
        }
    }

    private fun sendFragmented(
        gatt: BluetoothGatt,
        characteristic: BluetoothGattCharacteristic,
        data: ByteArray,
        mtu: Int
    ): Boolean {
        val chunkSize = mtu - 3
        val deviceAddress = gatt.device.address
        val chunks = mutableListOf<ByteArray>()
        var offset = 0

        while (offset < data.size) {
            val end = minOf(offset + chunkSize, data.size)
            chunks.add(data.copyOfRange(offset, end))
            offset = end
        }

        if (chunks.isEmpty()) return true

        pendingWrites[deviceAddress] = chunks.drop(1).toMutableList()
        characteristic.value = chunks[0]
        return gatt.writeCharacteristic(characteristic)
    }

    /**
     * Disconnect all active connections.
     */
    fun disconnectAll() {
        val addresses = activeConnections.keys.toList()
        addresses.forEach { disconnect(it) }
    }

    private val gattCallback = object : BluetoothGattCallback() {

        override fun onConnectionStateChange(gatt: BluetoothGatt, status: Int, newState: Int) {
            super.onConnectionStateChange(gatt, status, newState)

            val deviceAddress = gatt.device.address

            when (newState) {
                BluetoothProfile.STATE_CONNECTED -> {
                    Timber.d("Connected to $deviceAddress, requesting MTU...")
                    connectionStates[deviceAddress] = ConnectionState.DISCOVERING_SERVICES

                    try {
                        gatt.requestMtu(512)
                    } catch (e: SecurityException) {
                        Timber.e(e, "Security exception requesting MTU")
                        disconnect(deviceAddress)
                    }
                }

                BluetoothProfile.STATE_DISCONNECTED -> {
                    Timber.d("Disconnected from $deviceAddress")
                    activeConnections.remove(deviceAddress)
                    connectionStates.remove(deviceAddress)
                    gatt.close()
                }
            }
        }

        override fun onServicesDiscovered(gatt: BluetoothGatt, status: Int) {
            super.onServicesDiscovered(gatt, status)

            val deviceAddress = gatt.device.address

            if (status == BluetoothGatt.GATT_SUCCESS) {
                Timber.d("Services discovered on $deviceAddress")
                connectionStates[deviceAddress] = ConnectionState.CONNECTED

                // Check for SCMessenger service
                val service = gatt.getService(BleGattServer.SERVICE_UUID)
                if (service != null) {
                    Timber.d("SCMessenger service found on $deviceAddress")

                    // Read identity beacon
                    readIdentityBeacon(gatt)
                    scheduleIdentityRefreshReads(deviceAddress)

                    // Enable notifications for message characteristic
                    enableMessageNotifications(gatt)
                } else {
                    Timber.w("SCMessenger service not found on $deviceAddress")
                    disconnect(deviceAddress)
                }
            } else {
                Timber.e("Service discovery failed on $deviceAddress: $status")
                disconnect(deviceAddress)
            }
        }

        override fun onCharacteristicRead(
            gatt: BluetoothGatt,
            characteristic: BluetoothGattCharacteristic,
            status: Int
        ) {
            super.onCharacteristicRead(gatt, characteristic, status)

            val deviceAddress = gatt.device.address

            if (status == BluetoothGatt.GATT_SUCCESS) {
                when (characteristic.uuid) {
                    BleGattServer.IDENTITY_CHAR_UUID -> {
                        val identity = characteristic.value
                        Timber.d("Identity beacon from $deviceAddress: ${identity.size} bytes")
                        onIdentityReceived(deviceAddress, identity)
                    }

                    BleGattServer.SYNC_CHAR_UUID -> {
                        val syncData = characteristic.value
                        Timber.d("Sync handshake from $deviceAddress: ${syncData.size} bytes")
                    }
                }
            } else {
                Timber.e("Characteristic read failed on $deviceAddress: $status")
            }
        }

        override fun onCharacteristicWrite(
            gatt: BluetoothGatt,
            characteristic: BluetoothGattCharacteristic,
            status: Int
        ) {
            super.onCharacteristicWrite(gatt, characteristic, status)

            val deviceAddress = gatt.device.address

            if (status == BluetoothGatt.GATT_SUCCESS) {
                Timber.d("Characteristic write successful to $deviceAddress")
                val queue = pendingWrites[deviceAddress]
                if (queue != null && queue.isNotEmpty()) {
                    val nextChunk = queue.removeAt(0)
                    characteristic.value = nextChunk
                    gatt.writeCharacteristic(characteristic)
                } else {
                    pendingWrites.remove(deviceAddress)
                }
            } else {
                Timber.e("Characteristic write failed to $deviceAddress: $status")
                pendingWrites.remove(deviceAddress)
            }
        }

        override fun onCharacteristicChanged(
            gatt: BluetoothGatt,
            characteristic: BluetoothGattCharacteristic
        ) {
            super.onCharacteristicChanged(gatt, characteristic)

            val deviceAddress = gatt.device.address

            when (characteristic.uuid) {
                BleGattServer.MESSAGE_CHAR_UUID -> {
                    val data = characteristic.value
                    Timber.d("Message notification from $deviceAddress: ${data.size} bytes")
                    onDataReceived(deviceAddress, data)
                }
            }
        }

        override fun onMtuChanged(gatt: BluetoothGatt, mtu: Int, status: Int) {
            super.onMtuChanged(gatt, mtu, status)

            val deviceAddress = gatt.device.address

            if (status == BluetoothGatt.GATT_SUCCESS) {
                Timber.d("MTU changed to $mtu for $deviceAddress")
                try {
                    gatt.discoverServices()
                } catch (e: SecurityException) {
                    Timber.e(e, "Security exception discovering services")
                    disconnect(deviceAddress)
                }
            } else {
                Timber.w("MTU change failed for $deviceAddress: $status")
                disconnect(deviceAddress)
            }
        }
    }

    private fun readIdentityBeacon(gatt: BluetoothGatt) {
        try {
            val service = gatt.getService(BleGattServer.SERVICE_UUID) ?: return
            val characteristic = service.getCharacteristic(BleGattServer.IDENTITY_CHAR_UUID) ?: return
            gatt.readCharacteristic(characteristic)
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception reading identity")
        } catch (e: Exception) {
            Timber.e(e, "Failed to read identity beacon")
        }
    }

    private fun scheduleIdentityRefreshReads(deviceAddress: String) {
        val originalGatt = activeConnections[deviceAddress] ?: return
        IDENTITY_REFRESH_DELAYS_MS.forEach { delayMs ->
            scope.launch {
                delay(delayMs)
                val gatt = activeConnections[deviceAddress] ?: return@launch
                if (gatt !== originalGatt) return@launch
                if (connectionStates[deviceAddress] != ConnectionState.CONNECTED) return@launch
                readIdentityBeacon(gatt)
            }
        }
    }

    private fun enableMessageNotifications(gatt: BluetoothGatt) {
        try {
            val service = gatt.getService(BleGattServer.SERVICE_UUID) ?: return
            val characteristic = service.getCharacteristic(BleGattServer.MESSAGE_CHAR_UUID) ?: return

            // Enable local notifications
            gatt.setCharacteristicNotification(characteristic, true)

            // Write to CCCD
            val descriptor = characteristic.getDescriptor(BleGattServer.CLIENT_CONFIG_DESCRIPTOR_UUID)
            if (descriptor != null) {
                descriptor.value = BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE
                gatt.writeDescriptor(descriptor)
                Timber.d("Enabled notifications for ${gatt.device.address}")
            }
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception enabling notifications")
        } catch (e: Exception) {
            Timber.e(e, "Failed to enable notifications")
        }
    }

    fun cleanup() {
        scope.cancel()
        disconnectAll()
    }

    enum class ConnectionState {
        CONNECTING,
        DISCOVERING_SERVICES,
        CONNECTED,
        DISCONNECTED
    }
}
