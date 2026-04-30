package com.scmessenger.android.ui.dialogs

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.scmessenger.android.R
import com.scmessenger.android.network.DiagnosticsReporter
import com.scmessenger.android.network.DiagnosticsReporter.NetworkDiagnosticsReport

/**
 * P0_ANDROID_007: User-facing network diagnostics dialog.
 *
 * Shows connectivity test results, relay status, and actionable
 * recommendations when the user taps the network status indicator.
 */
@Composable
fun NetworkStatusDialog(
    diagnosticsReporter: DiagnosticsReporter,
    onDismiss: () -> Unit,
    onRetryBootstrap: () -> Unit = {}
) {
    var report by remember { mutableStateOf<NetworkDiagnosticsReport?>(null) }

    LaunchedEffect(Unit) {
        try {
            report = diagnosticsReporter.generateReport()
        } catch (e: Exception) {
            // Silently fail — diagnostics should never crash the UI
        }
    }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Network Diagnostics") },
        text = {
            if (report == null) {
                Text("Running diagnostics...")
            } else {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .verticalScroll(rememberScrollState()),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    // Network Type
                    DiagnosticRow(
                        label = "Network",
                        value = formatNetworkType(report!!.networkType),
                        isGood = report!!.networkType in listOf(
                            com.scmessenger.android.transport.NetworkType.WIFI,
                            com.scmessenger.android.transport.NetworkType.ETHERNET,
                            com.scmessenger.android.transport.NetworkType.VPN,
                            com.scmessenger.android.transport.NetworkType.CELLULAR
                        )
                    )

                    // Internet
                    DiagnosticRow(
                        label = "Internet",
                        value = if (report!!.hasInternet) "Connected" else "Disconnected",
                        isGood = report!!.hasInternet
                    )

                    // DNS Results
                    val failedDns = report!!.dnsResults.filterValues { !it }.keys
                    if (failedDns.isNotEmpty()) {
                        HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                        Text("DNS Failures:", style = MaterialTheme.typography.labelMedium)
                        failedDns.forEach { domain ->
                            Text("  - $domain", style = MaterialTheme.typography.bodySmall)
                        }
                    }

                    // Relay Results
                    val unreachableRelays = report!!.relayResults.filterValues { !it }.keys
                    if (unreachableRelays.isNotEmpty()) {
                        HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                        Text("Unreachable Relays:", style = MaterialTheme.typography.labelMedium)
                        unreachableRelays.forEach { relay ->
                            Text("  - $relay", style = MaterialTheme.typography.bodySmall)
                        }
                    }

                    // Recommendations
                    if (report!!.recommendations.isNotEmpty()) {
                        HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                        Text("Recommendations:", style = MaterialTheme.typography.labelMedium)
                        report!!.recommendations.forEach { rec ->
                            Text("  - $rec", style = MaterialTheme.typography.bodySmall)
                        }
                    }
                }
            }
        },
        confirmButton = {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedButton(onClick = onRetryBootstrap) {
                    Text("Retry")
                }
                Button(onClick = onDismiss) {
                    Text("OK")
                }
            }
        }
    )
}

@Composable
private fun DiagnosticRow(label: String, value: String, isGood: Boolean) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.fillMaxWidth()
    ) {
        Text(
            text = if (isGood) "OK" else "!!",
            color = if (isGood) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.error,
            style = MaterialTheme.typography.labelSmall,
            modifier = Modifier.size(24.dp)
        )
        Spacer(modifier = Modifier.width(8.dp))
        Text(label, style = MaterialTheme.typography.bodyMedium, modifier = Modifier.weight(1f))
        Text(value, style = MaterialTheme.typography.bodyMedium)
    }
}

@Composable
private fun formatNetworkType(type: com.scmessenger.android.transport.NetworkType): String = when (type) {
    com.scmessenger.android.transport.NetworkType.WIFI -> "Wi-Fi"
    com.scmessenger.android.transport.NetworkType.WIFI_RESTRICTED -> "Wi-Fi (Restricted)"
    com.scmessenger.android.transport.NetworkType.CELLULAR -> "Cellular"
    com.scmessenger.android.transport.NetworkType.CELLULAR_RESTRICTED -> "Cellular (Restricted)"
    com.scmessenger.android.transport.NetworkType.CELLULAR_NO_INTERNET -> "Cellular (No Internet)"
    com.scmessenger.android.transport.NetworkType.ETHERNET -> "Ethernet"
    com.scmessenger.android.transport.NetworkType.VPN -> "VPN"
    com.scmessenger.android.transport.NetworkType.BLUETOOTH -> "Bluetooth"
    com.scmessenger.android.transport.NetworkType.UNKNOWN -> stringResource(R.string.unknown_network_type)
}
