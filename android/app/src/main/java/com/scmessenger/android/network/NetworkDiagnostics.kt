package com.scmessenger.android.network

import android.content.Context
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import com.scmessenger.android.transport.NetworkType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import timber.log.Timber
import java.net.HttpURLConnection
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.Socket
import java.net.URL
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton

/**
 * P0_ANDROID_007: Comprehensive network diagnostics.
 *
 * Performs multi-stage connectivity testing (Internet, DNS, Ports, Relays)
 * to provide detailed failure reasons instead of generic "Network error".
 */
@Singleton
class NetworkDiagnostics @Inject constructor(
    @ApplicationContext private val context: Context
) {
    private val connectivityManager =
        context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

    data class NetworkTestResults(
        val internetConnectivity: Boolean = false,
        val dnsResolution: Map<String, Boolean> = emptyMap(),
        val portReachability: Map<Int, Boolean> = emptyMap(),
        val relayConnectivity: Map<String, Boolean> = emptyMap(),
        val networkType: NetworkType = NetworkType.UNKNOWN,
        val restrictions: List<String> = emptyList()
    )

    suspend fun testNetworkConnectivity(): NetworkTestResults = withContext(Dispatchers.IO) {
        val results = NetworkTestResults(
            internetConnectivity = testInternetConnectivity(),
            dnsResolution = testDnsResolution(),
            portReachability = testCommonPorts(),
            relayConnectivity = testRelaySpecificConnectivity(),
            networkType = detectNetworkType(),
            restrictions = detectNetworkRestrictions()
        )
        Timber.i("Network Diagnostics complete: internet=%b dns=%s ports=%s relay=%s type=%s",
            results.internetConnectivity,
            results.dnsResolution.filterValues { !it }.keys.ifEmpty { setOf("ok") },
            results.portReachability.filterValues { !it }.keys.ifEmpty { setOf("ok") },
            results.relayConnectivity.filterValues { !it }.keys.ifEmpty { setOf("ok") },
            results.networkType
        )
        results
    }

    private fun testInternetConnectivity(): Boolean {
        return try {
            val url = URL("https://www.google.com")
            val connection = url.openConnection() as HttpURLConnection
            connection.connectTimeout = 5000
            connection.readTimeout = 5000
            connection.requestMethod = "HEAD"
            connection.instanceFollowRedirects = true
            val code = connection.responseCode
            connection.disconnect()
            code in 200..399
        } catch (e: Exception) {
            Timber.d("Internet connectivity test failed: %s", e.message)
            false
        }
    }

    private fun testDnsResolution(): Map<String, Boolean> {
        val domains = listOf(
            "google.com",
            "cloudflare.com",
            "relay.scmessenger.net"
        )
        return domains.associateWith { domain ->
            try {
                InetAddress.getByName(domain) != null
            } catch (e: Exception) {
                Timber.d("DNS test failed for %s: %s", domain, e.message)
                false
            }
        }
    }

    private fun testCommonPorts(): Map<Int, Boolean> {
        val ports = listOf(80, 443, 9001, 9010)
        return ports.associateWith { port ->
            try {
                val socket = Socket()
                socket.connect(InetSocketAddress("8.8.8.8", port), 3000)
                socket.close()
                true
            } catch (e: Exception) {
                Timber.d("Port test failed for %d: %s", port, e.message)
                false
            }
        }
    }

    private fun testRelaySpecificConnectivity(): Map<String, Boolean> {
        val relayHosts = mapOf<String, Pair<String, Int>>()
        return relayHosts.mapValues { (_, hostPort) ->
            val (host, port) = hostPort
            try {
                val socket = Socket()
                socket.connect(InetSocketAddress(host, port), 5000)
                socket.close()
                true
            } catch (e: Exception) {
                Timber.d("Relay connectivity test failed for %s:%d: %s", host, port, e.message)
                false
            }
        }
    }

    private fun detectNetworkType(): NetworkType {
        val network = connectivityManager.activeNetwork ?: return NetworkType.UNKNOWN
        val caps = connectivityManager.getNetworkCapabilities(network) ?: return NetworkType.UNKNOWN
        return when {
            caps.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) -> NetworkType.WIFI
            caps.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) -> NetworkType.CELLULAR
            caps.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) -> NetworkType.ETHERNET
            caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN) -> NetworkType.VPN
            else -> NetworkType.UNKNOWN
        }
    }

    private fun detectNetworkRestrictions(): List<String> {
        val restrictions = mutableListOf<String>()
        val network = connectivityManager.activeNetwork ?: return restrictions
        val caps = connectivityManager.getNetworkCapabilities(network) ?: return restrictions

        if (!caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)) {
            restrictions.add("No internet capability")
        }
        if (!caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED)) {
            restrictions.add("Internet not validated by OS")
        }
        if (caps.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR)) {
            // Heuristic: check if common relay ports are blocked on this cellular network
            if (!isPortOpen("8.8.8.8", 9001)) {
                restrictions.add("Cellular network likely blocking non-standard ports")
            }
        }
        return restrictions
    }

    private fun isPortOpen(host: String, port: Int): Boolean {
        return try {
            val socket = Socket()
            socket.connect(InetSocketAddress(host, port), 2000)
            socket.close()
            true
        } catch (e: Exception) {
            false
        }
    }
}
