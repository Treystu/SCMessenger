package com.scmessenger.android.transport

import android.annotation.TargetApi
import android.content.Context
import android.net.*
import android.net.wifi.aware.*
import android.os.Build
import timber.log.Timber
import java.io.InputStream
import java.io.OutputStream
import java.net.InetSocketAddress
import java.net.ServerSocket
import java.net.Socket
import java.util.concurrent.ConcurrentHashMap
import kotlinx.coroutines.*

/**
 * WiFi Aware transport implementation (API 26+).
 *
 * WiFi Aware enables device-to-device discovery and communication without
 * requiring traditional WiFi infrastructure or pairing.
 *
 * Process:
 * 1. Attach to WiFi Aware system
 * 2. Publish SCMessenger service (as provider)
 * 3. Subscribe to SCMessenger service (as consumer)
 * 4. On discovery, create data path via NetworkRequest
 * 5. Use socket for bi-directional data exchange
 *
 * Gracefully falls back on unsupported devices.
 */
@TargetApi(Build.VERSION_CODES.O)
class WifiAwareTransport(
    private val context: Context,
    private val onPeerDiscovered: (peerId: String) -> Unit,
    private val onDataReceived: (peerId: String, data: ByteArray) -> Unit
) {

    private val wifiAwareManager: WifiAwareManager? = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
        context.getSystemService(Context.WIFI_AWARE_SERVICE) as? WifiAwareManager
    } else {
        null
    }

    private val connectivityManager = context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

    private var awareSession: WifiAwareSession? = null
    private var publishSession: PublishDiscoverySession? = null
    private var subscribeSession: SubscribeDiscoverySession? = null

    private val activeConnections = ConcurrentHashMap<String, AwareConnection>()

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    private var isRunning = false

    private val callbackLock = Any()
    private val registeredCallbacks = ConcurrentHashMap<String, ConnectivityManager.NetworkCallback>()

    // Guards the initiator path in onCapabilitiesChanged against duplicate launches.
    // putIfAbsent acts as an atomic check-and-set, preventing a second callback
    // from racing past the activeConnections.containsKey guard before the first
    // coroutine has inserted the completed connection.
    private val pendingInitiators = ConcurrentHashMap<String, Boolean>()

    /**
     * Check if WiFi Aware is available on this device.
     */
    fun isAvailable(): Boolean {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) {
            return false
        }

        return wifiAwareManager?.isAvailable == true
    }

    /**
     * Start WiFi Aware transport.
     * Attaches to WiFi Aware, publishes and subscribes to SCMessenger service.
     */
    fun start() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) {
            Timber.w("WiFi Aware not supported on this device (API < 26)")
            return
        }

        if (isRunning) {
            Timber.w("WiFi Aware already running")
            return
        }

        if (!isAvailable()) {
            Timber.w("WiFi Aware not available on this device")
            return
        }

        try {
            wifiAwareManager?.attach(attachCallback, null)
            Timber.i("Attaching to WiFi Aware...")
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception attaching to WiFi Aware - missing permissions?")
        } catch (e: Exception) {
            Timber.e(e, "Failed to attach to WiFi Aware")
        }
    }

    /**
     * Stop WiFi Aware transport.
     */
    fun stop() {
        if (!isRunning) {
            return
        }

        isRunning = false

        scope.cancel()

        // Close all connections
        activeConnections.values.forEach { it.close() }
        activeConnections.clear()

        // Stop publish/subscribe sessions
        publishSession?.close()
        subscribeSession?.close()

        // Unregister network callbacks
        val callbacksToUnregister = synchronized(callbackLock) {
            val callbacks = registeredCallbacks.values.toList()
            registeredCallbacks.clear()
            callbacks
        }
        callbacksToUnregister.forEach { callback ->
            try {
                connectivityManager.unregisterNetworkCallback(callback)
            } catch (e: Exception) {
                Timber.w(e, "Failed to unregister network callback")
            }
        }

        // Detach from WiFi Aware
        awareSession?.close()

        publishSession = null
        subscribeSession = null
        awareSession = null

        Timber.i("WiFi Aware stopped")
    }

    /**
     * Send data to a peer via WiFi Aware.
     */
    fun sendData(peerId: String, data: ByteArray): Boolean {
        val connection = activeConnections[peerId] ?: run {
            Timber.w("No WiFi Aware connection to $peerId")
            return false
        }

        return connection.send(data)
    }

    private val attachCallback = object : AttachCallback() {
        override fun onAttached(session: WifiAwareSession) {
            super.onAttached(session)

            awareSession = session
            isRunning = true

            Timber.i("WiFi Aware attached successfully")

            // Start publishing our service
            startPublishing()

            // Start subscribing to discover peers
            startSubscribing()
        }

        override fun onAttachFailed() {
            super.onAttachFailed()
            Timber.e("WiFi Aware attach failed")
            isRunning = false
        }
    }

    private fun startPublishing() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return

        val config = PublishConfig.Builder()
            .setServiceName(SERVICE_NAME)
            .setPublishType(PublishConfig.PUBLISH_TYPE_UNSOLICITED)
            .build()

        try {
            awareSession?.publish(config, publishDiscoveryCallback, null)
            Timber.d("Publishing WiFi Aware service: $SERVICE_NAME")
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception publishing WiFi Aware service")
        } catch (e: Exception) {
            Timber.e(e, "Failed to publish WiFi Aware service")
        }
    }

    private fun startSubscribing() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return

        val config = SubscribeConfig.Builder()
            .setServiceName(SERVICE_NAME)
            .setSubscribeType(SubscribeConfig.SUBSCRIBE_TYPE_PASSIVE)
            .build()

        try {
            awareSession?.subscribe(config, subscribeDiscoveryCallback, null)
            Timber.d("Subscribing to WiFi Aware service: $SERVICE_NAME")
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception subscribing to WiFi Aware service")
        } catch (e: Exception) {
            Timber.e(e, "Failed to subscribe to WiFi Aware service")
        }
    }

    private val publishDiscoveryCallback = object : DiscoverySessionCallback() {
        override fun onPublishStarted(session: PublishDiscoverySession) {
            super.onPublishStarted(session)
            publishSession = session
            Timber.i("WiFi Aware publish started")
        }

        override fun onServiceDiscovered(peerId: PeerHandle, serviceSpecificInfo: ByteArray?, matchFilter: MutableList<ByteArray>?) {
            super.onServiceDiscovered(peerId, serviceSpecificInfo, matchFilter)

            Timber.d("Peer discovered via WiFi Aware: $peerId")

            // Notify discovery
            val peerIdString = peerId.toString()
            onPeerDiscovered(peerIdString)

            // Initiate data path — Publisher is the RESPONDER (server socket)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val session = publishSession
                if (session != null) {
                    initiateDataPath(session, peerId, peerIdString, isPublisher = true)
                } else {
                    Timber.w("Publish session unavailable for WiFi Aware data path to $peerIdString")
                }
            }
        }
    }

    private val subscribeDiscoveryCallback = object : DiscoverySessionCallback() {
        override fun onSubscribeStarted(session: SubscribeDiscoverySession) {
            super.onSubscribeStarted(session)
            subscribeSession = session
            Timber.i("WiFi Aware subscribe started")
        }

        override fun onServiceDiscovered(peerId: PeerHandle, serviceSpecificInfo: ByteArray?, matchFilter: MutableList<ByteArray>?) {
            super.onServiceDiscovered(peerId, serviceSpecificInfo, matchFilter)

            Timber.d("Service discovered via WiFi Aware: $peerId")

            // Notify discovery
            val peerIdString = peerId.toString()
            onPeerDiscovered(peerIdString)

            // Initiate data path — Subscriber is the INITIATOR (client socket)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val session = subscribeSession
                if (session != null) {
                    initiateDataPath(session, peerId, peerIdString, isPublisher = false)
                } else {
                    Timber.w("Subscribe session unavailable for WiFi Aware data path to $peerIdString")
                }
            }
        }
    }

    @TargetApi(Build.VERSION_CODES.Q)
    private fun initiateDataPath(
        session: DiscoverySession,
        peerHandle: PeerHandle,
        peerIdString: String,
        isPublisher: Boolean
    ) {
        val config = WifiAwareNetworkSpecifier.Builder(session, peerHandle)
            .build()

        val request = NetworkRequest.Builder()
            .addTransportType(NetworkCapabilities.TRANSPORT_WIFI_AWARE)
            .setNetworkSpecifier(config)
            .build()

        val callback = object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: Network) {
                super.onAvailable(network)
                Timber.i("WiFi Aware data path available for $peerIdString (publisher=$isPublisher)")

                if (isPublisher) {
                    // Publisher is RESPONDER: open a ServerSocket and wait for the initiator
                    scope.launch {
                        createResponderSocket(network, peerIdString)
                    }
                }
                // Initiator path is deferred to onCapabilitiesChanged where peer IPv6 is available
            }

            override fun onCapabilitiesChanged(network: Network, networkCapabilities: NetworkCapabilities) {
                super.onCapabilitiesChanged(network, networkCapabilities)

                if (!isPublisher) {
                    // Subscriber is INITIATOR: extract peer IPv6 from WifiAwareNetworkInfo
                    // and connect as a client socket
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                        val info = networkCapabilities.transportInfo as? WifiAwareNetworkInfo
                        val peerIpv6 = info?.peerIpv6Addr
                        if (peerIpv6 != null &&
                            !activeConnections.containsKey(peerIdString) &&
                            pendingInitiators.putIfAbsent(peerIdString, true) == null
                        ) {
                            Timber.d("WiFi Aware initiator: peer IPv6=$peerIpv6 for $peerIdString")
                            scope.launch {
                                try {
                                    createInitiatorSocket(network, peerIdString, peerIpv6.hostAddress ?: return@launch)
                                } finally {
                                    pendingInitiators.remove(peerIdString)
                                }
                            }
                        }
                    }
                }
            }

            override fun onLost(network: Network) {
                super.onLost(network)
                Timber.d("WiFi Aware data path lost for $peerIdString")
                activeConnections.remove(peerIdString)?.close()
                val callbackToRemove = synchronized(callbackLock) {
                    registeredCallbacks.remove(peerIdString)
                }
                if (callbackToRemove != null) {
                    try {
                        connectivityManager.unregisterNetworkCallback(callbackToRemove)
                    } catch (e: Exception) {
                        Timber.w(e, "Failed to unregister network callback for $peerIdString")
                    }
                }
            }
        }

        val existingCallback = synchronized(callbackLock) {
            registeredCallbacks.put(peerIdString, callback)
        }
        existingCallback?.let {
            try {
                connectivityManager.unregisterNetworkCallback(it)
            } catch (e: Exception) {
                Timber.w(e, "Failed to replace existing network callback for $peerIdString")
            }
        }
        connectivityManager.requestNetwork(request, callback)
    }

    /**
     * Publisher (RESPONDER) path: open a ServerSocket bound to the Aware network,
     * accept the single incoming connection from the Subscriber, then close the
     * server socket and hand the accepted socket to AwareConnection.
     */
    private suspend fun createResponderSocket(network: Network, peerId: String) {
        withContext(Dispatchers.IO) {
            var serverSocket: ServerSocket? = null
            try {
                serverSocket = ServerSocket(AWARE_PORT)
                network.bindSocket(serverSocket)
                // Prevent accept() from blocking indefinitely if the initiator never connects.
                serverSocket.soTimeout = CONNECT_TIMEOUT_MS

                Timber.d("WiFi Aware responder waiting for connection from $peerId on port $AWARE_PORT")

                val socket = serverSocket.accept()
                serverSocket.close()

                val connection = AwareConnection(peerId, socket)
                activeConnections[peerId] = connection
                connection.startReading()

                Timber.i("WiFi Aware responder connected to $peerId")
            } catch (e: Exception) {
                serverSocket?.close()
                Timber.e(e, "Failed to create WiFi Aware responder socket for $peerId")
            }
        }
    }

    /**
     * Subscriber (INITIATOR) path: connect a client Socket to the Responder's
     * well-known port using the peer's link-local IPv6 address obtained from
     * WifiAwareNetworkInfo in onCapabilitiesChanged.
     */
    private suspend fun createInitiatorSocket(network: Network, peerId: String, peerIpv6: String) {
        withContext(Dispatchers.IO) {
            try {
                val socket = network.socketFactory.createSocket()
                socket.connect(InetSocketAddress(peerIpv6, AWARE_PORT), CONNECT_TIMEOUT_MS)

                val connection = AwareConnection(peerId, socket)
                activeConnections[peerId] = connection
                connection.startReading()

                Timber.i("WiFi Aware initiator connected to $peerId at [$peerIpv6]:$AWARE_PORT")
            } catch (e: Exception) {
                Timber.e(e, "Failed to create WiFi Aware initiator socket for $peerId at [$peerIpv6]")
            }
        }
    }

    /**
     * Represents an active WiFi Aware socket connection.
     */
    private inner class AwareConnection(
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
                        Timber.e(e, "WiFi Aware read error from $peerId")
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
                Timber.e(e, "Failed to send WiFi Aware data to $peerId")
                false
            }
        }

        fun close() {
            isReading = false
            try {
                socket.close()
            } catch (e: Exception) {
                Timber.w(e, "Error closing WiFi Aware socket")
            }
        }
    }

    fun cleanup() {
        stop()
        scope.cancel()
    }

    companion object {
        private const val SERVICE_NAME = "scmessenger"
        private const val AWARE_PORT = 8765
        private const val CONNECT_TIMEOUT_MS = 5000
    }
}
