package com.scmessenger.android.test

import io.mockk.*
import uniffi.api.*

/**
 * Helper functions for setting up mocks in tests.
 * Provides common mock configurations for UniFFI objects.
 */
object MockTestHelper {

    /**
     * Create a mock MeshSettings with sensible defaults.
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
        return mockk<MeshSettings>(relaxed = true) {
            every { this@mockk.relayEnabled } returns relayEnabled
            every { this@mockk.maxRelayBudget } returns maxRelayBudget
            every { this@mockk.batteryFloor } returns batteryFloor
            every { this@mockk.bleEnabled } returns bleEnabled
            every { this@mockk.wifiAwareEnabled } returns wifiAwareEnabled
            every { this@mockk.wifiDirectEnabled } returns wifiDirectEnabled
            every { this@mockk.internetEnabled } returns internetEnabled
            every { this@mockk.discoveryMode } returns discoveryMode
            every { this@mockk.onionRouting } returns onionRouting
        }
    }

    /**
     * Create a mock Contact with sensible defaults.
     */
    fun createMockContact(
        peerId: String = "test-peer-123",
        nickname: String? = "Test User",
        publicKey: String = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
    ): Contact {
        return mockk<Contact>(relaxed = true) {
            every { this@mockk.peerId } returns peerId
            every { this@mockk.nickname } returns nickname
            every { this@mockk.publicKey } returns publicKey
        }
    }

    /**
     * Create a mock IronCore instance.
     */
    fun createMockIronCore(): IronCore {
        return mockk<IronCore>(relaxed = true) {
            every { prepareMessage(any(), any()) } returns ByteArray(64) { it.toByte() }
            // receiveMessage is no longer available or has different signature
            /*
            every { receiveMessage(any(), any()) } returns MessageEnvelope(
                messageId = "msg-123",
                senderId = "sender-456",
                recipientId = "recipient-789",
                content = "Test message".toByteArray(),
                timestamp = System.currentTimeMillis().toULong(),
                signature = ByteArray(64)
            )
            */
        }
    }

    /**
     * Create a mock MeshSettingsManager.
     */
    fun createMockSettingsManager(
        initialSettings: MeshSettings? = null
    ): Any {
        return mockk<Any>(relaxed = true) {
            every { this@mockk.toString().contains("load") } returns true
        }
    }
}
