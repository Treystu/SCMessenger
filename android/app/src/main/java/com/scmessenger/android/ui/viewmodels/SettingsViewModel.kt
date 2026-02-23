package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.data.PreferencesRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import timber.log.Timber
import javax.inject.Inject

/**
 * ViewModel for the settings screen.
 *
 * Manages mesh settings, app preferences, and configuration.
 */
@HiltViewModel
class SettingsViewModel @Inject constructor(
    private val meshRepository: MeshRepository,
    private val preferencesRepository: PreferencesRepository
) : ViewModel() {

    // Mesh settings
    private val _settings = MutableStateFlow<uniffi.api.MeshSettings?>(null)
    val settings: StateFlow<uniffi.api.MeshSettings?> = _settings.asStateFlow()

    // App preferences
    val autoStart = preferencesRepository.serviceAutoStart
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = false
        )

    val vpnMode = preferencesRepository.vpnModeEnabled
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = false
        )

    val themeMode = preferencesRepository.themeMode
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = PreferencesRepository.ThemeMode.SYSTEM
        )

    val notificationsEnabled = preferencesRepository.notificationsEnabled
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = true
        )

    val showPeerCount = preferencesRepository.showPeerCount
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = true
        )

    val autoAdjustEnabled = preferencesRepository.autoAdjustEnabled
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = true
        )

    val adjustmentProfile = preferencesRepository.manualAdjustmentProfile
        .map { name ->
            try {
                uniffi.api.AdjustmentProfile.valueOf(name)
            } catch (e: IllegalArgumentException) {
                uniffi.api.AdjustmentProfile.STANDARD
            }
        }
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = uniffi.api.AdjustmentProfile.STANDARD
        )

    // BLE Privacy (mirrors iOS BLE rotation settings)
    val bleRotationEnabled = preferencesRepository.bleRotationEnabled
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = true
        )

    val bleRotationIntervalSec = preferencesRepository.bleRotationIntervalSec
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = 900
        )

    // Loading state
    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()
    val isSaving: StateFlow<Boolean> = _isLoading.asStateFlow()

    // Error state
    private val _error = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = _error.asStateFlow()

    // Identity
    private val _identityInfo = MutableStateFlow<uniffi.api.IdentityInfo?>(null)
    val identityInfo: StateFlow<uniffi.api.IdentityInfo?> = _identityInfo.asStateFlow()

    init {
        loadSettings()
        loadIdentity()
    }

    fun loadIdentity() {
        viewModelScope.launch {
            try {
                _identityInfo.value = meshRepository.getIdentityInfo()
            } catch (e: Exception) {
                Timber.e(e, "Failed to load identity")
            }
        }
    }

    fun getIdentityExportString(): String {
        return meshRepository.getIdentityExportString()
    }

    fun updateNickname(name: String) {
        viewModelScope.launch {
            try {
                meshRepository.setNickname(name)
                loadIdentity() // Refresh to reflect change
            } catch (e: Exception) {
                _error.value = "Failed to update nickname: ${e.message}"
                Timber.e(e, "Failed to update nickname")
            }
        }
    }

    // ========================================================================
    // MESH SETTINGS
    // ========================================================================

    /**
     * Load mesh settings from repository.
     */
    fun loadSettings() {
        viewModelScope.launch {
            try {
                _isLoading.value = true
                _error.value = null

                val settings = meshRepository.loadSettings()
                _settings.value = settings

                Timber.d("Loaded mesh settings: $settings")
            } catch (e: Exception) {
                _error.value = "Failed to load settings: ${e.message}"
                Timber.e(e, "Failed to load mesh settings")
            } finally {
                _isLoading.value = false
            }
        }
    }

    /**
     * Update mesh settings.
     */
    fun updateSettings(settings: uniffi.api.MeshSettings) {
        viewModelScope.launch {
            try {
                // Validate settings
                if (!meshRepository.validateSettings(settings)) {
                    _error.value = "Invalid settings configuration"
                    return@launch
                }

                meshRepository.saveSettings(settings)
                _settings.value = settings

                Timber.i("Mesh settings saved")
            } catch (e: Exception) {
                _error.value = "Failed to save settings: ${e.message}"
                Timber.e(e, "Failed to save mesh settings")
            }
        }
    }

    /**
     * Update specific mesh setting fields.
     */
    fun updateRelayEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            // Allow toggling - when disabled, stops ALL communication (bidirectional)
            updateSettings(current.copy(relayEnabled = enabled))
        }
    }

    fun updateMaxRelayBudget(budget: UInt) {
        _settings.value?.let { current ->
            updateSettings(current.copy(maxRelayBudget = budget))
        }
    }

    fun updateBatteryFloor(floor: UByte) {
        _settings.value?.let { current ->
            updateSettings(current.copy(batteryFloor = floor))
        }
    }

    fun updateBleEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            updateSettings(current.copy(bleEnabled = enabled))
        }
    }

    fun updateWifiAwareEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            updateSettings(current.copy(wifiAwareEnabled = enabled))
        }
    }

    fun updateWifiDirectEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            updateSettings(current.copy(wifiDirectEnabled = enabled))
        }
    }

    fun updateInternetEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            updateSettings(current.copy(internetEnabled = enabled))
        }
    }

    fun updateDiscoveryMode(mode: uniffi.api.DiscoveryMode) {
        _settings.value?.let { current ->
            updateSettings(current.copy(discoveryMode = mode))
        }
    }

    fun updateOnionRouting(enabled: Boolean) {
        _settings.value?.let { current ->
            updateSettings(current.copy(onionRouting = enabled))
        }
    }

    fun updateCoverTrafficEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            updateSettings(current.copy(coverTrafficEnabled = enabled))
        }
    }

    fun updateMessagePaddingEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            updateSettings(current.copy(messagePaddingEnabled = enabled))
        }
    }

    fun updateTimingObfuscationEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            updateSettings(current.copy(timingObfuscationEnabled = enabled))
        }
    }

    // ========================================================================
    // APP PREFERENCES
    // ========================================================================

    fun setAutoStart(enabled: Boolean) {
        viewModelScope.launch {
            preferencesRepository.setServiceAutoStart(enabled)
        }
    }

    fun setVpnMode(enabled: Boolean) {
        viewModelScope.launch {
            preferencesRepository.setVpnMode(enabled)
        }
    }

    fun setThemeMode(mode: PreferencesRepository.ThemeMode) {
        viewModelScope.launch {
            preferencesRepository.setThemeMode(mode)
        }
    }

    fun setNotificationsEnabled(enabled: Boolean) {
        viewModelScope.launch {
            preferencesRepository.setNotificationsEnabled(enabled)
        }
    }

    fun setShowPeerCount(show: Boolean) {
        viewModelScope.launch {
            preferencesRepository.setShowPeerCount(show)
        }
    }

    // ========================================================================
    // POWER SETTINGS
    // ========================================================================

    fun setAutoAdjust(enabled: Boolean) {
        viewModelScope.launch {
            preferencesRepository.setAutoAdjustEnabled(enabled)
            if (enabled) {
                meshRepository.clearAdjustmentOverrides()
            }
        }
    }

    fun setManualProfile(profile: uniffi.api.AdjustmentProfile) {
        viewModelScope.launch {
            preferencesRepository.setManualAdjustmentProfile(profile.name)

            // Apply profile settings if manual
            // Note: In a real implementation, we might apply these to the engine directly
            // or the engine observes this preference.
            // For now, we just save the preference.
        }
    }

    fun overrideBleScanInterval(intervalMs: UInt) {
        meshRepository.overrideBleInterval(intervalMs)
    }

    fun overrideRelayMax(max: UInt) {
        meshRepository.overrideRelayMax(max)
    }

    fun clearAdjustmentOverrides() {
        meshRepository.clearAdjustmentOverrides()
    }

    // ========================================================================
    // UTILITIES
    // ========================================================================

    /**
     * Clear error state.
     */
    fun clearError() {
        _error.value = null
    }

    /**
     * Get ledger summary for diagnostics.
     */
    fun getLedgerSummary(): String {
        return meshRepository.getLedgerSummary()
    }



    // MARK: - Identity Helpers
    /**
     * Get contact count for info display.
     */
    fun getContactCount(): UInt {
        return meshRepository.getContactCount()
    }

    /**
     * Get message count for info display.
     */
    fun getMessageCount(): UInt {
        return meshRepository.getMessageCount()
    }

    // ========================================================================
    // BLE PRIVACY (mirrors iOS SettingsViewModel BLE rotation)
    // ========================================================================

    fun setBleRotationEnabled(enabled: Boolean) {
        viewModelScope.launch {
            preferencesRepository.setBleRotationEnabled(enabled)
        }
    }

    fun setBleRotationIntervalSec(intervalSec: Int) {
        viewModelScope.launch {
            preferencesRepository.setBleRotationIntervalSec(intervalSec)
        }
    }
}
