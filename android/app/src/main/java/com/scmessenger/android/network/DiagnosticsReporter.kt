package com.scmessenger.android.network

import android.content.Context
import com.scmessenger.android.transport.NetworkType
import com.scmessenger.android.utils.NetworkFailureMetrics
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Singleton

/**
 * P0_ANDROID_007: Aggregates network diagnostics into a user-facing report.
 *
 * Combines results from NetworkDiagnostics, NetworkTypeDetector, and NetworkFailureMetrics
 * to identify the root cause of connectivity failures and provide recommendations.
 */
@Singleton
class DiagnosticsReporter @Inject constructor(
    private val context: Context,
    private val networkDiagnostics: NetworkDiagnostics,
    private val networkTypeDetector: NetworkTypeDetector,
    private val failureMetrics: NetworkFailureMetrics
) {
    data class NetworkDiagnosticsReport(
        val timestampMs: Long = System.currentTimeMillis(),
        val networkType: NetworkType,
        val hasInternet: Boolean,
        val dnsResults: Map<String, Boolean>,
        val portResults: Map<Int, Boolean>,
        val relayResults: Map<String, Boolean>,
        val failureSummary: NetworkFailureMetrics.Summary,
        val recommendations: List<String>
    )

    suspend fun generateReport(): NetworkDiagnosticsReport {
        val testResults = networkDiagnostics.testNetworkConnectivity()
        val networkType = networkTypeDetector.detectNetworkType()
        val failureSummary = failureMetrics.getSummary()

        val recommendations = generateRecommendations(testResults, failureSummary, networkType)

        val report = NetworkDiagnosticsReport(
            networkType = networkType,
            hasInternet = testResults.internetConnectivity,
            dnsResults = testResults.dnsResolution,
            portResults = testResults.portReachability,
            relayResults = testResults.relayConnectivity,
            failureSummary = failureSummary,
            recommendations = recommendations
        )

        Timber.i("Diagnostics report generated: internet=%b dns_fail=%d port_block=%d relay_fail=%d recs=%d",
            report.hasInternet,
            failureSummary.totalDnsFailures,
            failureSummary.totalPortBlockedFailures,
            report.relayResults.count { !it.value },
            recommendations.size
        )

        return report
    }

    private fun generateRecommendations(
        testResults: NetworkDiagnostics.NetworkTestResults,
        failureSummary: NetworkFailureMetrics.Summary,
        networkType: NetworkType
    ): List<String> {
        val recs = mutableListOf<String>()

        if (!testResults.internetConnectivity) {
            recs.add("No internet connectivity — check Wi-Fi or cellular data")
        }

        when (networkType) {
            NetworkType.CELLULAR_RESTRICTED ->
                recs.add("Cellular network is restricting non-standard ports — WebSocket on port 443 should work")
            NetworkType.CELLULAR_NO_INTERNET ->
                recs.add("Cellular network has no internet — enable data or switch to Wi-Fi")
            NetworkType.WIFI_RESTRICTED ->
                recs.add("Wi-Fi connected but no internet validation — check captive portal or login page")
            else -> { /* no specific network-type recommendation */ }
        }

        if (failureSummary.totalDnsFailures > 0) {
            recs.add("DNS resolution failing — try alternative DNS (8.8.8.8 or 1.1.1.1)")
        }

        val blockedPorts = testResults.portReachability.filterValues { !it }.keys
        if (blockedPorts.isNotEmpty() && networkType in listOf(NetworkType.CELLULAR, NetworkType.CELLULAR_RESTRICTED)) {
            recs.add("Ports ${blockedPorts.joinToString(",")} blocked on cellular — enable WebSocket fallback")
        }

        if (failureSummary.totalTlsFailures > 0) {
            recs.add("TLS handshake failures detected — check device date/time and certificate stores")
        }

        if (failureSummary.totalConnectionRefusedFailures > 0) {
            recs.add("Connection refused — relay servers may be down, try again later")
        }

        if (testResults.relayConnectivity.values.all { !it }) {
            recs.add("All relay servers unreachable — check firewall or try a different network")
        }

        return recs
    }

    fun formatReportForUser(report: NetworkDiagnosticsReport): String = buildString {
        appendLine("Network Status: ${report.networkType}")
        appendLine("Internet: ${if (report.hasInternet) "Connected" else "Disconnected"}")

        val failedDns = report.dnsResults.filterValues { !it }.keys
        if (failedDns.isNotEmpty()) {
            appendLine("DNS Failed: ${failedDns.joinToString(", ")}")
        }

        val failedRelays = report.relayResults.filterValues { !it }.keys
        if (failedRelays.isNotEmpty()) {
            appendLine("Relays Unreachable: ${failedRelays.joinToString(", ")}")
        }

        if (report.recommendations.isNotEmpty()) {
            appendLine()
            appendLine("Recommendations:")
            report.recommendations.forEach { appendLine("  - $it") }
        }
    }
}
