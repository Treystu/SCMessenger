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

    // Collect QR code data from a coroutine to avoid blocking Main thread on FFI calls
    var qrCodeData by remember { mutableStateOf<String?>(null) }
    LaunchedEffect(identityInfo) {
        if (identityInfo?.initialized == true) {
            qrCodeData = viewModel.getQrCodeData()
        } else {
            qrCodeData = null
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
            ProofOfWorkList(
                currentStage = progressStage,
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
private fun ProofOfWorkList(
    // v0.3.4 (P0_ANDROID_CRASHFIX): parameter is now non-nullable. The previous
    // `IdentityProgressStage?` allowed a null to reach `currentStage.id` at
    // line 234, crashing the activity. With IdentityViewModel._progressStage
    // typed as non-nullable StateFlow<IdentityProgressStage> and the
    // IdentityNotInitializedView call site gated on `!is Idle`, the compiler
    // now enforces non-null at this call site.
    currentStage: com.scmessenger.android.ui.viewmodels.IdentityProgressStage,
    modifier: Modifier = Modifier
) {
    // v0.3.4: the previous `val stage = currentStage ?: return` defense-in-depth
    // is no longer needed — the type system guarantees currentStage is non-null.
    // Replaced with a direct alias for readability of the lines below.
    val stage = currentStage
    val allStages = com.scmessenger.android.ui.viewmodels.IdentityProgressStage.ALL

    // Sum of etaMs for stages strictly before the current one, divided by total.
    // The bar fills as stages complete, regardless of how long each actually
    // takes on this device.
    val completedEtaMs = allStages
        .filter { it.id < stage.id }
        .sumOf { it.etaMs }
    val rawFraction = completedEtaMs.toFloat() /
        com.scmessenger.android.ui.viewmodels.IdentityProgressStage.TOTAL_ETA_MS.toFloat()
    val fraction = rawFraction.coerceIn(0f, 1f)
    val percentComplete = (fraction * 100f).toInt().coerceIn(0, 99)

    // ETA: total minus the sum of completed etas. Floor at "a few seconds" so
    // the user never sees "About 0s remaining" while the spinner is still
    // running on the longest step.
    val remainingMs = (com.scmessenger.android.ui.viewmodels.IdentityProgressStage.TOTAL_ETA_MS - completedEtaMs)
        .coerceAtLeast(500L)
    val remainingSec = (remainingMs + 999L) / 1000L // round up
    val etaText = if (remainingSec <= 1L) "Less than a second remaining"
                  else "About $remainingSec seconds remaining"

    Card(
        modifier = modifier,
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant
        )
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            // Header row: step counter + percent-complete
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = "Step ${stage.id} of ${com.scmessenger.android.ui.viewmodels.IdentityProgressStage.TOTAL} — ${stage.label}",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold,
                    modifier = Modifier.weight(1f)
                )
                Text(
                    text = "$percentComplete%",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.primary
                )
            }
            // Smooth progress bar
            LinearProgressIndicator(
                progress = { fraction },
                modifier = Modifier
                    .fillMaxWidth()
                    .height(6.dp),
                color = MaterialTheme.colorScheme.primary,
                trackColor = MaterialTheme.colorScheme.surfaceVariant,
            )
            // Detail line: what the current step is doing
            Text(
                text = stage.detail,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            // ETA hint
            Text(
                text = etaText,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(modifier = Modifier.height(4.dp))
            // 6-row stage list
            allStages.forEach { s ->
                ProofOfWorkRow(
                    stage = s,
                    isDone = s.id < stage.id,
                    isActive = s.id == stage.id
                )
            }
        }
    }
}

@Composable
private fun ProofOfWorkRow(
    stage: com.scmessenger.android.ui.viewmodels.IdentityProgressStage,
    isDone: Boolean,
    isActive: Boolean
) {
    val rowColor = when {
        isDone -> MaterialTheme.colorScheme.primary
        isActive -> MaterialTheme.colorScheme.onSurface
        else -> MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f)
    }
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        when {
            isDone -> Text("✓", color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Bold)
            isActive -> CircularProgressIndicator(modifier = Modifier.size(14.dp), strokeWidth = 2.dp)
            else -> Text("·", color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f), fontWeight = FontWeight.Bold)
        }
        Text(
            text = "${stage.id}. ${stage.label}",
            style = MaterialTheme.typography.bodyMedium,
            color = rowColor,
            fontWeight = if (isActive) FontWeight.SemiBold else FontWeight.Normal
        )
    }
}

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
