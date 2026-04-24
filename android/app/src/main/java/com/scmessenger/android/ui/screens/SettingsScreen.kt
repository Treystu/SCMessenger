package com.scmessenger.android.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Block
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material.icons.filled.Share
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import kotlinx.coroutines.launch
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.BuildConfig
import com.scmessenger.android.R
import com.scmessenger.android.ui.viewmodels.MeshServiceViewModel
import com.scmessenger.android.ui.viewmodels.SettingsViewModel
import com.scmessenger.android.data.PreferencesRepository

/**
 * Settings screen with mesh configuration and app preferences.
 */
@Composable
fun SettingsScreen(
    settingsViewModel: SettingsViewModel = hiltViewModel(),
    serviceViewModel: MeshServiceViewModel = hiltViewModel(),
    onNavigateToIdentity: () -> Unit = {},
    onNavigateToDiagnostics: () -> Unit = {},
    onNavigateToBlockedPeers: () -> Unit = {}
) {
    val meshSettings by settingsViewModel.settings.collectAsState()
    val identityInfo by settingsViewModel.identityInfo.collectAsState()
    val autoStart by settingsViewModel.autoStart.collectAsState()
    val notificationsEnabled by settingsViewModel.notificationsEnabled.collectAsState()
    val themeMode by settingsViewModel.themeMode.collectAsState()
    val isLoading by settingsViewModel.isLoading.collectAsState()
    val serviceState by serviceViewModel.serviceState.collectAsState()
    val isRunning by serviceViewModel.isRunning.collectAsState()
    val serviceStats by serviceViewModel.serviceStats.collectAsState()

    val context = LocalContext.current
    val scope = rememberCoroutineScope()

    val importResult by settingsViewModel.importResult.collectAsState()
    val snackbarHostState = remember { SnackbarHostState() }

    var showImportDialog by remember { mutableStateOf(false) }
    var importText by remember { mutableStateOf("") }

    LaunchedEffect(importResult) {
        importResult?.let {
            snackbarHostState.showSnackbar(it)
            settingsViewModel.clearImportResult()
        }
    }

    // Refresh identity whenever Settings screen is entered (nickname may have changed during onboarding)
    LaunchedEffect(Unit) {
        settingsViewModel.loadIdentity()
    }

    // Reload identity when service comes online (identity may become available after startup)
    LaunchedEffect(serviceState) {
        if (serviceState == uniffi.api.ServiceState.RUNNING) {
            settingsViewModel.loadIdentity()
        }
    }

    val statsText = remember(serviceStats) { serviceViewModel.getStatsText() }

    Box(modifier = Modifier.fillMaxSize()) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(16.dp)
        ) {
        Text(
            text = "Settings",
            style = MaterialTheme.typography.headlineMedium,
            modifier = Modifier.padding(bottom = 16.dp)
        )

        // Service Control Section
        ServiceControlSection(
            isRunning = isRunning,
            serviceState = serviceState,
            onToggleService = { serviceViewModel.toggleService() },
            stats = statsText
        )

        Spacer(modifier = Modifier.height(24.dp))

        // Identity Section
        val resolvedIdentityInfo = identityInfo
        if (resolvedIdentityInfo?.initialized == true) {
            IdentitySection(
                identityInfo = resolvedIdentityInfo,
                onNicknameChange = { settingsViewModel.updateNickname(it) },
                onCopyExport = {
                    scope.launch {
                        val export = settingsViewModel.getIdentityExportString()
                        val clipboard = context.getSystemService(android.content.Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
                        val clip = android.content.ClipData.newPlainText("Identity Export", export)
                        clipboard.setPrimaryClip(clip)
                    }
                },
                onShowIdentityQr = onNavigateToIdentity,
                onImportIdentity = { showImportDialog = true }
            )
        } else {
            IdentityUnavailableSection(
                onCreateIdentity = onNavigateToIdentity
            )
        }

        Spacer(modifier = Modifier.height(24.dp))

        // Mesh Settings Section
        meshSettings?.let { settings ->
            MeshSettingsSection(
                settings = settings,
                onUpdateSetting = { updater -> updater(settingsViewModel) },
                isLoading = isLoading
            )
        }

        Spacer(modifier = Modifier.height(24.dp))



        // Theme Section
        ThemeSection(
            themeMode = themeMode,
            onThemeModeChange = { settingsViewModel.setThemeMode(it) }
        )

        Spacer(modifier = Modifier.height(24.dp))

        // App Preferences Section
        AppPreferencesSection(
            autoStart = autoStart,
            notificationsEnabled = notificationsEnabled,
            onAutoStartChange = { settingsViewModel.setAutoStart(it) },
            onNotificationsChange = { settingsViewModel.setNotificationsEnabled(it) }
        )

        Spacer(modifier = Modifier.height(24.dp))

        // Privacy Section
        PrivacySection(
            onNavigateToBlockedPeers = onNavigateToBlockedPeers
        )

        Spacer(modifier = Modifier.height(24.dp))

        // Info Section
        InfoSection(
            contactCount = settingsViewModel.getContactCount(),
            messageCount = settingsViewModel.getMessageCount()
        )

        Spacer(modifier = Modifier.height(24.dp))

        // Advanced / Diagnostics Section
        AdvancedSettingsSection(
            onNavigateToDiagnostics = onNavigateToDiagnostics
        )

        Spacer(modifier = Modifier.height(24.dp))

        // Data Management Section
        DataManagementSection(
            onResetAll = { settingsViewModel.resetAllData() }
        )
    }

    if (showImportDialog) {
        AlertDialog(
            onDismissRequest = { showImportDialog = false },
            title = { Text("Import Identity Backup") },
            text = {
                OutlinedTextField(
                    value = importText,
                    onValueChange = { importText = it },
                    label = { Text("Paste backup string") },
                    minLines = 3,
                    maxLines = 6
                )
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        settingsViewModel.importIdentityBackup(importText)
                        showImportDialog = false
                        importText = ""
                    },
                    enabled = importText.isNotBlank()
                ) {
                    Text("Import")
                }
            },
            dismissButton = {
                TextButton(onClick = { showImportDialog = false }) {
                    Text("Cancel")
                }
            }
        )
    }

    SnackbarHost(
        hostState = snackbarHostState,
        modifier = Modifier.align(Alignment.BottomCenter)
    )
}
}

@Composable
fun DataManagementSection(
    onResetAll: () -> Unit
) {
    var showConfirmDialog by remember { mutableStateOf(false) }

    if (showConfirmDialog) {
        AlertDialog(
            onDismissRequest = { showConfirmDialog = false },
            title = { Text("Reset All Data?") },
            text = { Text("This will permanently delete your identity, messages, contacts, and settings. This action cannot be undone.") },
            confirmButton = {
                TextButton(
                    onClick = {
                        showConfirmDialog = false
                        onResetAll()
                    },
                    colors = ButtonDefaults.textButtonColors(contentColor = MaterialTheme.colorScheme.error)
                ) {
                    Text("RESET")
                }
            },
            dismissButton = {
                TextButton(onClick = { showConfirmDialog = false }) {
                    Text("Cancel")
                }
            }
        )
    }

    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.2f)
        )
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = "Data Management",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.error,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            Text(
                text = "DANGER ZONE: This permanently wipes your identity and all local history.",
                style = MaterialTheme.typography.bodySmall,
                modifier = Modifier.padding(bottom = 16.dp)
            )

            Button(
                onClick = { showConfirmDialog = true },
                modifier = Modifier.fillMaxWidth(),
                colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.error)
            ) {
                Text("Delete All Data & Reset App")
            }
        }
    }
}

@Composable
fun ServiceControlSection(
    isRunning: Boolean,
    serviceState: uniffi.api.ServiceState,
    onToggleService: () -> Unit,
    stats: String
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = "Mesh Service",
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column {
                    Text("Status: ${serviceState.name}")
                    if (isRunning) {
                        Text(
                            text = "Active",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.primary
                        )
                    }
                }

                Button(onClick = onToggleService) {
                    Text(if (isRunning) "Stop" else "Start")
                }
            }

            if (isRunning) {
                Spacer(modifier = Modifier.height(8.dp))
                HorizontalDivider()
                Spacer(modifier = Modifier.height(8.dp))
                Text(
                    text = stats,
                    style = MaterialTheme.typography.bodySmall,
                    modifier = Modifier.fillMaxWidth()
                )
            }
        }
    }
}

@Composable
fun MeshSettingsSection(
    settings: uniffi.api.MeshSettings,
    onUpdateSetting: ((SettingsViewModel) -> Unit) -> Unit,
    isLoading: Boolean
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = "Mesh Network",
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            // Warning about relay requirement
            Card(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 8.dp),
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.errorContainer
                )
            ) {
                Column(
                    modifier = Modifier.padding(12.dp)
                ) {
                    Text(
                        text = "⚠️ Relay = Messaging (Bidirectional)",
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.Medium,
                        color = MaterialTheme.colorScheme.onErrorContainer
                    )
                    Text(
                        text = "OFF = no sending OR receiving. Complete shutdown both directions. ON = full participation.",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onErrorContainer
                    )
                }
            }

            SwitchPreference(
                title = "Mesh Participation",
                subtitle = "Controls ALL communication. OFF = bidirectional shutdown.",
                checked = settings.relayEnabled,
                onCheckedChange = { onUpdateSetting { vm -> vm.updateRelayEnabled(it) } },
                enabled = !isLoading
            )

            SwitchPreference(
                title = "Bluetooth LE",
                subtitle = "Discover peers via Bluetooth",
                checked = settings.bleEnabled,
                onCheckedChange = { onUpdateSetting { vm -> vm.updateBleEnabled(it) } },
                enabled = !isLoading
            )

            SwitchPreference(
                title = "WiFi Aware",
                subtitle = "Direct WiFi peer discovery",
                checked = settings.wifiAwareEnabled,
                onCheckedChange = { onUpdateSetting { vm -> vm.updateWifiAwareEnabled(it) } },
                enabled = !isLoading
            )

            SwitchPreference(
                title = "WiFi Direct",
                subtitle = "WiFi Direct connections",
                checked = settings.wifiDirectEnabled,
                onCheckedChange = { onUpdateSetting { vm -> vm.updateWifiDirectEnabled(it) } },
                enabled = !isLoading
            )

            SwitchPreference(
                title = "Internet Relay",
                subtitle = "Use internet as fallback",
                checked = settings.internetEnabled,
                onCheckedChange = { onUpdateSetting { vm -> vm.updateInternetEnabled(it) } },
                enabled = !isLoading
            )
        }
    }
}



@Composable
fun AppPreferencesSection(
    autoStart: Boolean,
    notificationsEnabled: Boolean,
    onAutoStartChange: (Boolean) -> Unit,
    onNotificationsChange: (Boolean) -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = "App Preferences",
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            SwitchPreference(
                title = "Auto-start on Boot",
                subtitle = "Start mesh service when device boots",
                checked = autoStart,
                onCheckedChange = onAutoStartChange
            )

            SwitchPreference(
                title = "Notifications",
                subtitle = "Show message notifications",
                checked = notificationsEnabled,
                onCheckedChange = onNotificationsChange
            )
        }
    }
}

@Composable
fun ThemeSection(
    themeMode: PreferencesRepository.ThemeMode,
    onThemeModeChange: (PreferencesRepository.ThemeMode) -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = "Theme",
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            ThemeRadioOption(
                label = "System Default",
                selected = themeMode == PreferencesRepository.ThemeMode.SYSTEM,
                onClick = { onThemeModeChange(PreferencesRepository.ThemeMode.SYSTEM) }
            )
            ThemeRadioOption(
                label = "Light",
                selected = themeMode == PreferencesRepository.ThemeMode.LIGHT,
                onClick = { onThemeModeChange(PreferencesRepository.ThemeMode.LIGHT) }
            )
            ThemeRadioOption(
                label = "Dark",
                selected = themeMode == PreferencesRepository.ThemeMode.DARK,
                onClick = { onThemeModeChange(PreferencesRepository.ThemeMode.DARK) }
            )
        }
    }
}

@Composable
fun ThemeRadioOption(
    label: String,
    selected: Boolean,
    onClick: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        RadioButton(
            selected = selected,
            onClick = onClick
        )
        Spacer(modifier = Modifier.width(8.dp))
        Text(
            text = label,
            style = MaterialTheme.typography.bodyLarge
        )
    }
}

@Composable
fun SwitchPreference(
    title: String,
    subtitle: String,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
    enabled: Boolean = true
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 8.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(
            modifier = Modifier.weight(1f)
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge
            )
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }

        Switch(
            checked = checked,
            onCheckedChange = onCheckedChange,
            enabled = enabled
        )
    }
}

@Composable
fun InfoSection(
    contactCount: UInt,
    messageCount: UInt
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = "Information",
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            InfoRow("Contacts", contactCount.toString())
            InfoRow("Messages", messageCount.toString())
            InfoRow("Version", BuildConfig.VERSION_NAME)
        }
    }
}

@Composable
fun InfoRow(label: String, value: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.bodyMedium
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

@Composable
fun IdentitySection(
    identityInfo: uniffi.api.IdentityInfo,
    onNicknameChange: (String) -> Unit,
    onCopyExport: () -> Unit,
    onShowIdentityQr: () -> Unit,
    onImportIdentity: () -> Unit
) {
    val context = LocalContext.current

    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = "Identity",
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            // Nickname Input
            var pendingNickname by remember(identityInfo.nickname) { mutableStateOf(identityInfo.nickname ?: "") }
            OutlinedTextField(
                value = pendingNickname,
                onValueChange = {
                    pendingNickname = it
                    // DO NOT call onNicknameChange here — it triggers a Rust FFI roundtrip per keystroke
                },
                label = { Text("Nickname") },
                modifier = Modifier.fillMaxWidth(),
                singleLine = true,
                keyboardOptions = androidx.compose.foundation.text.KeyboardOptions(
                    imeAction = androidx.compose.ui.text.input.ImeAction.Done
                ),
                keyboardActions = androidx.compose.foundation.text.KeyboardActions(
                    onDone = {
                        onNicknameChange(pendingNickname.trim())
                    }
                )
            )
            Spacer(modifier = Modifier.height(4.dp))
            Button(
                onClick = { onNicknameChange(pendingNickname.trim()) },
                modifier = Modifier.fillMaxWidth()
            ) {
                Text("Save Nickname")
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Peer ID (Network)
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "Peer ID (Network)",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = identityInfo.libp2pPeerId?.take(16) ?: "????????",
                        style = MaterialTheme.typography.bodyMedium,
                        fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace
                    )
                }

                IconButton(onClick = {
                    val clipboard = context.getSystemService(android.content.Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
                    val clip = android.content.ClipData.newPlainText("Peer ID", identityInfo.libp2pPeerId ?: "")
                    clipboard.setPrimaryClip(clip)
                }) {
                    Icon(Icons.Default.ContentCopy, contentDescription = "Copy Peer ID")
                }
            }

            HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp))

            // Identity Hash
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "Identity Hash",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = identityInfo.identityId?.take(8) ?: "????????",
                        style = MaterialTheme.typography.bodyMedium,
                        fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace
                    )
                }

                IconButton(onClick = {
                    val clipboard = context.getSystemService(android.content.Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
                    val clip = android.content.ClipData.newPlainText("Identity Hash", identityInfo.identityId ?: "")
                    clipboard.setPrimaryClip(clip)
                }) {
                    Icon(Icons.Default.ContentCopy, contentDescription = "Copy Identity Hash")
                }
            }

            HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp))

            // Public Key
             Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "Public Key",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = identityInfo.publicKeyHex?.take(8) ?: "????????",
                        style = MaterialTheme.typography.bodyMedium,
                        fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace
                    )
                }

                IconButton(onClick = {
                    val clipboard = context.getSystemService(android.content.Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
                    val clip = android.content.ClipData.newPlainText("Public Key", identityInfo.publicKeyHex ?: "")
                    clipboard.setPrimaryClip(clip)
                }) {
                    Icon(Icons.Default.ContentCopy, contentDescription = "Copy Key")
                }
            }

            Spacer(modifier = Modifier.height(8.dp))

            // Full Export Button
            OutlinedButton(
                onClick = onCopyExport,
                modifier = Modifier.fillMaxWidth()
            ) {
                Icon(Icons.Default.Share, contentDescription = "Share identity export", modifier = Modifier.size(16.dp))
                Spacer(modifier = Modifier.width(8.dp))
                Text("Copy Full Identity Export")
            }

            Spacer(modifier = Modifier.height(8.dp))

            Button(
                onClick = onShowIdentityQr,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text("Show Identity QR")
            }

            Spacer(modifier = Modifier.height(8.dp))

            OutlinedButton(
                onClick = onImportIdentity,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text("Import Identity")
            }
        }
    }
}

@Composable
fun IdentityUnavailableSection(
    onCreateIdentity: () -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(
                text = "Identity",
                style = MaterialTheme.typography.titleMedium
            )
            Text(
                text = "This node is currently relay-only. Create an identity to enable chats and contacts.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Button(
                onClick = onCreateIdentity,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text("Create Identity")
            }
        }
    }
}
@Composable
fun PrivacySection(
    onNavigateToBlockedPeers: () -> Unit
) {
    val context = LocalContext.current
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = "Privacy",
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            Button(
                onClick = onNavigateToBlockedPeers,
                modifier = Modifier.fillMaxWidth()
            ) {
                Icon(Icons.Filled.Block, contentDescription = "Manage blocked peers", modifier = Modifier.size(16.dp))
                Spacer(modifier = Modifier.width(8.dp))
                Text("Manage Blocked Peers")
            }

            Spacer(modifier = Modifier.height(8.dp))

            OutlinedButton(
                onClick = {
                    val intent = android.content.Intent(
                        android.content.Intent.ACTION_VIEW,
                        android.net.Uri.parse(context.getString(R.string.privacy_policy_url))
                    )
                    context.startActivity(intent)
                },
                modifier = Modifier.fillMaxWidth()
            ) {
                Text("Privacy Policy")
            }
        }
    }
}

@Composable
fun AdvancedSettingsSection(
    onNavigateToDiagnostics: () -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = "Advanced",
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            Text(
                text = "Reliability: messages may appear as pending/stored/forwarding before delivered while routes are discovered.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            Text(
                text = "Permissions rationale: Bluetooth, Location, and Nearby WiFi are required for peer discovery and transport fallback.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(bottom = 12.dp)
            )

            Button(
                onClick = onNavigateToDiagnostics,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text("Diagnostics & Logs")
            }
        }
    }
}
