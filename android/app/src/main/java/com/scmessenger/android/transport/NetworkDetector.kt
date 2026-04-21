package com.scmessenger.android.transport

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.Build
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.Dispatchers
import timber.log.Timber
import java.util.concurrent.ConcurrentHashMap
import javax.inject.Inject
import javax.inject.Singleton

/**
 * P0_NETWORK_001: Network detection for cellular-aware transport selection.
 *
 * Detects network type (cellular vs WiFi) and identifies commonly blocked
 * ports to enable intelligent fallback from QUIC/TCP on non-standard ports
 * to WebSocket on standard HTTP ports (80/443).
 */
@Singleton
class NetworkDetector @Inject constructor(
    private val context: Context
) {
    private val connectivityManager =
        context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

    /** Current network type detected */
    private val _networkType = MutableStateFlow(NetworkType.UNKNOWN)
    val networkType: StateFlow<NetworkType> = _networkType.asStateFlow()

    /** Currently blocked ports (populated on cellular networks) */
    private val _blockedPorts = MutableStateFlow<Set<Int>>(emptySet())
    val blockedPorts: StateFlow<Set<Int>> = _blockedPorts.asStateFlow()

    /** Network capability cache */
    private val networkCapabilities = ConcurrentHashMap<Network, NetworkCapabilities>()

    /** Network callback for real-time updates */
    private var networkCallback: ConnectivityManager.NetworkCallback? = null

    /** Whether we've detected a cellular network that likely blocks non-standard ports */
    val isCellularNetwork: Boolean
        get() = _networkType.value == NetworkType.CELLULAR

    /** Whether we should prefer WebSocket on standard ports */
    val shouldPreferWebSocket: Boolean
        get() = isCellularNetwork || _blockedPorts.value.isNotEmpty()

    /** Ports commonly blocked by cellular carriers */
    private val commonlyBlockedPorts = setOf(
        9001, // SCMessenger TCP/QUIC
        9010, // SCMessenger TCP/QUIC (secondary)
        4001, // libp2p TCP default
        5001, // libp2p WebSocket default
    )

    /** Standard ports that are typically allowed on cellular */
    private val allowedStandardPorts = setOf(
        80,  // HTTP
        443, // HTTPS
    )

    /**
     * Start monitoring network changes.
     * Registers a NetworkCallback to detect network type changes in real-time.
     */
    fun startMonitoring() {
        if (networkCallback != null) {
            Timber.d("NetworkDetector already monitoring")
            return
        }

        val request = NetworkRequest.Builder()
            .addCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
            .build()

        val callback = object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: Network) {
                detectNetworkType(network)
            }

            override fun onLost(network: Network) {
                networkCapabilities.remove(network)
                Timber.d("Network lost: %s", network)
                redetectCurrentNetwork()
            }

            override fun onCapabilitiesChanged(
                network: Network,
                capabilities: NetworkCapabilities
            ) {
                networkCapabilities[network] = capabilities
                detectNetworkType(network)
            }
        }

        connectivityManager.registerNetworkCallback(request, callback)
        networkCallback = callback

        // Detect current network immediately
        redetectCurrentNetwork()

        Timber.i("NetworkDetector monitoring started")
    }

    /**
     * Stop monitoring network changes.
     */
    fun stopMonitoring() {
        networkCallback?.let {
            connectivityManager.unregisterNetworkCallback(it)
        }
        networkCallback = null
        Timber.i("NetworkDetector monitoring stopped")
    }

    /**
     * Detect the type of the active network.
     */
    private fun detectNetworkType(network: Network) {
        val capabilities = connectivityManager.getNetworkCapabilities(network) ?: return
        val type = classifyNetworkType(capabilities)

        _networkType.value = type

        // If cellular, populate blocked ports
        if (type == NetworkType.CELLULAR) {
            _blockedPorts.value = commonlyBlockedPorts
            Timber.w("Cellular network detected — blocking ports: %s", commonlyBlockedPorts)
        } else {
            _blockedPorts.value = emptySet()
        }

        Timber.d("Network type: %s, blocked ports: %s", type, _blockedPorts.value)
    }

    /**
     * Re-detect the current active network (e.g., after network loss).
     */
    private fun redetectCurrentNetwork() {
        val activeNetwork = connectivityManager.activeNetwork
        if (activeNetwork != null) {
            detectNetworkType(activeNetwork)
        } else {
            _networkType.value = NetworkType.UNKNOWN
            _blockedPorts.value = emptySet()
        }
    }

    /**
     * Classify network type from capabilities.
     */
    private fun classifyNetworkType(capabilities: NetworkCapabilities): NetworkType {
        return when {
            capabilities.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) -> NetworkType.WIFI
            capabilities.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) -> NetworkType.CELLULAR
            capabilities.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) -> NetworkType.ETHERNET
            capabilities.hasTransport(NetworkCapabilities.TRANSPORT_BLUETOOTH) -> NetworkType.BLUETOOTH
            capabilities.hasTransport(NetworkCapabilities.TRANSPORT_VPN) -> NetworkType.VPN
            else -> NetworkType.UNKNOWN
        }
    }

    /**
     * Check if a specific port is likely blocked on the current network.
     */
    fun isPortLikelyBlocked(port: Int): Boolean {
        // On cellular, non-standard ports are commonly blocked
        return isCellularNetwork && port !in allowedStandardPorts
    }

    /**
     * Get the recommended transport protocol for the current network.
     *
     * Returns the transport priority order:
     * - WiFi: QUIC → TCP → WebSocket (full connectivity expected)
     * - Cellular: WebSocket(443) → TCP(443) → QUIC → TCP (standard ports first)
     */
    fun getTransportPriority(): List<FallbackTransport> {
        return when (_networkType.value) {
            NetworkType.WIFI, NetworkType.ETHERNET -> listOf(
                FallbackTransport.QUIC,
                FallbackTransport.TCP,
                FallbackTransport.WEBSOCKET_WSS,
                FallbackTransport.WEBSOCKET_WS,
            )
            NetworkType.CELLULAR -> listOf(
                // On cellular, prefer WebSocket on standard ports first
                FallbackTransport.WEBSOCKET_WSS,
                FallbackTransport.TCP_STANDARD,
                FallbackTransport.QUIC,
                FallbackTransport.TCP,
                FallbackTransport.WEBSOCKET_WS,
            )
            NetworkType.BLUETOOTH, NetworkType.VPN -> listOf(
                FallbackTransport.WEBSOCKET_WSS,
                FallbackTransport.QUIC,
                FallbackTransport.TCP,
            )
            NetworkType.UNKNOWN -> listOf(
                FallbackTransport.WEBSOCKET_WSS,
                FallbackTransport.QUIC,
                FallbackTransport.TCP,
            )
        }
    }

    /**
     * Get detailed network diagnostics for logging.
     */
    fun getNetworkDiagnostics(): NetworkDiagnostics {
        val activeNetwork = connectivityManager.activeNetwork
        val capabilities = activeNetwork?.let {
            connectivityManager.getNetworkCapabilities(it)
        }

        return NetworkDiagnostics(
            networkType = _networkType.value,
            blockedPorts = _blockedPorts.value,
            hasInternet = capabilities?.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) ?: false,
            hasValidated = capabilities?.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED) ?: false,
            isMetered = !(
                capabilities?.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_METERED) ?: false
            ),
            upstreamBandwidth = capabilities?.linkUpstreamBandwidthKbps ?: 0,
            downstreamBandwidth = capabilities?.linkDownstreamBandwidthKbps ?: 0,
            recommendedTransports = getTransportPriority()
        )
    }

    /**
     * P0_NETWORK_001: Probe a set of host:port pairs for reachability.
     * Uses parallel TCP socket probes with a short timeout.
     * Returns a map of "host:port" to reachability (true = reachable).
     * Advisory only — used to deprioritize blocked addresses, not exclude them.
     */
    suspend fun probePorts(
        targets: List<Pair<String, Int>>,
        timeoutMs: Long = 1500L
    ): Map<String, Boolean> = kotlinx.coroutines.coroutineScope {
        val results = ConcurrentHashMap<String, Boolean>()
        targets.map { (host, port) ->
            kotlinx.coroutines.async(kotlinx.coroutines.Dispatchers.IO) {
                val key = "$host:$port"
                val reachable = try {
                    val socket = java.net.Socket()
                    socket.connect(java.net.InetSocketAddress(host, port), timeoutMs.toInt())
                    socket.close()
                    true
                } catch (_: Exception) {
                    false
                }
                results[key] = reachable
                Timber.d("Port probe: %s = %s", key, if (reachable) "open" else "blocked")
            }
        }.awaitAll()
        results.toMap()
    }

    companion object {
        private const val TAG = "NetworkDetector"
    }
}

/** Network types detected by the NetworkDetector */
enum class NetworkType {
    WIFI,
    CELLULAR,
    ETHERNET,
    BLUETOOTH,
    VPN,
    UNKNOWN
}

/** Fallback transport protocols in priority order */
enum class FallbackTransport(val scheme: String, val defaultPort: Int) {
    QUIC("quic", 9001),
    TCP("tcp", 9001),
    TCP_STANDARD("tcp", 443),
    WEBSOCKET_WS("ws", 80),
    WEBSOCKET_WSS("wss", 443)
}

/** Network diagnostic information */
data class NetworkDiagnostics(
    val networkType: NetworkType,
    val blockedPorts: Set<Int>,
    val hasInternet: Boolean,
    val hasValidated: Boolean,
    val isMetered: Boolean,
    val upstreamBandwidth: Int,
    val downstreamBandwidth: Int,
    val recommendedTransports: List<FallbackTransport>
) {
    /** Format diagnostics for logging */
    fun toLogString(): String {
        return """
            |Network Diagnostics:
            |  Type: $networkType
            |  Has Internet: $hasInternet
            |  Validated: $hasValidated
            |  Metered: $isMetered
            |  Blocked Ports: $blockedPorts
            |  Bandwidth: ${downstreamBandwidth}Kbps down / ${upstreamBandwidth}Kbps up
            |  Transport Priority: ${recommendedTransports.joinToString(" → ")}
        """.trimMargin()
    }
}