package com.scmessenger.android.data

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
}
