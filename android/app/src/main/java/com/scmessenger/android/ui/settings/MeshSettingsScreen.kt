package com.scmessenger.android.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.ui.components.ErrorBanner
import com.scmessenger.android.ui.viewmodels.SettingsViewModel

/**
 * Mesh Settings screen - Relay and transport toggles.
 * 
 * Provides controls for:
 * - Relay enable/disable and budget configuration
 * - Individual transport toggles (BLE, WiFi Aware, WiFi Direct, Internet)
 * - Discovery mode selection
 * - Onion routing (privacy feature)
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MeshSettingsScreen(
    onNavigateBack: () -> Unit,
    viewModel: SettingsViewModel = hiltViewModel()
) {
    val settings by viewModel.settings.collectAsState()
    val error by viewModel.error.collectAsState()
    val isSaving by viewModel.isSaving.collectAsState()
    
    LaunchedEffect(Unit) {
        viewModel.loadSettings()
    }
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Mesh Settings") },
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
                .verticalScroll(rememberScrollState())
        ) {
            // Error banner
            error?.let {
                ErrorBanner(
                    message = it,
                    onDismiss = { viewModel.clearError() }
                )
            }
            
            if (settings == null) {
                Box(
                    modifier = Modifier.fillMaxWidth().height(200.dp),
                    contentAlignment = Alignment.Center
                ) {
                    CircularProgressIndicator()
                }
            } else {
                val currentSettings = settings!!
                
                // Relay Settings
                SettingsSection(title = "Network Participation") {
                    // Warning/Info about relay requirement
                    Card(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 16.dp, vertical = 8.dp),
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
                                fontWeight = FontWeight.Bold,
                                color = MaterialTheme.colorScheme.onErrorContainer
                            )
                            Spacer(modifier = Modifier.height(4.dp))
                            Text(
                                text = "This single toggle controls ALL communication in BOTH directions. When OFF: you cannot send OR receive messages, cannot relay for others, and others cannot relay for you. Complete network shutdown. When ON: full mesh participation with bidirectional messaging.",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onErrorContainer
                            )
                        }
                    }
                    
                    SwitchSetting(
                        title = "Mesh Participation",
                        description = "Controls ALL sending AND receiving. OFF = complete communication shutdown in both directions.",
                        checked = currentSettings.relayEnabled,
                        onCheckedChange = {
                            viewModel.updateSettings(currentSettings.copy(relayEnabled = it))
                        },
                        enabled = !isSaving
                    )
                    
                    if (currentSettings.relayEnabled) {
                        SliderSetting(
                            title = "Relay Budget",
                            description = "Maximum messages to relay per hour",
                            value = currentSettings.maxRelayBudget.toFloat(),
                            valueRange = 0f..500f,
                            steps = 49,
                            onValueChange = {
                                viewModel.updateSettings(currentSettings.copy(maxRelayBudget = it.toUInt()))
                            },
                            valueLabel = "${currentSettings.maxRelayBudget} msg/hr",
                            enabled = !isSaving
                        )
                        
                        SliderSetting(
                            title = "Battery Floor",
                            description = "Stop relaying below this battery level",
                            value = currentSettings.batteryFloor.toFloat(),
                            valueRange = 0f..50f,
                            steps = 49,
                            onValueChange = {
                                viewModel.updateSettings(currentSettings.copy(batteryFloor = it.toInt().toUByte()))
                            },
                            valueLabel = "${currentSettings.batteryFloor}%",
                            enabled = !isSaving
                        )
                    }
                }
                
                // Transport Settings
                SettingsSection(title = "Transport Settings") {
                    SwitchSetting(
                        title = "Bluetooth Low Energy",
                        description = "Peer discovery and communication via BLE",
                        checked = currentSettings.bleEnabled,
                        onCheckedChange = {
                            viewModel.updateSettings(currentSettings.copy(bleEnabled = it))
                        },
                        enabled = !isSaving
                    )
                    
                    SwitchSetting(
                        title = "WiFi Aware",
                        description = "Peer discovery using WiFi Aware (Android 8+)",
                        checked = currentSettings.wifiAwareEnabled,
                        onCheckedChange = {
                            viewModel.updateSettings(currentSettings.copy(wifiAwareEnabled = it))
                        },
                        enabled = !isSaving
                    )
                    
                    SwitchSetting(
                        title = "WiFi Direct",
                        description = "Direct peer-to-peer WiFi connections",
                        checked = currentSettings.wifiDirectEnabled,
                        onCheckedChange = {
                            viewModel.updateSettings(currentSettings.copy(wifiDirectEnabled = it))
                        },
                        enabled = !isSaving
                    )
                    
                    SwitchSetting(
                        title = "Internet (libp2p)",
                        description = "Connect to peers over the internet",
                        checked = currentSettings.internetEnabled,
                        onCheckedChange = {
                            viewModel.updateSettings(currentSettings.copy(internetEnabled = it))
                        },
                        enabled = !isSaving
                    )
                }
                
                // Discovery Settings
                SettingsSection(title = "Discovery Settings") {
                    DiscoveryModeSetting(
                        currentMode = currentSettings.discoveryMode,
                        onModeChange = {
                            viewModel.updateSettings(currentSettings.copy(discoveryMode = it))
                        },
                        enabled = !isSaving
                    )
                }
                
                // Privacy Settings
                SettingsSection(title = "Privacy Settings") {
                    SwitchSetting(
                        title = "Onion Routing",
                        description = "Route messages through multiple hops for privacy",
                        checked = currentSettings.onionRouting,
                        onCheckedChange = {
                            viewModel.updateSettings(currentSettings.copy(onionRouting = it))
                        },
                        enabled = !isSaving
                    )
                }
            }
            
            Spacer(modifier = Modifier.height(16.dp))
        }
    }
}

@Composable
private fun SettingsSection(
    title: String,
    modifier: Modifier = Modifier,
    content: @Composable ColumnScope.() -> Unit
) {
    Column(modifier = modifier.fillMaxWidth()) {
        Text(
            text = title,
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 12.dp),
            style = MaterialTheme.typography.titleSmall,
            fontWeight = FontWeight.Bold,
            color = MaterialTheme.colorScheme.primary
        )
        
        content()
        
        Divider()
    }
}

@Composable
private fun SwitchSetting(
    title: String,
    description: String,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
    enabled: Boolean = true,
    modifier: Modifier = Modifier
) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 12.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Medium
            )
            Text(
                text = description,
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
private fun SliderSetting(
    title: String,
    description: String,
    value: Float,
    valueRange: ClosedFloatingPointRange<Float>,
    steps: Int,
    onValueChange: (Float) -> Unit,
    valueLabel: String,
    enabled: Boolean = true,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 12.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Medium
            )
            Text(
                text = valueLabel,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.primary
            )
        }
        
        Text(
            text = description,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        
        Slider(
            value = value,
            onValueChange = onValueChange,
            valueRange = valueRange,
            steps = steps,
            enabled = enabled,
            modifier = Modifier.fillMaxWidth()
        )
    }
}

@Composable
private fun DiscoveryModeSetting(
    currentMode: uniffi.api.DiscoveryMode,
    onModeChange: (uniffi.api.DiscoveryMode) -> Unit,
    enabled: Boolean = true,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 12.dp)
    ) {
        Text(
            text = "Discovery Mode",
            style = MaterialTheme.typography.bodyLarge,
            fontWeight = FontWeight.Medium
        )
        
        Text(
            text = "Control how aggressively this device discovers peers",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        
        Spacer(modifier = Modifier.height(8.dp))
        
        val modes = listOf(
            uniffi.api.DiscoveryMode.CAUTIOUS to "Cautious",
            uniffi.api.DiscoveryMode.NORMAL to "Normal",
            uniffi.api.DiscoveryMode.PARANOID to "Paranoid"
        )
        
        modes.forEach { (mode, label) ->
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                RadioButton(
                    selected = currentMode == mode,
                    onClick = { onModeChange(mode) },
                    enabled = enabled
                )
                Text(
                    text = label,
                    modifier = Modifier.padding(start = 8.dp)
                )
            }
        }
    }
}
