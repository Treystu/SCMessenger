package com.scmessenger.android.transport

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.wifi.p2p.*
import android.net.wifi.p2p.nsd.WifiP2pDnsSdServiceInfo
import android.net.wifi.p2p.nsd.WifiP2pDnsSdServiceRequest
import timber.log.Timber
import java.io.InputStream
import java.io.OutputStream
import java.net.InetSocketAddress
import java.net.ServerSocket
import java.net.Socket
import java.util.concurrent.ConcurrentHashMap
import kotlinx.coroutines.*

/**
 * WiFi Direct (WiFi P2P) transport implementation.
 * 
 * Provides device-to-device communication using WiFi Direct:
 * - DNS-SD service discovery
 * - Group formation and negotiation
 * - Socket-based data exchange
 * - Auto-negotiation for group owner
 * 
 * Works on most Android devices (API 14+, but we target 26+).
 */
class WifiDirectTransport(
    private val context: Context,
    private val onPeerDiscovered: (peerId: String) -> Unit,
    private val onDataReceived: (peerId: String, data: ByteArray) -> Unit
) {
    
    private val wifiP2pManager: WifiP2pManager? = context.getSystemService(Context.WIFI_P2P_SERVICE) as? WifiP2pManager
    private var channel: WifiP2pManager.Channel? = null
    
    private var isRunning = false
    private var isGroupOwner = false
    
    private val discoveredPeers = ConcurrentHashMap<String, WifiP2pDevice>()
    private val activeConnections = ConcurrentHashMap<String, P2pConnection>()
    
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    
    // Server socket for group owner
    private var serverSocket: ServerSocket? = null
    
    /**
     * Start WiFi Direct discovery and service advertisement.
     */
    fun start() {
        if (isRunning) {
            Timber.w("WiFi Direct already running")
            return
        }
        
        if (wifiP2pManager == null) {
            Timber.e("WiFi P2P not available on this device")
            return
        }
        
        try {
            channel = wifiP2pManager.initialize(context, context.mainLooper, null)
            
            if (channel == null) {
                Timber.e("Failed to initialize WiFi P2P channel")
                return
            }
            
            // Register broadcast receiver for P2P events
            val intentFilter = IntentFilter().apply {
                addAction(WifiP2pManager.WIFI_P2P_STATE_CHANGED_ACTION)
                addAction(WifiP2pManager.WIFI_P2P_PEERS_CHANGED_ACTION)
                addAction(WifiP2pManager.WIFI_P2P_CONNECTION_CHANGED_ACTION)
                addAction(WifiP2pManager.WIFI_P2P_THIS_DEVICE_CHANGED_ACTION)
            }
            
            context.registerReceiver(p2pReceiver, intentFilter)
            
            isRunning = true
            
            // Start service discovery
            startServiceDiscovery()
            
            // Register our service
            registerService()
            
            Timber.i("WiFi Direct started")
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception starting WiFi Direct - missing permissions?")
        } catch (e: Exception) {
            Timber.e(e, "Failed to start WiFi Direct")
        }
    }
    
    /**
     * Stop WiFi Direct.
     */
    fun stop() {
        if (!isRunning) {
            return
        }
        
        isRunning = false
        
        try {
            // Stop discovery
            wifiP2pManager?.stopPeerDiscovery(channel, null)
            
            // Clear service
            wifiP2pManager?.clearLocalServices(channel, null)
            
            // Disconnect from group
            wifiP2pManager?.removeGroup(channel, null)
            
            // Close all connections
            activeConnections.values.forEach { it.close() }
            activeConnections.clear()
            
            // Close server socket
            serverSocket?.close()
            serverSocket = null
            
            // Unregister receiver
            context.unregisterReceiver(p2pReceiver)
            
            discoveredPeers.clear()
            
            Timber.i("WiFi Direct stopped")
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception stopping WiFi Direct")
        } catch (e: Exception) {
            Timber.e(e, "Failed to stop WiFi Direct")
        }
    }
    
    /**
     * Send data to a peer via WiFi Direct.
     */
    fun sendData(peerId: String, data: ByteArray): Boolean {
        val connection = activeConnections[peerId] ?: run {
            Timber.w("No WiFi Direct connection to $peerId")
            return false
        }
        
        return connection.send(data)
    }
    
    private fun startServiceDiscovery() {
        if (channel == null) return
        
        try {
            val serviceRequest = WifiP2pDnsSdServiceRequest.newInstance()
            
            wifiP2pManager?.setDnsSdResponseListeners(
                channel,
                { _, _, _ -> },
                { fullDomainName, record, device ->
                    Timber.d("WiFi Direct service discovered: $fullDomainName from ${device.deviceName}")
                    
                    if (record["service"] == SERVICE_TYPE) {
                        discoveredPeers[device.deviceAddress] = device
                        onPeerDiscovered(device.deviceAddress)
                        
                        // Initiate connection
                        connectToPeer(device)
                    }
                }
            )
            
            wifiP2pManager?.addServiceRequest(channel, serviceRequest, object : WifiP2pManager.ActionListener {
                override fun onSuccess() {
                    Timber.d("Service discovery request added")
                    startPeerDiscovery()
                }
                
                override fun onFailure(reason: Int) {
                    Timber.e("Failed to add service discovery request: $reason")
                }
            })
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception in service discovery")
        } catch (e: Exception) {
            Timber.e(e, "Failed to start service discovery")
        }
    }
    
    private fun startPeerDiscovery() {
        if (channel == null) return
        
        try {
            wifiP2pManager?.discoverPeers(channel, object : WifiP2pManager.ActionListener {
                override fun onSuccess() {
                    Timber.d("Peer discovery started")
                }
                
                override fun onFailure(reason: Int) {
                    Timber.e("Peer discovery failed: $reason")
                }
            })
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception starting peer discovery")
        } catch (e: Exception) {
            Timber.e(e, "Failed to start peer discovery")
        }
    }
    
    private fun registerService() {
        if (channel == null) return
        
        try {
            val record = mutableMapOf<String, String>().apply {
                put("service", SERVICE_TYPE)
                put("version", "1.0")
            }
            
            val serviceInfo = WifiP2pDnsSdServiceInfo.newInstance(
                SERVICE_NAME,
                SERVICE_TYPE,
                record
            )
            
            wifiP2pManager?.addLocalService(channel, serviceInfo, object : WifiP2pManager.ActionListener {
                override fun onSuccess() {
                    Timber.d("WiFi Direct service registered: $SERVICE_NAME")
                }
                
                override fun onFailure(reason: Int) {
                    Timber.e("Failed to register service: $reason")
                }
            })
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception registering service")
        } catch (e: Exception) {
            Timber.e(e, "Failed to register service")
        }
    }
    
    private fun connectToPeer(device: WifiP2pDevice) {
        if (channel == null) return
        
        if (activeConnections.containsKey(device.deviceAddress)) {
            Timber.d("Already connected to ${device.deviceAddress}")
            return
        }
        
        try {
            val config = WifiP2pConfig().apply {
                deviceAddress = device.deviceAddress
                groupOwnerIntent = 0 // We want to be client if possible
            }
            
            wifiP2pManager?.connect(channel, config, object : WifiP2pManager.ActionListener {
                override fun onSuccess() {
                    Timber.d("Connection initiated to ${device.deviceAddress}")
                }
                
                override fun onFailure(reason: Int) {
                    Timber.e("Failed to connect to ${device.deviceAddress}: $reason")
                }
            })
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception connecting to peer")
        } catch (e: Exception) {
            Timber.e(e, "Failed to connect to peer")
        }
    }
    
    private val p2pReceiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            when (intent.action) {
                WifiP2pManager.WIFI_P2P_STATE_CHANGED_ACTION -> {
                    val state = intent.getIntExtra(WifiP2pManager.EXTRA_WIFI_STATE, -1)
                    val enabled = state == WifiP2pManager.WIFI_P2P_STATE_ENABLED
                    Timber.d("WiFi P2P state changed: enabled=$enabled")
                }
                
                WifiP2pManager.WIFI_P2P_PEERS_CHANGED_ACTION -> {
                    Timber.d("Peer list changed")
                }
                
                WifiP2pManager.WIFI_P2P_CONNECTION_CHANGED_ACTION -> {
                    val networkInfo = intent.getParcelableExtra<android.net.NetworkInfo>(WifiP2pManager.EXTRA_NETWORK_INFO)
                    
                    if (networkInfo?.isConnected == true) {
                        Timber.d("Connected to WiFi P2P group")
                        wifiP2pManager?.requestConnectionInfo(channel) { info ->
                            handleConnectionInfo(info)
                        }
                    } else {
                        Timber.d("Disconnected from WiFi P2P group")
                    }
                }
                
                WifiP2pManager.WIFI_P2P_THIS_DEVICE_CHANGED_ACTION -> {
                    Timber.d("This device changed")
                }
            }
        }
    }
    
    private fun handleConnectionInfo(info: WifiP2pInfo) {
        isGroupOwner = info.isGroupOwner
        
        Timber.d("Connection info - Group owner: $isGroupOwner, Owner address: ${info.groupOwnerAddress}")
        
        if (isGroupOwner) {
            // We are group owner - start server
            startServer()
        } else {
            // We are client - connect to group owner
            connectToGroupOwner(info.groupOwnerAddress.hostAddress ?: "")
        }
    }
    
    private fun startServer() {
        scope.launch {
            try {
                serverSocket = ServerSocket(P2P_PORT)
                Timber.i("WiFi Direct server started on port $P2P_PORT")
                
                while (isRunning) {
                    val socket = serverSocket?.accept()
                    if (socket != null) {
                        val peerId = socket.inetAddress.hostAddress ?: "unknown"
                        Timber.d("Incoming WiFi Direct connection from $peerId")
                        
                        val connection = P2pConnection(peerId, socket)
                        activeConnections[peerId] = connection
                        connection.startReading()
                    }
                }
            } catch (e: Exception) {
                if (isRunning) {
                    Timber.e(e, "WiFi Direct server error")
                }
            }
        }
    }
    
    private fun connectToGroupOwner(address: String) {
        scope.launch {
            try {
                val socket = Socket()
                socket.connect(InetSocketAddress(address, P2P_PORT), 5000)
                
                Timber.i("Connected to WiFi Direct group owner at $address")
                
                val connection = P2pConnection(address, socket)
                activeConnections[address] = connection
                connection.startReading()
            } catch (e: Exception) {
                Timber.e(e, "Failed to connect to group owner at $address")
            }
        }
    }
    
    /**
     * Represents an active WiFi Direct connection.
     */
    private inner class P2pConnection(
        val peerId: String,
        private val socket: Socket
    ) {
        private val inputStream: InputStream = socket.getInputStream()
        private val outputStream: OutputStream = socket.getOutputStream()
        
        @Volatile
        private var isReading = false
        
        fun startReading() {
            if (isReading) return
            
            isReading = true
            
            scope.launch {
                try {
                    val buffer = ByteArray(8192)
                    
                    while (isReading && socket.isConnected) {
                        val bytesRead = inputStream.read(buffer)
                        if (bytesRead > 0) {
                            val data = buffer.copyOfRange(0, bytesRead)
                            onDataReceived(peerId, data)
                        } else if (bytesRead < 0) {
                            break
                        }
                    }
                } catch (e: Exception) {
                    if (isReading) {
                        Timber.e(e, "WiFi Direct read error from $peerId")
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
                true
            } catch (e: Exception) {
                Timber.e(e, "Failed to send WiFi Direct data to $peerId")
                false
            }
        }
        
        fun close() {
            isReading = false
            try {
                socket.close()
            } catch (e: Exception) {
                Timber.w(e, "Error closing WiFi Direct socket")
            }
            activeConnections.remove(peerId)
        }
    }
    
    fun cleanup() {
        stop()
        scope.cancel()
    }
    
    companion object {
        private const val SERVICE_NAME = "scmessenger"
        private const val SERVICE_TYPE = "_scmessenger._tcp"
        private const val P2P_PORT = 8888
    }
}
