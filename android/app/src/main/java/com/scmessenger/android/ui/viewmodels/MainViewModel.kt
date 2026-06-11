package com.scmessenger.android.ui.viewmodels

import android.content.Context
import android.net.Uri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.data.PreferencesRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import timber.log.Timber
import com.scmessenger.android.utils.StorageManager
import java.security.SecureRandom
import java.util.concurrent.atomic.AtomicBoolean
import javax.inject.Inject

@HiltViewModel
class MainViewModel @Inject constructor(
    private val meshRepository: MeshRepository,
    private val preferencesRepository: PreferencesRepository,
    @ApplicationContext private val context: Context
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

    // P0_ANDROID_IDENTITY_PROOF_OF_WORK: high-level proof-of-work stages emitted
    // by createIdentity() — single source of truth shared with IdentityViewModel.
    //
    // v0.3.4 (P0_ANDROID_CRASHFIX): changed from MutableStateFlow<IdentityProgressStage?>(null)
    // to MutableStateFlow<IdentityProgressStage>(Idle). See the long comment in
    // IdentityViewModel for the full rationale. The previous nullable type caused
    // the NPE in IdentityScreen.ProofOfWorkList at `currentStage.id` line 234
    // because the screen rendered IdentityNotInitializedView before the user
    // tapped Create (when progressStage was still null), and the smart-cast at
    // the call site could not survive Compose's recomposition. Type-system
    // enforcement via Idle is the only durable fix.
    private val _identityProgressStage = MutableStateFlow<IdentityProgressStage>(IdentityProgressStage.Idle)
    val identityProgressStage: StateFlow<IdentityProgressStage> = _identityProgressStage.asStateFlow()

    private val _importError = MutableStateFlow<String?>(null)
    val importError = _importError.asStateFlow()

    private val _importSuccess = MutableStateFlow(false)
    val importSuccess = _importSuccess.asStateFlow()

    val identityInfo: uniffi.api.IdentityInfo?
        get() = meshRepository.getIdentityInfoNonBlocking()

    private val _isStorageLow = MutableStateFlow(false)
    val isStorageLow = _isStorageLow.asStateFlow()

    private val _availableStorageMB = MutableStateFlow(0L)
    val availableStorageMB = _availableStorageMB.asStateFlow()

    private val _pendingDeepLink = MutableStateFlow<DeepLinkData?>(null)
    val pendingDeepLink: StateFlow<DeepLinkData?> = _pendingDeepLink.asStateFlow()

    private val _pendingRequestsInbox = MutableStateFlow<String?>(null)
    val pendingRequestsInbox: StateFlow<String?> = _pendingRequestsInbox.asStateFlow()

    private val _themeMode = MutableStateFlow(PreferencesRepository.ThemeMode.SYSTEM)
    val themeMode: StateFlow<PreferencesRepository.ThemeMode> = _themeMode.asStateFlow()

    // Guards against concurrent refreshIdentityState() calls that spam FFI
    // and drop 160+ UI frames during startup.
    private val isRefreshing = AtomicBoolean(false)

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

        // Auto-refresh identity state when service transitions to RUNNING (lazy start).
        // Track prior state to avoid triggering refreshIdentityState on repeated
        // RUNNING emissions, which can cascade through _isReady → MeshApp recomposition.
        var lastServiceState: uniffi.api.ServiceState? = null
        viewModelScope.launch {
            meshRepository.serviceState.collect { state ->
                Timber.d("MeshRepository service state: $state")
                if (state == uniffi.api.ServiceState.RUNNING &&
                    lastServiceState != uniffi.api.ServiceState.RUNNING) {
                    refreshIdentityState()
                }
                lastServiceState = state
            }
        }

        // P0_SHARED_IDENTITY: observe the centralized identityInfo flow. This
        // ensures _isReady (which drives hasIdentity → showOnboarding → MeshApp
        // nav graph) flips true the moment the repo knows about the identity,
        // even if it happens via a different code path (e.g., IdentityViewModel's
        // own loadIdentity call publishes first). This eliminates the race where
        // a freshly created VM reads hasIdentity=false because its own
        // refreshIdentityState hasn't run yet.
        viewModelScope.launch {
            meshRepository.identityInfo.collect { info ->
                val initialized = info?.initialized == true
                if (initialized && !_isReady.value) {
                    Timber.d("P0_SHARED_IDENTITY: meshRepository.identityInfo reports initialized; updating _isReady")
                    _isReady.value = true
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
        // Drop duplicate FFI requests: concurrent refreshIdentityState() calls
        // spike isIdentityInitialized() 4-5×, dropping 160+ UI frames at startup.
        if (!isRefreshing.compareAndSet(false, true)) {
            Timber.d("refreshIdentityState() skipped — already refreshing")
            return
        }
        viewModelScope.launch(Dispatchers.IO) {
            try {
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
            } finally {
                isRefreshing.set(false)
            }
        }
    }

    /**
     * Create a new identity with retry logic for _isReady verification.
     * Issue #5: Added retry/verify loop to ensure _isReady reflects actual state
     * after identity creation (service may still be starting when first checked).
     */
    fun createIdentity(nickname: String, salt: ByteArray? = null) {
        // P0_ANDROID_024: Re-entrancy guard. Compose recomposition or a fast double-tap on
        // the "Generate Identity" button can fire createIdentity() multiple times before
        // _isCreatingIdentity flips to true (it is set inside the coroutine, not before
        // launch). Without this guard, two coroutines race through initializeIdentity,
        // setNickname, and the _isReady / preferences writes, which can leave the
        // onboarding flow in a broken intermediate state.
        //
        // P0_ANDROID_IDENTITY_PROGRESS: We now set _isCreatingIdentity.value = true
        // SYNCHRONOUSLY on the Main thread (compareAndSet is atomic and thread-safe)
        // BEFORE the IO coroutine launches. Previously the state flipped to true inside
        // the IO coroutine, so there was a 5-50ms window between the button tap and the
        // first recomposition where the user saw no progress indication at all. The
        // button looked frozen for a few seconds until the FFI call returned.
        if (!_isCreatingIdentity.compareAndSet(expect = false, update = true)) {
            Timber.d("createIdentity: ignored re-entrant call (already in progress)")
            return
        }
        // Mirror the state flip on the Main thread immediately so Compose recomposes
        // before any IO work begins. (compareAndSet above already updated the StateFlow
        // atomically; this comment exists to make the synchronous-flip intent explicit
        // for future maintainers.)
        _identityError.value = null
        // P0_ANDROID_IDENTITY_PROOF_OF_WORK: emit the first proof-of-work stage
        // SYNCHRONOUSLY so the UI recomposes with stage 1 visible before the IO
        // coroutine begins. This is the single source of truth for "we are working".
        _identityProgressStage.value = IdentityProgressStage.PreparingStorage
        viewModelScope.launch(Dispatchers.IO) {
            try {
                val trimmedNickname = nickname.trim()
                if (trimmedNickname.isEmpty()) {
                    Timber.w("Refusing identity creation with blank nickname")
                    _identityError.value = "Nickname is required"
                    _isReady.value = false
                    return@launch
                }
                // P0_ANDROID_IDENTITY_PROOF_OF_WORK: salt. If the entropy canvas
                // produced a salt, use that; otherwise draw 32 bytes of fresh entropy
                // from the platform CSPRNG. This guarantees every call (onboarding
                // AND in-settings) gets a random salt — fixing the regression where
                // the in-settings path was silently dropping the salt parameter.
                val effectiveSalt = salt ?: com.scmessenger.android.ui.viewmodels.IdentityViewModel.generateSecureRandomSalt()
                Timber.i("P0_IDENTITY: effective salt size=${effectiveSalt.size} bytes (user_entropy=${salt != null})")
                _identityProgressStage.value = IdentityProgressStage.GeneratingSalt

                _identityProgressStage.value = IdentityProgressStage.GeneratingKeypair
                meshRepository.createIdentity(effectiveSalt)
                meshRepository.setNickname(trimmedNickname)
                _identityProgressStage.value = IdentityProgressStage.ComputingFingerprint

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
                _identityProgressStage.value = IdentityProgressStage.PersistingToStorage
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

                _identityProgressStage.value = IdentityProgressStage.VerifyingIdentity
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
                // Hold the final stage visible for a moment so the user can see
                // the proof-of-work completed.
                // v0.3.4: set to Idle (was null) — see type-system comment on
                // _identityProgressStage above. Non-nullable contract.
                kotlinx.coroutines.delay(600)
                _identityProgressStage.value = IdentityProgressStage.Idle
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
            _isStorageLow.value = StorageManager.isStorageStateCritical(context)
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

    fun navigateToRequestsInbox(peerId: String? = null) {
        _pendingRequestsInbox.value = peerId
    }

    fun consumeRequestsInboxNav(): String? {
        val peerId = _pendingRequestsInbox.value
        _pendingRequestsInbox.value = null
        return peerId
    }
}

data class DeepLinkData(
    val publicKey: String,
    val peerId: String?,
    val nickname: String?,
    val identityId: String?
)
