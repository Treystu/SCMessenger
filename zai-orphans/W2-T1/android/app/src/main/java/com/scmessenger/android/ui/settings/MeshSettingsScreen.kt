package com.scmessenger.android.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.compose.ui.res.stringResource
import com.scmessenger.android.R
import com.scmessenger.android.ui.components.ErrorBanner
import com.scmessenger.android.ui.components.WarningBanner
import com.scmessenger.android.ui.viewmodels.SettingsViewModel

/**
 * Mesh Settings screen - Relay and transport toggles.
 *
 * Provides controls for:
 * - Relay enable/disable and budget configuration
 * - Individual transport toggles (BLE, WiFi Aware, WiFi Direct, Internet)
 * - Discovery mode selection
 * 
 * Note: Privacy features (Onion Routing, Cover Traffic, Message Padding, Timing Obfuscation)
 * have been temporarily removed as they require full implementation in the Rust core.
 */

@OptIn(ExperimentalMaterial3Api::class)
@Suppress("USELESS_ELVIS")
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
                title = { Text(stringResource(R.string.mesh_settings_title)) },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = stringResource(R.string.chat_action_dismiss))
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
            
            // Bottom padding for scroll grace zone - allows pulling content up slightly
            // for better visibility of bottom elements on smaller screens
            Spacer(modifier = Modifier.height(80.dp))

            settings?.let { currentSettings ->
                // Warning banner about relay mode
                if (!currentSettings.relayEnabled) {
                    WarningBanner(
                        message = stringResource(R.string.mesh_settings_error_relay_disabled),
                        onDismiss = {}
                    )
                }

                // Relay Settings

                // Relay Settings
                SettingsSection(title = stringResource(R.string.mesh_settings_section_network_participation)) {
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
                                text = stringResource(R.string.settings_mesh_relay_warning_title),
                                style = MaterialTheme.typography.titleSmall,
                                fontWeight = FontWeight.Medium,
                                color = MaterialTheme.colorScheme.onErrorContainer
                            )
                            Spacer(modifier = Modifier.height(4.dp))
                            Text(
                                text = stringResource(R.string.settings_mesh_relay_warning_description),
                                style = MaterialTheme.typography.bodySmall,
                                fontWeight = FontWeight.Medium,
                                color = MaterialTheme.colorScheme.onErrorContainer
                            )
                            Spacer(modifier = Modifier.height(4.dp))
                            Text(
                                text = stringResource(R.string.mesh_settings_relay_warning_full_description),
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onErrorContainer
                            )
                        }
                    }

                    SwitchSetting(
                        title = stringResource(R.string.settings_label_mesh_participation),
                        description = stringResource(R.string.settings_description_mesh_participation),
                        checked = currentSettings.relayEnabled,
                        onCheckedChange = {
                            viewModel.updateSettings(currentSettings.copy(relayEnabled = it))
                        },
                        enabled = !isSaving
                    )

                    if (currentSettings.relayEnabled) {
                        SliderSetting(
                            title = stringResource(R.string.power_settings_label_battery_floor),
                            description = stringResource(R.string.power_settings_description_battery_floor),
                            value = currentSettings.batteryFloor.toFloat(),
                            valueRange = 0f..50f,
                            steps = 49,
                            onValueChange = {
                                viewModel.updateBatteryFloor(it.toInt().toUByte())
                            },
                            valueLabel = "${currentSettings.batteryFloor}%",
                            enabled = !isSaving
                        )

                        SliderSetting(
                            title = stringResource(R.string.mesh_settings_label_relay_budget),
                            description = stringResource(R.string.mesh_settings_description_relay_budget),
                            value = currentSettings.maxRelayBudget.toFloat(),
                            valueRange = 0f..500f,
                            steps = 49,
                            onValueChange = {
                                viewModel.updateMaxRelayBudget(it.toInt().toUInt())
                            },
                            valueLabel = "${currentSettings.maxRelayBudget} msg/hr",
                            enabled = !isSaving
                        )
                    }
                }

                // Transport Settings
                SettingsSection(title = stringResource(R.string.mesh_settings_section_transport)) {
                    SwitchSetting(
                        title = stringResource(R.string.mesh_settings_label_ble),
                        description = stringResource(R.string.mesh_settings_description_ble),
                        checked = currentSettings.bleEnabled,
                        onCheckedChange = {
                            viewModel.updateSettings(currentSettings.copy(bleEnabled = it))
                        },
                        enabled = !isSaving
                    )

                    SwitchSetting(
                        title = stringResource(R.string.settings_label_wifi_aware),
                        description = stringResource(R.string.mesh_settings_description_wifi_aware),
                        checked = currentSettings.wifiAwareEnabled,
                        onCheckedChange = {
                            viewModel.updateSettings(currentSettings.copy(wifiAwareEnabled = it))
                        },
                        enabled = !isSaving
                    )

                    SwitchSetting(
                        title = stringResource(R.string.settings_label_wifi_direct),
                        description = stringResource(R.string.mesh_settings_description_wifi_direct),
                        checked = currentSettings.wifiDirectEnabled,
                        onCheckedChange = {
                            viewModel.updateSettings(currentSettings.copy(wifiDirectEnabled = it))
                        },
                        enabled = !isSaving
                    )

                    SwitchSetting(
                        title = stringResource(R.string.mesh_settings_label_internet),
                        description = stringResource(R.string.mesh_settings_description_internet),
                        checked = currentSettings.internetEnabled,
                        onCheckedChange = {
                            viewModel.updateSettings(currentSettings.copy(internetEnabled = it))
                        },
                        enabled = !isSaving
                    )
                }

                // Discovery Settings
                SettingsSection(title = stringResource(R.string.mesh_settings_section_discovery)) {
                    DiscoveryModeSetting(
                        currentMode = currentSettings.discoveryMode,
                        onModeChange = {
                            viewModel.updateDiscoveryMode(it)
                        },
                        enabled = !isSaving
                    )
                }
            }
        } ?: Box(
            modifier = Modifier.fillMaxWidth().height(200.dp),
            contentAlignment = Alignment.Center
        ) {
            CircularProgressIndicator()
        }

        // Extra bottom padding for scroll grace zone
        Spacer(modifier = Modifier.height(80.dp))
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

        HorizontalDivider()
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
            text = stringResource(R.string.mesh_settings_label_discovery_mode),
            style = MaterialTheme.typography.bodyLarge,
            fontWeight = FontWeight.Medium
        )

        Text(
            text = stringResource(R.string.mesh_settings_description_discovery_mode),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )

        Spacer(modifier = Modifier.height(8.dp))

        val modes = listOf(
            uniffi.api.DiscoveryMode.CAUTIOUS to stringResource(R.string.mesh_settings_mode_cautious),
            uniffi.api.DiscoveryMode.NORMAL to stringResource(R.string.mesh_settings_mode_normal),
            uniffi.api.DiscoveryMode.PARANOID to stringResource(R.string.mesh_settings_mode_paranoid)
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
