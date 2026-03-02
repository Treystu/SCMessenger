package com.scmessenger.android.data

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class MeshRepositoryTest {

    private fun meshSettings(relayEnabled: Boolean): uniffi.api.MeshSettings {
        return uniffi.api.MeshSettings(
            relayEnabled = relayEnabled,
            maxRelayBudget = 200u,
            batteryFloor = 20u,
            bleEnabled = true,
            wifiAwareEnabled = true,
            wifiDirectEnabled = true,
            internetEnabled = true,
            discoveryMode = uniffi.api.DiscoveryMode.NORMAL,
            onionRouting = false,
            coverTrafficEnabled = false,
            messagePaddingEnabled = false,
            timingObfuscationEnabled = false
        )
    }

    @Test
    fun `isMeshParticipationEnabled true when relay enabled`() {
        assertTrue(MeshRepository.isMeshParticipationEnabled(meshSettings(true)))
    }

    @Test
    fun `isMeshParticipationEnabled false when relay disabled`() {
        assertFalse(MeshRepository.isMeshParticipationEnabled(meshSettings(false)))
    }

    @Test
    fun `isMeshParticipationEnabled false when settings null`() {
        assertFalse(MeshRepository.isMeshParticipationEnabled(null))
    }

    @Test
    fun `requireMeshParticipationEnabled allows enabled settings`() {
        MeshRepository.requireMeshParticipationEnabled(meshSettings(true))
    }

    @Test(expected = IllegalStateException::class)
    fun `requireMeshParticipationEnabled throws for disabled settings`() {
        MeshRepository.requireMeshParticipationEnabled(meshSettings(false))
    }

    @Test(expected = IllegalStateException::class)
    fun `requireMeshParticipationEnabled throws for null settings`() {
        MeshRepository.requireMeshParticipationEnabled(null)
    }

    @Test
    fun `enabled helper remains true regardless of unrelated setting fields`() {
        val settings = meshSettings(relayEnabled = true).copy(
            bleEnabled = false,
            wifiAwareEnabled = false,
            wifiDirectEnabled = false,
            internetEnabled = false
        )
        assertTrue(MeshRepository.isMeshParticipationEnabled(settings))
    }

    @Test
    fun `disabled helper remains false regardless of budget values`() {
        val settings = meshSettings(relayEnabled = false).copy(
            maxRelayBudget = 999u,
            batteryFloor = 0.toUByte()
        )
        assertFalse(MeshRepository.isMeshParticipationEnabled(settings))
    }

    @Test
    fun `require helper throws consistently across repeated calls`() {
        repeat(3) {
            val thrown = kotlin.runCatching {
                MeshRepository.requireMeshParticipationEnabled(null)
            }.exceptionOrNull()
            assertTrue(thrown is IllegalStateException)
        }
    }

    @Test
    fun `require helper never throws for enabled settings across repeated calls`() {
        val settings = meshSettings(relayEnabled = true)
        repeat(10) {
            MeshRepository.requireMeshParticipationEnabled(settings)
        }
    }

    @Test
    fun `error message for disabled participation is descriptive`() {
        val err = kotlin.runCatching {
            MeshRepository.requireMeshParticipationEnabled(null)
        }.exceptionOrNull()

        assertTrue(err is IllegalStateException)
        assertTrue(err?.message?.contains("mesh participation is disabled") == true)
    }

    @Test
    fun `mesh participation helper is deterministic`() {
        val enabled = meshSettings(true)
        val disabled = meshSettings(false)
        repeat(10) {
            assertTrue(MeshRepository.isMeshParticipationEnabled(enabled))
            assertFalse(MeshRepository.isMeshParticipationEnabled(disabled))
        }
    }

    @Test
    fun `wifi local path succeeds without BLE fallback`() {
        val attempted = mutableListOf<String>()

        val result = MeshRepository.attemptWifiThenBleFallback(
            wifiPeerId = "192.168.49.23",
            blePeerId = "6d1564ca-10f5-4af9-8a2f-9a50bbf024f5",
            tryWifi = {
                attempted.add("wifi")
                true
            },
            tryBle = {
                attempted.add("ble")
                true
            }
        )

        assertTrue(result.wifiAttempted)
        assertTrue(result.wifiAcked)
        assertFalse(result.bleAttempted)
        assertFalse(result.bleAcked)
        assertTrue(result.acked)
        assertEquals(listOf("wifi"), attempted)
    }

    @Test
    fun `wifi unavailable falls back deterministically to BLE`() {
        val attempted = mutableListOf<String>()

        val result = MeshRepository.attemptWifiThenBleFallback(
            wifiPeerId = "192.168.49.42",
            blePeerId = "1fd24e84-4927-4a18-bf4b-0619d706d8a1",
            tryWifi = {
                attempted.add("wifi")
                false
            },
            tryBle = {
                attempted.add("ble")
                true
            }
        )

        assertTrue(result.wifiAttempted)
        assertFalse(result.wifiAcked)
        assertTrue(result.bleAttempted)
        assertTrue(result.bleAcked)
        assertTrue(result.acked)
        assertEquals(listOf("wifi", "ble"), attempted)
    }

    @Test
    fun `high volume local sync fallback remains stable`() {
        var wifiCalls = 0
        var bleCalls = 0
        var wifiSuccesses = 0
        var bleFallbackSuccesses = 0

        repeat(1_500) { index ->
            val wifiShouldSucceed = index % 3 != 0
            val result = MeshRepository.attemptWifiThenBleFallback(
                wifiPeerId = "192.168.49.5",
                blePeerId = "e05d1580-fdc0-4c9a-9991-f2f5f67b6d10",
                tryWifi = {
                    wifiCalls += 1
                    wifiShouldSucceed
                },
                tryBle = {
                    bleCalls += 1
                    true
                }
            )

            if (wifiShouldSucceed) {
                wifiSuccesses += 1
                assertTrue(result.wifiAcked)
                assertFalse(result.bleAttempted)
            } else {
                bleFallbackSuccesses += 1
                assertFalse(result.wifiAcked)
                assertTrue(result.bleAttempted)
                assertTrue(result.bleAcked)
            }
            assertTrue(result.acked)
        }

        assertEquals(1_500, wifiCalls)
        assertEquals(500, bleCalls)
        assertEquals(1_000, wifiSuccesses)
        assertEquals(500, bleFallbackSuccesses)
    }
}
