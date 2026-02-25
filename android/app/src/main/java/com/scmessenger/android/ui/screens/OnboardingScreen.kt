package com.scmessenger.android.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Shield
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.ui.viewmodels.MainViewModel

@OptIn(com.google.accompanist.permissions.ExperimentalPermissionsApi::class)
@Composable
fun OnboardingScreen(
    onOnboardingComplete: () -> Unit,
    viewModel: MainViewModel = hiltViewModel()
) {
    val permissionsToRequest = remember {
        val list = mutableListOf(
            android.Manifest.permission.ACCESS_FINE_LOCATION
            // Add Bluetooth Scan/Connect/Advertise if API >= 31
        ).apply {
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.S) {
                add(android.Manifest.permission.BLUETOOTH_SCAN)
                add(android.Manifest.permission.BLUETOOTH_ADVERTISE)
                add(android.Manifest.permission.BLUETOOTH_CONNECT)
            }
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU) {
                add(android.Manifest.permission.NEARBY_WIFI_DEVICES)
                add(android.Manifest.permission.POST_NOTIFICATIONS)
            }
        }
        list.toList()
    }

    val permissionsState = com.google.accompanist.permissions.rememberMultiplePermissionsState(
        permissions = permissionsToRequest
    )

    val importError by viewModel.importError.collectAsState()
    val importSuccess by viewModel.importSuccess.collectAsState()
    val isReady by viewModel.isReady.collectAsState()
    val isCreating by viewModel.isCreatingIdentity.collectAsState()
    val identityError by viewModel.identityError.collectAsState()
    var showImportDialog by remember { mutableStateOf(false) }
    var importCode by remember { mutableStateOf("") }
    var nickname by remember { mutableStateOf("") }
    var hasAcceptedConsent by remember { mutableStateOf(false) }
    var consentChecked by remember { mutableStateOf(false) }

    LaunchedEffect(isReady) {
        if (isReady) {
            onOnboardingComplete()
        }
    }

    LaunchedEffect(importSuccess) {
        if (importSuccess) {
            viewModel.clearImportState()
            showImportDialog = false
            importCode = ""
            onOnboardingComplete()
        }
    }

    if (showImportDialog) {
        ImportContactDialog(
            importCode = importCode,
            onImportCodeChange = { importCode = it },
            importError = importError,
            onImport = { if (importCode.isNotBlank()) viewModel.importContact(importCode) },
            onDismiss = {
                showImportDialog = false
                importCode = ""
                viewModel.clearImportState()
            }
        )
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp),
        contentAlignment = Alignment.Center
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center,
            modifier = Modifier
                .fillMaxWidth()
                .verticalScroll(rememberScrollState())
        ) {
            Icon(
                imageVector = Icons.Filled.Lock,
                contentDescription = null,
                modifier = Modifier.size(80.dp),
                tint = MaterialTheme.colorScheme.primary
            )

            Spacer(modifier = Modifier.height(32.dp))

            Text(
                text = "Welcome to SCMessenger",
                style = MaterialTheme.typography.headlineMedium,
                textAlign = TextAlign.Center
            )

            Spacer(modifier = Modifier.height(16.dp))

            Text(
                text = "Secure, private communication without central servers. Your identity is generated locally and never leaves your device.",
                style = MaterialTheme.typography.bodyLarge,
                textAlign = TextAlign.Center,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )

            Spacer(modifier = Modifier.height(32.dp))

            // ── Consent Gate ──
            if (!hasAcceptedConsent) {
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.surfaceVariant
                    )
                ) {
                    Column(
                        modifier = Modifier.padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Text(
                            text = "Before You Begin",
                            style = MaterialTheme.typography.titleMedium
                        )

                        ConsentInfoItem(
                            icon = Icons.Filled.Lock,
                            title = "Keypair Identity",
                            detail = "Your identity is a cryptographic keypair stored only on this device. No phone numbers, emails, or accounts."
                        )
                        ConsentInfoItem(
                            icon = Icons.Filled.Shield,
                            title = "Local-Only Data & E2E Encryption",
                            detail = "All data is stored locally. Messages are end-to-end encrypted. Only the recipient can read them."
                        )
                        ConsentInfoItem(
                            icon = Icons.Filled.CheckCircle,
                            title = "Relay Participation",
                            detail = "Your device relays encrypted messages for others. This is how the mesh network operates."
                        )
                        ConsentInfoItem(
                            icon = Icons.Filled.Warning,
                            title = "Alpha Software",
                            detail = "Expect bugs and breaking changes. Do not rely on this for critical communications."
                        )

                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            Checkbox(
                                checked = consentChecked,
                                onCheckedChange = { consentChecked = it }
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text(
                                text = "I understand and accept these terms",
                                style = MaterialTheme.typography.bodyMedium
                            )
                        }

                        Button(
                            onClick = { hasAcceptedConsent = true },
                            enabled = consentChecked,
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            Text("Continue")
                        }
                    }
                }
            } else {

            if (isCreating) {
                CircularProgressIndicator()
                Spacer(modifier = Modifier.height(16.dp))
                Text("Generating Identity keys...")
            } else {
                if (permissionsState.allPermissionsGranted) {
                    OutlinedTextField(
                        value = nickname,
                        onValueChange = { nickname = it },
                        label = { Text("Your nickname") },
                        placeholder = { Text("e.g. christy") },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth()
                    )

                    Spacer(modifier = Modifier.height(12.dp))

                    Button(
                        onClick = {
                            viewModel.clearIdentityError()
                            viewModel.createIdentity(nickname)
                        },
                        enabled = nickname.trim().isNotEmpty(),
                        modifier = Modifier.fillMaxWidth().height(56.dp)
                    ) {
                        Text("Create New Identity")
                    }
                    identityError?.let { error ->
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = error,
                            style = MaterialTheme.typography.bodySmall,
                            textAlign = TextAlign.Center,
                            color = MaterialTheme.colorScheme.error
                        )
                    }
                } else {
                    Button(
                        onClick = {
                            permissionsState.launchMultiplePermissionRequest()
                        },
                        modifier = Modifier.fillMaxWidth().height(56.dp)
                    ) {
                        Text("Grant Permissions")
                    }
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = "Bluetooth and Location permissions are required for mesh networking.",
                        style = MaterialTheme.typography.bodySmall,
                        textAlign = TextAlign.Center,
                        color = MaterialTheme.colorScheme.error
                    )
                }

                Spacer(modifier = Modifier.height(16.dp))

                OutlinedButton(
                    onClick = {
                        importCode = ""
                        viewModel.clearImportState()
                        showImportDialog = true
                    },
                    modifier = Modifier.fillMaxWidth().height(56.dp)
                ) {
                    Text("Import Contact / Join Existing Mesh")
                }
            }
            } // end consent else
        }
    }
}

@Composable
private fun ConsentInfoItem(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    title: String,
    detail: String
) {
    Row(
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        modifier = Modifier.fillMaxWidth()
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            modifier = Modifier.size(24.dp),
            tint = MaterialTheme.colorScheme.primary
        )
        Column {
            Text(
                text = title,
                style = MaterialTheme.typography.titleSmall
            )
            Text(
                text = detail,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

@Composable
private fun ImportContactDialog(
    importCode: String,
    onImportCodeChange: (String) -> Unit,
    importError: String?,
    onImport: () -> Unit,
    onDismiss: () -> Unit
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Import Contact") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(
                    text = "Paste the identity JSON shared by your contact.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                OutlinedTextField(
                    value = importCode,
                    onValueChange = onImportCodeChange,
                    label = { Text("Identity JSON") },
                    modifier = Modifier.fillMaxWidth().heightIn(min = 120.dp),
                    minLines = 4,
                    maxLines = 8,
                    textStyle = LocalTextStyle.current.copy(fontFamily = FontFamily.Monospace),
                    placeholder = {
                        Text(
                            text = "{\"public_key\":\"...\",\"identity_id\":\"...\"}",
                            fontFamily = FontFamily.Monospace,
                            style = MaterialTheme.typography.bodySmall
                        )
                    }
                )
                importError?.let { error ->
                    Text(
                        text = error,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.error
                    )
                }
            }
        },
        confirmButton = {
            Button(onClick = onImport, enabled = importCode.isNotBlank()) {
                Text("Import")
            }
        },
        dismissButton = {
            OutlinedButton(onClick = onDismiss) { Text("Cancel") }
        }
    )
}
