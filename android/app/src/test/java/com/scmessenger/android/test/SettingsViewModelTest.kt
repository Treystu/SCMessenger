package com.scmessenger.android.test

import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.data.PreferencesRepository
import com.scmessenger.android.ui.viewmodels.SettingsViewModel
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class SettingsViewModelTest {

    private lateinit var repository: MeshRepository
    private lateinit var preferences: PreferencesRepository
    private lateinit var viewModel: SettingsViewModel
    private val testDispatcher = StandardTestDispatcher()

    private fun settings(relayEnabled: Boolean = true) = uniffi.api.MeshSettings(
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

    @Before
    fun setup() {
        Dispatchers.setMain(testDispatcher)
        repository = mockk(relaxed = true)
        preferences = mockk(relaxed = true)

        every { repository.loadSettings() } returns settings()
        every { repository.validateSettings(any()) } returns true
        every { repository.getIdentityInfo() } returns null

        every { preferences.serviceAutoStart } returns MutableStateFlow(false)
        every { preferences.vpnModeEnabled } returns MutableStateFlow(false)
        every { preferences.themeMode } returns MutableStateFlow(PreferencesRepository.ThemeMode.SYSTEM)
        every { preferences.notificationsEnabled } returns MutableStateFlow(true)
        every { preferences.showPeerCount } returns MutableStateFlow(true)
        every { preferences.autoAdjustEnabled } returns MutableStateFlow(true)
        every { preferences.manualAdjustmentProfile } returns MutableStateFlow("STANDARD")
        every { preferences.bleRotationEnabled } returns MutableStateFlow(true)
        every { preferences.bleRotationIntervalSec } returns MutableStateFlow(900)

        viewModel = SettingsViewModel(repository, preferences)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `loadSettings populates UI state`() = runTest {
        advanceUntilIdle()
        assertEquals(true, viewModel.settings.value?.relayEnabled)
        assertNull(viewModel.error.value)
    }

    @Test
    fun `updateSettings saves when valid`() = runTest {
        val updated = settings(relayEnabled = false)
        viewModel.updateSettings(updated)
        advanceUntilIdle()

        verify(exactly = 1) { repository.saveSettings(updated) }
        assertEquals(false, viewModel.settings.value?.relayEnabled)
    }

    @Test
    fun `updateSettings rejects invalid configuration`() = runTest {
        every { repository.validateSettings(any()) } returns false

        viewModel.updateSettings(settings(relayEnabled = false))
        advanceUntilIdle()

        verify(exactly = 0) { repository.saveSettings(any()) }
        assertEquals("Invalid settings configuration", viewModel.error.value)
    }

    @Test
    fun `setAutoAdjust true clears overrides`() = runTest {
        coEvery { preferences.setAutoAdjustEnabled(true) } returns Unit

        viewModel.setAutoAdjust(true)
        advanceUntilIdle()

        coVerify(exactly = 1) { preferences.setAutoAdjustEnabled(true) }
        verify(exactly = 1) { repository.clearAdjustmentOverrides() }
    }

    @Test
    fun `setManualProfile persists preference`() = runTest {
        coEvery { preferences.setManualAdjustmentProfile(any()) } returns Unit

        viewModel.setManualProfile(uniffi.api.AdjustmentProfile.MINIMAL)
        advanceUntilIdle()

        coVerify(atLeast = 1) { preferences.setManualAdjustmentProfile("MINIMAL") }
    }

    @Test
    fun `clearAdjustmentOverrides delegates to repository`() {
        viewModel.clearAdjustmentOverrides()
        verify(exactly = 1) { repository.clearAdjustmentOverrides() }
    }
}
