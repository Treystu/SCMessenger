package com.scmessenger.android.utils

import timber.log.Timber
import java.util.concurrent.ConcurrentHashMap
import javax.inject.Inject
import javax.inject.Singleton

/**
 * P0_ANDROID_007: Tracks network failure metrics per node for diagnostics.
 *
 * Complements CircuitBreaker by recording *why* failures happen (DNS, timeout,
 * TLS, port-blocked) so DiagnosticsReporter can surface actionable recommendations.
 */
@Singleton
class NetworkFailureMetrics @Inject constructor() {

    private val entries = ConcurrentHashMap<String, NodeFailureLog>()

    data class FailureRecord(
        val timestampMs: Long = System.currentTimeMillis(),
        val reason: String,
        val exceptionClass: String
    )

    data class NodeFailureLog(
        val records: MutableList<FailureRecord> = mutableListOf(),
        var totalFailures: Int = 0,
        var dnsFailures: Int = 0,
        var timeoutFailures: Int = 0,
        var tlsFailures: Int = 0,
        var connectionRefusedFailures: Int = 0,
        var portBlockedFailures: Int = 0
    )

    fun recordFailure(nodeId: String, reason: String, exception: Exception) {
        val log = entries.getOrPut(nodeId) { NodeFailureLog() }
        synchronized(log) {
            log.totalFailures++
            log.records.add(FailureRecord(reason = reason, exceptionClass = exception.javaClass.simpleName))
            when {
                reason.contains("DNS", ignoreCase = true) -> log.dnsFailures++
                reason.contains("timeout", ignoreCase = true) -> log.timeoutFailures++
                reason.contains("TLS", ignoreCase = true) -> log.tlsFailures++
                reason.contains("refused", ignoreCase = true) -> log.connectionRefusedFailures++
                reason.contains("port", ignoreCase = true) ||
                    reason.contains("blocked", ignoreCase = true) -> log.portBlockedFailures++
            }
            // Keep only last 50 records per node
            if (log.records.size > 50) {
                log.records.removeAt(0)
            }
        }
        Timber.d("Failure metric recorded: node=%s reason=%s total=%d", nodeId, reason, log.totalFailures)
    }

    fun isNodeUnreachable(nodeId: String): Boolean {
        val log = entries[nodeId] ?: return false
        return log.totalFailures >= 5
    }

    fun hasDnsFailures(): Boolean = entries.values.any { it.dnsFailures > 0 }

    fun hasPortBlocking(): Boolean = entries.values.any { it.portBlockedFailures > 0 }

    fun getFailureCount(nodeId: String): Int = entries[nodeId]?.totalFailures ?: 0

    fun getLastFailure(nodeId: String): FailureRecord? {
        val log = entries[nodeId] ?: return null
        synchronized(log) { return log.records.lastOrNull() }
    }

    data class Summary(
        val totalNodes: Int,
        val unreachableNodes: Int,
        val totalDnsFailures: Int,
        val totalTimeoutFailures: Int,
        val totalTlsFailures: Int,
        val totalPortBlockedFailures: Int,
        val totalConnectionRefusedFailures: Int,
        val nodeDetails: Map<String, NodeSummary>
    )

    data class NodeSummary(
        val totalFailures: Int,
        val lastReason: String?,
        val isUnreachable: Boolean
    )

    fun getSummary(): Summary {
        val nodeDetails = mutableMapOf<String, NodeSummary>()
        var totalDns = 0
        var totalTimeout = 0
        var totalTls = 0
        var totalPortBlocked = 0
        var totalConnRefused = 0
        var unreachable = 0

        for ((nodeId, log) in entries) {
            synchronized(log) {
                totalDns += log.dnsFailures
                totalTimeout += log.timeoutFailures
                totalTls += log.tlsFailures
                totalPortBlocked += log.portBlockedFailures
                totalConnRefused += log.connectionRefusedFailures
                val isUnreachable = log.totalFailures >= 5
                if (isUnreachable) unreachable++
                nodeDetails[nodeId] = NodeSummary(
                    totalFailures = log.totalFailures,
                    lastReason = log.records.lastOrNull()?.reason,
                    isUnreachable = isUnreachable
                )
            }
        }

        return Summary(
            totalNodes = entries.size,
            unreachableNodes = unreachable,
            totalDnsFailures = totalDns,
            totalTimeoutFailures = totalTimeout,
            totalTlsFailures = totalTls,
            totalPortBlockedFailures = totalPortBlocked,
            totalConnectionRefusedFailures = totalConnRefused,
            nodeDetails = nodeDetails
        )
    }

    fun reset() {
        entries.clear()
        Timber.i("Network failure metrics reset")
    }
}