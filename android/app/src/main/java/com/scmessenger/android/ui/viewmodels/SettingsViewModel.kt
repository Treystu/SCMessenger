package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.BuildConfig
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.data.PreferencesRepository
import com.scmessenger.android.network.DiagnosticsReporter
import com.scmessenger.android.ui.diagnostics.DiagnosticsBundleFormatter
import com.scmessenger.android.ui.diagnostics.DiagnosticsBundleInput
import com.scmessenger.android.utils.Permissions
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import kotlinx.coroutines.delay
import timber.log.Timber
import javax.inject.Inject
import java.util.concurrent.atomic.AtomicLong
import kotlin.concurrent.Volatile

/**
 * ViewModel for the settings screen.
 *
 * Manages mesh settings, app preferences, and configuration.
 */
@HiltViewModel
class SettingsViewModel @Inject constructor(
    private val meshRepository: MeshRepository,
    private val preferencesRepository: PreferencesRepository,
    private val diagnosticsReporter: DiagnosticsReporter
) : ViewModel() {

    // Mesh settings
    private val _settings = MutableStateFlow<uniffi.api.MeshSettings?>(null)
    val settings: StateFlow<uniffi.api.MeshSettings?> = _settings.asStateFlow()

    // ANR FIX: Debouncing for settings updates to prevent UI thread spew
    // Use AtomicLong for thread-safe timestamp tracking without locks
    private val lastSettingUpdateNs = AtomicLong(0L)
    private val settingDebounceNs = 500_000_000L  // 500ms debounce

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
    val hasIdentity: StateFlow<Boolean> = _identityInfo.map { it?.initialized == true }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), false)

    // Import result
    private val _importResult = MutableStateFlow<String?>(null)
    val importResult: StateFlow<String?> = _importResult.asStateFlow()

    // ANR FIX: Cached settings to avoid re-calculating on every composition
    private var cachedSettings: uniffi.api.MeshSettings? = null
    private var cachedIdentityInfo: uniffi.api.IdentityInfo? = null
    private var isCacheValid = false

    // ANR FIX: Guard identity emission — only emit when data actually changed.
    // Prevents infinite recomposition loops when getIdentityInfo() returns
    // structurally equal objects that would otherwise cascade through
    // StateFlow → hasIdentity → LaunchedEffect(hasIdentity) → loadIdentity().
    private fun emitIdentityInfo(info: uniffi.api.IdentityInfo?) {
        if (_identityInfo.value != info) {
            _identityInfo.value = info
            cachedIdentityInfo = info
        }
    }

    init {
        // ANR FIX (P0_ANDROID_017): Defer heavy initialization to background thread
        // Settings screen should appear within 500ms for good UX
        // Set defaults immediately so UI never shows missing sections
        _settings.value = uniffi.api.MeshSettings(
            relayEnabled = true,
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
            timingObfuscationEnabled = false,
            notificationsEnabled = true,
            notifyDmEnabled = true,
            notifyDmRequestEnabled = true,
            notifyDmInForeground = false,
            notifyDmRequestInForeground = true,
            soundEnabled = true,
            badgeEnabled = true
        )

        viewModelScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            try {
                _isLoading.value = true
                // Load settings first (depends on MeshRepository which may trigger service startup)
                loadSettingsInternal()
                // Then load identity
                loadIdentityInternal()
                // Mark cache as valid after initial load
                isCacheValid = true
            } catch (e: Exception) {
                Timber.e(e, "Failed to initialize SettingsViewModel")
            } finally {
                _isLoading.value = false
            }
        }
    }

    /**
     * Internal settings loader that runs on IO dispatcher.
     * ANR FIX: Uses cached settings when available to avoid redundant I/O.
     */
    private suspend fun loadSettingsInternal() {
        // ANR FIX: Return cached settings immediately if already loaded
        if (cachedSettings != null && isCacheValid) {
            _settings.value = cachedSettings
            return
        }
        try {
            val settings = meshRepository.loadSettings()
            cachedSettings = settings  // ANR FIX: Cache for future use
            _settings.value = settings
            Timber.d("Loaded mesh settings: $settings")
        } catch (e: Exception) {
            _error.value = "Failed to load settings: ${e.message}"
            Timber.e(e, "Failed to load mesh settings")
        }
    }

    /**
     * Internal identity loader that runs on IO dispatcher.
     * ANR FIX: Uses cached identity info when available to avoid redundant FFI calls.
     * Uses non-blocking identity access to avoid main thread blocking during service startup.
     */
    private suspend fun loadIdentityInternal() {
        // ANR FIX: Return cached identity if already loaded
        if (cachedIdentityInfo != null && isCacheValid) {
            emitIdentityInfo(cachedIdentityInfo)
            return
        }
        try {
            // Use non-blocking identity access to avoid blocking main thread
            // during service startup when identity might not be fully initialized yet
            val info = meshRepository.getIdentityInfoNonBlocking()
            // Defensive fallback: if Rust core returns blank nickname, use cached preference
            if (info != null && info.nickname.isNullOrBlank()) {
                val cached = preferencesRepository.identityNickname.first()
                if (!cached.isNullOrBlank()) {
                    Timber.w("Rust core nickname blank; using DataStore fallback: $cached")
                    // Push fallback to Rust Core to permanently fix the null state
                    meshRepository.setNickname(cached)
                    emitIdentityInfo(info.copy(nickname = cached))
                    return
                }
            }
            emitIdentityInfo(info)
        } catch (e: Exception) {
            Timber.e(e, "Failed to load identity")
        }
    }

    fun loadIdentity() {
        viewModelScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            try {
                // ANR FIX: Return cached identity if already loaded — prevents
                // redundant FFI calls during recomposition cascades.
                if (cachedIdentityInfo != null && isCacheValid) {
                    emitIdentityInfo(cachedIdentityInfo)
                    return@launch
                }
                // Use non-blocking variant to avoid triggering service init
                // from the UI layer (prevents serviceState → loadIdentity loop).
                val info = meshRepository.getIdentityInfoNonBlocking()
                // Defensive fallback: if Rust core returns blank nickname, use cached preference
                if (info != null && info.nickname.isNullOrBlank()) {
                    val cached = preferencesRepository.identityNickname.first()
                    if (!cached.isNullOrBlank()) {
                        Timber.w("Rust core nickname blank; using DataStore fallback: $cached")
                        // Push fallback to Rust Core to permanently fix the null state
                        meshRepository.setNickname(cached)
                        emitIdentityInfo(info.copy(nickname = cached))
                        return@launch
                    }
                }
                emitIdentityInfo(info)
            } catch (e: Exception) {
                Timber.e(e, "Failed to load identity")
            }
        }
    }

    suspend fun getIdentityExportString(): String {
        // ANR FIX: Run on IO dispatcher to avoid blocking Main thread
        return kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
            meshRepository.getIdentityExportString()
        }
    }

    /** P1_ANDROID_003: Import identity from a backup string. */
    fun importIdentityBackup(backup: String) {
        viewModelScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            try {
                meshRepository.restoreIdentityFromBackup(backup)
                _importResult.value = "Identity restored successfully"
                loadIdentity() // Refresh UI after import
            } catch (e: Exception) {
                _importResult.value = "Failed to restore identity: ${e.message}"
                Timber.e(e, "Failed to restore identity from backup")
            }
        }
    }

    fun clearImportResult() {
        _importResult.value = null
    }

    fun updateNickname(name: String) {
        viewModelScope.launch(Dispatchers.IO) {
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
     * ANR FIX (P0_ANDROID_017): Debounced settings update.
     * Prevents rapid successive updates from causing excessive I/O and UI thread blocking.
     * Uses timestamp-based throttling with 500ms minimum interval between updates.
     */
    fun debouncedUpdateSettings(settings: uniffi.api.MeshSettings) {
        val nowNs = System.nanoTime()
        val lastUpdateNs = lastSettingUpdateNs.get()
        val timeSinceLastUpdateNs = nowNs - lastUpdateNs

        if (timeSinceLastUpdateNs < settingDebounceNs) {
            // Still within debounce window - log and skip
            Timber.d("Settings update throttled (${timeSinceLastUpdateNs / 1_000_000}ms since last)")
            return
        }

        // Try to atomically update the timestamp
        if (lastSettingUpdateNs.compareAndSet(lastUpdateNs, nowNs)) {
            // Success - we got the lock, proceed with update
            viewModelScope.launch {
                try {
                    // Validate settings
                    if (!meshRepository.validateSettings(settings)) {
                        _error.value = "Invalid settings configuration"
                        Timber.w("Invalid settings configuration")
                        return@launch
                    }

                    meshRepository.saveSettings(settings)
                    _settings.value = settings
                    Timber.i("Mesh settings saved (debounced)")
                } catch (e: Exception) {
                    _error.value = "Failed to save settings: ${e.message}"
                    Timber.e(e, "Failed to save mesh settings")
                }
            }
        } else {
            // Failed to atomically update - someone else beat us, skip this update
            Timber.d("Settings update skipped (concurrent update in progress)")
        }
    }

    /**
     * Update mesh settings - now redirects to debounced version.
     */
    fun updateSettings(settings: uniffi.api.MeshSettings) {
        debouncedUpdateSettings(settings)
    }

    /**
     * Update specific mesh setting fields - now uses debounced updates.
     */
    fun updateRelayEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            // Allow toggling - when disabled, stops ALL communication (bidirectional)
            debouncedUpdateSettings(current.copy(relayEnabled = enabled))
            // Apply transport settings when mesh participation changes
            meshRepository.applyTransportSettings(current.copy(relayEnabled = enabled))
        }
    }

    fun updateMaxRelayBudget(budget: UInt) {
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(maxRelayBudget = budget))
        }
    }

    fun updateBatteryFloor(floor: UByte) {
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(batteryFloor = floor))
        }
    }

    fun updateBleEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(bleEnabled = enabled))
            // Apply transport settings to enable/disable BLE
            meshRepository.applyTransportSettings(current.copy(bleEnabled = enabled))
        }
    }

    fun updateWifiAwareEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(wifiAwareEnabled = enabled))
            // Apply transport settings to enable/disable WiFi Aware
            meshRepository.applyTransportSettings(current.copy(wifiAwareEnabled = enabled))
        }
    }

    fun updateWifiDirectEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(wifiDirectEnabled = enabled))
            // Apply transport settings to enable/disable WiFi Direct
            meshRepository.applyTransportSettings(current.copy(wifiDirectEnabled = enabled))
        }
    }

    fun updateInternetEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(internetEnabled = enabled))
            // Apply transport settings to enable/disable the internet transport
            meshRepository.applyTransportSettings(current.copy(internetEnabled = enabled))
        }
    }

    fun updateDiscoveryMode(mode: uniffi.api.DiscoveryMode) {
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(discoveryMode = mode))
        }
    }

    /**
     * Reset mesh settings to factory defaults.
     */
    fun resetSettingsToDefault() {
        viewModelScope.launch(Dispatchers.IO) {
            try {
                _isLoading.value = true
                val defaults = meshRepository.getDefaultSettings()
                debouncedUpdateSettings(defaults)
                Timber.i("Mesh settings reset to defaults")
            } catch (e: Exception) {
                _error.value = "Failed to reset settings: ${e.message}"
                Timber.e(e, "Failed to reset mesh settings")
            } finally {
                _isLoading.value = false
            }
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

    fun getConnectionPathState(): uniffi.api.ConnectionPathState {
        return meshRepository.getConnectionPathState()
    }

    fun getNatStatus(): String {
        return meshRepository.getNatStatus()
    }

    /**
     * ANR FIX (P0_ANDROID_017): Export diagnostics asynchronously.
     * Original exportDiagnostics() was blocking on file I/O and Rust FFI calls.
     */
    suspend fun exportDiagnostics(): String {
        return kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
            meshRepository.exportDiagnostics()
        }
    }

    fun getDiagnosticsLogPath(): String {
        return meshRepository.getDiagnosticsLogPath()
    }

    /**
     * ANR FIX (P0_ANDROID_017): Get diagnostics logs asynchronously.
     * Reading log files was blocking on Main thread.
     */
    suspend fun getDiagnosticsLogs(limit: Int = 500): String {
        return kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
            meshRepository.getDiagnosticsLogs(limit)
        }
    }

    fun clearDiagnosticsLogs() {
        meshRepository.clearDiagnosticsLogs()
    }

    /** P0_ANDROID_007: Retry bootstrap with fallback strategy from diagnostics UI. */
    fun retryBootstrap() {
        viewModelScope.launch {
            try {
                meshRepository.bootstrapWithFallbackStrategy()
                Timber.i("Bootstrap retry initiated")
            } catch (e: Exception) {
                Timber.e(e, "Bootstrap retry failed")
            }
        }
    }

    /**
     * ANR FIX (P0_ANDROID_017): Build diagnostics bundle asynchronously.
     * All blocking I/O operations (exportDiagnostics, getPendingOutboxCount, getDiagnosticsLogs)
     * are moved to IO dispatcher to prevent UI thread blocking.
     */
    suspend fun buildTesterDiagnosticsBundle(): String {
        // Run the entire operation on IO dispatcher to avoid Main thread I/O
        return kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
            val missingPermissions = meshRepository.getMissingRuntimePermissions().map { permission ->
                "${Permissions.getPermissionName(permission)} ($permission)"
            }
            DiagnosticsBundleFormatter.format(
                DiagnosticsBundleInput(
                    generatedAtEpochMs = System.currentTimeMillis(),
                    appVersion = BuildConfig.VERSION_NAME,
                    serviceState = meshRepository.getServiceStateName(),
                    connectionPathState = meshRepository.getConnectionPathState().name,
                    natStatus = meshRepository.getNatStatus(),
                    discoveredPeers = meshRepository.getDiscoveredPeerCount(),
                    pendingOutbox = meshRepository.loadPendingOutbox().size,
                    missingPermissions = missingPermissions,
                    coreDiagnosticsJson = meshRepository.exportDiagnostics(),
                    recentLogs = meshRepository.getDiagnosticsLogs(limit = 1500)
                )
            )
        }
    }

    // ========================================================================
    // DIAGNOSTICS
    // ========================================================================

    /**
     * Get network diagnostics report from DiagnosticsReporter.
     */
    suspend fun getNetworkDiagnosticsReport(): DiagnosticsReporter.NetworkDiagnosticsReport? {
        return try {
            diagnosticsReporter.generateReport()
        } catch (e: Exception) {
            Timber.e(e, "Failed to generate diagnostics report")
            null
        }
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

    /**
     * Resets all application data and preferences.
     */
    fun resetAllData() {
        viewModelScope.launch {
            try {
                _isLoading.value = true

                // Clear app-level preferences
                preferencesRepository.clearAll()

                // Reset mesh data (identity, history, etc)
                meshRepository.resetAllData()

                Timber.i("Application reset complete")
            } catch (e: Exception) {
                _error.value = "Failed to reset application: ${e.message}"
                Timber.e(e, "Reset failed")
            } finally {
                _isLoading.value = false
            }
        }
    }
}
