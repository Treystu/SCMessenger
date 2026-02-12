package com.scmessenger.android.transport.ble

import android.bluetooth.*
import android.content.Context
import timber.log.Timber
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap

/**
 * GATT server implementing SCMessenger BLE service.
 * 
 * Provides characteristics for:
 * - Identity beacon (read): Public key + node info
 * - Message exchange (write/notify): Encrypted message frames
 * - Sync handshake (read/write): Drift protocol synchronization
 * 
 * Handles:
 * - MTU negotiation (request 512, handle fragmentation)
 * - Connection multiplexing (multiple clients)
 * - Data forwarding to Rust via PlatformBridge
 */
class BleGattServer(
    private val context: Context,
    private val onDataReceived: (peerId: String, data: ByteArray) -> Unit
) {
    
    private var bluetoothManager: BluetoothManager? = null
    private var gattServer: BluetoothGattServer? = null
    
    // Track connected devices
    private val connectedDevices = ConcurrentHashMap<String, BluetoothDevice>()
    
    // Pending write operations (for fragmented writes)
    private val pendingWrites = ConcurrentHashMap<String, ByteArray>()
    
    private var isRunning = false
    
    fun start() {
        if (isRunning) {
            Timber.w("GATT server already running")
            return
        }
        
        bluetoothManager = context.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager
        if (bluetoothManager == null) {
            Timber.e("BluetoothManager not available")
            return
        }
        
        try {
            gattServer = bluetoothManager?.openGattServer(context, gattServerCallback)
            
            if (gattServer == null) {
                Timber.e("Failed to open GATT server")
                return
            }
            
            // Add SCMessenger service
            val service = BluetoothGattService(
                SERVICE_UUID,
                BluetoothGattService.SERVICE_TYPE_PRIMARY
            )
            
            // Identity beacon characteristic (read)
            val identityChar = BluetoothGattCharacteristic(
                IDENTITY_CHAR_UUID,
                BluetoothGattCharacteristic.PROPERTY_READ,
                BluetoothGattCharacteristic.PERMISSION_READ
            )
            service.addCharacteristic(identityChar)
            
            // Message exchange characteristic (write + notify)
            val messageChar = BluetoothGattCharacteristic(
                MESSAGE_CHAR_UUID,
                BluetoothGattCharacteristic.PROPERTY_WRITE or
                        BluetoothGattCharacteristic.PROPERTY_NOTIFY,
                BluetoothGattCharacteristic.PERMISSION_WRITE
            )
            // Add descriptor for notifications
            val messageDescriptor = BluetoothGattDescriptor(
                CLIENT_CONFIG_DESCRIPTOR_UUID,
                BluetoothGattDescriptor.PERMISSION_READ or BluetoothGattDescriptor.PERMISSION_WRITE
            )
            messageChar.addDescriptor(messageDescriptor)
            service.addCharacteristic(messageChar)
            
            // Sync handshake characteristic (read + write)
            val syncChar = BluetoothGattCharacteristic(
                SYNC_CHAR_UUID,
                BluetoothGattCharacteristic.PROPERTY_READ or
                        BluetoothGattCharacteristic.PROPERTY_WRITE,
                BluetoothGattCharacteristic.PERMISSION_READ or
                        BluetoothGattCharacteristic.PERMISSION_WRITE
            )
            service.addCharacteristic(syncChar)
            
            gattServer?.addService(service)
            isRunning = true
            
            Timber.i("GATT server started with SCMessenger service")
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception starting GATT server - missing permissions?")
        } catch (e: Exception) {
            Timber.e(e, "Failed to start GATT server")
        }
    }
    
    fun stop() {
        if (!isRunning) {
            return
        }
        
        try {
            // Disconnect all clients
            connectedDevices.values.forEach { device ->
                try {
                    gattServer?.cancelConnection(device)
                } catch (e: Exception) {
                    Timber.w(e, "Error disconnecting device")
                }
            }
            connectedDevices.clear()
            pendingWrites.clear()
            
            gattServer?.close()
            gattServer = null
            isRunning = false
            
            Timber.i("GATT server stopped")
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception stopping GATT server")
        } catch (e: Exception) {
            Timber.e(e, "Failed to stop GATT server")
        }
    }
    
    /**
     * Send data to a specific connected device via notify.
     */
    fun sendData(deviceAddress: String, data: ByteArray): Boolean {
        val device = connectedDevices[deviceAddress] ?: run {
            Timber.w("Device not connected: $deviceAddress")
            return false
        }
        
        return try {
            val service = gattServer?.getService(SERVICE_UUID)
            val characteristic = service?.getCharacteristic(MESSAGE_CHAR_UUID)
            
            if (characteristic == null) {
                Timber.e("Message characteristic not found")
                return false
            }
            
            // Handle MTU fragmentation if needed
            val mtu = 512 // Negotiated MTU
            if (data.size > mtu - 3) {
                // Fragment and send in chunks
                sendFragmented(device, characteristic, data, mtu)
            } else {
                characteristic.value = data
                gattServer?.notifyCharacteristicChanged(device, characteristic, false)
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
        device: BluetoothDevice,
        characteristic: BluetoothGattCharacteristic,
        data: ByteArray,
        mtu: Int
    ): Boolean {
        val chunkSize = mtu - 3 // Account for ATT overhead
        var offset = 0
        
        while (offset < data.size) {
            val end = minOf(offset + chunkSize, data.size)
            val chunk = data.copyOfRange(offset, end)
            
            characteristic.value = chunk
            gattServer?.notifyCharacteristicChanged(device, characteristic, false)
            
            offset = end
            Thread.sleep(10) // Small delay between chunks
        }
        
        return true
    }
    
    private val gattServerCallback = object : BluetoothGattServerCallback() {
        
        override fun onConnectionStateChange(device: BluetoothDevice, status: Int, newState: Int) {
            super.onConnectionStateChange(device, status, newState)
            
            when (newState) {
                BluetoothProfile.STATE_CONNECTED -> {
                    connectedDevices[device.address] = device
                    Timber.d("GATT client connected: ${device.address}")
                    
                    // Request MTU change to 512
                    // Note: MTU request must come from client, but we track it here
                }
                
                BluetoothProfile.STATE_DISCONNECTED -> {
                    connectedDevices.remove(device.address)
                    pendingWrites.remove(device.address)
                    Timber.d("GATT client disconnected: ${device.address}")
                }
            }
        }
        
        override fun onCharacteristicReadRequest(
            device: BluetoothDevice,
            requestId: Int,
            offset: Int,
            characteristic: BluetoothGattCharacteristic
        ) {
            super.onCharacteristicReadRequest(device, requestId, offset, characteristic)
            
            when (characteristic.uuid) {
                IDENTITY_CHAR_UUID -> {
                    // Return our identity beacon
                    // Use a placeholder beacon - in production this would come from IronCore.getIdentityInfo()
                    val identityBeacon = "SCM_IDENTITY_BEACON".toByteArray()
                    gattServer?.sendResponse(
                        device,
                        requestId,
                        BluetoothGatt.GATT_SUCCESS,
                        offset,
                        identityBeacon
                    )
                }
                
                SYNC_CHAR_UUID -> {
                    // Return sync handshake data
                    val syncData = "SYNC_HANDSHAKE".toByteArray()
                    gattServer?.sendResponse(
                        device,
                        requestId,
                        BluetoothGatt.GATT_SUCCESS,
                        offset,
                        syncData
                    )
                }
                
                else -> {
                    gattServer?.sendResponse(
                        device,
                        requestId,
                        BluetoothGatt.GATT_FAILURE,
                        offset,
                        null
                    )
                }
            }
        }
        
        override fun onCharacteristicWriteRequest(
            device: BluetoothDevice,
            requestId: Int,
            characteristic: BluetoothGattCharacteristic,
            preparedWrite: Boolean,
            responseNeeded: Boolean,
            offset: Int,
            value: ByteArray?
        ) {
            super.onCharacteristicWriteRequest(device, requestId, characteristic, preparedWrite, responseNeeded, offset, value)
            
            if (value == null) {
                if (responseNeeded) {
                    gattServer?.sendResponse(device, requestId, BluetoothGatt.GATT_FAILURE, offset, null)
                }
                return
            }
            
            when (characteristic.uuid) {
                MESSAGE_CHAR_UUID -> {
                    // Receive message data (possibly fragmented)
                    if (preparedWrite) {
                        // Accumulate chunks
                        val existing = pendingWrites[device.address] ?: ByteArray(0)
                        pendingWrites[device.address] = existing + value
                    } else {
                        // Complete write
                        val completeData = (pendingWrites.remove(device.address) ?: ByteArray(0)) + value
                        
                        // Forward to Rust
                        onDataReceived(device.address, completeData)
                        
                        Timber.d("Received ${completeData.size} bytes from ${device.address}")
                    }
                    
                    if (responseNeeded) {
                        gattServer?.sendResponse(device, requestId, BluetoothGatt.GATT_SUCCESS, offset, value)
                    }
                }
                
                SYNC_CHAR_UUID -> {
                    // Process sync handshake
                    Timber.d("Sync handshake from ${device.address}: ${value.size} bytes")
                    
                    if (responseNeeded) {
                        gattServer?.sendResponse(device, requestId, BluetoothGatt.GATT_SUCCESS, offset, value)
                    }
                }
                
                else -> {
                    if (responseNeeded) {
                        gattServer?.sendResponse(device, requestId, BluetoothGatt.GATT_FAILURE, offset, null)
                    }
                }
            }
        }
        
        override fun onMtuChanged(device: BluetoothDevice, mtu: Int) {
            super.onMtuChanged(device, mtu)
            Timber.d("MTU changed for ${device.address}: $mtu")
        }
    }
    
    companion object {
        // SCMessenger GATT Service UUID (0xDF01)
        val SERVICE_UUID: UUID = UUID.fromString("0000df01-0000-1000-8000-00805f9b34fb")
        
        // Characteristics
        val IDENTITY_CHAR_UUID: UUID = UUID.fromString("0000df02-0000-1000-8000-00805f9b34fb")
        val MESSAGE_CHAR_UUID: UUID = UUID.fromString("0000df03-0000-1000-8000-00805f9b34fb")
        val SYNC_CHAR_UUID: UUID = UUID.fromString("0000df04-0000-1000-8000-00805f9b34fb")
        
        // Client Configuration Descriptor
        val CLIENT_CONFIG_DESCRIPTOR_UUID: UUID = UUID.fromString("00002902-0000-1000-8000-00805f9b34fb")
    }
}
