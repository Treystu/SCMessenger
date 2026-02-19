package com.scmessenger.android.utils

import android.Manifest
import android.os.Build

/**
 * Permissions required for SCMessenger functionality.
 *
 * Organized by feature and API level.
 */
object Permissions {

    /**
     * Bluetooth permissions (varies by API level).
     */
    val bluetooth: List<String> = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        // Android 12+ (API 31+)
        listOf(
            Manifest.permission.BLUETOOTH_SCAN,
            Manifest.permission.BLUETOOTH_ADVERTISE,
            Manifest.permission.BLUETOOTH_CONNECT
        )
    } else {
        // Android 11 and below
        listOf(
            Manifest.permission.BLUETOOTH,
            Manifest.permission.BLUETOOTH_ADMIN
        )
    }

    /**
     * Location permissions (required for WiFi Aware).
     */
    val location: List<String> = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
        // Android 10+ (API 29+)
        listOf(
            Manifest.permission.ACCESS_FINE_LOCATION,
            Manifest.permission.ACCESS_COARSE_LOCATION
        )
    } else {
        listOf(
            Manifest.permission.ACCESS_FINE_LOCATION
        )
    }

    /**
     * Nearby WiFi devices (Android 13+).
     */
    val nearbyWifi: List<String> = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        listOf(Manifest.permission.NEARBY_WIFI_DEVICES)
    } else {
        emptyList()
    }

    /**
     * Notifications permission (Android 13+).
     */
    val notifications: List<String> = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        listOf(Manifest.permission.POST_NOTIFICATIONS)
    } else {
        emptyList()
    }

    /**
     * All required permissions for core functionality.
     */
    val required: List<String> = bluetooth + location + nearbyWifi

    /**
     * All permissions including optional ones.
     */
    val all: List<String> = required + notifications

    /**
     * Get user-friendly permission names for rationale dialogs.
     */
    fun getPermissionName(permission: String): String = when (permission) {
        Manifest.permission.BLUETOOTH,
        Manifest.permission.BLUETOOTH_ADMIN,
        Manifest.permission.BLUETOOTH_SCAN,
        Manifest.permission.BLUETOOTH_ADVERTISE,
        Manifest.permission.BLUETOOTH_CONNECT -> "Bluetooth"

        Manifest.permission.ACCESS_FINE_LOCATION,
        Manifest.permission.ACCESS_COARSE_LOCATION -> "Location"

        Manifest.permission.NEARBY_WIFI_DEVICES -> "Nearby WiFi Devices"

        Manifest.permission.POST_NOTIFICATIONS -> "Notifications"

        else -> permission
    }

    /**
     * Get rationale explanation for a permission.
     */
    fun getRationale(permission: String): String = when (permission) {
        in bluetooth -> "Bluetooth is required for discovering and messaging with nearby peers."

        in location -> "Location access is required for WiFi Aware peer discovery. " +
                      "SCMessenger does not track or store your location."

        in nearbyWifi -> "Nearby WiFi devices permission enables WiFi Aware for " +
                        "direct peer-to-peer messaging without infrastructure."

        in notifications -> "Notifications keep you informed when new messages arrive."

        else -> "This permission is required for mesh networking."
    }
}
