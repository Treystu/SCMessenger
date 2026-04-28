package com.scmessenger.android.ui.viewmodels

import android.net.Uri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.data.PreferencesRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import timber.log.Timber
import com.scmessenger.android.utils.StorageManager
import javax.inject.Inject

@HiltViewModel
class MainViewModel @Inject constructor(
    private val meshRepository: MeshRepository,
    private val preferencesRepository: PreferencesRepository
) : ViewModel() {

    private val _isReady = MutableStateFlow(false)
    val isReady = _isReady.asStateFlow()
    val hasIdentity = _isReady.asStateFlow()

    private val _onboardingCompleted = MutableStateFlow(false)
    val onboardingCompleted = _onboardingCompleted.asStateFlow()

    private val _installChoiceCompleted = MutableStateFlow(false)
    val installChoiceCompleted = _installChoiceCompleted.asStateFlow()

    // showOnboarding is true if NOT ready AND NOT install choice completed
    val showOnboarding = combine(
        _isReady,
        _installChoiceCompleted
    ) { ready, choiceCompleted ->
        !ready && !choiceCompleted
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = true
    )

    private val _isCreatingIdentity = MutableStateFlow(false)
    val isCreatingIdentity = _isCreatingIdentity.asStateFlow()

    private val _identityError = MutableStateFlow<String?>(null)
    val identityError = _identityError.asStateFlow()

    private val _importError = MutableStateFlow<String?>(null)
    val importError = _importError.asStateFlow()

    private val _importSuccess = MutableStateFlow(false)
    val importSuccess = _importSuccess.asStateFlow()

    val identityInfo: uniffi.api.IdentityInfo?
        get() = meshRepository.getIdentityInfo()

    private val _isStorageLow = MutableStateFlow(false)
    val isStorageLow = _isStorageLow.asStateFlow()

    private val _availableStorageMB = MutableStateFlow(0L)
    val availableStorageMB = _availableStorageMB.asStateFlow()

    private val _pendingDeepLink = MutableStateFlow<DeepLinkData?>(null)
    val pendingDeepLink: StateFlow<DeepLinkData?> = _pendingDeepLink.asStateFlow()

    private val _themeMode = MutableStateFlow(PreferencesRepository.ThemeMode.SYSTEM)
    val themeMode: StateFlow<PreferencesRepository.ThemeMode> = _themeMode.asStateFlow()

    init {
        Timber.d("MainViewModel init")
        refreshStorageStatus()

        // Observe preferences
        viewModelScope.launch {
            preferencesRepository.onboardingCompleted.collect { completed ->
                Timber.d("Preference onboardingCompleted: $completed")
                _onboardingCompleted.value = completed
            }
        }
        viewModelScope.launch {
            preferencesRepository.installChoiceCompleted.collect { completed ->
                Timber.d("Preference installChoiceCompleted: $completed")
                _installChoiceCompleted.value = completed
            }
        }
        viewModelScope.launch {
            preferencesRepository.themeMode.collect { mode ->
                Timber.d("Preference themeMode: $mode")
                _themeMode.value = mode
            }
        }

        // Auto-refresh identity state when service state changes (important for lazy start)
        viewModelScope.launch {
            meshRepository.serviceState.collect { state ->
                Timber.d("MeshRepository service state: $state")
                if (state == uniffi.api.ServiceState.RUNNING) {
                    refreshIdentityState()
                }
            }
        }

        refreshIdentityState()
    }

    fun grantConsent() {
        viewModelScope.launch(Dispatchers.IO) {
            try {
                meshRepository.grantConsent()
                Timber.i("Consent granted via MainViewModel")
            } catch (e: Exception) {
                Timber.e(e, "Failed to grant consent")
            }
        }
    }

    fun refreshIdentityState() {
        viewModelScope.launch(Dispatchers.IO) {
            Timber.d("refreshIdentityState() called")
            val initialized = meshRepository.isIdentityInitialized()
            Timber.d("Identity initialized state: $initialized")
            _identityError.value = null
            _isReady.value = initialized

            if (initialized) {
                if (!_installChoiceCompleted.value) {
                    Timber.d("Identity is initialized but install choice not completed, fixing preference...")
                    preferencesRepository.setInstallChoiceCompleted(true)
                }
                if (!_onboardingCompleted.value) {
                    Timber.d("Identity is initialized but onboarding not completed, fixing preference...")
                    preferencesRepository.setOnboardingCompleted(true)
                }
            }
        }
    }

    /**
     * Create a new identity with retry logic for _isReady verification.
     * Issue #5: Added retry/verify loop to ensure _isReady reflects actual state
     * after identity creation (service may still be starting when first checked).
     */
    fun createIdentity(nickname: String) {
        viewModelScope.launch(Dispatchers.IO) {
            _isCreatingIdentity.value = true
            _identityError.value = null
            try {
                val trimmedNickname = nickname.trim()
                if (trimmedNickname.isEmpty()) {
                    Timber.w("Refusing identity creation with blank nickname")
                    _identityError.value = "Nickname is required"
                    _isReady.value = false
                    return@launch
                }
                Timber.i("Creating identity for nickname: $trimmedNickname")
                meshRepository.createIdentity()
                meshRepository.setNickname(trimmedNickname)

                // Verify nickname persisted (defensive: catch silent Rust-core failures)
                var info = meshRepository.getIdentityInfo()
                if (info?.nickname.isNullOrBlank()) {
                    Timber.w("Nickname was blank after setNickname; retrying once")
                    meshRepository.setNickname(trimmedNickname)
                    info = meshRepository.getIdentityInfo()
                }

                // Issue #5: Retry loop for _isReady verification
                // The service may still be starting, so we poll for up to 2 seconds
                // to ensure _isReady accurately reflects the identity state.
                var initialized = meshRepository.isIdentityInitialized()
                var retryCount = 0
                val maxRetries = 5
                val retryDelayMs = 100L

                while (!initialized && retryCount < maxRetries) {
                    kotlinx.coroutines.delay(retryDelayMs)
                    initialized = meshRepository.isIdentityInitialized()
                    retryCount++
                    Timber.d("isIdentityInitialized retry $retryCount/$maxRetries: $initialized")
                }

                Timber.i("Identity creation result initialized: $initialized; nickname=${info?.nickname}; retries=$retryCount")
                _isReady.value = initialized
                if (_isReady.value) {
                    preferencesRepository.setOnboardingCompleted(true)
                    preferencesRepository.setInstallChoiceCompleted(true)
                    // Defensive: cache nickname in DataStore as fallback for Rust-core regression
                    preferencesRepository.setIdentityNickname(trimmedNickname)
                }
            } catch (e: Exception) {
                Timber.e(e, "Failed to create identity")
                _identityError.value = e.message ?: "Failed to create identity"
                _isReady.value = false
            } finally {
                _isCreatingIdentity.value = false
            }
        }
    }

    fun clearIdentityError() {
        _identityError.value = null
    }

    fun refreshStorageStatus() {
        viewModelScope.launch(Dispatchers.IO) {
            val available = meshRepository.getAvailableStorageMB()
            _availableStorageMB.value = available
            _isStorageLow.value = available < StorageManager.CRITICAL_STORAGE_THRESHOLD_MB
            Timber.d("Storage refreshed: $available MB available (Low=${_isStorageLow.value})")
        }
    }

    fun importContact(jsonString: String) {
        viewModelScope.launch {
            try {
                _importError.value = null
                _importSuccess.value = false
                val json = org.json.JSONObject(jsonString)
                val publicKey = json.optString("public_key")
                // UNIFIED ID FIX: public_key is the canonical contact key.
                // identity_id is secondary (human fingerprint). peer_id is the libp2p network ID.
                val peerId = json.optString("peer_id").takeIf { it.isNotBlank() }
                val identityId = json.optString("identity_id")
                if (publicKey.isBlank()) {
                    _importError.value = "Invalid identity format — missing public_key"
                    return@launch
                }
                val nickname = json.optString("nickname").takeIf { it.isNotBlank() }
                val libp2pPeerId = peerId
                    ?: json.optString("libp2p_peer_id").takeIf { it.isNotBlank() }
                val listenersArr = json.optJSONArray("listeners")
                val listeners = listenersArr?.let { arr ->
                    (0 until arr.length()).map { i -> arr.getString(i) }
                } ?: emptyList()
                val notes = libp2pPeerId?.let { pid ->
                    buildString {
                        append("libp2p_peer_id:$pid")
                        if (listeners.isNotEmpty()) append(";listeners:${listeners.joinToString(",")}")
                    }
                }
                // Store contact with public_key as the canonical peerId
                val contact = uniffi.api.Contact(
                    peerId = publicKey,
                    nickname = nickname,
                    localNickname = null,
                    publicKey = publicKey,
                    addedAt = (System.currentTimeMillis() / 1000).toULong(),
                    lastSeen = null,
                    notes = notes,
                    lastKnownDeviceId = null
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

    fun skipOnboardingForRelayOnlyInstall() {
        viewModelScope.launch {
            Timber.i("Skipping onboarding for relay-only install")
            preferencesRepository.setInstallChoiceCompleted(true)
            preferencesRepository.setOnboardingCompleted(true)
        }
    }

    fun handleDeepLink(uri: Uri) {
        val publicKey = uri.getQueryParameter("public_key")?.trim()
        if (publicKey.isNullOrBlank()) {
            Timber.w("Deep link missing public_key: $uri")
            return
        }
        val data = DeepLinkData(
            publicKey = publicKey,
            peerId = uri.getQueryParameter("peer_id")?.trim(),
            nickname = uri.getQueryParameter("nickname")?.trim(),
            identityId = uri.getQueryParameter("identity_id")?.trim()
        )
        Timber.i("Deep link parsed: peerId=${data.peerId}, nickname=${data.nickname}")
        _pendingDeepLink.value = data
    }

    fun consumeDeepLink(): DeepLinkData? {
        val data = _pendingDeepLink.value
        _pendingDeepLink.value = null
        return data
    }
}

data class DeepLinkData(
    val publicKey: String,
    val peerId: String?,
    val nickname: String?,
    val identityId: String?
)
