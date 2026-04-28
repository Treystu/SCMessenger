package com.scmessenger.android.ui.screens

import android.content.Context
import android.content.Intent
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
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

    fun refreshLogs() {
        scope.launch {
            logText = viewModel.getDiagnosticsLogs(limit = 250)
        }
    }

    LaunchedEffect(Unit) {
        refreshLogs()
    }

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
                        Icon(Icons.Default.ArrowBack, contentDescription = "Back")
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
