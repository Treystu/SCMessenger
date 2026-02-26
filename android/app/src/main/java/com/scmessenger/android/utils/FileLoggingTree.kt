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
    private val logFile: File = File(context.filesDir, "mesh_diagnostics.log")
    private val dateFormat = SimpleDateFormat("yyyy-MM-dd HH:mm:ss.SSS", Locale.US)

    override fun log(priority: Int, tag: String?, message: String, t: Throwable?) {
        try {
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

            val logLine = "$timestamp $priorityStr/${tag ?: "App"}: $message\n"
            
            FileWriter(logFile, true).use { writer ->
                writer.write(logLine)
                t?.let {
                    val pw = PrintWriter(writer)
                    it.printStackTrace(pw)
                    pw.flush()
                }
            }
            
            // Limit file size to ~1MB by truncating if it gets too large
            if (logFile.length() > 1024 * 1024) {
                truncateLogFile()
            }
        } catch (e: Exception) {
            android.util.Log.e("FileLoggingTree", "Error writing to log file", e)
        }
    }

    private fun truncateLogFile() {
        try {
            val lines = logFile.readLines()
            if (lines.size > 500) {
                val keptLines = lines.takeLast(500)
                logFile.writeText(keptLines.joinToString("\n") + "\n")
            }
        } catch (e: Exception) {
            android.util.Log.e("FileLoggingTree", "Error truncating log file", e)
        }
    }
}
