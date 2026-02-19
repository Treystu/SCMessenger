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
 * Power Settings screen - AutoAdjust engine and battery management.
 *
 * Provides controls for:
 * - AutoAdjust enable/disable (automatic resource management)
 * - Manual adjustment profile selection
 * - BLE scan interval override
 * - Relay budget override
 * - Battery optimization settings
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PowerSettingsScreen(
    onNavigateBack: () -> Unit,
    viewModel: SettingsViewModel = hiltViewModel()
) {
    val autoAdjustEnabled by viewModel.autoAdjustEnabled.collectAsState()
    val currentProfile by viewModel.adjustmentProfile.collectAsState()
    val error by viewModel.error.collectAsState()
    val settings by viewModel.settings.collectAsState()

    var bleScanInterval by remember { mutableStateOf(2000u) }
    var relayMaxPerHour by remember { mutableStateOf(200u) }

    LaunchedEffect(Unit) {
        viewModel.loadSettings()
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Power Settings") },
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

            // AutoAdjust Settings
            SettingsSection(title = "AutoAdjust Engine") {
                SwitchSetting(
                    title = "Enable AutoAdjust",
                    description = "Automatically adjust settings based on battery and network conditions",
                    checked = autoAdjustEnabled,
                    onCheckedChange = { viewModel.setAutoAdjust(it) }
                )

                if (autoAdjustEnabled) {
                    Card(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 16.dp, vertical = 8.dp)
                    ) {
                        Column(modifier = Modifier.padding(16.dp)) {
                            Text(
                                text = "Current Profile",
                                style = MaterialTheme.typography.labelMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                            Text(
                                text = currentProfile?.name ?: "Unknown",
                                style = MaterialTheme.typography.titleLarge,
                                fontWeight = FontWeight.Bold,
                                color = MaterialTheme.colorScheme.primary
                            )
                        }
                    }
                }
            }

            // Manual Profile Override
            if (!autoAdjustEnabled) {
                SettingsSection(title = "Manual Profile") {
                    ProfileSelector(
                        currentProfile = currentProfile,
                        onProfileSelected = { viewModel.setManualProfile(it) }
                    )
                }
            }

            // Advanced Overrides
            SettingsSection(title = "Advanced Overrides") {
                SliderSetting(
                    title = "BLE Scan Interval",
                    description = "How often to scan for BLE peers (lower = more battery)",
                    value = bleScanInterval.toFloat(),
                    valueRange = 500f..10000f,
                    steps = 18,
                    onValueChange = {
                        bleScanInterval = it.toUInt()
                        viewModel.overrideBleScanInterval(it.toUInt())
                    },
                    valueLabel = "${bleScanInterval}ms",
                    enabled = !autoAdjustEnabled
                )

                SliderSetting(
                    title = "Relay Max Per Hour",
                    description = "Maximum messages to relay per hour",
                    value = relayMaxPerHour.toFloat(),
                    valueRange = 0f..500f,
                    steps = 49,
                    onValueChange = {
                        relayMaxPerHour = it.toUInt()
                        viewModel.overrideRelayMax(it.toUInt())
                    },
                    valueLabel = "${relayMaxPerHour} msg/hr",
                    enabled = !autoAdjustEnabled
                )

                Button(
                    onClick = { viewModel.clearAdjustmentOverrides() },
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 8.dp),
                    enabled = !autoAdjustEnabled
                ) {
                    Text("Reset to Defaults")
                }
            }

            // Battery Settings
            settings?.let { currentSettings ->
                SettingsSection(title = "Battery Management") {
                    SliderSetting(
                        title = "Battery Floor",
                        description = "Stop relaying when battery drops below this level",
                        value = currentSettings.batteryFloor.toFloat(),
                        valueRange = 0f..50f,
                        steps = 49,
                        onValueChange = {
                            viewModel.updateSettings(currentSettings.copy(batteryFloor = it.toInt().toUByte()))
                        },
                        valueLabel = "${currentSettings.batteryFloor}%"
                    )

                    InfoCard(
                        title = "Power Saving Tips",
                        message = """
                            • Enable AutoAdjust for automatic optimization
                            • Increase BLE scan interval to save battery
                            • Disable unused transports
                            • Lower relay budget on battery power
                        """.trimIndent()
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
private fun ProfileSelector(
    currentProfile: uniffi.api.AdjustmentProfile?,
    onProfileSelected: (uniffi.api.AdjustmentProfile) -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 12.dp)
    ) {
        val profiles = listOf(
            uniffi.api.AdjustmentProfile.MINIMAL to "Minimal (Battery Saver)",
            uniffi.api.AdjustmentProfile.STANDARD to "Standard (Balanced)",
            uniffi.api.AdjustmentProfile.MAXIMUM to "Maximum (Performance)"
        )

        profiles.forEach { (profile, label) ->
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                RadioButton(
                    selected = currentProfile == profile,
                    onClick = { onProfileSelected(profile) }
                )
                Text(
                    text = label,
                    modifier = Modifier.padding(start = 8.dp)
                )
            }
        }
    }
}

@Composable
private fun InfoCard(
    title: String,
    message: String,
    modifier: Modifier = Modifier
) {
    Card(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 8.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.secondaryContainer
        )
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = title,
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onSecondaryContainer
            )
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = message,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSecondaryContainer
            )
        }
    }
}
