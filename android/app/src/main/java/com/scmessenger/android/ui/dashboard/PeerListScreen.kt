package com.scmessenger.android.ui.dashboard

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.ui.components.ErrorBanner
import com.scmessenger.android.ui.components.IdenticonFromPeerId
import com.scmessenger.android.ui.components.StatusIndicator
import com.scmessenger.android.ui.theme.*
import com.scmessenger.android.ui.viewmodels.DashboardViewModel
import java.text.SimpleDateFormat
import java.util.*

/**
 * Peer List screen - Display connected peers with transport info.
 * 
 * Shows all active peers in the mesh network with:
 * - Identicon and peer ID
 * - Connection status and transport type
 * - Last seen timestamp
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PeerListScreen(
    onNavigateBack: () -> Unit,
    viewModel: DashboardViewModel = hiltViewModel()
) {
    val peers by viewModel.peers.collectAsState()
    val isLoading by viewModel.isLoading.collectAsState()
    val error by viewModel.error.collectAsState()
    
    LaunchedEffect(Unit) {
        viewModel.refreshData()
    }
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Connected Peers") },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "Back")
                    }
                },
                actions = {
                    IconButton(onClick = { viewModel.refreshData() }) {
                        Icon(Icons.Default.Refresh, contentDescription = "Refresh")
                    }
                }
            )
        }
    ) { paddingValues ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
        ) {
            when {
                isLoading -> {
                    CircularProgressIndicator(
                        modifier = Modifier.align(Alignment.Center)
                    )
                }
                
                peers.isEmpty() -> {
                    Column(
                        modifier = Modifier
                            .align(Alignment.Center)
                            .padding(32.dp),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Text(
                            text = "No peers connected",
                            style = MaterialTheme.typography.titleLarge
                        )
                        
                        Spacer(modifier = Modifier.height(8.dp))
                        
                        Text(
                            text = "Start the mesh service to discover peers",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
                
                else -> {
                    Column(modifier = Modifier.fillMaxSize()) {
                        // Error banner
                        error?.let {
                            ErrorBanner(
                                message = it,
                                onDismiss = { viewModel.clearError() }
                            )
                        }
                        
                        // Peer count
                        Surface(
                            modifier = Modifier.fillMaxWidth(),
                            tonalElevation = 1.dp
                        ) {
                            Text(
                                text = "${peers.size} peer${if (peers.size != 1) "s" else ""} connected",
                                modifier = Modifier.padding(16.dp),
                                style = MaterialTheme.typography.titleSmall,
                                fontWeight = FontWeight.Bold
                            )
                        }
                        
                        // Peer list
                        LazyColumn(
                            modifier = Modifier.fillMaxSize(),
                            contentPadding = PaddingValues(16.dp),
                            verticalArrangement = Arrangement.spacedBy(12.dp)
                        ) {
                            items(peers, key = { it.peerId }) { peer ->
                                PeerCard(peer = peer)
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun PeerCard(
    peer: com.scmessenger.android.ui.viewmodels.PeerInfo,
    modifier: Modifier = Modifier
) {
    Card(
        modifier = modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            horizontalArrangement = Arrangement.spacedBy(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            // Identicon
            IdenticonFromPeerId(
                peerId = peer.peerId,
                size = 56.dp
            )
            
            // Info
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(4.dp)
            ) {
                // Peer ID
                Text(
                    text = peer.peerId.take(16) + "...",
                    style = MaterialTheme.typography.bodyMedium,
                    fontFamily = FontFamily.Monospace,
                    fontWeight = FontWeight.Medium
                )
                
                // Transport
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    TransportBadge(transport = peer.transport)
                    
                    StatusIndicator(
                        isOnline = peer.isOnline
                    )
                    Text(
                        text = if (peer.isOnline) "Online" else "Offline",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                
                // Last seen
                peer.lastSeen?.let {
                    Text(
                        text = "Last seen: ${formatTimestamp(it)}",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }
    }
}

@Composable
private fun TransportBadge(
    transport: String,
    modifier: Modifier = Modifier
) {
    val color = when (transport) {
        "BLE" -> TransportBLE
        "WiFi Aware" -> TransportWiFiAware
        "WiFi Direct" -> TransportWiFiDirect
        "Internet" -> TransportInternet
        else -> MaterialTheme.colorScheme.surfaceVariant
    }
    
    Surface(
        modifier = modifier,
        color = color,
        shape = MaterialTheme.shapes.small
    ) {
        Text(
            text = transport,
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
            style = MaterialTheme.typography.labelSmall,
            color = if (transport == "Unknown") MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.onPrimary
        )
    }
}

private fun formatTimestamp(timestamp: ULong): String {
    val millis = timestamp.toLong() * 1000
    val date = Date(millis)
    val now = Date()
    
    val diff = (now.time - date.time) / 1000
    
    return when {
        diff < 60 -> "just now"
        diff < 3600 -> "${diff / 60}m ago"
        diff < 86400 -> "${diff / 3600}h ago"
        else -> {
            val sdf = SimpleDateFormat("MMM d", Locale.getDefault())
            sdf.format(date)
        }
    }
}
