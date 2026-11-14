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
import com.scmessenger.android.utils.CircuitBreaker
import com.scmessenger.android.transport.FallbackTransport

/**
 * P0_ANDROID_007 / P0_NETWORK_001 Phase 7: User-facing network diagnostics dialog.
 *
 * Shows connectivity test results, relay status, transport priority, circuit breaker
 * states, port probe results, and actionable recommendations when the user taps the
 * network status indicator.
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
            // Silently fail -- diagnostics should never crash the UI
        }
    }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(stringResource(R.string.network_diagnostics_title)) },
        text = {
            if (report == null) {
                Text(stringResource(R.string.network_diagnostics_running))
            } else {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .verticalScroll(rememberScrollState()),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    // Network Type
                    DiagnosticRow(
                        label = stringResource(R.string.network_diagnostics_label_network),
                        value = formatNetworkType(report!!.networkType),
                        isGood = report!!.networkType in listOf(
                            com.scmessenger.android.transport.NetworkType.WIFI,
                            com.scmessenger.android.transport.NetworkType.ETHERNET,
                            com.scmessenger.android.transport.NetworkType.VPN,
                            com.scmessenger.android.transport.NetworkType.CELLULAR
                        )
                    )

                    // Internet
                    val internetConnectedLabel = stringResource(R.string.network_diagnostics_status_connected)
                    val internetDisconnectedLabel = stringResource(R.string.network_diagnostics_status_disconnected)
                    DiagnosticRow(
                        label = stringResource(R.string.network_diagnostics_label_internet),
                        value = if (report!!.hasInternet) internetConnectedLabel else internetDisconnectedLabel,
                        isGood = report!!.hasInternet
                    )

                    // P0_NETWORK_001 Phase 7: Transport Priority
                    if (report!!.transportPriority.isNotEmpty()) {
                        HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                        Text(stringResource(R.string.network_diagnostics_section_priority), style = MaterialTheme.typography.labelMedium)
                        val transportStr = report!!.transportPriority.joinToString(" > ") {
                            formatTransport(it)
                        }
                        Text(transportStr, style = MaterialTheme.typography.bodySmall)
                    }

                    // P0_NETWORK_001 Phase 7: Port Probe Results
                    if (report!!.portProbeResults.isNotEmpty()) {
                        HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                        Text(stringResource(R.string.network_diagnostics_section_port_probe), style = MaterialTheme.typography.labelMedium)
                        val openLabel = stringResource(R.string.network_diagnostics_status_open)
                        val blockedLabel = stringResource(R.string.network_diagnostics_status_blocked)
                        report!!.portProbeResults.forEach { (hostPort, reachable) ->
                            val label = if (reachable) openLabel else blockedLabel
                            val isGood = reachable
                            DiagnosticRow(
                                label = hostPort,
                                value = label,
                                isGood = isGood
                            )
                        }
                    }

                    // P0_NETWORK_001 Phase 7: Circuit Breaker States
                    if (report!!.circuitBreakerEntries.isNotEmpty()) {
                        HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                        Text(stringResource(R.string.network_diagnostics_section_circuits), style = MaterialTheme.typography.labelMedium)
                        val okLabel = stringResource(R.string.ok)
                        val blockedLabel = stringResource(R.string.network_diagnostics_status_blocked).uppercase()
                        val probingLabel = stringResource(R.string.network_diagnostics_status_probing)
                        report!!.circuitBreakerEntries.forEach { entry ->
                            val stateLabel = when (entry.state) {
                                CircuitBreaker.CircuitState.CLOSED -> okLabel
                                CircuitBreaker.CircuitState.OPEN -> blockedLabel
                                CircuitBreaker.CircuitState.HALF_OPEN -> probingLabel
                            }
                            val isGood = entry.state == CircuitBreaker.CircuitState.CLOSED
                            DiagnosticRow(
                                label = entry.address,
                                value = stateLabel,
                                isGood = isGood
                            )
                            if (entry.lastFailureReason != null) {
                                Text(
                                    "  ${entry.lastFailureReason}",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.error
                                )
                            }
                        }
                    }

                    // DNS Results
                    val failedDns = report!!.dnsResults.filterValues { !it }.keys
                    if (failedDns.isNotEmpty()) {
                        HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                        Text(stringResource(R.string.network_diagnostics_section_dns_failures), style = MaterialTheme.typography.labelMedium)
                        failedDns.forEach { domain ->
                            Text(stringResource(R.string.network_diagnostics_item, domain), style = MaterialTheme.typography.bodySmall)
                        }
                    }

                    // Relay Results
                    val unreachableRelays = report!!.relayResults.filterValues { !it }.keys
                    if (unreachableRelays.isNotEmpty()) {
                        HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                        Text(stringResource(R.string.network_diagnostics_section_unreachable_relays), style = MaterialTheme.typography.labelMedium)
                        unreachableRelays.forEach { relay ->
                            Text(stringResource(R.string.network_diagnostics_item, relay), style = MaterialTheme.typography.bodySmall)
                        }
                    }

                    // Recommendations
                    if (report!!.recommendations.isNotEmpty()) {
                        HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                        Text(stringResource(R.string.network_diagnostics_section_recommendations), style = MaterialTheme.typography.labelMedium)
                        report!!.recommendations.forEach { rec ->
                            Text(stringResource(R.string.network_diagnostics_item, rec), style = MaterialTheme.typography.bodySmall)
                        }
                    }

                    // Full report (wired from DiagnosticsReporter.formatReportForUser)
                    HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                    Text(
                        text = diagnosticsReporter.formatReportForUser(report!!),
                        style = MaterialTheme.typography.bodySmall,
                        fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace
                    )
                }
            }
        },
        confirmButton = {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedButton(onClick = onRetryBootstrap) {
                    Text(stringResource(R.string.retry))
                }
                Button(onClick = onDismiss) {
                    Text(stringResource(R.string.ok))
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

private fun formatTransport(transport: FallbackTransport): String {
    return when (transport) {
        FallbackTransport.QUIC -> "QUIC:${transport.defaultPort}"
        FallbackTransport.TCP -> "TCP:${transport.defaultPort}"
        FallbackTransport.TCP_STANDARD -> "TCP:443"
        FallbackTransport.WEBSOCKET_WS -> "WS:${transport.defaultPort}"
        FallbackTransport.WEBSOCKET_WSS -> "WSS:${transport.defaultPort}"
    }
}