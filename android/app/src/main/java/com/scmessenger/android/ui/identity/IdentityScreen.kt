package com.scmessenger.android.ui.identity

import android.graphics.Bitmap
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.google.zxing.BarcodeFormat
import com.google.zxing.qrcode.QRCodeWriter
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
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("My Identity") },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "Back")
                    }
                },
                actions = {
                    IconButton(onClick = { viewModel.loadIdentity() }) {
                        Icon(Icons.Default.Refresh, contentDescription = "Refresh")
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
                
                identityInfo == null || !identityInfo!!.initialized -> {
                    // Identity not initialized
                    IdentityNotInitializedView(
                        onCreateIdentity = { viewModel.createIdentity() },
                        modifier = Modifier.align(Alignment.Center)
                    )
                }
                
                else -> {
                    // Show identity
                    IdentityContent(
                        identityInfo = identityInfo!!,
                        qrCodeData = viewModel.getQrCodeData(),
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
    onCreateIdentity: () -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier.padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text(
            text = "Identity Not Initialized",
            style = MaterialTheme.typography.titleLarge
        )
        
        Text(
            text = "Create your identity to start using SCMessenger",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        
        Button(onClick = onCreateIdentity) {
            Text("Create Identity")
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
    onClearSuccess: () -> Unit
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
            peerId = identityInfo.identityId ?: "Unknown",
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
        
        // Peer ID
        Card {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    text = "Peer ID",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )
                
                Spacer(modifier = Modifier.height(8.dp))
                
                CopyableText(
                    text = identityInfo.identityId ?: "Unknown",
                    monospace = true
                )
            }
        }
        
        // Public Key
        Card {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    text = "Public Key",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )
                
                Spacer(modifier = Modifier.height(8.dp))
                
                CopyableText(
                    text = identityInfo.publicKeyHex ?: "Unknown",
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
                contentDescription = "QR Code",
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
