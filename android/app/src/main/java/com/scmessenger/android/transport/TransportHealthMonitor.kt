package com.scmessenger.android.transport

import timber.log.Timber

/**
 * Monitors transport health metrics to inform routing decisions.
 *
 * Tracks success/failure counts and latency per transport type.
 * Used by SmartTransportRouter to deprioritize failing transports
 * and trigger graceful degradation when BLE or other transports fail.
 */
class TransportHealthMonitor {
    private val transportHealth = java.util.concurrent.ConcurrentHashMap<String, TransportHealth>()

    data class TransportHealth(
        var successCount: Int = 0,
        var failureCount: Int = 0,
        var totalLatencyMs: Long = 0,
        var lastUpdated: Long = System.currentTimeMillis(),
        var consecutiveFailures: Int = 0
    )

    fun recordSuccess(transport: String, latencyMs: Long? = null) {
        val health = transportHealth.getOrPut(transport) { TransportHealth() }
        health.successCount++
        health.consecutiveFailures = 0
        latencyMs?.let { health.totalLatencyMs += it }
        health.lastUpdated = System.currentTimeMillis()
    }

    fun recordFailure(transport: String) {
        val health = transportHealth.getOrPut(transport) { TransportHealth() }
        health.failureCount++
        health.consecutiveFailures++
        health.lastUpdated = System.currentTimeMillis()

        if (health.consecutiveFailures >= CONSECUTIVE_FAILURE_THRESHOLD) {
            Timber.w("Transport $transport has ${health.consecutiveFailures} consecutive failures")
        }
    }

    fun getHealth(transport: String): TransportHealth {
        return transportHealth[transport] ?: TransportHealth()
    }

    /**
     * Whether a transport should be used for new connections.
     * Returns true if there isn't enough data yet, or if the success rate is above 30%.
     */
    fun shouldUseTransport(transport: String): Boolean {
        val health = getHealth(transport)
        val totalAttempts = health.successCount + health.failureCount
        if (totalAttempts < MIN_SAMPLES) return true
        val successRate = health.successCount.toDouble() / totalAttempts.toDouble()
        return successRate >= MIN_SUCCESS_RATE
    }

    /**
     * Whether a transport is in a degraded state and should trigger fallback.
     */
    fun isDegraded(transport: String): Boolean {
        val health = getHealth(transport)
        return health.consecutiveFailures >= CONSECUTIVE_FAILURE_THRESHOLD
    }

    fun getSummary(): Map<String, TransportHealth> = transportHealth.toMap()

    companion object {
        private const val MIN_SAMPLES = 5
        private const val MIN_SUCCESS_RATE = 0.3
        private const val CONSECUTIVE_FAILURE_THRESHOLD = 3
    }
}