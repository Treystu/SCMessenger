package com.scmessenger.android.ui.contacts

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.google.android.gms.common.api.CommonStatusCodes
import com.google.mlkit.common.MlKitException
import com.google.mlkit.vision.codescanner.GmsBarcodeScannerOptions
import com.google.mlkit.vision.codescanner.GmsBarcodeScanning
import com.google.mlkit.vision.barcode.common.Barcode
import com.scmessenger.android.ui.components.ErrorBanner
import com.scmessenger.android.ui.components.IdenticonFromPeerId
import com.scmessenger.android.ui.viewmodels.ContactsViewModel
import com.scmessenger.android.utils.ContactImportParseResult
import com.scmessenger.android.utils.parseContactImportPayload
import timber.log.Timber

/**
 * Add Contact screen - QR scan, manual entry, nearby discovery.
 *
 * Provides multiple methods to add contacts:
 * - Manual entry of peer ID and public key
 * - QR code scanning
 * - Nearby peer discovery (future)
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AddContactScreen(
    onNavigateBack: () -> Unit,
    onContactAdded: () -> Unit = {},
    viewModel: ContactsViewModel = hiltViewModel()
) {
    val error by viewModel.error.collectAsState()

    var selectedTab by remember { mutableStateOf(0) }
    var peerId by remember { mutableStateOf("") }
    var publicKey by remember { mutableStateOf("") }
    var nickname by remember { mutableStateOf("") }
    var notes by remember { mutableStateOf("") }
    var libp2pPeerId by remember { mutableStateOf<String?>(null) }
    var listeners by remember { mutableStateOf<List<String>>(emptyList()) }
    var isAdding by remember { mutableStateOf(false) }
    var qrError by remember { mutableStateOf<String?>(null) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Add Contact") },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "Back")
                    }
                }
            )
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
        ) {
            // Tab selector
            TabRow(selectedTabIndex = selectedTab) {
                Tab(
                    selected = selectedTab == 0,
                    onClick = { selectedTab = 0 },
                    text = { Text("Manual Entry") }
                )
                Tab(
                    selected = selectedTab == 1,
                    onClick = { selectedTab = 1 },
                    text = { Text("QR Scan") }
                )
                Tab(
                    selected = selectedTab == 2,
                    onClick = { selectedTab = 2 },
                    text = { Text("Nearby") }
                )
            }

            // Error banner
            error?.let {
                ErrorBanner(
                    message = it,
                    onDismiss = { viewModel.clearError() }
                )
            }
            qrError?.let {
                ErrorBanner(
                    message = it,
                    onDismiss = { qrError = null }
                )
            }

            // Content based on selected tab
            when (selectedTab) {
                0 -> ManualEntryTab(
                    peerId = peerId,
                    onPeerIdChange = { peerId = it },
                    publicKey = publicKey,
                    onPublicKeyChange = { publicKey = it },
                    nickname = nickname,
                    onNicknameChange = { nickname = it },
                    notes = notes,
                    onNotesChange = { notes = it },
                    isAdding = isAdding,
                    onAdd = {
                        if (peerId.isNotBlank() && publicKey.isNotBlank()) {
                            isAdding = true
                            viewModel.addContact(
                                peerId = peerId,
                                publicKey = publicKey,
                                nickname = nickname.takeIf { it.isNotBlank() },
                                libp2pPeerId = libp2pPeerId,
                                listeners = listeners,
                                notes = notes.takeIf { it.isNotBlank() }
                            )
                            // Reset form
                            peerId = ""
                            publicKey = ""
                            nickname = ""
                            notes = ""
                            isAdding = false
                            onContactAdded()
                        }
                    }
                )
                1 -> QRScanTab(
                    onScanned = { scanned ->
                        qrError = null
                        when (val parsed = parseContactImportPayload(scanned)) {
                            is ContactImportParseResult.Invalid -> {
                                qrError = parsed.reason
                                Timber.w("Invalid contact QR data: ${parsed.reason}")
                            }
                            is ContactImportParseResult.Valid -> {
                                peerId = parsed.payload.peerId
                                publicKey = parsed.payload.publicKey
                                nickname = parsed.payload.nickname ?: nickname
                                libp2pPeerId = parsed.payload.libp2pPeerId
                                listeners = parsed.payload.listeners
                                selectedTab = 0
                            }
                        }
                    },
                    onScanError = { message ->
                        qrError = message
                    }
                )
                2 -> NearbyDiscoveryTab()
            }
        }
    }
}

@Composable
private fun ManualEntryTab(
    peerId: String,
    onPeerIdChange: (String) -> Unit,
    publicKey: String,
    onPublicKeyChange: (String) -> Unit,
    nickname: String,
    onNicknameChange: (String) -> Unit,
    notes: String,
    onNotesChange: (String) -> Unit,
    isAdding: Boolean,
    onAdd: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        // Preview
        if (peerId.isNotBlank()) {
            Card {
                Row(
                    modifier = Modifier.padding(16.dp),
                    horizontalArrangement = Arrangement.spacedBy(16.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    IdenticonFromPeerId(peerId = peerId, size = 64.dp)

                    Column {
                        Text(
                            text = nickname.ifBlank { "Unknown" },
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold
                        )
                        Text(
                            text = peerId.take(16) + "...",
                            style = MaterialTheme.typography.bodySmall,
                            fontFamily = FontFamily.Monospace
                        )
                    }
                }
            }
        }

        // Peer ID input
        OutlinedTextField(
            value = peerId,
            onValueChange = onPeerIdChange,
            label = { Text("Peer ID *") },
            modifier = Modifier.fillMaxWidth(),
            singleLine = true,
            enabled = !isAdding
        )

        // Public Key input
        OutlinedTextField(
            value = publicKey,
            onValueChange = onPublicKeyChange,
            label = { Text("Public Key *") },
            modifier = Modifier.fillMaxWidth(),
            minLines = 2,
            maxLines = 4,
            enabled = !isAdding
        )

        // Nickname input
        OutlinedTextField(
            value = nickname,
            onValueChange = onNicknameChange,
            label = { Text("Nickname (optional)") },
            modifier = Modifier.fillMaxWidth(),
            singleLine = true,
            enabled = !isAdding
        )

        // Notes input
        OutlinedTextField(
            value = notes,
            onValueChange = onNotesChange,
            label = { Text("Notes (optional)") },
            modifier = Modifier.fillMaxWidth(),
            minLines = 3,
            maxLines = 5,
            enabled = !isAdding
        )

        // Add button
        Button(
            onClick = onAdd,
            modifier = Modifier.fillMaxWidth(),
            enabled = !isAdding && peerId.isNotBlank() && publicKey.isNotBlank()
        ) {
            if (isAdding) {
                CircularProgressIndicator(
                    modifier = Modifier.size(20.dp),
                    color = MaterialTheme.colorScheme.onPrimary
                )
            } else {
                Icon(Icons.Default.Add, contentDescription = null)
                Spacer(modifier = Modifier.width(8.dp))
                Text("Add Contact")
            }
        }
    }
}

@Composable
private fun QRScanTab(
    onScanned: (String) -> Unit,
    onScanError: (String) -> Unit
) {
    val context = LocalContext.current
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Icon(
            imageVector = Icons.Default.Info,
            contentDescription = null,
            modifier = Modifier.size(64.dp),
            tint = MaterialTheme.colorScheme.primary
        )

        Spacer(modifier = Modifier.height(16.dp))

        Text(
            text = "QR Code Scanning",
            style = MaterialTheme.typography.titleLarge
        )

        Spacer(modifier = Modifier.height(8.dp))

        Text(
            text = "Scan a contact export QR to auto-fill peer ID and public key.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )

        Spacer(modifier = Modifier.height(24.dp))

        Button(onClick = {
            val options = GmsBarcodeScannerOptions.Builder()
                .setBarcodeFormats(Barcode.FORMAT_QR_CODE)
                .build()
            val scanner = GmsBarcodeScanning.getClient(context, options)
            scanner.startScan()
                .addOnSuccessListener { barcode ->
                    val rawValue = barcode.rawValue
                    if (rawValue.isNullOrBlank()) {
                        onScanError("QR code was empty. Please try again.")
                    } else {
                        onScanned(rawValue)
                    }
                }
                .addOnFailureListener { e ->
                    Timber.w(e, "QR scan failed")
                    if (e is MlKitException && e.errorCode == CommonStatusCodes.CANCELED) {
                        return@addOnFailureListener
                    }
                    onScanError("Unable to scan QR code. Please try again.")
                }
        }) {
            Icon(Icons.Default.CameraAlt, contentDescription = null)
            Spacer(modifier = Modifier.width(8.dp))
            Text("Scan QR Code")
        }
    }
}

@Composable
private fun NearbyDiscoveryTab() {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Icon(
            imageVector = Icons.Default.Search,
            contentDescription = null,
            modifier = Modifier.size(64.dp),
            tint = MaterialTheme.colorScheme.primary
        )

        Spacer(modifier = Modifier.height(16.dp))

        Text(
            text = "Nearby Discovery",
            style = MaterialTheme.typography.titleLarge
        )

        Spacer(modifier = Modifier.height(8.dp))

        Text(
            text = "Automatic discovery of nearby peers will be available in a future update",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}
