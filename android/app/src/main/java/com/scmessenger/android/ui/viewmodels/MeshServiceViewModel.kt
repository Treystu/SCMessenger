package com.scmessenger.android.ui.viewmodels

import android.content.Context
import android.content.Intent
import android.os.Build
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.data.PreferencesRepository
import com.scmessenger.android.service.MeshForegroundService
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import timber.log.Timber
import javax.inject.Inject

/**
 * ViewModel managing the mesh service lifecycle and state.
 * 
 * This is shared across the app to provide consistent service state
 * and control methods.
 */
@HiltViewModel
class MeshServiceViewModel @Inject constructor(
    @ApplicationContext private val context: Context,
    private val meshRepository: MeshRepository,
    private val preferencesRepository: PreferencesRepository
) : ViewModel() {
    
    // Service state from repository
    val serviceState = meshRepository.serviceState
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = uniffi.api.ServiceState.STOPPED
        )
    
    // Service stats from repository
    val serviceStats = meshRepository.serviceStats
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = null
        )
    
    // Preferences
    val autoStart = preferencesRepository.serviceAutoStart
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = false
        )
    
    // Derived state: is service running
    val isRunning = serviceState.map { it == uniffi.api.ServiceState.RUNNING }
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = false
        )
    
    init {
        Timber.d("MeshServiceViewModel initialized")
    }
    
    /**
     * Start the mesh service.
     */
    fun startService() {
        viewModelScope.launch {
            try {
                val intent = Intent(context, MeshForegroundService::class.java).apply {
                    action = MeshForegroundService.ACTION_START
                }
                
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                    context.startForegroundService(intent)
                } else {
                    context.startService(intent)
                }
                
                Timber.i("Mesh service start requested")
            } catch (e: Exception) {
                Timber.e(e, "Failed to start mesh service")
            }
        }
    }
    
    /**
     * Stop the mesh service.
     */
    fun stopService() {
        viewModelScope.launch {
            try {
                val intent = Intent(context, MeshForegroundService::class.java).apply {
                    action = MeshForegroundService.ACTION_STOP
                }
                context.startService(intent)
                
                Timber.i("Mesh service stop requested")
            } catch (e: Exception) {
                Timber.e(e, "Failed to stop mesh service")
            }
        }
    }
    
    /**
     * Toggle the mesh service on/off.
     */
    fun toggleService() {
        when (serviceState.value) {
            uniffi.api.ServiceState.STOPPED -> startService()
            uniffi.api.ServiceState.RUNNING -> stopService()
            else -> {
                Timber.w("Cannot toggle service in state: ${serviceState.value}")
            }
        }
    }
    
    /**
     * Set auto-start preference.
     */
    fun setAutoStart(enabled: Boolean) {
        viewModelScope.launch {
            preferencesRepository.setServiceAutoStart(enabled)
            Timber.d("Auto-start set to: $enabled")
        }
    }
    
    /**
     * Get formatted stats for display.
     */
    fun getStatsText(): String {
        val stats = serviceStats.value ?: return "No stats available"
        
        return buildString {
            appendLine("Peers Discovered: ${stats.peersDiscovered}")
            appendLine("Messages Relayed: ${stats.messagesRelayed}")
            appendLine("Bytes Transferred: ${formatBytes(stats.bytesTransferred)}")
            appendLine("Uptime: ${formatDuration(stats.uptimeSecs)}")
        }
    }
    
    private fun formatBytes(bytes: ULong): String {
        return when {
            bytes < 1024u -> "$bytes B"
            bytes < 1024u * 1024u -> "${bytes / 1024u} KB"
            bytes < 1024u * 1024u * 1024u -> "${bytes / (1024u * 1024u)} MB"
            else -> "${bytes / (1024u * 1024u * 1024u)} GB"
        }
    }
    
    private fun formatDuration(seconds: ULong): String {
        val secs = seconds.toLong()
        val hours = secs / 3600
        val minutes = (secs % 3600) / 60
        val remainingSeconds = secs % 60
        
        return when {
            hours > 0 -> "${hours}h ${minutes}m"
            minutes > 0 -> "${minutes}m ${remainingSeconds}s"
            else -> "${remainingSeconds}s"
        }
    }
}
