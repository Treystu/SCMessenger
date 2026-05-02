package com.scmessenger.android.transport

import android.content.Context
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import android.os.Build
import android.os.Handler
import android.os.Looper
import timber.log.Timber

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

    // Service type must match libp2p-mdns default (_p2p._udp.) so Rust peers discover us.
    // Android's NsdManager appends .local. automatically — do not include it here.
    private val serviceType = "_p2p._udp."
    private val serviceName = "SCMessenger"
    private val servicePort = 9001 // Must match the actual libp2p swarm listen port (startSwarm /ip4/0.0.0.0/tcp/9001)

    // Handler for retrying operations
    private val handler = Handler(Looper.getMainLooper())

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
                Timber.e("mDNS service registration failed: $errorCode")
                isRegistered = false
            }

            override fun onUnregistrationFailed(serviceInfo: NsdServiceInfo, errorCode: Int) {
                Timber.e("mDNS service unregistration failed: $errorCode")
            }

            override fun onServiceRegistered(serviceInfo: NsdServiceInfo) {
                Timber.d("mDNS service registered: ${serviceInfo.serviceName}")
                isRegistered = true
            }

            override fun onServiceUnregistered(serviceInfo: NsdServiceInfo) {
                Timber.d("mDNS service unregistered: ${serviceInfo.serviceName}")
                isRegistered = false
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
                Timber.d("mDNS discovery started for type: $regType")
                isDiscovering = true
            }

            override fun onDiscoveryStopped(regType: String) {
                Timber.d("mDNS discovery stopped for type: $regType")
                isDiscovering = false
            }

            override fun onServiceFound(serviceInfo: NsdServiceInfo) {
                Timber.d("mDNS service found: ${serviceInfo.serviceName} type: ${serviceInfo.serviceType}")

                // Only process services of our type
                if (serviceInfo.serviceType == serviceType) {
                    // Resolve the service to get the address
                    resolveService(serviceInfo)
                }
            }

            override fun onServiceLost(serviceInfo: NsdServiceInfo) {
                Timber.d("mDNS service lost: ${serviceInfo.serviceName}")
            }

            override fun onStartDiscoveryFailed(serviceType: String, errorCode: Int) {
                Timber.e("mDNS discovery failed to start: $errorCode")
                isDiscovering = false
            }

            override fun onStopDiscoveryFailed(serviceType: String, errorCode: Int) {
                Timber.e("mDNS discovery failed to stop: $errorCode")
                // Reset discovering state so we can retry
                isDiscovering = false
                // Schedule a retry to restart discovery
                handler.postDelayed({
                    if (isRunning && !isDiscovering) {
                        Timber.d("Retrying mDNS discovery after stop failure")
                        startDiscovery()
                    }
                }, 1000) // 1 second delay before retry
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
                Timber.e("mDNS service resolve failed: $errorCode")
            }

            override fun onServiceResolved(resolvedInfo: NsdServiceInfo) {
                @Suppress("DEPRECATION")
                Timber.d("mDNS service resolved: ${resolvedInfo.serviceName} at ${resolvedInfo.host}:${resolvedInfo.port}")

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
                    Timber.i("mDNS: LAN peer resolved $peerId → $multiaddr — notifying for SwarmBridge dial")
                    onLanPeerResolved?.invoke(peerId, host, port)
                }
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
