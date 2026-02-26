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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.core.content.FileProvider
import java.io.File

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DiagnosticsScreen(
    onNavigateBack: () -> Unit
) {
    val context = LocalContext.current
    var logText by remember { mutableStateOf("Loading logs...") }

    fun refreshLogs() {
        val logFile = File(context.filesDir, "mesh_diagnostics.log")
        if (logFile.exists()) {
            val lines = logFile.readLines().takeLast(100)
            logText = if (lines.isEmpty()) "Log file empty." else lines.joinToString("\n")
        } else {
            logText = "Log file not found."
        }
    }

    LaunchedEffect(Unit) {
        refreshLogs()
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
                        val logFile = File(context.filesDir, "mesh_diagnostics.log")
                        if (logFile.exists()) {
                            logFile.writeText("")
                            refreshLogs()
                        }
                    }) {
                        Icon(imageVector = Icons.Default.Delete, contentDescription = "Clear")
                    }
                    IconButton(onClick = { shareLogFile(context) }) {
                        Icon(Icons.Default.Share, contentDescription = "Share")
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
            Text(
                text = logText,
                style = MaterialTheme.typography.bodySmall,
                fontFamily = FontFamily.Monospace
            )
        }
    }
}

private fun shareLogFile(context: Context) {
    val logFile = File(context.filesDir, "mesh_diagnostics.log")
    if (!logFile.exists()) return

    val uri = FileProvider.getUriForFile(
        context,
        "${context.packageName}.fileprovider",
        logFile
    )

    val intent = Intent(Intent.ACTION_SEND).apply {
        type = "text/plain"
        putExtra(Intent.EXTRA_STREAM, uri)
        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
    }
    context.startActivity(Intent.createChooser(intent, "Share Logs"))
}
