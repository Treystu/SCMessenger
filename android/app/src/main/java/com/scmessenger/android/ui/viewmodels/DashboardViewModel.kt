package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.service.MeshEventBus
import com.scmessenger.android.service.PeerEvent
import com.scmessenger.android.service.StatusEvent
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import timber.log.Timber
import uniffi.api.*
import javax.inject.Inject

/**
 * ViewModel for the dashboard screen.
 * 
 * Provides service statistics, peer list, mesh topology data,
 * and real-time network health metrics.
 */
@HiltViewModel
class DashboardViewModel @Inject constructor(
    private val meshRepository: MeshRepository
) : ViewModel() {
    
    // Service stats
    private val _stats = MutableStateFlow<uniffi.api.ServiceStats?>(null)
    val stats: StateFlow<uniffi.api.ServiceStats?> = _stats.asStateFlow()
    
    // Active peers
    private val _peers = MutableStateFlow<List<PeerInfo>>(emptyList())
    val peers: StateFlow<List<PeerInfo>> = _peers.asStateFlow()
    
    // Network topology data (for graph visualization)
    private val _topology = MutableStateFlow<NetworkTopology>(NetworkTopology())
    val topology: StateFlow<NetworkTopology> = _topology.asStateFlow()
    
    // Loading state
    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()
    
    // Error state
    private val _error = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = _error.asStateFlow()
    
    init {
        observeNetworkEvents()
        refreshData()
    }
    
    /**
     * Refresh all dashboard data.
     */
    fun refreshData() {
        viewModelScope.launch {
            try {
                _isLoading.value = true
                _error.value = null
                
                // Get service stats
                _stats.value = meshRepository.serviceStats.value
                
                // Get peer information
                loadPeers()
                
                // Build topology
                buildTopology()
                
                Timber.d("Dashboard data refreshed")
            } catch (e: Exception) {
                _error.value = "Failed to refresh data: ${e.message}"
                Timber.e(e, "Failed to refresh dashboard data")
            } finally {
                _isLoading.value = false
            }
        }
    }
    
    /**
     * Load active peers from ledger.
     */
    private fun loadPeers() {
        try {
            val ledgerEntries = meshRepository.getDialableAddresses()
            val peerList = ledgerEntries.map { entry ->
                PeerInfo(
                    peerId = entry.peerId,
                    multiaddr = entry.multiaddr,
                    lastSeen = entry.lastSuccessTime,
                    transport = determineTransport(entry.multiaddr),
                    isOnline = isRecent(entry.lastSuccessTime)
                )
            }
            _peers.value = peerList
            
            Timber.d("Loaded ${peerList.size} peers")
        } catch (e: Exception) {
            Timber.e(e, "Failed to load peers")
        }
    }
    
    /**
     * Build network topology from ledger and stats.
     */
    private fun buildTopology() {
        try {
            val nodes = mutableListOf<TopologyNode>()
            val edges = mutableListOf<TopologyEdge>()
            
            // Add self node
            val identityInfo = meshRepository.getIdentityInfo()
            if (identityInfo != null) {
                nodes.add(
                    TopologyNode(
                        id = identityInfo.peerId,
                        isSelf = true,
                        isOnline = true
                    )
                )
            }
            
            // Add peer nodes and edges
            _peers.value.forEach { peer ->
                nodes.add(
                    TopologyNode(
                        id = peer.peerId,
                        isSelf = false,
                        isOnline = peer.isOnline
                    )
                )
                
                // Add edge from self to peer
                identityInfo?.let {
                    edges.add(
                        TopologyEdge(
                            source = it.peerId,
                            target = peer.peerId,
                            transport = peer.transport
                        )
                    )
                }
            }
            
            _topology.value = NetworkTopology(nodes, edges)
            
            Timber.d("Topology built: ${nodes.size} nodes, ${edges.size} edges")
        } catch (e: Exception) {
            Timber.e(e, "Failed to build topology")
        }
    }
    
    /**
     * Observe network events for real-time updates.
     */
    private fun observeNetworkEvents() {
        viewModelScope.launch {
            MeshEventBus.peerEvents.collect { event ->
                when (event) {
                    is PeerEvent.Discovered, is PeerEvent.Connected -> {
                        refreshData()
                    }
                    is PeerEvent.Disconnected -> {
                        refreshData()
                    }
                    else -> {}
                }
            }
        }
        
        viewModelScope.launch {
            MeshEventBus.statusEvents.collect { event ->
                if (event is StatusEvent.StatsUpdated) {
                    _stats.value = event.stats
                }
            }
        }
    }
    
    /**
     * Determine transport type from multiaddr.
     */
    private fun determineTransport(multiaddr: String): String {
        return when {
            "/ble/" in multiaddr -> "BLE"
            "/wifi-aware/" in multiaddr -> "WiFi Aware"
            "/wifi-direct/" in multiaddr -> "WiFi Direct"
            "/ip4/" in multiaddr || "/ip6/" in multiaddr -> "Internet"
            else -> "Unknown"
        }
    }
    
    /**
     * Check if timestamp is recent (within last 5 minutes).
     */
    private fun isRecent(timestamp: ULong?): Boolean {
        if (timestamp == null) return false
        val now = (System.currentTimeMillis() / 1000).toULong()
        val fiveMinutes = 300u
        return (now - timestamp) < fiveMinutes
    }
    
    /**
     * Clear error state.
     */
    fun clearError() {
        _error.value = null
    }
}

/**
 * Peer information for display.
 */
data class PeerInfo(
    val peerId: String,
    val multiaddr: String,
    val lastSeen: ULong?,
    val transport: String,
    val isOnline: Boolean
)

/**
 * Network topology data structure.
 */
data class NetworkTopology(
    val nodes: List<TopologyNode> = emptyList(),
    val edges: List<TopologyEdge> = emptyList()
)

/**
 * Topology node (peer in the network).
 */
data class TopologyNode(
    val id: String,
    val isSelf: Boolean,
    val isOnline: Boolean
)

/**
 * Topology edge (connection between peers).
 */
data class TopologyEdge(
    val source: String,
    val target: String,
    val transport: String
)
