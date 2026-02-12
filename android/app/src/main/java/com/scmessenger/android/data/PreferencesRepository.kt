package com.scmessenger.android.data

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.*
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import timber.log.Timber

/**
 * Repository for Android-specific preferences using DataStore.
 * 
 * Stores UI and app-level preferences separate from mesh settings
 * (which are stored via MeshSettingsManager in Rust).
 */
class PreferencesRepository(private val context: Context) {
    
    private val Context.dataStore: DataStore<Preferences> by preferencesDataStore(name = "app_preferences")
    
    // ========================================================================
    // KEYS
    // ========================================================================
    
    companion object {
        private val SERVICE_AUTO_START = booleanPreferencesKey("service_auto_start")
        private val VPN_MODE_ENABLED = booleanPreferencesKey("vpn_mode_enabled")
        private val ONBOARDING_COMPLETED = booleanPreferencesKey("onboarding_completed")
        private val THEME_MODE = stringPreferencesKey("theme_mode")
        private val NOTIFICATIONS_ENABLED = booleanPreferencesKey("notifications_enabled")
        private val SHOW_PEER_COUNT = booleanPreferencesKey("show_peer_count")
    }
    
    // ========================================================================
    // SERVICE PREFERENCES
    // ========================================================================
    
    val serviceAutoStart: Flow<Boolean> = context.dataStore.data.map { prefs ->
        prefs[SERVICE_AUTO_START] ?: false
    }
    
    suspend fun setServiceAutoStart(enabled: Boolean) {
        context.dataStore.edit { prefs ->
            prefs[SERVICE_AUTO_START] = enabled
        }
        Timber.d("Service auto-start: $enabled")
    }
    
    val vpnModeEnabled: Flow<Boolean> = context.dataStore.data.map { prefs ->
        prefs[VPN_MODE_ENABLED] ?: false
    }
    
    suspend fun setVpnMode(enabled: Boolean) {
        context.dataStore.edit { prefs ->
            prefs[VPN_MODE_ENABLED] = enabled
        }
        Timber.d("VPN mode: $enabled")
    }
    
    // ========================================================================
    // ONBOARDING
    // ========================================================================
    
    val onboardingCompleted: Flow<Boolean> = context.dataStore.data.map { prefs ->
        prefs[ONBOARDING_COMPLETED] ?: false
    }
    
    suspend fun setOnboardingCompleted(completed: Boolean) {
        context.dataStore.edit { prefs ->
            prefs[ONBOARDING_COMPLETED] = completed
        }
        Timber.i("Onboarding completed: $completed")
    }
    
    // ========================================================================
    // UI PREFERENCES
    // ========================================================================
    
    enum class ThemeMode {
        SYSTEM, LIGHT, DARK
    }
    
    val themeMode: Flow<ThemeMode> = context.dataStore.data.map { prefs ->
        when (prefs[THEME_MODE]) {
            "light" -> ThemeMode.LIGHT
            "dark" -> ThemeMode.DARK
            else -> ThemeMode.SYSTEM
        }
    }
    
    suspend fun setThemeMode(mode: ThemeMode) {
        context.dataStore.edit { prefs ->
            prefs[THEME_MODE] = mode.name.lowercase()
        }
        Timber.d("Theme mode: $mode")
    }
    
    val notificationsEnabled: Flow<Boolean> = context.dataStore.data.map { prefs ->
        prefs[NOTIFICATIONS_ENABLED] ?: true
    }
    
    suspend fun setNotificationsEnabled(enabled: Boolean) {
        context.dataStore.edit { prefs ->
            prefs[NOTIFICATIONS_ENABLED] = enabled
        }
        Timber.d("Notifications: $enabled")
    }
    
    val showPeerCount: Flow<Boolean> = context.dataStore.data.map { prefs ->
        prefs[SHOW_PEER_COUNT] ?: true
    }
    
    suspend fun setShowPeerCount(show: Boolean) {
        context.dataStore.edit { prefs ->
            prefs[SHOW_PEER_COUNT] = show
        }
        Timber.d("Show peer count: $show")
    }
    
    // ========================================================================
    // UTILITIES
    // ========================================================================
    
    suspend fun clearAll() {
        context.dataStore.edit { it.clear() }
        Timber.w("All preferences cleared")
    }
}
