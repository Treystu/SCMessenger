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
                isLoading -> {
                    CircularProgressIndicator(
                        modifier = Modifier.align(Alignment.Center)
                    )
                }

                identityInfo == null || identityInfo?.initialized != true -> {
                    // Identity not initialized
                    IdentityNotInitializedView(
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
    onCreateIdentity: (nickname: String) -> Unit,
    modifier: Modifier = Modifier
) {
    var nickname by remember { mutableStateOf("") }

    Column(
        modifier = modifier.padding(32.dp),
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
            singleLine = true
        )

        Button(onClick = { onCreateIdentity(nickname) }) {
            Text(stringResource(R.string.identity_action_create))
        }
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
