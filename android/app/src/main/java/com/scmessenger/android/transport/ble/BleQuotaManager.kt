package com.scmessenger.android.transport.ble

import timber.log.Timber

/**
 * Enforces Android 12+ BLE scan quota: max 5 scan starts per 30-second window.
 *
 * Tracks scan start timestamps in a sliding window and blocks attempts
 * that would exceed the quota, returning a cooldown delay instead.
 */
class BleQuotaManager(
    private val maxScansPerWindow: Int = DEFAULT_MAX_SCANS,
    private val windowMs: Long = DEFAULT_WINDOW_MS
) {
    private val scanStartTimestamps = java.util.LinkedList<Long>()

    /**
     * Check if a new scan start is allowed under the current quota.
     * Returns 0 if allowed, or a delay in ms the caller should wait before retrying.
     */
    @Synchronized
    fun checkQuota(): Long {
        val now = System.currentTimeMillis()
        pruneOldTimestamps(now)

        return if (scanStartTimestamps.size >= maxScansPerWindow) {
            val oldestInWindow = scanStartTimestamps.first()
            val cooldownMs = windowMs - (now - oldestInWindow) + 500 // 500ms safety margin
            Timber.w("BLE scan quota exhausted ($maxScansPerWindow starts / ${windowMs}ms). Cooldown: ${cooldownMs}ms")
            maxOf(cooldownMs, 1000)
        } else {
            0L
        }
    }

    /**
     * Record a scan start attempt. Call after successfully starting a scan.
     */
    @Synchronized
    fun recordScanStart() {
        val now = System.currentTimeMillis()
        pruneOldTimestamps(now)
        scanStartTimestamps.addLast(now)
    }

    /**
     * Get the current number of scan starts within the window.
     */
    @Synchronized
    fun currentCount(): Int {
        pruneOldTimestamps(System.currentTimeMillis())
        return scanStartTimestamps.size
    }

    private fun pruneOldTimestamps(now: Long) {
        while (scanStartTimestamps.isNotEmpty() && (now - scanStartTimestamps.first()) > windowMs) {
            scanStartTimestamps.removeFirst()
        }
    }

    companion object {
        const val DEFAULT_MAX_SCANS = 5
        const val DEFAULT_WINDOW_MS = 30_000L
    }
}