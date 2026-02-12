package com.scmessenger.android.ui.dashboard

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.ui.components.ErrorBanner
import com.scmessenger.android.ui.theme.*
import com.scmessenger.android.ui.viewmodels.DashboardViewModel
import com.scmessenger.android.ui.viewmodels.NetworkTopology
import com.scmessenger.android.ui.viewmodels.TopologyNode
import kotlin.math.cos
import kotlin.math.sin

/**
 * Topology screen - Canvas-based network graph visualization.
 * 
 * Displays the mesh network topology as an interactive graph:
 * - Nodes represent peers (self node highlighted)
 * - Edges represent connections (colored by transport type)
 * - Layout uses circular arrangement for clarity
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TopologyScreen(
    onNavigateBack: () -> Unit,
    viewModel: DashboardViewModel = hiltViewModel()
) {
    val topology by viewModel.topology.collectAsState()
    val isLoading by viewModel.isLoading.collectAsState()
    val error by viewModel.error.collectAsState()
    
    LaunchedEffect(Unit) {
        viewModel.refreshData()
    }
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Network Topology") },
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
                
                topology.nodes.isEmpty() -> {
                    Column(
                        modifier = Modifier
                            .align(Alignment.Center)
                            .padding(32.dp),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Text(
                            text = "No topology data",
                            style = MaterialTheme.typography.titleLarge
                        )
                        
                        Spacer(modifier = Modifier.height(8.dp))
                        
                        Text(
                            text = "Start the mesh service to see network topology",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
                
                else -> {
                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .verticalScroll(rememberScrollState())
                    ) {
                        // Error banner
                        error?.let {
                            ErrorBanner(
                                message = it,
                                onDismiss = { viewModel.clearError() }
                            )
                        }
                        
                        // Stats
                        TopologyStats(topology = topology)
                        
                        // Graph visualization
                        TopologyGraph(
                            topology = topology,
                            modifier = Modifier
                                .fillMaxWidth()
                                .height(500.dp)
                                .padding(16.dp)
                        )
                        
                        // Legend
                        TopologyLegend(
                            modifier = Modifier.padding(16.dp)
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun TopologyStats(
    topology: NetworkTopology,
    modifier: Modifier = Modifier
) {
    Surface(
        modifier = modifier.fillMaxWidth(),
        tonalElevation = 1.dp
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            horizontalArrangement = Arrangement.SpaceEvenly
        ) {
            StatItem(
                label = "Nodes",
                value = topology.nodes.size.toString()
            )
            
            StatItem(
                label = "Connections",
                value = topology.edges.size.toString()
            )
            
            StatItem(
                label = "Online",
                value = topology.nodes.count { it.isOnline }.toString()
            )
        }
    }
}

@Composable
private fun StatItem(
    label: String,
    value: String,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(
            text = value,
            style = MaterialTheme.typography.headlineMedium,
            fontWeight = FontWeight.Bold,
            color = MaterialTheme.colorScheme.primary
        )
        Text(
            text = label,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

@Composable
private fun TopologyGraph(
    topology: NetworkTopology,
    modifier: Modifier = Modifier
) {
    val nodeColor = MaterialTheme.colorScheme.primary
    val selfNodeColor = MaterialTheme.colorScheme.secondary
    val offlineNodeColor = MaterialTheme.colorScheme.surfaceVariant
    
    Canvas(modifier = modifier.background(MaterialTheme.colorScheme.surface)) {
        val centerX = size.width / 2
        val centerY = size.height / 2
        val radius = minOf(size.width, size.height) / 2 - 60f
        
        // Calculate node positions in a circle
        val nodePositions = mutableMapOf<String, Offset>()
        
        topology.nodes.forEachIndexed { index, node ->
            if (node.isSelf) {
                // Place self node in center
                nodePositions[node.id] = Offset(centerX, centerY)
            } else {
                // Place other nodes in a circle
                val otherCount = topology.nodes.size - 1
                if (otherCount > 0) {
                    val angle = (2 * Math.PI * index) / otherCount
                    val x = centerX + (radius * cos(angle)).toFloat()
                    val y = centerY + (radius * sin(angle)).toFloat()
                    nodePositions[node.id] = Offset(x, y)
                }
            }
        }
        
        // Draw edges first (behind nodes)
        topology.edges.forEach { edge ->
            val source = nodePositions[edge.source]
            val target = nodePositions[edge.target]
            
            if (source != null && target != null) {
                val edgeColor = getTransportColor(edge.transport)
                
                drawLine(
                    color = edgeColor,
                    start = source,
                    end = target,
                    strokeWidth = 3f,
                    cap = StrokeCap.Round
                )
            }
        }
        
        // Draw nodes on top
        topology.nodes.forEach { node ->
            val position = nodePositions[node.id] ?: return@forEach
            val color = when {
                node.isSelf -> selfNodeColor
                !node.isOnline -> offlineNodeColor
                else -> nodeColor
            }
            
            // Outer circle (highlight for self)
            if (node.isSelf) {
                drawCircle(
                    color = color.copy(alpha = 0.3f),
                    radius = 35f,
                    center = position
                )
            }
            
            // Main node circle
            drawCircle(
                color = color,
                radius = 25f,
                center = position,
                style = Stroke(width = 4f)
            )
            
            // Inner fill
            drawCircle(
                color = color.copy(alpha = 0.2f),
                radius = 21f,
                center = position
            )
        }
    }
}

@Composable
private fun TopologyLegend(
    modifier: Modifier = Modifier
) {
    Card(modifier = modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(
                text = "Legend",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold
            )
            
            // Node types
            LegendItem(
                color = MaterialTheme.colorScheme.secondary,
                label = "This Device"
            )
            LegendItem(
                color = MaterialTheme.colorScheme.primary,
                label = "Connected Peer"
            )
            LegendItem(
                color = MaterialTheme.colorScheme.surfaceVariant,
                label = "Offline Peer"
            )
            
            Divider()
            
            // Transport types
            Text(
                text = "Connection Types",
                style = MaterialTheme.typography.titleSmall
            )
            
            LegendItem(color = TransportBLE, label = "BLE")
            LegendItem(color = TransportWiFiAware, label = "WiFi Aware")
            LegendItem(color = TransportWiFiDirect, label = "WiFi Direct")
            LegendItem(color = TransportInternet, label = "Internet")
        }
    }
}

@Composable
private fun LegendItem(
    color: Color,
    label: String,
    modifier: Modifier = Modifier
) {
    Row(
        modifier = modifier,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            modifier = Modifier
                .size(20.dp)
                .background(color, shape = MaterialTheme.shapes.small)
        )
        
        Text(
            text = label,
            style = MaterialTheme.typography.bodyMedium
        )
    }
}

/**
 * Get color for transport type.
 */
private fun getTransportColor(transport: String): Color {
    return when (transport) {
        "BLE" -> TransportBLE
        "WiFi Aware" -> TransportWiFiAware
        "WiFi Direct" -> TransportWiFiDirect
        "Internet" -> TransportInternet
        else -> Color.Gray
    }
}
