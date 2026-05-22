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
import com.scmessenger.android.ui.components.ErrorState
import com.scmessenger.android.ui.components.IdenticonFromHex
import com.scmessenger.android.ui.components.InfoBanner
import com.scmessenger.android.ui.components.LabeledCopyableText
import com.scmessenger.android.ui.components.TruncatedCopyableText
import com.scmessenger.android.ui.components.WarningBanner
import com.scmessenger.android.ui.viewmodels.SettingsViewModel
import android.net.VpnService
import android.app.Activity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.ui.platform.LocalContext

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

    val context = LocalContext.current
    val vpnModeEnabled by viewModel.vpnMode.collectAsState(initial = false)

    val vpnLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.StartActivityForResult()
    ) { result ->
        if (result.resultCode == Activity.RESULT_OK) {
            viewModel.setVpnMode(true)
        }
    }

    var bleScanInterval by remember { mutableStateOf(2000u) }
    var relayMaxPerHour by remember { mutableStateOf(200u) }

    LaunchedEffect(Unit) {
        viewModel.loadSettings()
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.power_settings_title)) },
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

            // Info banner about AutoAdjust
            if (autoAdjustEnabled) {
                InfoBanner(
                    message = stringResource(R.string.power_settings_info_autoadjust_active),
                    onDismiss = {}
                )
            }

            // Error state for persistent errors
            val currentError = error
            if (currentError != null && currentError.isNotEmpty()) {
                ErrorState(
                    error = currentError,
                    onDismiss = { viewModel.clearError() }
                )
            }

            // AutoAdjust Settings
            SettingsSection(title = stringResource(R.string.power_settings_section_autoadjust)) {
                SwitchSetting(
                    title = stringResource(R.string.power_settings_label_enable_autoadjust),
                    description = stringResource(R.string.power_settings_description_enable_autoadjust),
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
                                text = stringResource(R.string.power_settings_label_current_profile),
                                style = MaterialTheme.typography.labelMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                            Text(
                                text = currentProfile.name,
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
                SettingsSection(title = stringResource(R.string.power_settings_section_manual_profile)) {
                    ProfileSelector(
                        currentProfile = currentProfile,
                        onProfileSelected = { viewModel.setManualProfile(it) }
                    )
                }
            }

            // Advanced Overrides
            SettingsSection(title = stringResource(R.string.power_settings_section_advanced_overrides)) {
                SliderSetting(
                    title = stringResource(R.string.power_settings_label_ble_scan_interval),
                    description = stringResource(R.string.power_settings_description_ble_scan_interval),
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
                    title = stringResource(R.string.power_settings_label_relay_max_per_hour),
                    description = stringResource(R.string.power_settings_description_relay_max_per_hour),
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
                    Text(stringResource(R.string.power_settings_action_reset_defaults))
                }
            }

            // Component Examples (for demonstration)
            if (autoAdjustEnabled) {
                SettingsSection(title = "Component Examples") {
                    Card(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 16.dp, vertical = 8.dp)
                    ) {
                        Column(modifier = Modifier.padding(16.dp)) {
                            Text(
                                text = "Identicon Example",
                                style = MaterialTheme.typography.labelMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                            IdenticonFromHex(
                                hexString = "0123456789abcdef0123456789abcdef",
                                size = 48.dp,
                                modifier = Modifier.align(Alignment.CenterHorizontally)
                            )
                            Text(
                                text = "IdenticonFromHex(0123456789abcdef...)",
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                modifier = Modifier.align(Alignment.CenterHorizontally)
                            )
                        }
                    }

                    Card(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 16.dp, vertical = 8.dp)
                    ) {
                        Column(modifier = Modifier.padding(16.dp)) {
                            LabeledCopyableText(
                                label = "BLE Scan Interval",
                                text = "${bleScanInterval}ms",
                                monospace = true
                            )
                        }
                    }

                    Card(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 16.dp, vertical = 8.dp)
                    ) {
                        Column(modifier = Modifier.padding(16.dp)) {
                            TruncatedCopyableText(
                                text = "0123456789abcdef0123456789abcdef",
                                label = "Identity Hash",
                                maxLength = 16
                            )
                        }
                    }
                }
            }

            // Battery Settings
            settings?.let { currentSettings ->
                SettingsSection(title = stringResource(R.string.power_settings_section_battery_management)) {
                    SliderSetting(
                        title = stringResource(R.string.power_settings_label_battery_floor),
                        description = stringResource(R.string.power_settings_description_battery_floor),
                        value = currentSettings.batteryFloor.toFloat(),
                        valueRange = 0f..50f,
                        steps = 49,
                        onValueChange = {
                            viewModel.updateBatteryFloor(it.toInt().toUByte())
                        },
                        valueLabel = "${currentSettings.batteryFloor}%"
                    )

                    SwitchSetting(
                        title = stringResource(R.string.power_settings_label_vpn_tunnel),
                        description = stringResource(R.string.power_settings_description_vpn_tunnel),
                        checked = vpnModeEnabled,
                        onCheckedChange = { enabled ->
                            if (enabled) {
                                val intent = VpnService.prepare(context)
                                if (intent != null) {
                                    vpnLauncher.launch(intent)
                                } else {
                                    viewModel.setVpnMode(true)
                                }
                            } else {
                                viewModel.setVpnMode(false)
                            }
                        }
                    )

                    InfoCard(
                        title = stringResource(R.string.power_settings_section_tips),
                        message = stringResource(R.string.power_settings_tips_content)
                    )
                }
            }

            // Extra bottom padding for scroll grace zone
            Spacer(modifier = Modifier.height(80.dp))
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
            uniffi.api.AdjustmentProfile.MINIMAL to stringResource(R.string.power_settings_profile_minimal),
            uniffi.api.AdjustmentProfile.STANDARD to stringResource(R.string.power_settings_profile_standard),
            uniffi.api.AdjustmentProfile.MAXIMUM to stringResource(R.string.power_settings_profile_maximum)
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
