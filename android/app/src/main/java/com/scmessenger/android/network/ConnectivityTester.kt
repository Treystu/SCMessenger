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
import javax.inject.Inject
import javax.inject.Singleton

/**
 * P0_ANDROID_007: Startup connectivity testing.
 *
 * Tests basic internet, DNS resolution, common ports, and relay-specific
 * connectivity so that failures are diagnosed with specifics rather than
 * generic "Network error".
 */
@Singleton
class ConnectivityTester @Inject constructor(
    private val context: Context
) {
    private val connectivityManager =
        context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

    data class TestResults(
        val internetConnectivity: Boolean = false,
        val dnsResolution: Map<String, Boolean> = emptyMap(),
        val portReachability: Map<Int, Boolean> = emptyMap(),
        val relayConnectivity: Map<String, Boolean> = emptyMap(),
        val networkType: NetworkType = NetworkType.UNKNOWN,
        val hasValidatedInternet: Boolean = false,
        val isMetered: Boolean = false
    )

    suspend fun testNetworkConnectivity(): TestResults = withContext(Dispatchers.IO) {
        val results = TestResults(
            internetConnectivity = testInternetConnectivity(),
            dnsResolution = testDnsResolution(),
            portReachability = testCommonPorts(),
            relayConnectivity = testRelayConnectivity(),
            networkType = detectNetworkType(),
            hasValidatedInternet = hasValidatedInternet(),
            isMetered = isMetered()
        )
        Timber.i("Connectivity test complete: internet=%b dns=%s ports=%s relay=%s type=%s",
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
            val conn = url.openConnection() as HttpURLConnection
            conn.connectTimeout = 5000
            conn.readTimeout = 5000
            conn.requestMethod = "HEAD"
            conn.instanceFollowRedirects = true
            val code = conn.responseCode
            conn.disconnect()
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
            "bootstrap.scmessenger.net",
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

    private fun testRelayConnectivity(): Map<String, Boolean> {
        val relayHosts = mapOf(
            "GCP relay (34.135.34.73:9001)" to "34.135.34.73" to 9001,
            "GCP relay (34.135.34.73:443)" to "34.135.34.73" to 443,
            "OSX relay (104.28.216.43:9010)" to "104.28.216.43" to 9010,
            "bootstrap.scmessenger.net:443" to "bootstrap.scmessenger.net" to 443
        )
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

    private fun hasValidatedInternet(): Boolean {
        val network = connectivityManager.activeNetwork ?: return false
        val caps = connectivityManager.getNetworkCapabilities(network) ?: return false
        return caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED)
    }

    private fun isMetered(): Boolean {
        val network = connectivityManager.activeNetwork ?: return true
        val caps = connectivityManager.getNetworkCapabilities(network) ?: return true
        return !caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_METERED)
    }
}