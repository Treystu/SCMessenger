package com.scmessenger.android.utils

import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

private const val EPOCH_MILLIS_THRESHOLD = 1_000_000_000_000L

fun ULong.toEpochMillis(): Long {
    val raw = toLong()
    return if (raw >= EPOCH_MILLIS_THRESHOLD) raw else raw * 1000L
}

fun ULong.toEpochSeconds(): Long {
    val raw = toLong()
    return if (raw >= EPOCH_MILLIS_THRESHOLD) raw / 1000L else raw
}

/** Format a unix timestamp (seconds or millis) as e.g. "Jan 2, 2026 15:04". */
fun ULong.formatAsDateTime(): String {
    val sdf = SimpleDateFormat("MMM d, yyyy HH:mm", Locale.getDefault())
    return sdf.format(Date(toEpochMillis()))
}
