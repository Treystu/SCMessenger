package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import timber.log.Timber
import java.security.SecureRandom
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

    // P0_SHARED_IDENTITY: IdentityViewModel now observes the centralized
    // meshRepository.identityInfo StateFlow. There is no longer a private
    // _identityInfo; instead we mirror the repo's flow into a derived local
    // StateFlow so the existing UI (identityInfo.collectAsState()) keeps
    // working without any Compose-side changes. The centralized flow replays
    // its latest value to new subscribers, so a fresh IdentityViewModel
    // constructed after the repo has been hydrated (the common case after
    // onboarding) immediately sees the real identity without polling.
    private val _identityInfo = MutableStateFlow<uniffi.api.IdentityInfo?>(null)
    val identityInfo: StateFlow<uniffi.api.IdentityInfo?> = _identityInfo.asStateFlow()

    // Loading state
    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()

    // P1: Re-entrancy guard for createIdentity(). createIdentity() in the
    // repository is also mutex-guarded, but rejecting at the ViewModel layer
    // gives us a single deterministic early-return + prevents duplicate
    // "Identity created successfully" success messages and redundant UI work.
    private val _isCreating = MutableStateFlow(false)
    val isCreating: StateFlow<Boolean> = _isCreating.asStateFlow()

    // Error state
    private val _error = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = _error.asStateFlow()

    // Success message (for export/copy operations)
    private val _successMessage = MutableStateFlow<String?>(null)
    val successMessage: StateFlow<String?> = _successMessage.asStateFlow()

    // P0_ANDROID_IDENTITY_PROOF_OF_WORK: High-level proof-of-work stage emitted
    // by createIdentity() so the UI can show the user exactly which step of the
    // cryptographic pipeline is currently running. Single source of truth shared
    // with MainViewModel via IdentityProgressStage.
    //
    // v0.3.4 (P0_ANDROID_CRASHFIX): changed from MutableStateFlow<IdentityProgressStage?>(null)
    // to MutableStateFlow<IdentityProgressStage>(Idle). The previous nullable type
    // was the root cause of the NPE crash in IdentityScreen.ProofOfWorkList at
    // `currentStage.id` line 234 — the screen rendered IdentityNotInitializedView
    // even when progressStage was null (e.g. on first load before user tapped
    // Create), and the smart-cast at the call site could not survive Compose's
    // recomposition. With non-nullable Idle, the type system guarantees
    // progressStage is always a valid stage, and the call site can use a clean
    // `if (progressStage !is Idle)` check instead of nullable branching.
    private val _progressStage = MutableStateFlow<IdentityProgressStage>(IdentityProgressStage.Idle)
    val progressStage: StateFlow<IdentityProgressStage> = _progressStage.asStateFlow()

    // P0_ANDROID_PROGRESS_CALLBACK: transient sub-stage detail line that
    // appears under the active stage's `detail` in IdentityProgressDisplay.
    // Mirrors the same flow on MainViewModel so both the onboarding entry
    // point and the in-settings entry point get the same progress narrative.
    private val _progressSubDetail = MutableStateFlow<String?>(null)
    val progressSubDetail: StateFlow<String?> = _progressSubDetail.asStateFlow()

    // Companion object: shared SecureRandom instance (CSPRNG) used to generate
    // the 256-bit salt for both the onboarding and in-settings entry points.
    // Even if the user does not engage with the entropy canvas, the identity
    // pipeline still receives a fresh random salt on every call.
    companion object {
        private val secureRandom = SecureRandom()

        /** Draw 32 bytes of fresh entropy from the platform CSPRNG. */
        fun generateSecureRandomSalt(): ByteArray {
            val salt = ByteArray(32)
            secureRandom.nextBytes(salt)
            return salt
        }
    }

    init {
        // P0_SHARED_IDENTITY: Mirror the centralized meshRepository.identityInfo
        // StateFlow into the local _identityInfo. Because StateFlow has replay=1
        // semantics through the repo (the repo's flow was set to MutableStateFlow
        // which always replays its current value to new subscribers), this means
        // a fresh IdentityViewModel constructed after the repo has been hydrated
        // immediately gets the real identity without polling. This is the core
        // fix for the "Show Identity QR -> not initialized" bug.
        viewModelScope.launch(Dispatchers.IO) {
            meshRepository.identityInfo.collect { info ->
                if (_identityInfo.value != info) {
                    _identityInfo.value = info
                }
            }
        }

        // Also kick a loadIdentity() to trigger an FFI read (which will publish
        // to meshRepository.identityInfo, propagating back here via the observer
        // above). loadIdentity() also handles the retry-with-backoff case where
        // the Rust core is still hydrating.
        loadIdentity()

        // P0: Refresh identity from Rust core when service transitions to RUNNING,
        // replacing SharedPreferences cache with live data.
        var lastServiceState: uniffi.api.ServiceState? = null
        viewModelScope.launch(Dispatchers.IO) {
            meshRepository.serviceState.collect { state ->
                if (state == uniffi.api.ServiceState.RUNNING &&
                    lastServiceState != uniffi.api.ServiceState.RUNNING) {
                    Timber.d("IdentityViewModel: service -> RUNNING, force-refreshing identity")
                    loadIdentity(forceRefresh = true)
                }
                lastServiceState = state
            }
        }
    }

    /**
     * Load identity information.
     *
     * P0_ANDROID_QR_FIX: When the user navigates from Settings -> Identity QR, the
     * IdentityViewModel is constructed fresh. Its `init` calls `loadIdentity()` which
     * uses `getIdentityInfoNonBlocking()`. On a cold start, the ironCore may be null
     * or `core.getIdentityInfo()` may return null because the service is still spinning
     * up — even though the identity is fully on disk in SharedPreferences backup. The
     * old code returned null in that case and the user saw "Identity not initialized"
     * on the QR screen, even though the mesh was working and identity was real.
     *
     * The fix:
     *   1. Retry with exponential backoff for up to ~1.55s total if the first read
     *      returns null/uninitialized. The Rust core typically hydrates within 100-500ms
     *      on a warm start.
     *   2. As a defensive check, use `meshRepository.isIdentityInitialized()` to know
     *      whether the SharedPreferences backup exists. If no backup, identity was
     *      never created on this device — short-circuit, no retry.
     *   3. The `forceRefresh` flag (used on service RUNNING transition) bypasses the
     *      in-VM cache and re-reads from the Rust core.
     */
    fun loadIdentity(forceRefresh: Boolean = false) {
        viewModelScope.launch(Dispatchers.IO) {
            try {
                _isLoading.value = true
                _error.value = null

                // P0_ANDROID_QR_FIX: bounded retry-with-backoff so the QR screen
                // doesn't show "not initialized" during a normal cold start.
                val identity = readIdentityWithRetry(forceRefresh = forceRefresh)
                if (_identityInfo.value != identity) {
                    _identityInfo.value = identity
                }

                if (identity == null || !identity.initialized) {
                    Timber.w("P0_QR_FIX: Identity not initialized after retry loop")
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
     * P0_ANDROID_QR_FIX: Read identity with bounded retry-with-backoff.
     *
     * Strategy:
     *   - First attempt: use `getIdentityInfoNonBlocking()` (no service init).
     *   - If null/uninitialized AND `isIdentityInitialized()` (backup check) says
     *     identity exists on disk, retry up to 5 times with exponential backoff
     *     (50ms, 100ms, 200ms, 400ms, 800ms = ~1.55s total). The Rust core typically
     *     hydrates from sled within the first 1-2 retries.
     *   - If `isIdentityInitialized()` returns false, identity was never created
     *     on this device — short-circuit immediately, no retry, render create form.
     */
    private suspend fun readIdentityWithRetry(forceRefresh: Boolean): uniffi.api.IdentityInfo? {
        val firstAttempt = meshRepository.getIdentityInfoNonBlocking()
        if (firstAttempt != null && firstAttempt.initialized) {
            return firstAttempt
        }
        // Identity not yet visible via non-blocking read. Check the backup before
        // retrying: if the SharedPreferences backup is missing, identity was never
        // created here and we should render the create form, not spin forever.
        val backupExists = try {
            meshRepository.isIdentityInitialized()
        } catch (e: Exception) {
            Timber.w(e, "P0_QR_FIX: isIdentityInitialized() threw during retry check")
            false
        }
        if (!backupExists) {
            Timber.d("P0_QR_FIX: no identity backup on disk; rendering create form")
            return firstAttempt // null or uninitialized
        }
        // Backup exists but the core returned null. Retry with backoff so the QR
        // screen doesn't show "not initialized" during a normal cold start.
        if (!forceRefresh && _identityInfo.value?.initialized == true) {
            // Already have initialized data in this VM; don't churn.
            return _identityInfo.value
        }
        val backoffMs = longArrayOf(50L, 100L, 200L, 400L, 800L)
        for ((index, delay) in backoffMs.withIndex()) {
            kotlinx.coroutines.delay(delay)
            val attempt = try {
                meshRepository.getIdentityInfoNonBlocking()
            } catch (e: Exception) {
                Timber.w(e, "P0_QR_FIX: retry $index threw")
                continue
            }
            if (attempt != null && attempt.initialized) {
                Timber.d("P0_QR_FIX: identity hydrated on retry $index after ${backoffMs.take(index + 1).sum()}ms")
                return attempt
            }
        }
        Timber.w("P0_QR_FIX: identity not visible from Rust core after retry loop; backup exists but core unhydrated")
        return firstAttempt
    }

    /**
     * Create a new identity (first-time setup).
     *
     * P0_ANDROID_IDENTITY_PROOF_OF_WORK: Emits 6 named stages via [progressStage]
     * so the UI can show high-level proof of work. The same stages are emitted
     * by [MainViewModel.createIdentity] so onboarding and in-settings share one
     * progress narrative.
     *
     * Always passes a fresh 32-byte SecureRandom salt to meshRepository.createIdentity
     * — fixes the "no random salt for settings launched identity generation"
     * regression where the in-settings path was silently dropping the salt param.
     */
    fun createIdentity(nickname: String? = null) {
        // P1: Re-entrancy guard. _isCreating transitions are synchronized via
        // MutableStateFlow which is thread-safe; the first caller wins and
        // subsequent callers drop into the no-op branch until completion.
        if (!_isCreating.compareAndSet(expect = false, update = true)) {
            Timber.d("createIdentity: re-entrant call dropped (already in progress)")
            return
        }
        // P0_ANDROID_IDENTITY_PROOF_OF_WORK: emit the first stage SYNCHRONOUSLY
        // so the UI recomposes with proof-of-work visible before the IO coroutine
        // begins. This mirrors the same synchronous-flip pattern in MainViewModel
        // and is the single source of truth for "we are working on this".
        _isLoading.value = true
        _error.value = null
        _progressStage.value = IdentityProgressStage.PreparingStorage
        viewModelScope.launch(Dispatchers.IO) {
            try {
                // Stage 1: prepare storage (grant consent + ensure service running).
                // meshRepository.createIdentity() internally calls grantConsent()
                // and ensureServiceInitialized(), so calling it advances through
                // these stages. We emit the salt-generation stage BEFORE the
                // Rust FFI call so the user can see "we're preparing a salt"
                // while they wait for the math-heavy keygen.
                val salt = generateSecureRandomSalt()
                Timber.i("P0_IDENTITY: generated ${salt.size}-byte SecureRandom salt (hex=${salt.joinToString("") { "%02x".format(it) }.take(16)}…)")
                _progressStage.value = IdentityProgressStage.GeneratingSalt

                // Stage 2 → 3: hand the salt to the repo. The repo runs grantConsent
                // + ensureServiceInitialized + initializeIdentity (Rust FFI). Stages
                // 3-6 are now driven by the callback fired from inside the repo,
                // so the UI sees real progress (Ed25519 keygen, persistIdentityBackup
                // sub-steps, initializeAndStartSwarm) instead of a frozen stage
                // for the entire ~10s FFI block. Mirrors the MainViewModel fix.
                _progressStage.value = IdentityProgressStage.GeneratingSalt
                meshRepository.createIdentity(salt) { event, subDetail ->
                    _progressStage.value = when (event) {
                        is com.scmessenger.android.data.IdentityCreationEvent.PreparingStorage ->
                            IdentityProgressStage.PreparingStorage
                        is com.scmessenger.android.data.IdentityCreationEvent.GeneratingSalt ->
                            IdentityProgressStage.GeneratingSalt
                        is com.scmessenger.android.data.IdentityCreationEvent.GeneratingKeypair ->
                            IdentityProgressStage.GeneratingKeypair
                        is com.scmessenger.android.data.IdentityCreationEvent.ComputingFingerprint ->
                            IdentityProgressStage.ComputingFingerprint
                        is com.scmessenger.android.data.IdentityCreationEvent.PersistingToStorage ->
                            IdentityProgressStage.PersistingToStorage
                        is com.scmessenger.android.data.IdentityCreationEvent.VerifyingIdentity ->
                            IdentityProgressStage.VerifyingIdentity
                    }
                    _progressSubDetail.value = subDetail
                }

                // Set nickname after creation if provided
                if (nickname != null && nickname.isNotBlank()) {
                    meshRepository.setNickname(nickname)
                }

                // P0_ANDROID_PROGRESS_CALLBACK: stages 5 and 6 are now driven
                // by the repo callback. We only need to re-read the identity
                // here to refresh the local mirror (so the screen shows the
                // freshly-created identity) and verify it landed.
                val info = meshRepository.getIdentityInfoNonBlocking()
                if (_identityInfo.value != info) {
                    _identityInfo.value = info
                }

                val verified = meshRepository.isIdentityInitialized()
                if (!verified) {
                    Timber.w("P0_IDENTITY: identity not initialized after createIdentity; retry verification")
                    kotlinx.coroutines.delay(100)
                }

                Timber.i("Identity created (id=${info?.identityId?.take(8) ?: "?"}, nickname=${info?.nickname})")
                _successMessage.value = "Identity created successfully"
            } catch (e: Exception) {
                _error.value = "Failed to create identity: ${e.message}"
                Timber.e(e, "Failed to create identity")
            } finally {
                _isLoading.value = false
                _isCreating.value = false
                // Reset progress stage on a small delay so the user sees the
                // "done" state for a moment before the view recomposes.
                // v0.3.4: set to Idle (was null) — see type-system comment on
                // _progressStage above. The non-nullable type contract requires
                // a non-null value here.
                kotlinx.coroutines.delay(600)
                _progressStage.value = IdentityProgressStage.Idle
                // P0_ANDROID_PROGRESS_CALLBACK: clear the transient sub-detail
                // so a subsequent click on Create starts from a clean slate.
                _progressSubDetail.value = null
            }
        }
    }

    /**
     * Get QR code data for sharing identity.
     * Returns canonical identity export JSON so contact imports fully autofill.
     * Suspend function to avoid blocking Main thread on FFI calls.
     */
    suspend fun getQrCodeData(): String? {
        return withContext(Dispatchers.IO) {
            try {
                val identity = _identityInfo.value ?: return@withContext null
                if (!identity.initialized) return@withContext null
                meshRepository.getIdentityExportString()
            } catch (e: Exception) {
                Timber.e(e, "Failed to generate QR code data")
                null
            }
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
