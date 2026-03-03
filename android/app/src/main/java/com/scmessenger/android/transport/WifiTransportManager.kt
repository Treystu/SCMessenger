package com.scmessenger.android.transport

import android.Manifest
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.PackageManager
import android.net.wifi.p2p.WifiP2pManager
import android.os.Build
import android.os.Looper
import androidx.core.content.ContextCompat
import timber.log.Timber

/**
 * Manages WiFi Direct (P2P) transport for mesh networking.
 * Handles peer discovery and connection management.
 */
class WifiTransportManager(
    private val context: Context,
    private val onPeerDiscovered: (String) -> Unit,
    private val onDataReceived: ((String, ByteArray) -> Unit)? = null
) {

    private val manager: WifiP2pManager? by lazy {
        context.getSystemService(Context.WIFI_P2P_SERVICE) as? WifiP2pManager
    }

    private var channel: WifiP2pManager.Channel? = null
    private var isDiscovering = false
    private var receiverRegistered = false
    private var wifiDirectTransport: WifiDirectTransport? = null

    private val receiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            val action = intent.action
            if (WifiP2pManager.WIFI_P2P_PEERS_CHANGED_ACTION == action) {
                requestPeers()
            }
        }
    }

    fun initialize() {
        channel = manager?.initialize(context, Looper.getMainLooper(), null)
        if (channel == null) {
            Timber.e("Failed to initialize WiFi P2P Manager")
        } else {
            Timber.d("WifiTransportManager initialized")
        }
        if (wifiDirectTransport == null) {
            wifiDirectTransport = WifiDirectTransport(
                context = context,
                onPeerDiscovered = onPeerDiscovered,
                onDataReceived = { peerId, data ->
                    onDataReceived?.invoke(peerId, data)
                }
            )
        }
    }

    fun startDiscovery() {
        if (isDiscovering) {
            Timber.d("WiFi P2P discovery already active; skipping duplicate start")
            return
        }
        if (!hasDiscoveryPermissions()) {
            Timber.w("WiFi P2P discovery skipped: missing required runtime permissions")
            return
        }
        wifiDirectTransport?.start()
        val c = channel ?: return

        try {
            manager?.discoverPeers(c, object : WifiP2pManager.ActionListener {
                override fun onSuccess() {
                    Timber.i("WiFi P2P Discovery started")
                    isDiscovering = true
                    registerReceiver()
                }

                override fun onFailure(reasonCode: Int) {
                    Timber.e("WiFi P2P Discovery failed: $reasonCode")
                    isDiscovering = false
                }
            })
        } catch (e: SecurityException) {
            Timber.e(e, "Missing permission while starting WiFi P2P discovery")
            isDiscovering = false
        }
    }

    fun stopDiscovery() {
        val c = channel
        if (isDiscovering) {
            if (c != null) {
                manager?.stopPeerDiscovery(c, object : WifiP2pManager.ActionListener {
                    override fun onSuccess() {
                        isDiscovering = false
                        Timber.i("WiFi P2P Discovery stopped")
                    }
                    override fun onFailure(reason: Int) {
                        Timber.w("Failed to stop WiFi P2P discovery: $reason")
                    }
                })
            }
            isDiscovering = false
            try {
                if (receiverRegistered) {
                    context.unregisterReceiver(receiver)
                    receiverRegistered = false
                }
            } catch (e: IllegalArgumentException) {
                // Ignore if not registered
            }
        }
        wifiDirectTransport?.stop()
    }

    private fun registerReceiver() {
        if (receiverRegistered) return
        val intentFilter = IntentFilter().apply {
            addAction(WifiP2pManager.WIFI_P2P_PEERS_CHANGED_ACTION)
            addAction(WifiP2pManager.WIFI_P2P_CONNECTION_CHANGED_ACTION)
        }
        context.registerReceiver(receiver, intentFilter)
        receiverRegistered = true
    }

    private fun requestPeers() {
        if (!hasDiscoveryPermissions()) {
            Timber.w("WiFi peer request skipped: missing required runtime permissions")
            return
        }
        val c = channel ?: return
        try {
            manager?.requestPeers(c) { peers ->
                peers.deviceList.forEach { device ->
                    val peerId = device.deviceAddress // Use MAC as ID for now
                    Timber.v("WiFi Peer discovered: $peerId (${device.deviceName})")
                    onPeerDiscovered(peerId)
                }
            }
        } catch (e: SecurityException) {
            Timber.e(e, "Missing permission while requesting WiFi peers")
        }
    }

    private fun hasDiscoveryPermissions(): Boolean {
        val hasLocation = ContextCompat.checkSelfPermission(
            context,
            Manifest.permission.ACCESS_FINE_LOCATION
        ) == PackageManager.PERMISSION_GRANTED
        val hasNearbyWifi = Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU ||
            ContextCompat.checkSelfPermission(
                context,
                Manifest.permission.NEARBY_WIFI_DEVICES
            ) == PackageManager.PERMISSION_GRANTED
        return hasLocation && hasNearbyWifi
    }
    fun sendData(peerId: String, data: ByteArray): Boolean {
        val normalizedPeerId = peerId.trim()
        if (normalizedPeerId.isEmpty()) {
            Timber.w("WiFi send skipped: empty peer ID")
            return false
        }
        val direct = wifiDirectTransport
        if (direct == null) {
            Timber.d("WiFi send skipped for $normalizedPeerId: transport not initialized")
            return false
        }
        val sent = direct.sendData(normalizedPeerId, data)
        if (!sent) {
            Timber.d("WiFi send failed for $normalizedPeerId; fallback required")
        }
        return sent
    }
}
