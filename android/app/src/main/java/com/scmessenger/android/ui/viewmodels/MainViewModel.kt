package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class MainViewModel @Inject constructor(
    private val meshRepository: MeshRepository
) : ViewModel() {

    private val _isReady = MutableStateFlow(false)
    val isReady = _isReady.asStateFlow()

    private val _importError = MutableStateFlow<String?>(null)
    val importError = _importError.asStateFlow()

    private val _importSuccess = MutableStateFlow(false)
    val importSuccess = _importSuccess.asStateFlow()

    val identityInfo: uniffi.api.IdentityInfo?
        get() = meshRepository.getIdentityInfo()

    init {
        checkIdentity()
    }

    private fun checkIdentity() {
        viewModelScope.launch {
            val info = meshRepository.getIdentityInfo()
            val hasNickname = !info?.nickname.isNullOrBlank()
            if (meshRepository.isIdentityInitialized() && hasNickname) {
                _isReady.value = true
            } else {
                // Stay not ready, waiting for onboarding
                _isReady.value = false
            }
        }
    }

    fun createIdentity(nickname: String) {
        viewModelScope.launch {
            try {
                val trimmedNickname = nickname.trim()
                if (trimmedNickname.isEmpty()) {
                    Timber.w("Refusing identity creation with blank nickname")
                    _isReady.value = false
                    return@launch
                }
                meshRepository.createIdentity()
                meshRepository.setNickname(trimmedNickname)
                _isReady.value = true
            } catch (e: Exception) {
                Timber.e(e, "Failed to create identity")
            }
        }
    }

    fun importContact(jsonString: String) {
        viewModelScope.launch {
            try {
                _importError.value = null
                _importSuccess.value = false
                val json = org.json.JSONObject(jsonString)
                val publicKey = json.optString("public_key")
                val identityId = json.optString("identity_id")
                if (publicKey.isBlank() || identityId.isBlank()) {
                    _importError.value = "Invalid identity format — missing public_key or identity_id"
                    return@launch
                }
                val nickname = json.optString("nickname").takeIf { it.isNotBlank() }
                val libp2pPeerId = json.optString("libp2p_peer_id").takeIf { it.isNotBlank() }
                val listenersArr = json.optJSONArray("listeners")
                val listeners = (0 until (listenersArr?.length() ?: 0)).map { i -> listenersArr!!.getString(i) }
                val notes = libp2pPeerId?.let { pid ->
                    buildString {
                        append("libp2p_peer_id:$pid")
                        if (listeners.isNotEmpty()) append(";listeners:${listeners.joinToString(",")}")
                    }
                }
                val contact = uniffi.api.Contact(
                    peerId = identityId,
                    nickname = nickname,
                    localNickname = null,
                    publicKey = publicKey,
                    addedAt = (System.currentTimeMillis() / 1000).toULong(),
                    lastSeen = null,
                    notes = notes
                )
                meshRepository.addContact(contact)
                Timber.i("Contact imported: ${identityId.take(8)}...")
                if (!libp2pPeerId.isNullOrEmpty() && listeners.isNotEmpty()) {
                    meshRepository.connectToPeer(libp2pPeerId, listeners)
                }
                _importSuccess.value = true
            } catch (e: Exception) {
                Timber.e(e, "Failed to import contact")
                _importError.value = "Failed to import: ${e.message}"
            }
        }
    }

    fun clearImportState() {
        _importError.value = null
        _importSuccess.value = false
    }
}
