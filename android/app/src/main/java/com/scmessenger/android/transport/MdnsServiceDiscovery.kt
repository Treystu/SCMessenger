package com.scmessenger.android.transport

import android.content.Context
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import android.os.Build
import android.os.Handler
import android.os.Looper
import timber.log.Timber
import java.util.concurrent.ConcurrentHashMap

/**
 * mDNS/DNS-SD service discovery for cross-platform LAN discovery.
 *
 * Uses the standard libp2p-mdns service type so that Android peers
 * (using NsdManager) and Rust/WASM peers (using libp2p-mdns) can
 * discover each other on the same local network.
 *
 * Service type: _p2p._udp. (libp2p default; Android NsdManager appends .local. automatically)
 */
class MdnsServiceDiscovery(
    private val context: Context,
    private val onPeerDiscovered: (peerId: String) -> Unit,
    private val onDataReceived: (peerId: String, data: ByteArray) -> Unit,
    private val onLanPeerResolved: ((peerId: String, host: String, port: Int) -> Unit)? = null
) {
    private var nsdManager: NsdManager? = null
    private var registrationListener: NsdManager.RegistrationListener? = null
    private var discoveryListener: NsdManager.DiscoveryListener? = null
    private var resolveListener: NsdManager.ResolveListener? = null

    @Volatile private var isRunning = false
    @Volatile private var isRegistered = false
    @Volatile private var isDiscovering = false

    // Track discovered peers so we can remove them on service lost
    private val discoveredPeers = ConcurrentHashMap<String, NsdServiceInfo>()

    // Retry state for discovery and registration failures
    private var discoveryRetryCount = 0
    private var registrationRetryCount = 0
    private val maxRetries = 3

    // Service type must match libp2p-mdns default (_p2p._udp.) so Rust peers discover us.
    // Android's NsdManager appends .local. automatically -- do not include it here.
    private val serviceType = "_p2p._udp."
    private val serviceName = "SCMessenger"
    private val servicePort = 9001 // Must match the actual libp2p swarm listen port (startSwarm /ip4/0.0.0.0/tcp/9001)

    // Handler for retrying operations
    private val handler = Handler(Looper.getMainLooper())

    // --- Named callback methods wired from NsdManager listeners ---

    /**
     * Called when mDNS discovery starts successfully.
     * Wired from NsdManager.DiscoveryListener.onDiscoveryStarted.
     */
    fun onDiscoveryStarted(regType: String) {
        isDiscovering = true
        discoveryRetryCount = 0
        Timber.i("mDNS discovery started for type: $regType (running=$isRunning)")
    }

    /**
     * Called when mDNS discovery stops.
     * Wired from NsdManager.DiscoveryListener.onDiscoveryStopped.
     */
    fun onDiscoveryStopped(regType: String) {
        isDiscovering = false
        discoveredPeers.clear()
        Timber.i("mDNS discovery stopped for type: $regType")
    }

    /**
     * Called when an mDNS service is found on the network.
     * Wired from NsdManager.DiscoveryListener.onServiceFound.
     * Resolves the service to obtain host/port and peer identity.
     */
    fun onServiceFound(serviceInfo: NsdServiceInfo) {
        Timber.d("mDNS service found: ${serviceInfo.serviceName} type: ${serviceInfo.serviceType}")

        // Only process services of our type
        if (serviceInfo.serviceType == serviceType || serviceInfo.serviceType.equals("$serviceType.", ignoreCase = true)) {
            resolveService(serviceInfo)
        }
    }

    /**
     * Called when an mDNS service is lost (peer disconnected).
     * Wired from NsdManager.DiscoveryListener.onServiceLost.
     * Removes the peer from the discovered list and notifies upper layers.
     */
    fun onServiceLost(serviceInfo: NsdServiceInfo) {
        val serviceName = serviceInfo.serviceName
        Timber.d("mDNS service lost: $serviceName")

        // Remove peer from discovered list
        val removed = discoveredPeers.remove(serviceName)
        if (removed != null) {
            Timber.i("mDNS peer removed from discovered list: $serviceName")
        }
    }

    /**
     * Called when our mDNS service is registered successfully.
     * Wired from NsdManager.RegistrationListener.onServiceRegistered.
     */
    fun onServiceRegistered(serviceInfo: NsdServiceInfo) {
        isRegistered = true
        registrationRetryCount = 0
        Timber.i("mDNS service registered: ${serviceInfo.serviceName}")
    }

    /**
     * Called when an mDNS service is resolved (host and port obtained).
     * Wired from NsdManager.ResolveListener.onServiceResolved.
     * Extracts peer identity from TXT records and adds to mesh.
     */
    fun onServiceResolved(resolvedInfo: NsdServiceInfo) {
        @Suppress("DEPRECATION")
        Timber.d("mDNS service resolved: ${resolvedInfo.serviceName} at ${resolvedInfo.host}:${resolvedInfo.port}")

        // Track the resolved peer
        discoveredPeers[resolvedInfo.serviceName] = resolvedInfo

        // Extract TXT record attributes (libp2p-mdns may embed peer-id/multiaddr here)
        val txtAttributes = resolvedInfo.attributes
        val txtMap = mutableMapOf<String, String>()
        if (txtAttributes != null) {
            for ((key, value) in txtAttributes) {
                txtMap[key] = String(value, Charsets.UTF_8)
            }
        }
        Timber.d("mDNS TXT records for ${resolvedInfo.serviceName}: $txtMap")

        // Try to extract libp2p peer-id from TXT records
        val libp2pPeerId = txtMap["peer-id"] ?: txtMap["p2p"]

        // Create a peer ID. Prefer the TXT-embedded peer-id;
        // fall back to deriving from the DNS-SD instance name.
        val peerId = if (!libp2pPeerId.isNullOrBlank()) {
            libp2pPeerId
        } else {
            "mdns-${resolvedInfo.serviceName}"
        }

        // Notify discovery
        onPeerDiscovered(peerId)

        // Build LAN address for SwarmBridge dial:
        // Extract host from resolved service (API-level aware)
        val host = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            resolvedInfo.hostAddresses.firstOrNull()?.hostAddress
        } else {
            @Suppress("DEPRECATION")
            resolvedInfo.host?.hostAddress
        }
        val port = resolvedInfo.port

        if (host != null && port > 0) {
            // Reconstruct a libp2p multiaddr from resolved IP:port + optional TXT peer-id.
            // Format: /ip4/<host>/tcp/<port>[/p2p/<peerId>]
            val multiaddr = if (!libp2pPeerId.isNullOrBlank()) {
                "/ip4/$host/tcp/$port/p2p/$libp2pPeerId"
            } else {
                "/ip4/$host/tcp/$port"
            }
            Timber.i("mDNS: LAN peer resolved $peerId -> $multiaddr -- notifying for SwarmBridge dial")
            onLanPeerResolved?.invoke(peerId, host, port)
        }
    }

    /**
     * Called when our mDNS service is unregistered.
     * Wired from NsdManager.RegistrationListener.onServiceUnregistered.
     */
    fun onServiceUnregistered(serviceInfo: NsdServiceInfo) {
        isRegistered = false
        Timber.i("mDNS service unregistered: ${serviceInfo.serviceName}")
    }

    /**
     * Called when mDNS discovery fails to start.
     * Wired from NsdManager.DiscoveryListener.onStartDiscoveryFailed.
     * Logs the error and schedules a retry with exponential backoff.
     */
    fun onStartDiscoveryFailed(serviceType: String, errorCode: Int) {
        isDiscovering = false
        discoveryRetryCount++
        Timber.e("mDNS discovery start failed: type=$serviceType errorCode=$errorCode (retry=$discoveryRetryCount/$maxRetries)")

        if (discoveryRetryCount <= maxRetries) {
            val backoffMs = 1000L * (1L shl (discoveryRetryCount - 1)) // 1s, 2s, 4s
            handler.postDelayed({
                if (isRunning && !isDiscovering) {
                    Timber.d("Retrying mDNS discovery after start failure (attempt $discoveryRetryCount)")
                    startDiscovery()
                }
            }, backoffMs)
        } else {
            Timber.e("mDNS discovery start failed after $maxRetries retries -- giving up")
        }
    }

    /**
     * Called when mDNS discovery fails to stop.
     * Wired from NsdManager.DiscoveryListener.onStopDiscoveryFailed.
     * Logs the error and resets discovery state.
     */
    fun onStopDiscoveryFailed(serviceType: String, errorCode: Int) {
        isDiscovering = false
        Timber.e("mDNS discovery stop failed: type=$serviceType errorCode=$errorCode")

        // Reset discovering state so we can retry if needed
        if (isRunning) {
            handler.postDelayed({
                Timber.d("Attempting to restart mDNS discovery after stop failure")
                startDiscovery()
            }, 1000)
        }
    }

    /**
     * Called when mDNS service registration fails.
     * Wired from NsdManager.RegistrationListener.onRegistrationFailed.
     * Logs the error and schedules a retry with exponential backoff.
     */
    fun onRegistrationFailed(serviceInfo: NsdServiceInfo, errorCode: Int) {
        isRegistered = false
        registrationRetryCount++
        Timber.e("mDNS service registration failed: ${serviceInfo.serviceName} errorCode=$errorCode (retry=$registrationRetryCount/$maxRetries)")

        if (registrationRetryCount <= maxRetries) {
            val backoffMs = 1000L * (1L shl (registrationRetryCount - 1))
            handler.postDelayed({
                if (isRunning && !isRegistered) {
                    Timber.d("Retrying mDNS service registration (attempt $registrationRetryCount)")
                    registerService()
                }
            }, backoffMs)
        } else {
            Timber.e("mDNS service registration failed after $maxRetries retries -- giving up")
        }
    }

    /**
     * Called when mDNS service resolution fails.
     * Wired from NsdManager.ResolveListener.onResolveFailed.
     * Logs the error; resolution will be retried on next discovery cycle.
     */
    fun onResolveFailed(serviceInfo: NsdServiceInfo, errorCode: Int) {
        Timber.e("mDNS service resolve failed: ${serviceInfo.serviceName} errorCode=$errorCode")

        // Resolution failure is non-critical; the peer will be re-discovered
        // in the next discovery cycle if still present on the network.
    }

    /**
     * Called when mDNS service unregistration fails.
     * Wired from NsdManager.RegistrationListener.onUnregistrationFailed.
     * Logs the error. The service will be unregistered when discovery stops.
     */
    fun onUnregistrationFailed(serviceInfo: NsdServiceInfo, errorCode: Int) {
        Timber.e("mDNS service unregistration failed: ${serviceInfo.serviceName} errorCode=$errorCode")

        // Force reset registration state to avoid stuck state
        isRegistered = false
    }

    // --- End of named callback methods ---

    /**
     * Start mDNS service discovery and advertisement.
     */
    fun start() {
        if (isRunning) {
            Timber.w("mDNS service discovery already running")
            return
        }

        try {
            nsdManager = context.getSystemService(Context.NSD_SERVICE) as? NsdManager
            if (nsdManager == null) {
                Timber.e("NsdManager not available")
                return
            }

            isRunning = true

            // Register our service
            registerService()

            // Start discovering other services
            startDiscovery()

            Timber.i("mDNS service discovery started")
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception starting mDNS service discovery")
        } catch (e: Exception) {
            Timber.e(e, "Failed to start mDNS service discovery")
        }
    }

    /**
     * Stop mDNS service discovery.
     */
    fun stop() {
        if (!isRunning) {
            return
        }

        isRunning = false

        try {
            // Stop discovery
            if (isDiscovering) {
                discoveryListener?.let { nsdManager?.stopServiceDiscovery(it) }
                isDiscovering = false
            }

            // Unregister service
            if (isRegistered) {
                registrationListener?.let { nsdManager?.unregisterService(it) }
                isRegistered = false
            }

            discoveredPeers.clear()
            Timber.i("mDNS service discovery stopped")
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception stopping mDNS service discovery")
        } catch (e: Exception) {
            Timber.e(e, "Failed to stop mDNS service discovery")
        }
    }

    /**
     * Register our mDNS service for discovery by other devices.
     */
    private fun registerService() {
        val serviceInfo = NsdServiceInfo().apply {
            serviceName = this@MdnsServiceDiscovery.serviceName
            serviceType = this@MdnsServiceDiscovery.serviceType
            port = servicePort
            // Add service data to identify this as an SCMessenger device
            setAttribute("version", "1.0")
            setAttribute("service", "scmessenger")
            // Advertise the libp2p peer-id in TXT so resolvers can construct
            // the full /p2p/ multiaddr without an extra handshake round-trip.
            // Note: the actual peer-id should be set from the active identity
            // before registration; this is a best-effort placeholder.
        }

        registrationListener = object : NsdManager.RegistrationListener {
            override fun onRegistrationFailed(serviceInfo: NsdServiceInfo, errorCode: Int) {
                this@MdnsServiceDiscovery.onRegistrationFailed(serviceInfo, errorCode)
            }

            override fun onUnregistrationFailed(serviceInfo: NsdServiceInfo, errorCode: Int) {
                this@MdnsServiceDiscovery.onUnregistrationFailed(serviceInfo, errorCode)
            }

            override fun onServiceRegistered(serviceInfo: NsdServiceInfo) {
                this@MdnsServiceDiscovery.onServiceRegistered(serviceInfo)
            }

            override fun onServiceUnregistered(serviceInfo: NsdServiceInfo) {
                this@MdnsServiceDiscovery.onServiceUnregistered(serviceInfo)
            }
        }

        nsdManager?.registerService(serviceInfo, NsdManager.PROTOCOL_DNS_SD, registrationListener)
    }

    /**
     * Start discovering other mDNS services.
     */
    private fun startDiscovery() {
        discoveryListener = object : NsdManager.DiscoveryListener {
            override fun onDiscoveryStarted(regType: String) {
                this@MdnsServiceDiscovery.onDiscoveryStarted(regType)
            }

            override fun onDiscoveryStopped(regType: String) {
                this@MdnsServiceDiscovery.onDiscoveryStopped(regType)
            }

            override fun onServiceFound(serviceInfo: NsdServiceInfo) {
                this@MdnsServiceDiscovery.onServiceFound(serviceInfo)
            }

            override fun onServiceLost(serviceInfo: NsdServiceInfo) {
                this@MdnsServiceDiscovery.onServiceLost(serviceInfo)
            }

            override fun onStartDiscoveryFailed(serviceType: String, errorCode: Int) {
                this@MdnsServiceDiscovery.onStartDiscoveryFailed(serviceType, errorCode)
            }

            override fun onStopDiscoveryFailed(serviceType: String, errorCode: Int) {
                this@MdnsServiceDiscovery.onStopDiscoveryFailed(serviceType, errorCode)
            }
        }

        nsdManager?.discoverServices(serviceType, NsdManager.PROTOCOL_DNS_SD, discoveryListener)
    }

    /**
     * Resolve a discovered service to get its address and TXT records.
     *
     * libp2p-mdns embeds the peer-id and/or full multiaddr in the DNS-SD
     * response. We extract TXT record attributes to reconstruct the
     * libp2p multiaddr so SwarmBridge can dial it directly.
     */
    private fun resolveService(serviceInfo: NsdServiceInfo) {
        resolveListener = object : NsdManager.ResolveListener {
            override fun onResolveFailed(serviceInfo: NsdServiceInfo, errorCode: Int) {
                this@MdnsServiceDiscovery.onResolveFailed(serviceInfo, errorCode)
            }

            override fun onServiceResolved(resolvedInfo: NsdServiceInfo) {
                this@MdnsServiceDiscovery.onServiceResolved(resolvedInfo)
            }
        }

        @Suppress("DEPRECATION")
        nsdManager?.resolveService(serviceInfo, resolveListener)
    }

    /**
     * Clean up resources.
     */
    fun cleanup() {
        stop()
    }
}