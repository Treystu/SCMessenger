package com.scmessenger.android.transport.ble

import android.annotation.TargetApi
import android.bluetooth.*
import android.content.Context
import android.os.Build
import timber.log.Timber
import java.io.InputStream
import java.io.OutputStream
import java.util.concurrent.ConcurrentHashMap
import kotlinx.coroutines.*

/**
 * L2CAP Connection-Oriented Channel manager for high-throughput BLE.
 * 
 * Available on Android 10+ (API 29+).
 * Provides stream-oriented data transfer with higher throughput than GATT.
 * Falls back to GATT on older devices or if L2CAP fails.
 * 
 * Uses:
 * - BluetoothServerSocket for incoming connections
 * - BluetoothSocket for outgoing connections
 */
@TargetApi(Build.VERSION_CODES.Q)
class BleL2capManager(
    private val context: Context,
    private val onDataReceived: (deviceAddress: String, data: ByteArray) -> Unit
) {
    
    private val bluetoothManager = context.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager
    
    // L2CAP server socket (listening for incoming)
    private var serverSocket: BluetoothServerSocket? = null
    
    // Active L2CAP connections
    private val activeConnections = ConcurrentHashMap<String, L2capConnection>()
    
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    
    private var isListening = false
    
    /**
     * Check if L2CAP is supported on this device.
     */
    fun isSupported(): Boolean {
        return Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q
    }
    
    /**
     * Start listening for incoming L2CAP connections.
     */
    fun startListening() {
        if (!isSupported()) {
            Timber.w("L2CAP not supported on this device (API < 29)")
            return
        }
        
        if (isListening) {
            Timber.w("Already listening for L2CAP connections")
            return
        }
        
        val adapter = bluetoothManager?.adapter
        if (adapter == null) {
            Timber.e("Bluetooth adapter not available")
            return
        }
        
        scope.launch {
            try {
                serverSocket = adapter.listenUsingInsecureL2capChannel()
                isListening = true
                
                val psm = serverSocket?.psm ?: 0
                Timber.i("L2CAP server listening on PSM: $psm")
                
                // Accept loop
                while (isListening) {
                    try {
                        val socket = serverSocket?.accept()
                        if (socket != null) {
                            handleIncomingConnection(socket)
                        }
                    } catch (e: Exception) {
                        if (isListening) {
                            Timber.e(e, "Error accepting L2CAP connection")
                        }
                    }
                }
            } catch (e: SecurityException) {
                Timber.e(e, "Security exception starting L2CAP server")
                isListening = false
            } catch (e: Exception) {
                Timber.e(e, "Failed to start L2CAP server")
                isListening = false
            }
        }
    }
    
    /**
     * Stop listening for incoming connections.
     */
    fun stopListening() {
        if (!isListening) {
            return
        }
        
        isListening = false
        
        try {
            serverSocket?.close()
            serverSocket = null
            Timber.i("L2CAP server stopped")
        } catch (e: Exception) {
            Timber.e(e, "Error stopping L2CAP server")
        }
    }
    
    /**
     * Connect to a remote device via L2CAP.
     * Returns true if connection initiated successfully.
     */
    fun connect(deviceAddress: String, psm: Int): Boolean {
        if (!isSupported()) {
            Timber.w("L2CAP not supported on this device")
            return false
        }
        
        if (activeConnections.containsKey(deviceAddress)) {
            Timber.d("Already connected to $deviceAddress via L2CAP")
            return true
        }
        
        val adapter = bluetoothManager?.adapter
        if (adapter == null) {
            Timber.e("Bluetooth adapter not available")
            return false
        }
        
        scope.launch {
            try {
                val device = adapter.getRemoteDevice(deviceAddress)
                val socket = device.createInsecureL2capChannel(psm)
                
                socket.connect()
                
                val connection = L2capConnection(deviceAddress, socket)
                activeConnections[deviceAddress] = connection
                
                // Start read loop
                connection.startReading()
                
                Timber.i("L2CAP connected to $deviceAddress (PSM: $psm)")
            } catch (e: SecurityException) {
                Timber.e(e, "Security exception connecting L2CAP to $deviceAddress")
            } catch (e: Exception) {
                Timber.e(e, "Failed to connect L2CAP to $deviceAddress")
            }
        }
        
        return true
    }
    
    /**
     * Disconnect from a device.
     */
    fun disconnect(deviceAddress: String) {
        val connection = activeConnections.remove(deviceAddress) ?: return
        connection.close()
        Timber.d("L2CAP disconnected from $deviceAddress")
    }
    
    /**
     * Send data to a connected device.
     */
    fun sendData(deviceAddress: String, data: ByteArray): Boolean {
        val connection = activeConnections[deviceAddress] ?: run {
            Timber.w("No L2CAP connection to $deviceAddress")
            return false
        }
        
        return connection.send(data)
    }
    
    /**
     * Disconnect all connections and stop listening.
     */
    fun shutdown() {
        stopListening()
        
        val addresses = activeConnections.keys.toList()
        addresses.forEach { disconnect(it) }
        
        scope.cancel()
    }
    
    private fun handleIncomingConnection(socket: BluetoothSocket) {
        val deviceAddress = socket.remoteDevice.address
        Timber.d("Incoming L2CAP connection from $deviceAddress")
        
        if (activeConnections.containsKey(deviceAddress)) {
            Timber.w("Already have L2CAP connection to $deviceAddress, closing new one")
            socket.close()
            return
        }
        
        val connection = L2capConnection(deviceAddress, socket)
        activeConnections[deviceAddress] = connection
        connection.startReading()
    }
    
    /**
     * Represents an active L2CAP connection.
     */
    private inner class L2capConnection(
        val deviceAddress: String,
        private val socket: BluetoothSocket
    ) {
        private val inputStream: InputStream = socket.inputStream
        private val outputStream: OutputStream = socket.outputStream
        
        @Volatile
        private var isReading = false
        
        fun startReading() {
            if (isReading) {
                return
            }
            
            isReading = true
            
            scope.launch {
                try {
                    val buffer = ByteArray(8192) // 8KB buffer
                    
                    while (isReading && socket.isConnected) {
                        val bytesRead = inputStream.read(buffer)
                        if (bytesRead > 0) {
                            val data = buffer.copyOfRange(0, bytesRead)
                            onDataReceived(deviceAddress, data)
                            Timber.d("L2CAP received $bytesRead bytes from $deviceAddress")
                        } else if (bytesRead < 0) {
                            // End of stream
                            break
                        }
                    }
                } catch (e: Exception) {
                    if (isReading) {
                        Timber.e(e, "L2CAP read error from $deviceAddress")
                    }
                } finally {
                    close()
                }
            }
        }
        
        fun send(data: ByteArray): Boolean {
            return try {
                outputStream.write(data)
                outputStream.flush()
                Timber.d("L2CAP sent ${data.size} bytes to $deviceAddress")
                true
            } catch (e: Exception) {
                Timber.e(e, "Failed to send L2CAP data to $deviceAddress")
                false
            }
        }
        
        fun close() {
            isReading = false
            
            try {
                socket.close()
            } catch (e: Exception) {
                Timber.w(e, "Error closing L2CAP socket for $deviceAddress")
            }
            
            activeConnections.remove(deviceAddress)
        }
    }
}
