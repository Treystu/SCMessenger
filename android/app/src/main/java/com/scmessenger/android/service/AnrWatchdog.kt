package com.scmessenger.android.service

import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.os.SystemClock
import timber.log.Timber
import java.io.File
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicLong

/**
 * ANR watchdog that detects main thread freezes and triggers recovery.
 *
 * Pings the main thread every [checkIntervalMs]. If the main thread doesn't
 * respond within [anrThresholdMs], it's considered frozen. After
 * [maxConsecutiveBlocks] consecutive blocks, triggers service recovery
 * via MeshForegroundService restart.
 */
class AnrWatchdog(
    private val context: Context,
    private val checkIntervalMs: Long = 5000L,
    private val anrThresholdMs: Long = 10000L,
    private val maxConsecutiveBlocks: Int = 2
) {
    private val handler = Handler(Looper.getMainLooper())
    private val isRunning = AtomicBoolean(false)

    @Volatile private var lastPingTime = 0L
    @Volatile private var consecutiveBlocks = 0
    @Volatile private var totalAnrEvents = 0

    private val watchdogThread = Thread({ watchdogLoop() }, "ANR-Watchdog")

    private fun watchdogLoop() {
        while (isRunning.get()) {
            val pingStart = SystemClock.uptimeMillis()
            lastPingTime = pingStart

            // Post a runnable to the main thread
            val responded = AtomicBoolean(false)
            handler.post { responded.set(true) }

            // Sleep for the check interval
            try {
                Thread.sleep(checkIntervalMs)
            } catch (_: InterruptedException) {
                break
            }

            if (!responded.get()) {
                val blockedMs = SystemClock.uptimeMillis() - pingStart
                if (blockedMs > anrThresholdMs) {
                    consecutiveBlocks++
                    totalAnrEvents++
                    Timber.e(
                        "ANR detected: main thread blocked for %dms (consecutive=%d, total=%d)",
                        blockedMs, consecutiveBlocks, totalAnrEvents
                    )

                    if (consecutiveBlocks >= maxConsecutiveBlocks) {
                        triggerAnrRecovery(blockedMs)
                    }
                }
            } else {
                if (consecutiveBlocks > 0) {
                    Timber.i("Main thread recovered after %d ANR-like blocks", consecutiveBlocks)
                }
                consecutiveBlocks = 0
            }
        }
    }

    private fun triggerAnrRecovery(blockedMs: Long) {
        Timber.e(
            "ANR RECOVERY TRIGGERED - Main thread blocked %dms (threshold=%dms, consecutive=%d, total=%d)",
            blockedMs, anrThresholdMs, consecutiveBlocks, totalAnrEvents
        )

        // Collect ANR diagnostic information
        val anrInfo = buildAnrDiagnostics(blockedMs)
        Timber.e("ANR Diagnostics:\n%s", anrInfo)

        consecutiveBlocks = 0

        // Write ANR info to file for post-mortem analysis
        try {
            writeAnrDiagnostics(anrInfo)
        } catch (e: Exception) {
            Timber.e(e, "Failed to write ANR diagnostics")
        }

        // Request service restart via intent (safe from background thread)
        try {
            val restartIntent = Intent(context, MeshForegroundService::class.java).apply {
                action = MeshForegroundService.ACTION_START
            }
            context.startService(restartIntent)
        } catch (e: Exception) {
            Timber.e(e, "Failed to trigger ANR recovery restart")
        }
    }

    private fun buildAnrDiagnostics(blockedMs: Long): String {
        val sb = StringBuilder()
        sb.append("=== ANR DIAGNOSTICS ===\n")
        sb.append("Timestamp: ").append(System.currentTimeMillis()).append("\n")
        sb.append("Blocked Duration: ").append(blockedMs).append("ms\n")
        sb.append("Consecutive Blocks: ").append(consecutiveBlocks).append("\n")
        sb.append("Total ANR Events: ").append(totalAnrEvents).append("\n")
        sb.append("Android Version: ").append(Build.VERSION.SDK_INT).append("\n")
        sb.append("Device: ").append(Build.BRAND).append(" / ").append(Build.MODEL).append("\n")

        // Check for main thread stack info (if we could get it)
        sb.append("Main Thread Responsive: ").append(isMainThreadResponsive()).append("\n")

        // Memory info
        try {
            val activityManager = context.getSystemService(Context.ACTIVITY_SERVICE) as android.app.ActivityManager
            val memoryInfo = android.app.ActivityManager.MemoryInfo()
            activityManager.getMemoryInfo(memoryInfo)
            sb.append("Memory - Available: ").append(memoryInfo.availMem / 1024 / 1024).append("MB\n")
            sb.append("Memory - Low: ").append(memoryInfo.lowMemory).append("\n")
            sb.append("Memory - Threshold: ").append(memoryInfo.threshold / 1024 / 1024).append("MB\n")
        } catch (e: Exception) {
            sb.append("Memory info unavailable: ").append(e.message).append("\n")
        }

        sb.append("=== END DIAGNOSTICS ===")
        return sb.toString()
    }

    private fun writeAnrDiagnostics(anrInfo: String) {
        val anrDir = java.io.File(context.filesDir, "anr_diagnostics")
        anrDir.mkdirs()

        val timestamp = System.currentTimeMillis()
        val anrFile = java.io.File(anrDir, "anr_$timestamp.txt")

        anrFile.writeText(anrInfo)
        Timber.i("ANR diagnostics written to: %s", anrFile.absolutePath)

        // Keep only last 10 ANR files
        val anrFiles = anrDir.listFiles()?.sortedBy { it.lastModified() } ?: emptyArray()
        if (anrFiles.size > 10) {
            anrFiles.take(anrFiles.size - 10).forEach { file ->
                file.delete()
                Timber.d("Removed old ANR diagnostic: %s", file.name)
            }
        }
    }

    fun start() {
        if (isRunning.compareAndSet(false, true)) {
            lastPingTime = SystemClock.uptimeMillis()
            consecutiveBlocks = 0
            watchdogThread.start()
            Timber.i("ANR watchdog started (check=%dms, threshold=%dms)", checkIntervalMs, anrThresholdMs)
        }
    }

    fun stop() {
        if (isRunning.compareAndSet(true, false)) {
            watchdogThread.interrupt()
            handler.removeCallbacksAndMessages(null)
            Timber.i("ANR watchdog stopped (total ANR events=%d)", totalAnrEvents)
        }
    }

    fun getTotalAnrEvents(): Int = totalAnrEvents

    fun isMainThreadResponsive(): Boolean = consecutiveBlocks == 0
}