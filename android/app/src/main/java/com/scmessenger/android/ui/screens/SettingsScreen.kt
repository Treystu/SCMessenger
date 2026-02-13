package com.scmessenger.android.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.ui.viewmodels.MeshServiceViewModel
import com.scmessenger.android.ui.viewmodels.SettingsViewModel
import uniffi.api.*

/**
 * Settings screen with mesh configuration and app preferences.
 */
@Composable
fun SettingsScreen(
    settingsViewModel: SettingsViewModel = hiltViewModel(),
    serviceViewModel: MeshServiceViewModel = hiltViewModel()
) {
    val meshSettings by settingsViewModel.meshSettings.collectAsState()
    val autoStart by settingsViewModel.autoStart.collectAsState()
    val notificationsEnabled by settingsViewModel.notificationsEnabled.collectAsState()
    val isLoading by settingsViewModel.isLoading.collectAsState()
    val serviceState by serviceViewModel.serviceState.collectAsState()
    val isRunning by serviceViewModel.isRunning.collectAsState()
    
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
            stats = serviceViewModel.getStatsText()
        )
        
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
        
        // App Preferences Section
        AppPreferencesSection(
            autoStart = autoStart,
            notificationsEnabled = notificationsEnabled,
            onAutoStartChange = { settingsViewModel.setAutoStart(it) },
            onNotificationsChange = { settingsViewModel.setNotificationsEnabled(it) }
        )
        
        Spacer(modifier = Modifier.height(24.dp))
        
        // Info Section
        InfoSection(
            contactCount = settingsViewModel.getContactCount(),
            messageCount = settingsViewModel.getMessageCount()
        )
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
                Divider()
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
            
            SwitchPreference(
                title = "Relay Messages",
                subtitle = "Help forward messages for others",
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
            InfoRow("Version", "0.1.0")
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
