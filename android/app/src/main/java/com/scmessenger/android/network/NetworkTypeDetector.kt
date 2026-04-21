package com.scmessenger.android.network

import android.content.Context
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import com.scmessenger.android.transport.NetworkType
import javax.inject.Inject
import javax.inject.Singleton

/**
 * P0_ANDROID_007: Detects current network type and identifies carrier-level restrictions.
 *
 * Used by DiagnosticsReporter to provide actionable recommendations
 * (e.g., suggesting WebSocket fallback if cellular ports are blocked).
 */
@Singleton
class NetworkTypeDetector @Inject constructor(
    private val context: Context
) {
    private val connectivityManager =
        context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

    fun detectNetworkType(): NetworkType {
        val network = connectivityManager.activeNetwork ?: return NetworkType.UNKNOWN
        val caps = connectivityManager.getNetworkCapabilities(network) ?: return NetworkType.UNKNOWN

        return when {
            caps.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) -> {
                if (!caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED)) {
                    NetworkType.WIFI_RESTRICTED
                } else {
                    NetworkType.WIFI
                }
            }
            caps.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) -> {
                when {
                    !caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) ->
                        NetworkType.CELLULAR_NO_INTERNET
                    !caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED) ||
                        isCellularPortRestricted() ->
                        NetworkType.CELLULAR_RESTRICTED
                    else -> NetworkType.CELLULAR
                }
            }
            caps.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) -> NetworkType.ETHERNET
            caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN) -> NetworkType.VPN
            else -> NetworkType.UNKNOWN
        }
    }

    /** Heuristic: check if the cellular network blocks non-standard ports (9001, 9010). */
    private fun isCellularPortRestricted(): Boolean {
        return isPortBlocked("8.8.8.8", 9001) && isPortBlocked("8.8.8.8", 9010)
    }

    fun isCellularNetwork(): Boolean {
        val type = detectNetworkType()
        return type == NetworkType.CELLULAR || type == NetworkType.CELLULAR_RESTRICTED
    }

    fun isPortBlocked(host: String, port: Int): Boolean {
        return try {
            val socket = java.net.Socket()
            socket.connect(java.net.InetSocketAddress(host, port), 3000)
            socket.close()
            false // Port is open
        } catch (e: Exception) {
            true // Port is blocked
        }
    }
}
