package com.scmessenger.android.utils

private const val EPOCH_MILLIS_THRESHOLD = 1_000_000_000_000L

fun ULong.toEpochMillis(): Long {
    val raw = toLong()
    return if (raw >= EPOCH_MILLIS_THRESHOLD) raw else raw * 1000L
}

fun ULong.toEpochSeconds(): Long {
    val raw = toLong()
    return if (raw >= EPOCH_MILLIS_THRESHOLD) raw / 1000L else raw
}
