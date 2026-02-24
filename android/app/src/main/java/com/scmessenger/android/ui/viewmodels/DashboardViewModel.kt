package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.service.MeshEventBus
import com.scmessenger.android.service.PeerEvent
import com.scmessenger.android.service.StatusEvent
import com.scmessenger.android.utils.toEpochSeconds
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import timber.log.Timber
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
    private val bootstrapRelayPeerIds: Set<String> = MeshRepository.DEFAULT_BOOTSTRAP_NODES
        .mapNotNull { parseBootstrapRelayPeerId(it) }
        .toSet()

    // Service stats
    private val _stats = MutableStateFlow<uniffi.api.ServiceStats?>(null)
    val stats: StateFlow<uniffi.api.ServiceStats?> = _stats.asStateFlow()

    // Active peers
    private val _peers = MutableStateFlow<List<PeerInfo>>(emptyList())
    val peers: StateFlow<List<PeerInfo>> = _peers.asStateFlow()

    // Network topology data (for graph visualization)
    private val _topology = MutableStateFlow<NetworkTopology>(NetworkTopology())
    val topology: StateFlow<NetworkTopology> = _topology.asStateFlow()

    // Peer counts from discovery tracking
    val fullPeersCount = meshRepository.discoveredPeers.map { discovered ->
        deduplicateDiscoveredPeers(discovered).values.count { peer -> peer.isFull }
    }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), 0)

    val headlessPeersCount = meshRepository.discoveredPeers.map { discovered ->
        deduplicateDiscoveredPeers(discovered).values.count { peer -> !peer.isFull }
    }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), 0)

    val totalPeersCount = meshRepository.discoveredPeers.map { discovered ->
        deduplicateDiscoveredPeers(discovered).size
    }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), 0)

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
     * Load active peers from discovery map and ledger.
     */
    private fun loadPeers() {
        try {
            val discoveredSnapshot = meshRepository.discoveredPeers.value
            val discovered = deduplicateDiscoveredPeers(discoveredSnapshot)
            val ledgerEntries = meshRepository.getDialableAddresses()
            val routeAliasToCanonical = discoveredSnapshot
                .mapNotNull { (routeKey, info) ->
                    val alias = routeKey.trim()
                    val canonical = info.peerId.trim()
                    if (alias.isNotEmpty() && canonical.isNotEmpty() && alias != canonical) {
                        alias to canonical
                    } else {
                        null
                    }
                }
                .toMap()
            val canonicalByPublicKey = discovered.values
                .mapNotNull { info ->
                    normalizePublicKey(info.publicKey)?.let { publicKey ->
                        publicKey to info.peerId
                    }
                }
                .toMap()
           
            val peerMap = discovered.mapValues { (_, info) ->
                PeerInfo(
                    peerId = info.peerId,
                    nickname = info.nickname,
                    localNickname = info.localNickname,
                    multiaddr = "", // Might be empty for BLE/headless
                    lastSeen = info.lastSeen,
                    transport = when (info.transport) {
                        com.scmessenger.android.service.TransportType.BLE -> "BLE"
                        com.scmessenger.android.service.TransportType.WIFI_AWARE -> "WiFi Aware"
                        com.scmessenger.android.service.TransportType.WIFI_DIRECT -> "WiFi Direct"
                        com.scmessenger.android.service.TransportType.INTERNET -> "Internet"
                    },
                    isOnline = isRecent(info.lastSeen),
                    isFull = info.isFull,
                    isRelay = isBootstrapRelayPeer(info.peerId)
                )
            }.toMutableMap()
           
            // Enrich/Add with ledger entries (dialable peers)
            ledgerEntries.forEach { entry ->
                val rawPeerId = entry.peerId ?: return@forEach
                val peerId = routeAliasToCanonical[rawPeerId]
                    ?: normalizePublicKey(entry.publicKey)?.let { canonicalByPublicKey[it] }
                    ?: rawPeerId
                val existing = peerMap[peerId]
                if (existing != null) {
                    val entryLastSeen = entry.lastSeen
                    val existingLastSeen = existing.lastSeen
                    peerMap[peerId] = existing.copy(
                        nickname = existing.nickname ?: entry.nickname,
                        multiaddr = entry.multiaddr,
                        lastSeen = when {
                            entryLastSeen == null -> existingLastSeen
                            existingLastSeen == null || entryLastSeen > existingLastSeen -> entryLastSeen
                            else -> existingLastSeen
                        },
                        isOnline = isRecent(entry.lastSeen) || existing.isOnline,
                        isRelay = existing.isRelay || isBootstrapRelayPeer(peerId)
                    )
                } else {
                    peerMap[peerId] = PeerInfo(
                        peerId = peerId,
                        nickname = entry.nickname,
                        multiaddr = entry.multiaddr,
                        lastSeen = entry.lastSeen,
                        transport = determineTransport(entry.multiaddr),
                        isOnline = isRecent(entry.lastSeen),
                        isFull = false,
                        isRelay = isBootstrapRelayPeer(peerId)
                    )
                }
            }
           
            val peerList = peerMap.values.toList()
            _peers.value = peerList

            Timber.d("Loaded ${peerList.size} discovered peers (${peerList.count { it.isFull }} full)")
        } catch (e: Exception) {
            Timber.e(e, "Failed to load peers")
        }
    }

    private fun deduplicateDiscoveredPeers(
        discovered: Map<String, MeshRepository.PeerDiscoveryInfo>
    ): Map<String, MeshRepository.PeerDiscoveryInfo> {
        val merged = linkedMapOf<String, MeshRepository.PeerDiscoveryInfo>()
        discovered.values.forEach { info ->
            val canonicalPeerId = info.peerId.trim()
            if (canonicalPeerId.isEmpty()) return@forEach
            val existing = merged[canonicalPeerId]
            if (existing == null) {
                merged[canonicalPeerId] = info.copy(peerId = canonicalPeerId)
            } else {
                merged[canonicalPeerId] = existing.copy(
                    publicKey = existing.publicKey ?: info.publicKey,
                    nickname = existing.nickname ?: info.nickname,
                    localNickname = existing.localNickname ?: info.localNickname,
                    transport = if (
                        existing.transport == com.scmessenger.android.service.TransportType.INTERNET ||
                            info.transport == com.scmessenger.android.service.TransportType.INTERNET
                    ) {
                        com.scmessenger.android.service.TransportType.INTERNET
                    } else {
                        existing.transport
                    },
                    isFull = existing.isFull || info.isFull,
                    lastSeen = maxOf(existing.lastSeen, info.lastSeen)
                )
            }
        }
        return merged
    }

    private fun normalizePublicKey(value: String?): String? {
        val trimmed = value?.trim() ?: return null
        if (trimmed.length != 64) return null
        if (!trimmed.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' }) return null
        return trimmed.lowercase()
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
                        id = identityInfo.identityId ?: "Self",
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
                            source = it.identityId ?: "Self",
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
            meshRepository.discoveredPeers.collect {
                refreshData()
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
        val now = System.currentTimeMillis() / 1000
        val seenAt = timestamp.toEpochSeconds()
        val fiveMinutes = 300L
        return seenAt <= now && (now - seenAt) < fiveMinutes
    }

    private fun isBootstrapRelayPeer(peerId: String): Boolean {
        return bootstrapRelayPeerIds.contains(peerId)
    }

    private fun parseBootstrapRelayPeerId(multiaddr: String): String? {
        val marker = "/p2p/"
        val idx = multiaddr.lastIndexOf(marker)
        if (idx < 0 || idx + marker.length >= multiaddr.length) return null
        val relayPeerId = multiaddr.substring(idx + marker.length).trim()
        return relayPeerId.takeIf { it.isNotEmpty() }
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
    val nickname: String?,
    val localNickname: String? = null,
    val multiaddr: String,
    val lastSeen: ULong?,
    val transport: String,
    val isOnline: Boolean,
    val isFull: Boolean,
    val isRelay: Boolean = false
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
