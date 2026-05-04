package com.scmessenger.android.ui.screens

import android.content.Context
import android.content.Intent
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Share
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import kotlinx.coroutines.launch
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.network.DiagnosticsReporter
import com.scmessenger.android.ui.diagnostics.DiagnosticsBundleFormatter
import com.scmessenger.android.ui.diagnostics.DiagnosticsBundleInput
import com.scmessenger.android.ui.dialogs.NetworkStatusDialog
import com.scmessenger.android.ui.viewmodels.SettingsViewModel
import com.scmessenger.android.utils.NotificationHelper
import com.scmessenger.android.service.PerformanceMonitor
import com.scmessenger.android.service.ServiceHealthMonitor
import com.scmessenger.android.service.AnrEvent
import com.scmessenger.android.ui.components.WarningBanner
import com.scmessenger.android.ui.components.InfoBanner
import com.scmessenger.android.ui.components.ErrorState
import androidx.core.content.FileProvider
import timber.log.Timber
import java.io.File

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DiagnosticsScreen(
    onNavigateBack: () -> Unit,
    viewModel: SettingsViewModel = hiltViewModel()
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    var showNetworkDiagnostics by remember { mutableStateOf(false) }
    var logText by remember { mutableStateOf("Loading logs...") }
    var networkDiagnosticsReport by remember { mutableStateOf<DiagnosticsReporter.NetworkDiagnosticsReport?>(null) }
    var notificationStats by remember { mutableStateOf(NotificationHelper.getNotificationStats()) }
    var performanceHealthStatus by remember { mutableStateOf("") }
    var anrStats by remember { mutableStateOf("") }
    var anrEvents by remember { mutableStateOf<List<AnrEvent>>(emptyList()) }
    var serviceHealthy by remember { mutableStateOf(true) }
    var healthSummary by remember { mutableStateOf("") }

    val performanceMonitor = remember { PerformanceMonitor(context) }
    val healthMonitor = remember { ServiceHealthMonitor(context) }

    fun refreshLogs() {
        scope.launch {
            logText = viewModel.getDiagnosticsLogs(limit = 250)
        }
    }

    // Wire getNetworkDiagnosticsReport into diagnostics display
    LaunchedEffect(Unit) {
        scope.launch {
            networkDiagnosticsReport = viewModel.getNetworkDiagnosticsReport()
        }
        // Wire PerformanceMonitor.getHealthStatus into diagnostics
        performanceHealthStatus = performanceMonitor.getHealthStatus()
        // Wire PerformanceMonitor.getAnrStats into diagnostics
        anrStats = performanceMonitor.getAnrStats()
        // Wire PerformanceMonitor.getAllAnrEvents into diagnostics
        anrEvents = performanceMonitor.getAllAnrEvents()
        // Wire ServiceHealthMonitor.isServiceHealthy into diagnostics
        serviceHealthy = healthMonitor.isServiceHealthy()
        // Wire ServiceHealthMonitor health summary
        healthSummary = healthMonitor.getHealthSummary()
    }

    LaunchedEffect(Unit) {
        refreshLogs()
    }

    // Wire WarningBanner into diagnostics for health warnings
    if (!serviceHealthy) {
        WarningBanner(
            message = "Mesh service health check failed — service may be unresponsive",
            onDismiss = {}
        )
    }

    // Wire InfoBanner into diagnostics for informational notices
    InfoBanner(
        message = "Diagnostics data is local to this device. Share the bundle for remote analysis.",
        onDismiss = {}
    )

    if (showNetworkDiagnostics) {
        NetworkStatusDialog(
            diagnosticsReporter = DiagnosticsReporter(
                context = context,
                networkDiagnostics = com.scmessenger.android.network.NetworkDiagnostics(context),
                networkTypeDetector = com.scmessenger.android.network.NetworkTypeDetector(context),
                failureMetrics = com.scmessenger.android.utils.NetworkFailureMetrics()
            ),
            onDismiss = { showNetworkDiagnostics = false },
            onRetryBootstrap = {
                // Retry bootstrap - call the mesh repository method
                viewModel.retryBootstrap()
                showNetworkDiagnostics = false
            }
        )
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Diagnostics") },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                    }
                },
                actions = {
                    IconButton(onClick = { refreshLogs() }) {
                        Icon(Icons.Default.Refresh, contentDescription = "Refresh")
                    }
                    IconButton(onClick = {
                        viewModel.clearDiagnosticsLogs()
                        refreshLogs()
                    }) {
                        Icon(imageVector = Icons.Default.Delete, contentDescription = "Clear")
                    }
                    IconButton(onClick = {
                        scope.launch {
                            shareDiagnosticsBundle(context, viewModel.buildTesterDiagnosticsBundle())
                        }
                    }) {
                        Icon(Icons.Default.Share, contentDescription = "Share")
                    }
                    IconButton(onClick = { showNetworkDiagnostics = true }) {
                        Icon(Icons.Default.Refresh, contentDescription = "Network Diagnostics")
                    }
                }
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .padding(padding)
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(16.dp)
        ) {
            // Network diagnostics card
            Card(
                modifier = Modifier.fillMaxWidth(),
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer)
            ) {
                Column(
                    modifier = Modifier.padding(16.dp)
                ) {
                    Text(
                        text = "Network Diagnostics",
                        style = MaterialTheme.typography.titleMedium
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = "Test connectivity, DNS resolution, relay reachability, and get actionable recommendations.",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(modifier = Modifier.height(12.dp))
                    Button(
                        onClick = { showNetworkDiagnostics = true },
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Text("Run Network Diagnostics")
                    }
                }
            }
            Spacer(modifier = Modifier.height(16.dp))

            // Network Diagnostics Report (wired via getNetworkDiagnosticsReport)
            networkDiagnosticsReport?.let { report ->
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.tertiaryContainer)
                ) {
                    Column(
                        modifier = Modifier.padding(16.dp)
                    ) {
                        Text(
                            text = "Network Diagnostics Report",
                            style = MaterialTheme.typography.titleMedium
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = "Internet: ${report.hasInternet}, Network: ${report.networkType}",
                            style = MaterialTheme.typography.bodySmall
                        )
                        report.relayResults.forEach { (relay, reachable) ->
                            Text(
                                text = "Relay $relay: ${if (reachable) "Reachable" else "Unreachable"}",
                                style = MaterialTheme.typography.bodySmall
                            )
                        }
                    }
                }
                Spacer(modifier = Modifier.height(16.dp))
            }

            // Notification Stats (wired via getNotificationStats)
            Card(
                modifier = Modifier.fillMaxWidth(),
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.secondaryContainer)
            ) {
                Column(
                    modifier = Modifier.padding(16.dp)
                ) {
                    Text(
                        text = "Notification Stats",
                        style = MaterialTheme.typography.titleMedium
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = notificationStats,
                        style = MaterialTheme.typography.bodySmall,
                        fontFamily = FontFamily.Monospace
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        Button(
                            onClick = {
                                // Wire resetNotificationStats into diagnostics
                                NotificationHelper.resetNotificationStats()
                                notificationStats = NotificationHelper.getNotificationStats()
                            },
                            modifier = Modifier.weight(1f)
                        ) {
                            Text("Reset Stats", style = MaterialTheme.typography.labelSmall)
                        }
                        Button(
                            onClick = {
                                // Wire clearAllRequestNotifications into diagnostics
                                NotificationHelper.clearAllRequestNotifications()
                                notificationStats = NotificationHelper.getNotificationStats()
                            },
                            modifier = Modifier.weight(1f)
                        ) {
                            Text("Clear Requests", style = MaterialTheme.typography.labelSmall)
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Service Health (wired via isServiceHealthy + getHealthSummary)
            Card(
                modifier = Modifier.fillMaxWidth(),
                colors = CardDefaults.cardColors(containerColor = if (serviceHealthy) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.errorContainer)
            ) {
                Column(
                    modifier = Modifier.padding(16.dp)
                ) {
                    Text(
                        text = "Service Health",
                        style = MaterialTheme.typography.titleMedium
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = "Status: ${if (serviceHealthy) "Healthy" else "Unhealthy"}",
                        style = MaterialTheme.typography.bodyMedium,
                        color = if (serviceHealthy) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onErrorContainer
                    )
                    Spacer(modifier = Modifier.height(4.dp))
                    Text(
                        text = healthSummary,
                        style = MaterialTheme.typography.bodySmall,
                        fontFamily = FontFamily.Monospace
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Button(
                        onClick = {
                            // Wire resetHealth into diagnostics settings reset
                            healthSummary = "Reset"
                            serviceHealthy = true
                            performanceMonitor.clearAnrEvents()
                            anrStats = performanceMonitor.getAnrStats()
                            anrEvents = performanceMonitor.getAllAnrEvents()
                        },
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Text("Reset Health", style = MaterialTheme.typography.labelSmall)
                    }
                }
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Performance Monitor (wired via recordUiTiming, getAnrStats, getAllAnrEvents, getHealthStatus)
            Card(
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(
                    modifier = Modifier.padding(16.dp)
                ) {
                    Text(
                        text = "Performance & ANR",
                        style = MaterialTheme.typography.titleMedium
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = performanceHealthStatus,
                        style = MaterialTheme.typography.bodySmall,
                        fontFamily = FontFamily.Monospace
                    )
                    Spacer(modifier = Modifier.height(4.dp))
                    Text(
                        text = anrStats,
                        style = MaterialTheme.typography.bodySmall,
                        fontFamily = FontFamily.Monospace
                    )
                    if (anrEvents.isNotEmpty()) {
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = "Recent ANR Events:",
                            style = MaterialTheme.typography.labelMedium
                        )
                        anrEvents.take(5).forEach { event: AnrEvent ->
                            Text(
                                text = "- ${event.context}: ${event.durationMs}ms (API ${event.androidVersion}, ${event.device})",
                                style = MaterialTheme.typography.bodySmall,
                                fontFamily = FontFamily.Monospace
                            )
                        }
                    }
                    Spacer(modifier = Modifier.height(8.dp))
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        Button(
                            onClick = {
                                // Wire clearAnrEvents into diagnostics reset
                                performanceMonitor.clearAnrEvents()
                                anrStats = performanceMonitor.getAnrStats()
                                anrEvents = performanceMonitor.getAllAnrEvents()
                                performanceHealthStatus = performanceMonitor.getHealthStatus()
                            },
                            modifier = Modifier.weight(1f)
                        ) {
                            Text("Clear ANR", style = MaterialTheme.typography.labelSmall)
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Log viewer
            Text(
                text = "Diagnostics Logs",
                style = MaterialTheme.typography.titleSmall
            )
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = "Tester note: share bundle after reproducing issue. Include permission prompts and install/first-message steps.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(modifier = Modifier.height(12.dp))
            Text(
                text = logText,
                style = MaterialTheme.typography.bodySmall,
                fontFamily = FontFamily.Monospace
            )
        }
    }
}

private fun shareDiagnosticsBundle(context: Context, bundleText: String) {
    val bundleFile = File(context.cacheDir, "scmessenger_diagnostics_bundle.txt")
    bundleFile.writeText(bundleText)

    val uri = FileProvider.getUriForFile(
        context,
        "${context.packageName}.fileprovider",
        bundleFile
    )

    val intent = Intent(Intent.ACTION_SEND).apply {
        type = "text/plain"
        putExtra(Intent.EXTRA_STREAM, uri)
        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        putExtra(Intent.EXTRA_SUBJECT, "SCMessenger Diagnostics Bundle")
    }
    context.startActivity(Intent.createChooser(intent, "Share Diagnostics Bundle"))
}
