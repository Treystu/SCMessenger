package com.scmessenger.android.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Bluetooth
import androidx.compose.material.icons.filled.Bolt
import androidx.compose.material.icons.filled.NetworkWifi
import androidx.compose.material.icons.filled.People
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Router
import androidx.compose.material.icons.filled.Wifi
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.ui.viewmodels.MeshServiceViewModel
import com.scmessenger.android.ui.viewmodels.DashboardViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DashboardScreen(
    serviceViewModel: MeshServiceViewModel = hiltViewModel(),
    dashboardViewModel: DashboardViewModel = hiltViewModel()
) {
    val serviceState by serviceViewModel.serviceState.collectAsState()
    val isRunning by serviceViewModel.isRunning.collectAsState()
    val stats by serviceViewModel.serviceStats.collectAsState()
    
    val fullPeers by dashboardViewModel.fullPeersCount.collectAsState()
    val headlessPeers by dashboardViewModel.headlessPeersCount.collectAsState()
    val totalPeers by dashboardViewModel.totalPeersCount.collectAsState()

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Mesh") }
            )
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            // Service Status Card
            StatusCard(
                isRunning = isRunning,
                stateName = serviceState.name
            )

            // Quick Stats Grid
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                StatCard(
                    modifier = Modifier.weight(1.5f),
                    title = "Nodes ($fullPeers Full / $headlessPeers Headless)",
                    value = totalPeers.toString(),
                    icon = Icons.Filled.People,
                    color = MaterialTheme.colorScheme.primary
                )
                StatCard(
                    modifier = Modifier.weight(1f),
                    title = "Relayed",
                    value = stats?.messagesRelayed?.toString() ?: "0",
                    icon = Icons.Filled.Router,
                    color = MaterialTheme.colorScheme.tertiary
                )
            }

            // Connection Methods Status
            ConnectionStatusCard()

            // Detailed Stats
            Card(
                modifier = Modifier.fillMaxWidth(),
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
                )
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text(
                        text = "Mesh Performance",
                        style = MaterialTheme.typography.titleMedium,
                        modifier = Modifier.padding(bottom = 8.dp)
                    )

                    TextDetailRow("Uptime", formatDuration(stats?.uptimeSecs ?: 0uL))
                    TextDetailRow("Data Transferred", formatBytes(stats?.bytesTransferred ?: 0uL))
                }
            }

            // Discovered Nodes List
            Text(
                text = "Discovered Nodes",
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(top = 8.dp)
            )

            val peers by dashboardViewModel.peers.collectAsState()
            if (peers.isEmpty()) {
                Text(
                    text = "No nodes discovered yet",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            } else {
                peers.forEach { peer ->
                    PeerItem(peer)
                    Divider(modifier = Modifier.padding(vertical = 4.dp), color = MaterialTheme.colorScheme.surfaceVariant)
                }
            }
        }
    }
}

@Composable
fun PeerItem(peer: com.scmessenger.android.ui.viewmodels.PeerInfo) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            modifier = Modifier
                .size(40.dp)
                .background(
                    if (peer.isOnline) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceVariant,
                    CircleShape
                ),
            contentAlignment = Alignment.Center
        ) {
            Icon(
                when {
                    peer.isRelay -> Icons.Filled.Router
                    peer.isFull -> Icons.Filled.Person
                    else -> Icons.Filled.People
                },
                contentDescription = null,
                tint = if (peer.isOnline) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onSurfaceVariant
            )
        }

        Spacer(modifier = Modifier.width(12.dp))

        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = peer.localNickname
                    ?: peer.nickname
                    ?: if (peer.isRelay) "Headless Relay" else if (peer.isFull) "Full Node" else "Headless Node",
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Bold
            )
            if (peer.nickname != null && peer.localNickname != null) {
                Text(
                    text = "@${peer.nickname}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            Text(
                text = buildString {
                    append("ID: ")
                    append(peer.peerId.take(12))
                    append("... • ")
                    append(peer.transport)
                    append(" • ")
                    append(
                        when {
                            peer.isRelay -> "Relay"
                            peer.isFull -> "Full"
                            else -> "Headless"
                        }
                    )
                },
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }

        if (peer.isOnline) {
            Box(
                modifier = Modifier
                    .size(8.dp)
                    .background(Color.Green, CircleShape)
            )
        }
    }
}

@Composable
fun StatusCard(
    isRunning: Boolean,
    stateName: String
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = if (isRunning) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceVariant
        )
    ) {
        Row(
            modifier = Modifier
                .padding(24.dp)
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Column {
                Text(
                    text = if (isRunning) "Mesh Active" else "Mesh Stopped",
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold,
                    color = if (isRunning) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onSurfaceVariant
                )
                Text(
                    text = "State: $stateName",
                    style = MaterialTheme.typography.bodyMedium,
                    color = if (isRunning) MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.8f) else MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
    }
}

@Composable
fun StatCard(
    modifier: Modifier = Modifier,
    title: String,
    value: String,
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    color: Color
) {
    Card(
        modifier = modifier
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            horizontalAlignment = Alignment.Start
        ) {
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .background(color.copy(alpha = 0.2f), CircleShape),
                contentAlignment = Alignment.Center
            ) {
                Icon(icon, contentDescription = null, tint = color)
            }
            Spacer(modifier = Modifier.height(12.dp))
            Text(
                text = value,
                style = MaterialTheme.typography.headlineMedium,
                fontWeight = FontWeight.Bold
            )
            Text(
                text = title,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

@Composable
fun ConnectionStatusCard() {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = "Transports",
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 12.dp)
            )

            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                TransportItem("BLE", Icons.Filled.Bluetooth, true) // Ideally get from ViewModel
                TransportItem("WiFi Aware", Icons.Filled.Wifi, true)
                TransportItem("WiFi Direct", Icons.Filled.Router, true)
            }
        }
    }
}

@Composable
fun TransportItem(name: String, icon: androidx.compose.ui.graphics.vector.ImageVector, enabled: Boolean) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = if (enabled) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outline
        )
        Spacer(modifier = Modifier.height(4.dp))
        Text(text = name, style = MaterialTheme.typography.labelSmall)
    }
}

@Composable
fun TextDetailRow(label: String, value: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(text = label, style = MaterialTheme.typography.bodyMedium)
        Text(text = value, style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.SemiBold)
    }
}

private fun formatBytes(bytes: ULong): String {
    return when {
        bytes < 1024u -> "$bytes B"
        bytes < 1024u * 1024u -> "${bytes / 1024u} KB"
        bytes < 1024u * 1024u * 1024u -> "${bytes / (1024u * 1024u)} MB"
        else -> "${bytes / (1024u * 1024u * 1024u)} GB"
    }
}

private fun formatDuration(seconds: ULong): String {
    val secs = seconds.toLong()
    val hours = secs / 3600
    val minutes = (secs % 3600) / 60
    return "${hours}h ${minutes}m"
}
