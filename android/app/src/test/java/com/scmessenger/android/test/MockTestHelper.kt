package com.scmessenger.android.test

import io.mockk.*
import uniffi.api.*

/**
 * Helper functions for setting up test data.
 * Uses real UniFFI constructors instead of MockK mocks for records to avoid
 * "Missing mocked calls" errors on final classes.
 */
object MockTestHelper {

    /**
     * Create real MeshSettings with sensible defaults.
     */
    fun createMockMeshSettings(
        relayEnabled: Boolean = true,
        maxRelayBudget: UInt = 200u,
        batteryFloor: UByte = 20u.toUByte(),
        bleEnabled: Boolean = true,
        wifiAwareEnabled: Boolean = true,
        wifiDirectEnabled: Boolean = true,
        internetEnabled: Boolean = true,
        discoveryMode: DiscoveryMode = DiscoveryMode.NORMAL,
        onionRouting: Boolean = false
    ): MeshSettings {
        return MeshSettings(
            relayEnabled = relayEnabled,
            maxRelayBudget = maxRelayBudget,
            batteryFloor = batteryFloor,
            bleEnabled = bleEnabled,
            wifiAwareEnabled = wifiAwareEnabled,
            wifiDirectEnabled = wifiDirectEnabled,
            internetEnabled = internetEnabled,
            discoveryMode = discoveryMode,
            onionRouting = onionRouting,
            coverTrafficEnabled = false,
            messagePaddingEnabled = false,
            timingObfuscationEnabled = false,
            notificationsEnabled = true,
            notifyDmEnabled = true,
            notifyDmRequestEnabled = true,
            notifyDmInForeground = true,
            notifyDmRequestInForeground = true,
            soundEnabled = true,
            badgeEnabled = true
        )
    }

    /**
     * Create a real Contact instance.
     */
    fun createMockContact(
        peerId: String = "test-peer-123",
        nickname: String? = "Test User",
        publicKey: String = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
    ): Contact {
        return Contact(
            peerId = peerId,
            nickname = nickname,
            localNickname = null,
            publicKey = publicKey,
            addedAt = 1u,
            lastSeen = null,
            notes = null,
            lastKnownDeviceId = null
        )
    }

    /**
     * Create a mock IronCore instance.
     * Note: IronCore is an interface/class with methods, so it stays as a mock.
     */
    fun createMockIronCore(): IronCore {
        return mockk<IronCore>(relaxed = true) {
            every { prepareMessage(any(), any(), any(), any()) } returns PreparedMessage(
                messageId = "msg-123",
                envelopeData = ByteArray(64) { it.toByte() }
            )
        }
    }

    /**
     * Create a mock MeshSettingsManager.
     */
    fun createMockSettingsManager(
        @Suppress("UNUSED_PARAMETER") initialSettings: MeshSettings? = null
    ): Any {
        return mockk<Any>(relaxed = true) {
            every { this@mockk.toString().contains("load") } returns true
        }
    }
}
