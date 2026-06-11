package com.scmessenger.android.transport

import android.content.Context
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import android.os.Build
import android.os.Handler
import android.os.Looper
import androidx.core.content.ContextCompat
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
    private val onPeerDisconnected: ((peerId: String) -> Unit)? = null,
    private val onLanPeerResolved: ((peerId: String, host: String, port: Int) -> Unit)? = null,
    private val getLocalPeerId: (() -> String?)? = null
) {
    private var nsdManager: NsdManager? = null
    private var registrationListener: NsdManager.RegistrationListener? = null
    private var discoveryListener: NsdManager.DiscoveryListener? = null

    // P0_ANDROID_025: Track in-flight resolves by service name. NsdManager
    // rejects resolveService() calls that reuse a listener while a previous
    // resolve on the same listener is still in flight, throwing
    // IllegalArgumentException("listener already in use") on the
    // ConnectivityThread (crash). The previous code used a singleton listener
    // and crashed on the second onServiceFound. The canonical fix: a fresh
    // listener per resolveService() call, with the in-flight set guaranteeing
    // the listener instance is GC-eligible only after onComplete fires.
    private val inFlightResolves = ConcurrentHashMap<String, NsdManager.ResolveListener>()

    // Build a per-call listener. Each call gets a unique instance, and
    // the listener removes itself from the in-flight set on either terminal
    // callback. This avoids the "listener already in use" race entirely.
    private fun newResolveListener(serviceName: String): NsdManager.ResolveListener {
        return object : NsdManager.ResolveListener {
            override fun onResolveFailed(serviceInfo: NsdServiceInfo, errorCode: Int) {
                inFlightResolves.remove(serviceName)
                this@MdnsServiceDiscovery.onResolveFailed(serviceInfo, errorCode)
            }

            override fun onServiceResolved(resolvedInfo: NsdServiceInfo) {
                inFlightResolves.remove(resolvedInfo.serviceName)
                this@MdnsServiceDiscovery.onServiceResolved(resolvedInfo)
            }
        }
    }
    private var multicastLock: android.net.wifi.WifiManager.MulticastLock? = null

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
    private val serviceType = "_p2p._udp"
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
        val typeStripped = serviceInfo.serviceType.trimEnd('.')
        val targetStripped = serviceType.trimEnd('.')
        if (typeStripped.equals(targetStripped, ignoreCase = true)) {
            resolveService(serviceInfo)
        }
    }

    /**
     * Called when an mDNS service is lost (peer disconnected).
     * Wired from NsdManager.DiscoveryListener.onServiceLost.
     * Removes the peer from the discovered list and notifies upper layers
     * via onPeerDisconnected.
     *
     * P1 (Bug 5): Previously this only removed the entry from the local
     * `discoveredPeers` cache and never propagated the loss upward, so
     * MeshRepository.peersDisconnected never fired for mDNS-only peers and
     * the UI/connection state showed them as "connected" indefinitely. We
     * now derive the same peer-id that was emitted in onServiceResolved()
     * (TXT peer-id if present, else "mdns-<serviceName>") and forward the
     * disconnect to the upper layer.
     */
    fun onServiceLost(serviceInfo: NsdServiceInfo) {
        val serviceName = serviceInfo.serviceName
        Timber.d("mDNS service lost: $serviceName")

        // Remove peer from discovered list. If the peer was never resolved
        // (e.g. lost between onServiceFound and onServiceResolved), the
        // entry is absent — that's fine, we still forward a disconnect
        // hint below using the same id derivation.
        val removed = discoveredPeers.remove(serviceName)
        if (removed != null) {
            Timber.i("mDNS peer removed from discovered list: $serviceName")
        }

        // Also drop any in-flight resolve for this service so the listener
        // can't fire a stale onServiceResolved after we've already reported
        // the peer as gone (TOCTOU between lost and resolved).
        inFlightResolves.remove(serviceName)

        // Derive the same peer-id that onServiceResolved would have used,
        // so the upper layer can match by peer-id.
        val cachedAttributes = removed?.attributes
        val cachedPeerId = cachedAttributes?.get("peer-id")?.let { String(it, Charsets.UTF_8) }
            ?: cachedAttributes?.get("p2p")?.let { String(it, Charsets.UTF_8) }
        val peerId = if (!cachedPeerId.isNullOrBlank()) {
            cachedPeerId
        } else {
            "mdns-$serviceName"
        }
        onPeerDisconnected?.invoke(peerId)
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
        // Host is deprecated in API 33; use hostAddresses instead
        val hostAddress = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            resolvedInfo.hostAddresses.firstOrNull()?.hostAddress
        } else {
            resolvedInfo.host?.hostAddress
        }
        Timber.d("mDNS service resolved: ${resolvedInfo.serviceName} at ${hostAddress ?: "unknown"}:${resolvedInfo.port}")

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
        // host is deprecated in API 33; use hostAddresses instead
        val host = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            resolvedInfo.hostAddresses.firstOrNull()?.hostAddress
        } else {
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
            val wifiManager = context.applicationContext.getSystemService(Context.WIFI_SERVICE) as? android.net.wifi.WifiManager
            if (wifiManager != null) {
                multicastLock = wifiManager.createMulticastLock("scmessenger_mdns_lock").apply {
                    setReferenceCounted(true)
                    acquire()
                }
                Timber.i("mDNS Wifi MulticastLock acquired")
            }
        } catch (e: Exception) {
            Timber.w("Failed to acquire Wifi MulticastLock: ${e.message}")
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
            multicastLock?.let {
                if (it.isHeld) {
                    it.release()
                    Timber.i("mDNS Wifi MulticastLock released")
                }
            }
            multicastLock = null
        } catch (e: Exception) {
            Timber.w("Failed to release Wifi MulticastLock: ${e.message}")
        }

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
            // P0_ANDROID_025: clear any in-flight resolves so listeners held by
            // NsdManager don't get a callback after we stop. The in-flight set
            // would otherwise leak a few entries on stop-while-resolving, which
            // is harmless (the listener self-removes on next callback) but tidy.
            inFlightResolves.clear()
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
        val localId = getLocalPeerId?.invoke()
        val serviceInfo = NsdServiceInfo().apply {
            serviceName = if (!localId.isNullOrBlank()) localId else this@MdnsServiceDiscovery.serviceName
            serviceType = this@MdnsServiceDiscovery.serviceType
            port = servicePort
            // Add service data to identify this as an SCMessenger device
            setAttribute("version", "1.0")
            setAttribute("service", "scmessenger")
            
            // Set peer ID attributes so other peers (like Windows CLI) can discover us and associate our IP with our peer ID
            if (!localId.isNullOrBlank()) {
                setAttribute("peer-id", localId)
                setAttribute("p2p", localId)
                setAttribute("dnsaddr", "/ip4/0.0.0.0/tcp/$servicePort/p2p/$localId")
                Timber.i("mDNS advertising TXT peer-id: $localId")
            }
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
        // P0_ANDROID_025: NsdManager rejects reusing the same ResolveListener
        // while a previous resolve is in flight. Create a fresh listener per
        // call and track it in `inFlightResolves` so we never reuse one, and
        // the listener's terminal callbacks will self-remove from the set.
        val listener = newResolveListener(serviceInfo.serviceName)
        inFlightResolves[serviceInfo.serviceName] = listener

        // resolveService with Listener is deprecated in API 33; requires Executor overload
        // Use SDK version gate to support minSdk 26 while avoiding deprecation warnings on API 33+
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            // API 28+ has Context.getMainExecutor(), use Executor overload
            nsdManager?.resolveService(serviceInfo, context.getMainExecutor(), listener)
        } else {
            // Legacy API for API < 28 (minSdk 26, so this covers 26-27)
            // Kept for completeness but will not be called on API 28+
            @Suppress("DEPRECATION")
            nsdManager?.resolveService(serviceInfo, listener)
        }
    }

    /**
     * Clean up resources.
     */
    fun cleanup() {
        stop()
    }
}