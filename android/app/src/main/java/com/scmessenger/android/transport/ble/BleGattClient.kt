package com.scmessenger.android.transport.ble

import android.bluetooth.*
import android.content.Context
import timber.log.Timber
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicInteger
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

    // Negotiated MTU per device
    private val negotiatedMtus = ConcurrentHashMap<String, Int>()

    // Current connection states
    private val connectionStates = ConcurrentHashMap<String, ConnectionState>()

    // Pending write operations (for fragmented writes)
    private val pendingWrites = ConcurrentHashMap<String, MutableList<ByteArray>>()

    // Reassembly buffers per device
    private val reassemblyBuffers = ConcurrentHashMap<String, MutableMap<Int, ByteArray>>()
    private val expectedFragments = ConcurrentHashMap<String, Int>()

    // Per-device GATT operation queue.
    // Android GATT is strictly sequential per connection: initiating a new op
    // before the previous callback fires causes the new op to silently return
    // false. Each device gets a Channel<() -> Unit> that is consumed one-at-a-
    // time; the consumer holds a Semaphore(1) permit for the duration of each
    // in-flight operation, and the corresponding GATT callback releases it once
    // the result arrives so the next enqueued op can proceed.
    private val gattOpQueues = ConcurrentHashMap<String, Channel<() -> Unit>>()
    private val gattOpSemaphores = ConcurrentHashMap<String, Semaphore>()

    // Counts in-flight WRITE_TYPE_NO_RESPONSE writes per device. For those
    // writes the Android GATT stack does NOT reliably deliver
    // onCharacteristicWrite (behaviour varies by API level and peripheral).
    // The semaphore is released immediately after writeCharacteristic();
    // onCharacteristicWrite decrements this counter and skips its own
    // release when the pre-decrement count was > 0, preventing double-release
    // even when multiple no-response writes are pipelined back-to-back.
    private val noResponseWriteInFlight = ConcurrentHashMap<String, AtomicInteger>()

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
            negotiatedMtus.remove(deviceAddress)
            gattOpQueues.remove(deviceAddress)?.close()
            gattOpSemaphores.remove(deviceAddress)
            noResponseWriteInFlight.remove(deviceAddress)
            // Clear any in-progress reassembly for this device to prevent stale
            // fragments from a prior connection contaminating a new connection.
            reassemblyBuffers.remove(deviceAddress)
            expectedFragments.remove(deviceAddress)
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

        val mtu = negotiatedMtus[deviceAddress] ?: 23
        val fragments = fragmentData(data, mtu)

        var allEnqueued = true
        for (fragment in fragments) {
            val enqueued = enqueueGattOp(deviceAddress) {
                try {
                    characteristic.value = fragment
                    characteristic.writeType = BluetoothGattCharacteristic.WRITE_TYPE_NO_RESPONSE
                    val initiated = gatt.writeCharacteristic(characteristic)
                    // WRITE_TYPE_NO_RESPONSE: onCharacteristicWrite is not reliably
                    // called on all Android versions.  Release the semaphore here so
                    // the next fragment is not blocked waiting for a callback that may
                    // never arrive.  noResponseWriteInFlight tracks how many such
                    // writes are pending so onCharacteristicWrite can skip its own
                    // release and avoid a double-release.
                    noResponseWriteInFlight.computeIfAbsent(deviceAddress) { AtomicInteger(0) }.incrementAndGet()
                    releaseGattOp(deviceAddress)
                    if (!initiated) {
                        Timber.e("Failed to initiate characteristic write to $deviceAddress")
                    }
                } catch (e: SecurityException) {
                    Timber.e(e, "Security exception sending data to $deviceAddress")
                    releaseGattOp(deviceAddress)
                } catch (e: Exception) {
                    Timber.e(e, "Failed to send data to $deviceAddress")
                    releaseGattOp(deviceAddress)
                }
            }
            if (!enqueued) allEnqueued = false
        }
        return allEnqueued
    }

    private fun fragmentData(data: ByteArray, mtu: Int): List<ByteArray> {
        val maxChunk = minOf(512, mtu - 3)
        val maxPayload = maxChunk - 4
        if (maxPayload <= 0) return listOf(data) // Fallback for tiny MTU

        val totalFragments = (data.size + maxPayload - 1).let { if (it < 0) 0 else it / maxPayload }.coerceAtLeast(1)
        val fragments = mutableListOf<ByteArray>()

        for (i in 0 until totalFragments) {
            val start = i * maxPayload
            val end = minOf(start + maxPayload, data.size)
            val chunk = data.copyOfRange(start, end)

            val header = ByteArray(4)
            // total_fragments (u16 le)
            header[0] = (totalFragments and 0xFF).toByte()
            header[1] = ((totalFragments shr 8) and 0xFF).toByte()
            // fragment_index (u16 le)
            header[2] = (i and 0xFF).toByte()
            header[3] = ((i shr 8) and 0xFF).toByte()

            fragments.add(header + chunk)
        }
        return fragments
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
                if (characteristic.uuid == BleGattServer.IDENTITY_CHAR_UUID && (status == 6 || status == 133)) {
                    // Status 6: Request Not Supported (often transient on iOS while GATT database settles)
                    // Status 133: Generic communication error (common Android transient failure)
                    scope.launch {
                        delay(1000)
                        val gattRef = activeConnections[deviceAddress] ?: return@launch
                        Timber.d("Retrying identity read for $deviceAddress")
                        readIdentityBeacon(gattRef)
                    }
                }
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
            } else {
                Timber.e("Characteristic write failed to $deviceAddress: $status")
            }
            pendingWrites.remove(deviceAddress)

            // For WRITE_TYPE_NO_RESPONSE, the semaphore was already released
            // immediately after writeCharacteristic(). Decrement the in-flight
            // counter (clamped at 0); if it was > 0 we consumed a pending
            // write — skip the release to avoid double-releasing the semaphore.
            val inFlight = noResponseWriteInFlight[deviceAddress]?.getAndUpdate { count ->
                if (count > 0) count - 1 else 0
            } ?: 0
            if (inFlight > 0) {
                Timber.v("Skipping semaphore release for NO_RESPONSE write on $deviceAddress")
                return
            }
            releaseGattOp(deviceAddress)
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
                    val value = characteristic.value ?: return
                    if (value.size < 4) {
                        Timber.w("Received tiny BLE packet (<4 bytes) from $deviceAddress")
                        return
                    }

                    val totalFrags = (value[0].toInt() and 0xFF) or ((value[1].toInt() and 0xFF) shl 8)
                    val fragIndex = (value[2].toInt() and 0xFF) or ((value[3].toInt() and 0xFF) shl 8)
                    val payload = value.copyOfRange(4, value.size)

                    if (fragIndex == 0) {
                        reassemblyBuffers[deviceAddress]?.clear()
                        Timber.v("BLE-RX (Central): Message start ($totalFrags frags) from $deviceAddress")
                    }
                    val buffer = reassemblyBuffers.getOrPut(deviceAddress) { ConcurrentHashMap<Int, ByteArray>() }
                    buffer[fragIndex] = payload
                    expectedFragments[deviceAddress] = totalFrags
                    Timber.v("BLE-RX (Central): Frag $fragIndex/$totalFrags (${payload.size} bytes) from $deviceAddress (current buffer: ${buffer.size})")

                    if (buffer.size == totalFrags) {
                        // All fragments arrived
                        val sortedData = (0 until totalFrags).mapNotNull { buffer[it] }
                        val completeData = ByteArray(sortedData.sumOf { it.size })
                        var currentPos = 0
                        for (chunk in sortedData) {
                            System.arraycopy(chunk, 0, completeData, currentPos, chunk.size)
                            currentPos += chunk.size
                        }

                        reassemblyBuffers.remove(deviceAddress)
                        expectedFragments.remove(deviceAddress)

                        Timber.d("Reassembled complete message from $deviceAddress: ${completeData.size} bytes")
                        onDataReceived(deviceAddress, completeData)
                    }
                }
            }
        }

        override fun onMtuChanged(gatt: BluetoothGatt, mtu: Int, status: Int) {
            super.onMtuChanged(gatt, mtu, status)

            val deviceAddress = gatt.device.address

            if (status == BluetoothGatt.GATT_SUCCESS) {
                Timber.d("MTU changed to $mtu for $deviceAddress")
                negotiatedMtus[deviceAddress] = mtu
                
                // P5: Request high priority for faster GATT writes/handshakes
                try {
                    gatt.requestConnectionPriority(BluetoothGatt.CONNECTION_PRIORITY_HIGH)
                } catch (e: SecurityException) {
                    Timber.w("Security exception requesting connection priority")
                }

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
