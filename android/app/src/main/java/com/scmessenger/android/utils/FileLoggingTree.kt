package com.scmessenger.android.utils

import android.content.Context
import timber.log.Timber
import java.io.File
import java.io.FileWriter
import java.io.PrintWriter
import java.text.SimpleDateFormat
import java.util.*

/**
 * A Timber Tree that logs to a file in the app's internal storage.
 * Useful for diagnosing issues on devices without a debugger connected.
 */
class FileLoggingTree(context: Context) : Timber.Tree() {
    private val MAX_LOG_LINES = 10000
    private val logFile: File = File(context.filesDir, "mesh_diagnostics.log")
    private val dateFormat = SimpleDateFormat("yyyy-MM-dd HH:mm:ss.SSS", Locale.US)
    // Guard against recursion (Timber -> FileLoggingTree -> Timber -> ...)
    private val isLogging = ThreadLocal.withInitial { false }
    @Volatile
    private var ironCore: uniffi.api.IronCore? = null

    fun setIronCore(core: uniffi.api.IronCore?) {
        synchronized(this) { this.ironCore = core }
    }

    override fun log(priority: Int, tag: String?, message: String, t: Throwable?) {
        if (isLogging.get()) return // Prevent recursion

        try {
            isLogging.set(true)

            val timestamp = dateFormat.format(Date())
            val priorityStr = when (priority) {
                android.util.Log.VERBOSE -> "V"
                android.util.Log.DEBUG -> "D"
                android.util.Log.INFO -> "I"
                android.util.Log.WARN -> "W"
                android.util.Log.ERROR -> "E"
                android.util.Log.ASSERT -> "A"
                else -> "U"
            }

            val logLine = "$timestamp $priorityStr/${tag ?: "Mesh"}: $message\n"
            
            // WS12.41: Send to IronCore for summarized storage
            synchronized(this) {
                runCatching { ironCore?.recordLog(logLine) }
                    .onFailure { android.util.Log.w("FileLoggingTree", "IronCore logging failed; using file fallback", it) }
                
                // Fallback/Legacy: Still append to file but with smaller limit
                // The user wants "instead of saving all the log files, we only save the log once"
                // but for debugging it's useful to have some raw tail.
                FileWriter(logFile, true).use { writer ->
                    writer.write(logLine)
                    t?.let {
                        val pw = PrintWriter(writer)
                        it.printStackTrace(pw)
                        pw.flush()
                    }
                }

                // Limit file size to ~100KB (much smaller now that we have summarizer)
                if (logFile.length() > 100 * 1024) {
                    truncateLogFile()
                }
            }
        } catch (e: Exception) {
            android.util.Log.e("FileLoggingTree", "Error writing to log file", e)
        } finally {
            isLogging.set(false)
        }
    }

    private fun truncateLogFile() {
        try {
            android.util.Log.d("FileLoggingTree", "Truncating log file: $logFile")
            // Consolidate logs: .4 -> .5, .3 -> .4, etc.
            for (i in 4 downTo 1) {
                val current = File(logFile.parent, "${logFile.name}.$i")
                val next = File(logFile.parent, "${logFile.name}.${i + 1}")
                if (current.exists()) {
                    if (next.exists()) next.delete()
                    current.renameTo(next)
                }
            }
            // Move current to .1
            val firstHistory = File(logFile.parent, "${logFile.name}.1")
            if (firstHistory.exists()) firstHistory.delete()
            logFile.renameTo(firstHistory)

            // Re-create logFile if needed or it will be created on next write
        } catch (e: Exception) {
            android.util.Log.e("FileLoggingTree", "Error truncating log file", e)
        }
    }
}
