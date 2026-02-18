package com.scmessenger.android.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Info
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
 * Privacy Settings screen - Privacy controls and features.
 * 
 * Provides controls for:
 * - Onion routing (multi-hop message routing)
 * - Cover traffic (traffic analysis resistance)
 * - Message padding (metadata protection)
 * - Timing obfuscation
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PrivacySettingsScreen(
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
                title = { Text("Privacy Settings") },
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
                
                // Privacy Notice
                Card(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.primaryContainer
                    )
                ) {
                    Row(
                        modifier = Modifier.padding(16.dp),
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Icon(
                            imageVector = Icons.Default.Info,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onPrimaryContainer
                        )
                        
                        Column {
                            Text(
                                text = "Privacy by Design",
                                style = MaterialTheme.typography.titleSmall,
                                fontWeight = FontWeight.Bold,
                                color = MaterialTheme.colorScheme.onPrimaryContainer
                            )
                            
                            Spacer(modifier = Modifier.height(4.dp))
                            
                            Text(
                                text = "SCMessenger is built with privacy at its core. All messages are end-to-end encrypted. These settings provide additional privacy protections.",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onPrimaryContainer
                            )
                        }
                    }
                }
                
                // Onion Routing
                SettingsSection(title = "Onion Routing") {
                    SwitchSetting(
                        title = "Enable Onion Routing",
                        description = "Route messages through multiple hops to obscure sender and receiver",
                        checked = currentSettings.onionRouting,
                        onCheckedChange = {
                            viewModel.updateSettings(currentSettings.copy(onionRouting = it))
                        },
                        enabled = !isSaving
                    )

                    InfoCard(
                        message = "Onion routing provides anonymity by routing your messages through multiple peers before reaching the destination. This makes it harder to trace who is communicating with whom."
                    )
                }

                // BLE Identity Rotation (mirrors iOS PrivacySettingsView)
                val bleRotationEnabled by viewModel.bleRotationEnabled.collectAsState()
                val bleRotationIntervalSec by viewModel.bleRotationIntervalSec.collectAsState()

                SettingsSection(title = "BLE Identity Rotation") {
                    SwitchSetting(
                        title = "Rotate BLE Identity",
                        description = "Periodically rotate the BLE advertising identity to prevent tracking",
                        checked = bleRotationEnabled,
                        onCheckedChange = { viewModel.setBleRotationEnabled(it) },
                        enabled = !isSaving
                    )

                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 16.dp, vertical = 12.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            text = "Rotation Interval",
                            style = MaterialTheme.typography.bodyLarge,
                            fontWeight = FontWeight.Medium
                        )
                        Text(
                            text = "${bleRotationIntervalSec / 60} min",
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }

                    InfoCard(
                        message = "BLE identity rotation changes your device's Bluetooth advertising data periodically, making it harder for third parties to track your device over time."
                    )
                }
                
                // Future Privacy Features (Placeholders)
                SettingsSection(title = "Additional Privacy Features") {
                    FeaturePlaceholder(
                        title = "Cover Traffic",
                        description = "Send dummy traffic to resist traffic analysis (Coming Soon)"
                    )
                    
                    FeaturePlaceholder(
                        title = "Message Padding",
                        description = "Pad messages to hide actual message length (Coming Soon)"
                    )
                    
                    FeaturePlaceholder(
                        title = "Timing Obfuscation",
                        description = "Add random delays to obscure communication patterns (Coming Soon)"
                    )
                }
            }
            
            // Best Practices
            SettingsSection(title = "Privacy Best Practices") {
                InfoCard(
                    message = """
                        ✓ Enable onion routing for maximum anonymity
                        ✓ Use unique identities for different contexts
                        ✓ Be aware that metadata (timing, message size) can still leak information
                        ✓ Consider physical security of your device
                        ✓ SCMessenger does not collect any user data or analytics
                    """.trimIndent()
                )
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
private fun FeaturePlaceholder(
    title: String,
    description: String,
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
                fontWeight = FontWeight.Medium,
                color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.6f)
            )
            Text(
                text = description,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
        }
        
        Switch(
            checked = false,
            onCheckedChange = {},
            enabled = false
        )
    }
}

@Composable
private fun InfoCard(
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
        Text(
            text = message,
            modifier = Modifier.padding(16.dp),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSecondaryContainer
        )
    }
}
