package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.data.PreferencesRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import timber.log.Timber
import uniffi.api.*
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
    private val _meshSettings = MutableStateFlow<uniffi.api.MeshSettings?>(null)
    val meshSettings: StateFlow<uniffi.api.MeshSettings?> = _meshSettings.asStateFlow()
    
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
    
    // Loading state
    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()
    
    // Error state
    private val _error = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = _error.asStateFlow()
    
    // Identity
    private val _identityInfo = MutableStateFlow<uniffi.api.IdentityInfo?>(null)
    val identityInfo: StateFlow<uniffi.api.IdentityInfo?> = _identityInfo.asStateFlow()
    
    init {
        loadMeshSettings()
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
    
    // ========================================================================
    // MESH SETTINGS
    // ========================================================================
    
    /**
     * Load mesh settings from repository.
     */
    fun loadMeshSettings() {
        viewModelScope.launch {
            try {
                _isLoading.value = true
                _error.value = null
                
                val settings = meshRepository.loadSettings()
                _meshSettings.value = settings
                
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
     * Save mesh settings.
     */
    fun saveMeshSettings(settings: uniffi.api.MeshSettings) {
        viewModelScope.launch {
            try {
                // Validate first
                if (!meshRepository.validateSettings(settings)) {
                    _error.value = "Invalid settings configuration"
                    return@launch
                }
                
                meshRepository.saveSettings(settings)
                _meshSettings.value = settings
                
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
        _meshSettings.value?.let { current ->
            saveMeshSettings(current.copy(relayEnabled = enabled))
        }
    }
    
    fun updateMaxRelayBudget(budget: UInt) {
        _meshSettings.value?.let { current ->
            saveMeshSettings(current.copy(maxRelayBudget = budget))
        }
    }
    
    fun updateBatteryFloor(floor: UByte) {
        _meshSettings.value?.let { current ->
            saveMeshSettings(current.copy(batteryFloor = floor))
        }
    }
    
    fun updateBleEnabled(enabled: Boolean) {
        _meshSettings.value?.let { current ->
            saveMeshSettings(current.copy(bleEnabled = enabled))
        }
    }
    
    fun updateWifiAwareEnabled(enabled: Boolean) {
        _meshSettings.value?.let { current ->
            saveMeshSettings(current.copy(wifiAwareEnabled = enabled))
        }
    }
    
    fun updateWifiDirectEnabled(enabled: Boolean) {
        _meshSettings.value?.let { current ->
            saveMeshSettings(current.copy(wifiDirectEnabled = enabled))
        }
    }
    
    fun updateInternetEnabled(enabled: Boolean) {
        _meshSettings.value?.let { current ->
            saveMeshSettings(current.copy(internetEnabled = enabled))
        }
    }
    
    fun updateDiscoveryMode(mode: uniffi.api.DiscoveryMode) {
        _meshSettings.value?.let { current ->
            saveMeshSettings(current.copy(discoveryMode = mode))
        }
    }
    
    fun updateOnionRouting(enabled: Boolean) {
        _meshSettings.value?.let { current ->
            saveMeshSettings(current.copy(onionRouting = enabled))
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
}

// Helper extension for copying MeshSettings
private fun uniffi.api.MeshSettings.copy(
    relayEnabled: Boolean = this.relayEnabled,
    maxRelayBudget: UInt = this.maxRelayBudget,
    batteryFloor: UByte = this.batteryFloor,
    bleEnabled: Boolean = this.bleEnabled,
    wifiAwareEnabled: Boolean = this.wifiAwareEnabled,
    wifiDirectEnabled: Boolean = this.wifiDirectEnabled,
    internetEnabled: Boolean = this.internetEnabled,
    discoveryMode: uniffi.api.DiscoveryMode = this.discoveryMode,
    onionRouting: Boolean = this.onionRouting
) = uniffi.api.MeshSettings(
    relayEnabled = relayEnabled,
    maxRelayBudget = maxRelayBudget,
    batteryFloor = batteryFloor,
    bleEnabled = bleEnabled,
    wifiAwareEnabled = wifiAwareEnabled,
    wifiDirectEnabled = wifiDirectEnabled,
    internetEnabled = internetEnabled,
    discoveryMode = discoveryMode,
    onionRouting = onionRouting
)
