package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import timber.log.Timber
import javax.inject.Inject

/**
 * ViewModel for identity management.
 * 
 * Handles identity creation, display, QR code generation,
 * and key export/backup operations.
 */
@HiltViewModel
class IdentityViewModel @Inject constructor(
    private val meshRepository: MeshRepository
) : ViewModel() {
    
    // Identity info
    private val _identityInfo = MutableStateFlow<uniffi.api.IdentityInfo?>(null)
    val identityInfo: StateFlow<uniffi.api.IdentityInfo?> = _identityInfo.asStateFlow()
    
    // Loading state
    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()
    
    // Error state
    private val _error = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = _error.asStateFlow()
    
    // Success message (for export/copy operations)
    private val _successMessage = MutableStateFlow<String?>(null)
    val successMessage: StateFlow<String?> = _successMessage.asStateFlow()
    
    init {
        loadIdentity()
    }
    
    /**
     * Load identity information.
     */
    fun loadIdentity() {
        viewModelScope.launch {
            try {
                _isLoading.value = true
                _error.value = null
                
                val identity = meshRepository.getIdentityInfo()
                _identityInfo.value = identity
                
                if (identity == null || !identity.initialized) {
                    Timber.w("Identity not initialized")
                } else {
                    Timber.d("Identity loaded: ${identity.identityId ?: "Unknown"}")
                }
            } catch (e: Exception) {
                _error.value = "Failed to load identity: ${e.message}"
                Timber.e(e, "Failed to load identity")
            } finally {
                _isLoading.value = false
            }
        }
    }
    
    /**
     * Create a new identity (first-time setup).
     */
    fun createIdentity() {
        viewModelScope.launch {
            try {
                _isLoading.value = true
                _error.value = null
                
                meshRepository.createIdentity()
                loadIdentity()
                
                _successMessage.value = "Identity created successfully"
                Timber.i("Identity created")
            } catch (e: Exception) {
                _error.value = "Failed to create identity: ${e.message}"
                Timber.e(e, "Failed to create identity")
            } finally {
                _isLoading.value = false
            }
        }
    }
    
    /**
     * Get QR code data for sharing identity.
     * Returns JSON string with peer ID and public key.
     */
    fun getQrCodeData(): String? {
        val identity = _identityInfo.value ?: return null
        if (!identity.initialized) return null
        
        val id = identity.identityId ?: return null
        val pubKey = identity.publicKeyHex ?: return null
        
        return try {
            """{"peerId":"$id","publicKey":"$pubKey"}"""
        } catch (e: Exception) {
            Timber.e(e, "Failed to generate QR code data")
            null
        }
    }
    
    /**
     * Clear success message.
     */
    fun clearSuccessMessage() {
        _successMessage.value = null
    }
    
    /**
     * Clear error state.
     */
    fun clearError() {
        _error.value = null
    }
}
