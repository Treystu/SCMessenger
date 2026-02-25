package com.scmessenger.android.transport.ble

import android.bluetooth.*
import android.content.Context
import timber.log.Timber
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap
import kotlinx.coroutines.*
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.sync.Semaphore

/**
 * GATT client for connecting to discovered SCMessenger peripherals.
 *
 * Android GATT requires strictly sequential operations per connection: a second
 * operation must not be initiated until the callback for the first one fires.
 * All GATT reads and writes are funnelled through a per-device
 * [Channel]<[() -> Unit]> + [Semaphore](1) queue. The consumer coroutine
 * acquires the semaphore before launching each op, and the GATT callback
 * releases it once the result arrives, allowing the next op to proceed.
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

    // Write queue for handling async writeCharacteristic fragmentation
    private val pendingWrites = ConcurrentHashMap<String, MutableList<ByteArray>>()

    // Per-device GATT operation queue.
    // Android GATT is strictly sequential per connection: initiating a new op
    // before the previous callback fires causes the new op to silently return
    // false. Each device gets a Channel<() -> Unit> that is consumed one-at-a-
    // time; the consumer holds a Semaphore(1) permit for the duration of each
    // in-flight operation, and the corresponding GATT callback releases it once
    // the result arrives so the next enqueued op can proceed.
    private val gattOpQueues = ConcurrentHashMap<String, Channel<() -> Unit>>()
    private val gattOpSemaphores = ConcurrentHashMap<String, Semaphore>()

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    // ---------- GATT op queue helpers ----------

    /** Returns (creating if necessary) the serialised op channel for a device. */
    private fun gattQueue(deviceAddress: String): Channel<() -> Unit> =
        gattOpQueues.getOrPut(deviceAddress) {
            Channel<() -> Unit>(Channel.UNLIMITED).also { ch ->
                val sem = Semaphore(1)
                gattOpSemaphores[deviceAddress] = sem
                scope.launch {
                    for (op in ch) {
                        // Wait until the previous op's callback has fired before
                        // starting the next one.
                        sem.acquire()
                        op()
                    }
                }
            }
        }

    /** Enqueue a GATT operation for sequential execution on the given device.
     *
     * Uses [Channel.trySend] to avoid launching a coroutine that could be
     * scheduled after [disconnect] closes the channel, which would throw a
     * [kotlinx.coroutines.channels.ClosedSendChannelException].  Because the
     * consumer coroutine serialises execution, ops are always run in the order
     * they were accepted.
     *
     * @return `true` if the op was accepted into the queue, `false` if the
     *   channel is closed or full (caller should treat the send as failed).
     */
    private fun enqueueGattOp(deviceAddress: String, op: () -> Unit): Boolean {
        val result = gattQueue(deviceAddress).trySend(op)
        if (!result.isSuccess) {
            Timber.w("GATT op dropped for %s: channel closed or full", deviceAddress)
        }
        return result.isSuccess
    }

    /**
     * Signal that the in-flight GATT operation for [deviceAddress] has completed.
     * Must be called from every GATT callback path (success and failure) so the
     * next queued operation is unblocked.
     */
    private fun releaseGattOp(deviceAddress: String) {
        gattOpSemaphores[deviceAddress]?.release()
    }

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
            gattOpQueues.remove(deviceAddress)?.close()
            gattOpSemaphores.remove(deviceAddress)
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
     * The write is enqueued so it cannot interleave with concurrent reads or
     * other in-progress writes on the same device.
     *
     * Returns `true` if the write was accepted into the op queue, `false` if
     * the device is not connected, the characteristic is unavailable, or the
     * op queue has already been closed (e.g. after [disconnect]).
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

        val service = gatt.getService(BleGattServer.SERVICE_UUID)
        val characteristic = service?.getCharacteristic(BleGattServer.MESSAGE_CHAR_UUID)
        if (characteristic == null) {
            Timber.e("Message characteristic not found on $deviceAddress")
            return false
        }

        val mtu = 512 // Assumed negotiated MTU
        return enqueueGattOp(deviceAddress) {
            try {
                if (data.size > mtu - 3) {
                    if (!sendFragmented(gatt, characteristic, data, mtu)) {
                        releaseGattOp(deviceAddress)
                    }
                    // else: semaphore released by onCharacteristicWrite when last fragment ack'd
                } else {
                    characteristic.value = data
                    if (!gatt.writeCharacteristic(characteristic)) {
                        releaseGattOp(deviceAddress)
                    }
                    // else: semaphore released in onCharacteristicWrite
                }
            } catch (e: SecurityException) {
                Timber.e(e, "Security exception sending data to $deviceAddress")
                releaseGattOp(deviceAddress)
            } catch (e: Exception) {
                Timber.e(e, "Failed to send data to $deviceAddress")
                releaseGattOp(deviceAddress)
            }
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
                    // Route through disconnect() so the GATT op queue and semaphore
                    // are cleaned up, not just the connection map entries.
                    // Guard against double-close: disconnect() is a no-op when the
                    // address is not in activeConnections.
                    activeConnections[deviceAddress] = gatt   // re-register so disconnect() finds it
                    disconnect(deviceAddress)
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
            // Read op complete — unblock the next queued operation.
            releaseGattOp(deviceAddress)
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
                    // More fragments remain — write the next one and keep the
                    // semaphore held until the last fragment is acknowledged.
                    val nextChunk = queue.removeAt(0)
                    characteristic.value = nextChunk
                    if (!gatt.writeCharacteristic(characteristic)) {
                        pendingWrites.remove(deviceAddress)
                        releaseGattOp(deviceAddress)
                    }
                } else {
                    // Last (or only) fragment acknowledged — op complete.
                    pendingWrites.remove(deviceAddress)
                    releaseGattOp(deviceAddress)
                }
            } else {
                Timber.e("Characteristic write failed to $deviceAddress: $status")
                pendingWrites.remove(deviceAddress)
                releaseGattOp(deviceAddress)
            }
        }

        override fun onDescriptorWrite(
            gatt: BluetoothGatt,
            descriptor: BluetoothGattDescriptor,
            status: Int
        ) {
            super.onDescriptorWrite(gatt, descriptor, status)
            val deviceAddress = gatt.device.address
            if (status == BluetoothGatt.GATT_SUCCESS) {
                Timber.d("Descriptor write successful for $deviceAddress")
            } else {
                Timber.e("Descriptor write failed for $deviceAddress: $status")
            }
            // CCCD write op complete — unblock the next queued operation.
            releaseGattOp(deviceAddress)
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
        enqueueGattOp(gatt.device.address) {
            try {
                val service = gatt.getService(BleGattServer.SERVICE_UUID)
                val characteristic = service?.getCharacteristic(BleGattServer.IDENTITY_CHAR_UUID)
                if (characteristic == null || !gatt.readCharacteristic(characteristic)) {
                    // If the op couldn't be initiated, release immediately so the
                    // queue doesn't stall.
                    releaseGattOp(gatt.device.address)
                }
                // else: semaphore released in onCharacteristicRead
            } catch (e: SecurityException) {
                Timber.e(e, "Security exception reading identity")
                releaseGattOp(gatt.device.address)
            } catch (e: Exception) {
                Timber.e(e, "Failed to read identity beacon")
                releaseGattOp(gatt.device.address)
            }
        }
    }

    private fun scheduleIdentityRefreshReads(deviceAddress: String) {
        val originalGatt = activeConnections[deviceAddress] ?: return
        // All refresh reads run in a single sequential coroutine to preserve their
        // intended ordering (T+900 ms, then T+2200 ms after connection).  Each
        // read is submitted through enqueueGattOp so it cannot overlap with any
        // other in-flight operation on the same device.
        scope.launch {
            for (delayMs in IDENTITY_REFRESH_DELAYS_MS) {
                delay(delayMs)
                val gatt = activeConnections[deviceAddress] ?: return@launch
                if (gatt !== originalGatt) return@launch
                if (connectionStates[deviceAddress] != ConnectionState.CONNECTED) return@launch
                readIdentityBeacon(gatt)
            }
        }
    }

    private fun enableMessageNotifications(gatt: BluetoothGatt) {
        enqueueGattOp(gatt.device.address) {
            try {
                val service = gatt.getService(BleGattServer.SERVICE_UUID)
                val characteristic = service?.getCharacteristic(BleGattServer.MESSAGE_CHAR_UUID)
                if (characteristic == null) {
                    releaseGattOp(gatt.device.address)
                    return@enqueueGattOp
                }

                // Enable local notification routing (no GATT round-trip needed)
                gatt.setCharacteristicNotification(characteristic, true)

                // Write to CCCD — completion signalled via onDescriptorWrite
                val descriptor = characteristic.getDescriptor(BleGattServer.CLIENT_CONFIG_DESCRIPTOR_UUID)
                if (descriptor != null) {
                    descriptor.value = BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE
                    if (!gatt.writeDescriptor(descriptor)) {
                        releaseGattOp(gatt.device.address)
                    } else {
                        Timber.d("Enabling notifications for ${gatt.device.address}")
                    }
                    // else: semaphore released in onDescriptorWrite
                } else {
                    releaseGattOp(gatt.device.address)
                }
            } catch (e: SecurityException) {
                Timber.e(e, "Security exception enabling notifications")
                releaseGattOp(gatt.device.address)
            } catch (e: Exception) {
                Timber.e(e, "Failed to enable notifications")
                releaseGattOp(gatt.device.address)
            }
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
