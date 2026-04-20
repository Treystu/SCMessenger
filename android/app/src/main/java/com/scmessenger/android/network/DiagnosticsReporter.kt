package com.scmessenger.android.network

import com.scmessenger.android.transport.NetworkDetector
import com.scmessenger.android.utils.CircuitBreaker
import com.scmessenger.android.utils.NetworkFailureMetrics
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Singleton

/**
 * P0_ANDROID_007: Aggregates network diagnostics into reports with recommendations.
 *
 * Pulls from ConnectivityTester, NetworkDetector, CircuitBreaker, and
 * NetworkFailureMetrics to produce a comprehensive, user-friendly diagnostics report.
 */
@Singleton
class DiagnosticsReporter @Inject constructor(
    private val connectivityTester: ConnectivityTester,
    private val networkDetector: NetworkDetector,
    private val circuitBreaker: CircuitBreaker,
    private val failureMetrics: NetworkFailureMetrics
) {
    data class NetworkDiagnosticsReport(
        val timestampMs: Long = System.currentTimeMillis(),
        val networkType: String,
        val hasInternet: Boolean,
        val hasValidatedInternet: Boolean,
        val isMetered: Boolean,
        val dnsResults: Map<String, Boolean>,
        val portResults: Map<Int, Boolean>,
        val relayResults: Map<String, Boolean>,
        val circuitBreakerStats: CircuitBreaker.CircuitBreakerStats,
        val failureSummary: NetworkFailureMetrics.Summary,
        val recommendations: List<String>
    )

    suspend fun generateReport(): NetworkDiagnosticsReport {
        val testResults = connectivityTester.testNetworkConnectivity()
        val netDiags = networkDetector.getNetworkDiagnostics()
        val cbStats = circuitBreaker.getStats()
        val failureSummary = failureMetrics.getSummary()

        val recommendations = generateRecommendations(testResults, failureSummary)

        val report = NetworkDiagnosticsReport(
            networkType = testResults.networkType.name,
            hasInternet = testResults.internetConnectivity,
            hasValidatedInternet = testResults.hasValidatedInternet,
            isMetered = testResults.isMetered,
            dnsResults = testResults.dnsResolution,
            portResults = testResults.portReachability,
            relayResults = testResults.relayConnectivity,
            circuitBreakerStats = cbStats,
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
        testResults: ConnectivityTester.TestResults,
        failureSummary: NetworkFailureMetrics.Summary
    ): List<String> {
        val recs = mutableListOf<String>()

        if (!testResults.internetConnectivity) {
            recs.add("No internet connectivity — check Wi-Fi or cellular data")
        }

        if (failureSummary.totalDnsFailures > 0) {
            recs.add("DNS resolution failing — try alternative DNS (8.8.8.8 or 1.1.1.1)")
        }

        val blockedPorts = testResults.portReachability.filterValues { !it }.keys
        if (blockedPorts.isNotEmpty() && testResults.networkType == com.scmessenger.android.transport.NetworkType.CELLULAR) {
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

        if (testResults.isMetered && testResults.networkType == com.scmessenger.android.transport.NetworkType.CELLULAR) {
            recs.add("Metered cellular connection — app will prefer WebSocket on standard ports")
        }

        return recs
    }

    fun formatReportForUser(report: NetworkDiagnosticsReport): String = buildString {
        appendLine("Network Status: ${report.networkType}")
        appendLine("Internet: ${if (report.hasInternet) "Connected" else "Disconnected"}")
        appendLine("Validated: ${if (report.hasValidatedInternet) "Yes" else "No"}")

        val failedDns = report.dnsResults.filterValues { !it }.keys
        if (failedDns.isNotEmpty()) {
            appendLine("DNS Failed: ${failedDns.joinToString(", ")}")
        }

        val failedRelays = report.relayResults.filterValues { !it }.keys
        if (failedRelays.isNotEmpty()) {
            appendLine("Relays Unreachable: ${failedRelays.joinToString(", ")}")
        }

        if (report.circuitBreakerStats.openCount > 0) {
            appendLine("Circuits Open: ${report.circuitBreakerStats.openCount}")
        }

        if (report.recommendations.isNotEmpty()) {
            appendLine()
            appendLine("Recommendations:")
            report.recommendations.forEach { appendLine("  - $it") }
        }
    }
}