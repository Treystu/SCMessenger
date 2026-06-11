package com.scmessenger.android.ui.identity

import android.graphics.Bitmap
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.google.zxing.BarcodeFormat
import com.google.zxing.qrcode.QRCodeWriter
import com.scmessenger.android.R
import com.scmessenger.android.ui.components.CopyableText
import com.scmessenger.android.ui.components.ErrorBanner
import com.scmessenger.android.ui.components.IdenticonFromPeerId
import com.scmessenger.android.ui.viewmodels.IdentityViewModel
import timber.log.Timber

/**
 * Identity screen - Display public key, QR code, and export options.
 *
 * Shows the user's identity information including peer ID, public key,
 * and a scannable QR code for easy contact sharing.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun IdentityScreen(
    onNavigateBack: () -> Unit,
    viewModel: IdentityViewModel = hiltViewModel()
) {
    val identityInfo by viewModel.identityInfo.collectAsState()
    val isLoading by viewModel.isLoading.collectAsState()
    val error by viewModel.error.collectAsState()
    val successMessage by viewModel.successMessage.collectAsState()
    // P0_ANDROID_IDENTITY_PROOF_OF_WORK: pull the high-level proof-of-work stage
    // out of the ViewModel so we can show 6 named stages instead of a tiny
    // centered spinner. The user can see exactly which step of the cryptographic
    // pipeline is currently running.
    val progressStage by viewModel.progressStage.collectAsState()
    // P0_ANDROID_PROGRESS_CALLBACK: transient sub-stage detail from inside the
    // FFI call. Renders as a smaller brighter line under the active stage's
    // `detail` in IdentityProgressDisplay so the user sees motion during the
    // SharedPreferences commit() and the libp2p swarm bind.
    val progressSubDetail by viewModel.progressSubDetail.collectAsState()

    // Collect QR code data from a coroutine to avoid blocking Main thread on FFI calls
    var qrCodeData by remember { mutableStateOf<String?>(null) }
    LaunchedEffect(identityInfo) {
        if (identityInfo?.initialized == true) {
            qrCodeData = viewModel.getQrCodeData()
        } else {
            qrCodeData = null
        }
    }

    // P0_ANDROID_QR_FIX: belt-and-suspenders poll. IdentityViewModel.loadIdentity
    // has its own retry loop (~1.55s) but the Rust core may hydrate LATER if the
    // service transitions to RUNNING *after* the VM was constructed (e.g., a slow
    // cold start where the user reaches Settings before MeshService.onCreate finished).
    // This LaunchedEffect is keyed on identityInfo itself, so it re-fires whenever
    // the state changes — but it also fires once on first composition. We schedule
    // two delayed polls (2s, 4s) to catch late hydration without busy-spinning.
    LaunchedEffect(identityInfo?.initialized) {
        if (identityInfo?.initialized != true) {
            kotlinx.coroutines.delay(2_000L)
            viewModel.loadIdentity(forceRefresh = true)
            kotlinx.coroutines.delay(2_000L)
            viewModel.loadIdentity(forceRefresh = true)
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.identity_title)) },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = stringResource(R.string.chat_action_dismiss))
                    }
                },
                actions = {
                    IconButton(onClick = { viewModel.loadIdentity() }) {
                        Icon(Icons.Default.Refresh, contentDescription = stringResource(R.string.diagnostics_action_refresh))
                    }
                }
            )
        }
    ) { paddingValues ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
        ) {
            when {
                // P0_ANDROID_IDENTITY_PROOF_OF_WORK: distinguish between "initial
                // load" (isLoading + Idle progress stage) and "active creation"
                // (progressStage != Idle). The old code conflated them and
                // replaced the entire form with a tiny centered spinner during
                // creation, hiding the button + form + everything from the user.
                // Now we only show the centered spinner during the very first
                // identityInfo load; once we know identity is not initialized,
                // we render the form WITH the proof-of-work stages inline.
                //
                // v0.3.4 (P0_ANDROID_CRASHFIX): `progressStage == null` became
                // `progressStage is IdentityProgressStage.Idle` because
                // _progressStage is now non-nullable in IdentityViewModel.
                isLoading && progressStage is com.scmessenger.android.ui.viewmodels.IdentityProgressStage.Idle && identityInfo == null -> {
                    CircularProgressIndicator(
                        modifier = Modifier.align(Alignment.Center)
                    )
                }

                identityInfo == null || identityInfo?.initialized != true -> {
                    // Identity not initialized
                    // P0_ANDROID_IDENTITY_PROOF_OF_WORK: pass the progress stage
                    // down to IdentityNotInitializedView so the user sees the 6
                    // named stages of cryptographic work, not just a spinner.
                    IdentityNotInitializedView(
                        isCreating = isLoading,
                        progressStage = progressStage,
                        // P0_ANDROID_PROGRESS_CALLBACK: pass the parent's
                        // collected sub-stage detail down to the leaf
                        // composable so it can be passed to
                        // IdentityProgressDisplay. Collecting here keeps
                        // the leaf stateless w.r.t. the ViewModel.
                        progressSubDetail = progressSubDetail,
                        onCreateIdentity = { nickname -> viewModel.createIdentity(nickname) },
                        modifier = Modifier.align(Alignment.Center)
                    )
                }

                else -> {
                    // Show identity — identityInfo is non-null and initialized here
                    val resolvedIdentity = identityInfo ?: return@Box
                    IdentityContent(
                        identityInfo = resolvedIdentity,
                        qrCodeData = qrCodeData,
                        error = error,
                        successMessage = successMessage,
                        onClearError = { viewModel.clearError() },
                        onClearSuccess = { viewModel.clearSuccessMessage() }
                    )
                }
            }
        }
    }
}

@Composable
private fun IdentityNotInitializedView(
    isCreating: Boolean,
    // v0.3.4 (P0_ANDROID_CRASHFIX): parameter is now non-nullable. The previous
    // `IdentityProgressStage?` was the root cause of the NPE crash at
    // ProofOfWorkList.currentStage.id — the call site relied on a smart-cast
    // through a `if (progressStage != null)` guard that did not survive
    // Compose's recomposition, so a null value reached the call site. With
    // IdentityViewModel._progressStage typed as non-nullable StateFlow<IdentityProgressStage>
    // and initialized to IdentityProgressStage.Idle, the type system guarantees
    // progressStage is always a valid stage here. The Idle check below is the
    // only remaining null-equivalent branch we need.
    progressStage: com.scmessenger.android.ui.viewmodels.IdentityProgressStage,
    // P0_ANDROID_PROGRESS_CALLBACK: transient sub-stage detail from inside the
    // FFI call. Passed down from the parent composable (which collects from
    // IdentityViewModel.progressSubDetail) so we can render the secondary
    // line under the active stage's `detail`.
    progressSubDetail: String?,
    onCreateIdentity: (nickname: String) -> Unit,
    modifier: Modifier = Modifier
) {
    var nickname by remember { mutableStateOf("") }

    Column(
        modifier = modifier.padding(32.dp).verticalScroll(rememberScrollState()),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text(
            text = stringResource(R.string.identity_not_initialized_title),
            style = MaterialTheme.typography.titleLarge
        )

        Text(
            text = stringResource(R.string.identity_not_initialized_description),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )

        OutlinedTextField(
            value = nickname,
            onValueChange = { nickname = it },
            label = { Text(stringResource(R.string.identity_label_nickname)) },
            modifier = Modifier.fillMaxWidth(0.8f),
            singleLine = true,
            enabled = !isCreating
        )

        // P0_ANDROID_IDENTITY_PROOF_OF_WORK: Inline progress indicator. The
        // button now mirrors IdentityCreationFlow's pattern: while the FFI call
        // runs (several seconds for Ed25519 keygen + entropy + storage write),
        // the button shows a CircularProgressIndicator + "Generating Identity
        // keys…" text, and the button itself is disabled. Previously this button
        // had no progress indication at all, which made the in-settings
        // identity creation flow feel broken.
        Button(
            onClick = { onCreateIdentity(nickname) },
            enabled = nickname.isNotBlank() && !isCreating,
            modifier = Modifier.fillMaxWidth(0.8f).height(56.dp)
        ) {
            if (isCreating) {
                CircularProgressIndicator(
                    modifier = Modifier.size(20.dp),
                    strokeWidth = 2.dp,
                    color = MaterialTheme.colorScheme.onPrimary
                )
                Spacer(modifier = Modifier.size(8.dp))
                Text(stringResource(R.string.onboarding_generating_keys))
            } else {
                Text(stringResource(R.string.identity_action_create))
            }
        }

        // P0_ANDROID_IDENTITY_PROOF_OF_WORK: render the 6 named proof-of-work
        // stages below the button. Each stage shows:
        //   - ✓ checkmark (done)
        //   - spinner + label (active)
        //   - dimmed label (pending)
        //   - detail text under the active stage explaining what is happening
        //
        // This is the "high level proof of work" the user asked for. The user
        // can see exactly which step of the cryptographic pipeline is currently
        // running, and the 6 stages match MainViewModel/IdentityViewModel one
        // for one, so onboarding + in-settings have a unified narrative.
        // P0_ANDROID_CRASHFIX: full fix landed in v0.3.4. The previous
        // `progressStage?.let { stage -> ... }` was a defense-in-depth
        // workaround for the NPE; the real fix is to type the StateFlow as
        // non-nullable with Idle as the sentinel. Now the gate is a clean
        // `if (progressStage !is Idle)` — the type system enforces non-null,
        // and the !is Idle check enforces "we are creating right now".
        if (progressStage !is com.scmessenger.android.ui.viewmodels.IdentityProgressStage.Idle) {
            Spacer(modifier = Modifier.height(8.dp))
            IdentityProgressDisplay(
                currentStage = progressStage,
                // P0_ANDROID_PROGRESS_CALLBACK: in-settings create flow uses
                // the same callback plumbing as the onboarding flow, so the
                // sub-stage detail reaches the display here too.
                subStageDetail = progressSubDetail,
                modifier = Modifier.fillMaxWidth(0.95f)
            )
        }
    }
}

/**
 * Renders the 6-stage identity-generation proof-of-work list. Each stage is
 * marked as "done" (✓), "active" (spinner), or "pending" (dimmed), with the
 * current active stage's detail text displayed below the list so the user can
 * see exactly what cryptographic work is happening.
 *
 * Includes:
 *   - a stage counter "Step X of 6" so the user sees numeric progress
 *   - a LinearProgressIndicator driven by the per-stage etaMs so the bar moves
 *     smoothly even when the Rust FFI call blocks (the longest single step)
 *   - a "About Ns remaining" hint that updates as the active stage changes
 *   - a percent-complete number that climbs as stages complete
 */
@Composable
private fun IdentityContent(
    identityInfo: uniffi.api.IdentityInfo,
    qrCodeData: String?,
    error: String?,
    successMessage: String?,
    onClearError: () -> Unit,
    @Suppress("UNUSED_PARAMETER") onClearSuccess: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(24.dp)
    ) {
        // Error banner
        error?.let {
            ErrorBanner(
                message = it,
                onDismiss = onClearError
            )
        }

        // Success message
        successMessage?.let {
            Card(
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.primaryContainer
                )
            ) {
                Text(
                    text = it,
                    modifier = Modifier.padding(16.dp),
                    color = MaterialTheme.colorScheme.onPrimaryContainer
                )
            }
        }

        // Identicon
        IdenticonFromPeerId(
            peerId = identityInfo.libp2pPeerId ?: identityInfo.identityId ?: stringResource(R.string.identity_field_unavailable),
            size = 96.dp,
            modifier = Modifier.align(Alignment.CenterHorizontally)
        )

        // QR Code
        qrCodeData?.let { data ->
            QRCodeDisplay(
                data = data,
                modifier = Modifier.align(Alignment.CenterHorizontally)
            )
        }

        // Identity Hash (human fingerprint)
        Card {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    text = stringResource(R.string.identity_label_hash),
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )

                Spacer(modifier = Modifier.height(8.dp))

                CopyableText(
                    text = identityInfo.identityId ?: stringResource(R.string.identity_field_unavailable),
                    monospace = true
                )
            }
        }

        // Peer ID (Network) — libp2p Peer ID for contact add / routing
        Card {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    text = stringResource(R.string.identity_label_peer_id),
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )

                Spacer(modifier = Modifier.height(8.dp))

                CopyableText(
                    text = identityInfo.libp2pPeerId ?: stringResource(R.string.identity_field_unavailable),
                    monospace = true
                )
            }
        }

        // Public Key (canonical identity)
        Card {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    text = stringResource(R.string.identity_label_public_key),
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )

                Spacer(modifier = Modifier.height(8.dp))

                CopyableText(
                    text = identityInfo.publicKeyHex ?: stringResource(R.string.identity_field_unavailable),
                    monospace = true
                )
            }
        }
    }
}

/**
 * QR Code display component.
 */
@Composable
private fun QRCodeDisplay(
    data: String,
    modifier: Modifier = Modifier
) {
    val bitmap = remember(data) {
        try {
            generateQRCode(data, 512)
        } catch (e: Exception) {
            Timber.e(e, "Failed to generate QR code")
            null
        }
    }

    bitmap?.let {
        Card(modifier = modifier) {
            Image(
                bitmap = it.asImageBitmap(),
                contentDescription = stringResource(R.string.identity_label_qr_code),
                modifier = Modifier
                    .size(256.dp)
                    .padding(16.dp)
            )
        }
    }
}

/**
 * Generate QR code bitmap from string data.
 */
private fun generateQRCode(data: String, size: Int): Bitmap {
    val writer = QRCodeWriter()
    val bitMatrix = writer.encode(data, BarcodeFormat.QR_CODE, size, size)

    val width = bitMatrix.width
    val height = bitMatrix.height
    val bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.RGB_565)

    for (x in 0 until width) {
        for (y in 0 until height) {
            bitmap.setPixel(
                x,
                y,
                if (bitMatrix[x, y]) android.graphics.Color.BLACK else android.graphics.Color.WHITE
            )
        }
    }

    return bitmap
}
