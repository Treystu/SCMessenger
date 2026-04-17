package com.scmessenger.android.transport

import android.content.Context
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import timber.log.Timber

/**
 * mDNS/DNS-SD service discovery for cross-platform LAN discovery.
 *
 * This implements the same DNS-SD service type as iOS's mDNSServiceDiscovery,
 * allowing Android devices to discover iOS devices on the same local network.
 *
 * Service type: _scmessenger._tcp (matches iOS's mDNSServiceDiscovery.SERVICE_TYPE)
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

    // Service type (must match iOS's mDNSServiceDiscovery.SERVICE_TYPE)
    private val serviceType = "_scmessenger._tcp"
    private val serviceName = "SCMessenger"
    private val servicePort = 8888 // Matches Android's P2P_PORT

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
            }
        }

        nsdManager?.discoverServices(serviceType, NsdManager.PROTOCOL_DNS_SD, discoveryListener)
    }

    /**
     * Resolve a discovered service to get its address.
     */
    private fun resolveService(serviceInfo: NsdServiceInfo) {
        resolveListener = object : NsdManager.ResolveListener {
            override fun onResolveFailed(serviceInfo: NsdServiceInfo, errorCode: Int) {
                Timber.e("mDNS service resolve failed: $errorCode")
            }

            override fun onServiceResolved(serviceInfo: NsdServiceInfo) {
                Timber.d("mDNS service resolved: ${serviceInfo.serviceName} at ${serviceInfo.host}:${serviceInfo.port}")

                // Create a peer ID from the service name (matches iOS's service.name pattern)
                val peerId = "mdns-${serviceInfo.serviceName}"

                // Notify discovery
                onPeerDiscovered(peerId)

                // TCP/mDNS parity: Notify the resolved LAN address so the caller
                // can generate a libp2p multiaddr and dial via SwarmBridge.
                val host = serviceInfo.host?.hostAddress
                val port = serviceInfo.port
                if (host != null && port > 0) {
                    Timber.i("mDNS: LAN peer resolved $peerId at $host:$port — notifying for SwarmBridge dial")
                    onLanPeerResolved?.invoke(peerId, host, port)
                }
            }
        }

        nsdManager?.resolveService(serviceInfo, resolveListener)
    }

    /**
     * Clean up resources.
     */
    fun cleanup() {
        stop()
    }
}
