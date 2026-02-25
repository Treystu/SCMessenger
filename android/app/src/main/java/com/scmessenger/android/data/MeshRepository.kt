package com.scmessenger.android.data

import android.content.Context
import android.content.pm.PackageManager
import android.content.SharedPreferences
import androidx.core.content.ContextCompat
import com.scmessenger.android.utils.Permissions
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import timber.log.Timber
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.isActive
import java.io.File

/**
 * Repository abstracting access to the Rust core via UniFFI bindings.
 *
 * This is the single source of truth for:
 * - Mesh service lifecycle
 * - Contacts management
 * - Message history
 * - Connection ledger
 * - Network settings
 *
 * All UniFFI objects are initialized lazily and managed here to ensure
 * proper lifecycle and resource cleanup.
 */
class MeshRepository(private val context: Context) {

    companion object {
        private const val IDENTITY_BACKUP_PREFS = "identity_backup_prefs"
        private const val IDENTITY_BACKUP_KEY = "identity_backup_v1"
        /** Static fallback bootstrap nodes for NAT traversal and internet roaming.
         *  These are used if env override and remote fetch both fail/are absent. */
        private val STATIC_BOOTSTRAP_NODES: List<String> = listOf(
            "/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWMyngfNZajWRNRPdtc32uxn1sBYZE126NDD4b547BAMLj"
        )

        /** Resolve bootstrap nodes using the core BootstrapResolver.
         *  Priority: SC_BOOTSTRAP_NODES env var → remote URL → static fallback. */
        val DEFAULT_BOOTSTRAP_NODES: List<String> by lazy {
            try {
                val config = uniffi.api.BootstrapConfig(
                    staticNodes = STATIC_BOOTSTRAP_NODES,
                    remoteUrl = null,  // Set to a bootstrap-list URL when available
                    fetchTimeoutSecs = 5u,
                    envOverrideKey = "SC_BOOTSTRAP_NODES"
                )
                uniffi.api.BootstrapResolver(config).resolve()
            } catch (e: Exception) {
                Timber.w("BootstrapResolver failed, using static fallback: ${e.message}")
                STATIC_BOOTSTRAP_NODES
            }
        }

        internal fun isMeshParticipationEnabled(settings: uniffi.api.MeshSettings?): Boolean {
            return settings?.relayEnabled == true
        }

        internal fun requireMeshParticipationEnabled(settings: uniffi.api.MeshSettings?) {
            if (!isMeshParticipationEnabled(settings)) {
                throw IllegalStateException(
                    "Cannot send messages: mesh participation is disabled. Enable mesh participation in settings to send and receive messages."
                )
            }
        }
    }

    private val storagePath: String = context.filesDir.absolutePath
    private val identityBackupPrefs: SharedPreferences by lazy {
        context.getSharedPreferences(IDENTITY_BACKUP_PREFS, Context.MODE_PRIVATE)
    }

    // Mesh service instance (lazy init)
    private var meshService: uniffi.api.MeshService? = null

    // Managers (lazy init)
    private var contactManager: uniffi.api.ContactManager? = null
    private var historyManager: uniffi.api.HistoryManager? = null
    private var ledgerManager: uniffi.api.LedgerManager? = null
    private var settingsManager: uniffi.api.MeshSettingsManager? = null
    private var autoAdjustEngine: uniffi.api.AutoAdjustEngine? = null

    // Core & Network (lazy init)
    private var ironCore: uniffi.api.IronCore? = null
    // Swarm Bridge (Internet/Libp2p)
    private var swarmBridge: uniffi.api.SwarmBridge? = null

    // Wifi Transport
    private var wifiTransportManager: com.scmessenger.android.transport.WifiTransportManager? = null

    // Service state
    private val _serviceState = MutableStateFlow(uniffi.api.ServiceState.STOPPED)
    val serviceState: StateFlow<uniffi.api.ServiceState> = _serviceState.asStateFlow()

    // Service stats
    private val _serviceStats = MutableStateFlow<uniffi.api.ServiceStats?>(null)
    val serviceStats: StateFlow<uniffi.api.ServiceStats?> = _serviceStats.asStateFlow()

    // Message updates flow (both sent and received) used for UI updates
    private val _messageUpdates = kotlinx.coroutines.flow.MutableSharedFlow<uniffi.api.MessageRecord>(replay = 0)
    val messageUpdates = _messageUpdates.asSharedFlow()

    // Compatibility for notifications (incoming only)
    val incomingMessages = messageUpdates.filter { it.direction == uniffi.api.MessageDirection.RECEIVED }

    private val repoScope = kotlinx.coroutines.CoroutineScope(kotlinx.coroutines.Dispatchers.IO + kotlinx.coroutines.SupervisorJob())
    private var pendingOutboxRetryJob: kotlinx.coroutines.Job? = null
    private var coverTrafficJob: kotlinx.coroutines.Job? = null
    private val pendingOutboxFile = File(storagePath, "pending_outbox.json")
    private val receiptAwaitSeconds: Long = 8L
    private val identitySyncSentPeers = java.util.Collections.synchronizedSet(mutableSetOf<String>())
    @Volatile
    private var lastRelayBootstrapDialMs: Long = 0L
    private val dialThrottleState = java.util.concurrent.ConcurrentHashMap<String, Pair<Int, Long>>()

    // Core Delegate reference to prevent GC
    private var coreDelegate: uniffi.api.CoreDelegate? = null

    // BLE Components
    private var bleScanner: com.scmessenger.android.transport.ble.BleScanner? = null
    private var bleAdvertiser: com.scmessenger.android.transport.ble.BleAdvertiser? = null
    private var bleGattServer: com.scmessenger.android.transport.ble.BleGattServer? = null
    private var bleGattClient: com.scmessenger.android.transport.ble.BleGattClient? = null

    private data class RoutingHints(
        val blePeerId: String?,
        val libp2pPeerId: String?,
        val listeners: List<String>
    )

    private data class TransportIdentityResolution(
        val canonicalPeerId: String,
        val publicKey: String,
        val nickname: String?,
        val localNickname: String? = null
    )

    private data class PendingOutboundEnvelope(
        val queueId: String,
        val historyRecordId: String,
        val peerId: String,
        val routePeerId: String?,
        val listeners: List<String>,
        val envelopeBase64: String,
        val createdAtEpochSec: Long,
        val attemptCount: Int,
        val nextAttemptAtEpochSec: Long
    )

    private data class MessageIdentityHints(
        val identityId: String?,
        val publicKey: String?,
        val nickname: String?,
        val libp2pPeerId: String?,
        val listeners: List<String>,
        val externalAddresses: List<String>,
        val connectionHints: List<String>
    )

    private data class DecodedMessagePayload(
        val kind: String,
        val text: String,
        val hints: MessageIdentityHints?
    )

    // Track discovered peers (both headless and full)
    private val _discoveredPeers = MutableStateFlow<Map<String, PeerDiscoveryInfo>>(emptyMap())
    val discoveredPeers: StateFlow<Map<String, PeerDiscoveryInfo>> = _discoveredPeers.asStateFlow()

    data class PeerDiscoveryInfo(
        val peerId: String,          // Key (libp2p or canonical)
        val publicKey: String?,      // Extracted from PeerId or from identity beacon
        val nickname: String?,       // From Identity Beacon or Contact DB
        val localNickname: String? = null,
        val transport: com.scmessenger.android.service.TransportType,
        val isFull: Boolean,         // True if peer identity is authenticated (non-relay)
        val lastSeen: ULong = System.currentTimeMillis().toULong() / 1000u
    )

    private data class DeliveryAttemptResult(
        val acked: Boolean,
        val routePeerId: String?
    )

    private data class ReplayDiscoveredIdentity(
        var canonicalPeerId: String,
        var publicKey: String?,
        var nickname: String?,
        var localNickname: String?,
        var routePeerId: String?,
        var transport: com.scmessenger.android.service.TransportType
    )

    private data class IdentityEmissionSignature(
        val canonicalPeerId: String,
        val publicKey: String,
        val nickname: String?,
        val libp2pPeerId: String?,
        val blePeerId: String?
    )

    init {
        Timber.d("MeshRepository initialized with storage: $storagePath")
        initializeManagers()
    }

    private fun initializeManagers() {
        try {
            // Initialize Data Managers
            settingsManager = uniffi.api.MeshSettingsManager(storagePath)
            historyManager = uniffi.api.HistoryManager(storagePath)
            contactManager = uniffi.api.ContactManager(storagePath)
            ledgerManager = uniffi.api.LedgerManager(storagePath)
            autoAdjustEngine = uniffi.api.AutoAdjustEngine()

            // Pre-load data where applicable
            ledgerManager?.load()

            Timber.i("All managers initialized successfully")
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize managers")
        }
    }

    // ========================================================================
    // MESH SERVICE LIFECYCLE
    // ========================================================================

    /**
     * Start the mesh service with the given configuration.
     * This initializes the Rust core, starts BLE transport, and wires up events.
     */
    @Synchronized
    fun startMeshService(config: uniffi.api.MeshServiceConfig) {
        if (meshService?.getState() == uniffi.api.ServiceState.RUNNING) {
            _serviceState.value = uniffi.api.ServiceState.RUNNING
            Timber.d("MeshService is already running")
            return
        }

        try {
            Timber.d("Starting MeshService...")
            if (meshService == null) {
                // Recreate service instance after stop/failure so start is always clean.
                meshService = uniffi.api.MeshService.withStorage(config, storagePath)
            }

            // 1. Start the Rust Core service
            meshService?.start()

            // 2. Obtain shared IronCore instance
            ironCore = meshService?.getCore()
            if (ironCore == null) {
                throw IllegalStateException("IronCore instance is null after service start")
            }
            ensureLocalIdentityFederation()

            // 3. Wire up CoreDelegate (Rust -> Android Events)
            coreDelegate = object : uniffi.api.CoreDelegate {
                override fun onPeerDiscovered(peerId: String) {
                    Timber.d("Core notified discovery: $peerId")
                    repoScope.launch {
                        val selfLibp2pPeerId = ironCore?.getIdentityInfo()?.libp2pPeerId
                        if (!selfLibp2pPeerId.isNullOrBlank() && peerId == selfLibp2pPeerId) {
                            Timber.d("Ignoring self transport discovery: $peerId")
                            return@launch
                        }

                        val isRelay = isBootstrapRelayPeer(peerId)

                        val transportIdentity = resolveTransportIdentity(peerId)
                        val extractedKey = try { ironCore?.extractPublicKeyFromPeerId(peerId) } catch (_: Exception) { null }

                        val discoveredNickname = prepopulateDiscoveryNickname(
                            nickname = transportIdentity?.nickname,
                            peerId = transportIdentity?.canonicalPeerId ?: peerId,
                            publicKey = transportIdentity?.publicKey ?: extractedKey
                        )

                        // Update discovery map
                        val discoveryInfo = PeerDiscoveryInfo(
                            peerId = transportIdentity?.canonicalPeerId ?: peerId,
                            publicKey = transportIdentity?.publicKey ?: extractedKey,
                            nickname = discoveredNickname,
                            localNickname = transportIdentity?.localNickname,
                            transport = com.scmessenger.android.service.TransportType.INTERNET, // Default for swarm
                            isFull = !isRelay && (
                                transportIdentity != null ||
                                    !extractedKey.isNullOrBlank()
                                ),
                            lastSeen = System.currentTimeMillis().toULong() / 1000u
                        )
                        updateDiscoveredPeer(peerId, discoveryInfo)
                        if (discoveryInfo.peerId != peerId) {
                            updateDiscoveredPeer(discoveryInfo.peerId, discoveryInfo)
                        }

                        if (transportIdentity != null) {
                            val relayHints = buildDialCandidatesForPeer(
                                routePeerId = peerId,
                                rawAddresses = emptyList(),
                                includeRelayCircuits = true
                            )
                            emitIdentityDiscoveredIfChanged(
                                peerId = transportIdentity.canonicalPeerId,
                                publicKey = transportIdentity.publicKey,
                                nickname = discoveredNickname,
                                libp2pPeerId = peerId,
                                listeners = relayHints
                            )
                            annotateIdentityInLedger(
                                routePeerId = peerId,
                                listeners = relayHints,
                                publicKey = transportIdentity.publicKey,
                                nickname = discoveredNickname
                            )
                            persistRouteHintsForTransportPeer(
                                libp2pPeerId = peerId,
                                listeners = relayHints,
                                knownPublicKey = transportIdentity.publicKey
                            )
                                upsertFederatedContact(
                                    canonicalPeerId = transportIdentity.canonicalPeerId,
                                    publicKey = transportIdentity.publicKey,
                                    nickname = transportIdentity.nickname,
                                    libp2pPeerId = peerId,
                                    listeners = relayHints,
                                    createIfMissing = false
                                )
                            try { contactManager?.updateLastSeen(transportIdentity.canonicalPeerId) } catch (_: Exception) { }
                            try { contactManager?.updateLastSeen(peerId) } catch (_: Exception) { }
                            if (!isRelay && relayHints.isNotEmpty()) {
                                connectToPeer(peerId, relayHints)
                            }
                        } else {
                            com.scmessenger.android.service.MeshEventBus.emitPeerEvent(
                                com.scmessenger.android.service.PeerEvent.Discovered(
                                    peerId,
                                    com.scmessenger.android.service.TransportType.INTERNET
                                )
                            )
                        }
                    }
                }

                override fun onPeerIdentified(peerId: String, agentVersion: String, listenAddrs: List<String>) {
                    Timber.d("Core notified identified: $peerId (agent: $agentVersion) with ${listenAddrs.size} addresses")
                    repoScope.launch {
                        dialThrottleState.keys
                            .filter { it.endsWith("/p2p/$peerId") || it == peerId }
                            .forEach { dialThrottleState.remove(it) }
                        val selfLibp2pPeerId = ironCore?.getIdentityInfo()?.libp2pPeerId
                        if (!selfLibp2pPeerId.isNullOrBlank() && peerId == selfLibp2pPeerId) {
                            Timber.d("Ignoring self transport identity: $peerId")
                            return@launch
                        }

                        val dialCandidates = buildDialCandidatesForPeer(
                            routePeerId = peerId,
                            rawAddresses = listenAddrs,
                            includeRelayCircuits = true
                        )

                        val isHeadless = agentVersion.contains("/headless/")
                        if (isBootstrapRelayPeer(peerId) || isHeadless) {
                            Timber.i("Headless/Relay transport node identified: $peerId (agent: $agentVersion)")
                            val relayDiscovery = PeerDiscoveryInfo(
                                peerId = peerId,
                                publicKey = null,
                                nickname = null,
                                localNickname = null,
                                transport = com.scmessenger.android.service.TransportType.INTERNET,
                                isFull = false,
                                lastSeen = System.currentTimeMillis().toULong() / 1000u
                            )
                            updateDiscoveredPeer(peerId, relayDiscovery)
                            com.scmessenger.android.service.MeshEventBus.emitPeerEvent(
                                com.scmessenger.android.service.PeerEvent.Discovered(
                                    peerId,
                                    com.scmessenger.android.service.TransportType.INTERNET
                                )
                            )
                            emitConnectedIfChanged(
                                peerId = peerId,
                                transport = com.scmessenger.android.service.TransportType.INTERNET
                            )
                        } else {
                            val transportIdentity = resolveTransportIdentity(peerId)

                            val discoveredNickname = prepopulateDiscoveryNickname(
                                nickname = transportIdentity?.nickname,
                                peerId = transportIdentity?.canonicalPeerId ?: peerId,
                                publicKey = transportIdentity?.publicKey
                            )

                            // Update discovery map
                            val discoveryInfo = PeerDiscoveryInfo(
                                peerId = transportIdentity?.canonicalPeerId ?: peerId,
                                publicKey = transportIdentity?.publicKey,
                                nickname = discoveredNickname,
                                localNickname = transportIdentity?.localNickname,
                                transport = com.scmessenger.android.service.TransportType.INTERNET,
                                isFull = transportIdentity != null,
                                lastSeen = System.currentTimeMillis().toULong() / 1000u
                            )
                            updateDiscoveredPeer(peerId, discoveryInfo)
                            if (discoveryInfo.peerId != peerId) {
                                updateDiscoveredPeer(discoveryInfo.peerId, discoveryInfo)
                            }

                            if (transportIdentity != null) {
                                emitIdentityDiscoveredIfChanged(
                                    peerId = transportIdentity.canonicalPeerId,
                                    publicKey = transportIdentity.publicKey,
                                    nickname = discoveredNickname,
                                    libp2pPeerId = peerId,
                                    listeners = dialCandidates
                                )
                                annotateIdentityInLedger(
                                    routePeerId = peerId,
                                    listeners = dialCandidates,
                                    publicKey = transportIdentity.publicKey,
                                    nickname = discoveredNickname
                                )
                                try { contactManager?.updateLastSeen(transportIdentity.canonicalPeerId) } catch (_: Exception) { }
                                try { contactManager?.updateLastSeen(peerId) } catch (_: Exception) { }
                            } else {
                                Timber.d("Transport identity unavailable for $peerId")
                            }
                            emitConnectedIfChanged(
                                peerId = peerId,
                                transport = com.scmessenger.android.service.TransportType.INTERNET
                            )
                            persistRouteHintsForTransportPeer(
                                libp2pPeerId = peerId,
                                listeners = dialCandidates,
                                knownPublicKey = transportIdentity?.publicKey
                            )
                            if (transportIdentity != null) {
                                upsertFederatedContact(
                                    canonicalPeerId = transportIdentity.canonicalPeerId,
                                    publicKey = transportIdentity.publicKey,
                                    nickname = transportIdentity.nickname,
                                    libp2pPeerId = peerId,
                                    listeners = dialCandidates,
                                    createIfMissing = false
                                )
                            }
                            sendIdentitySyncIfNeeded(
                                routePeerId = peerId,
                                knownPublicKey = transportIdentity?.publicKey
                            )
                        }

                        // Identified implies an active session exists; avoid immediate re-dial loops.
                        flushPendingOutbox("peer_identified:$peerId")
                        updateBleIdentityBeacon()
                    }
                }

                override fun onPeerDisconnected(peerId: String) {
                    Timber.d("Core notified disconnect: $peerId")
                    repoScope.launch {
                        connectedEmissionCache.remove(peerId.trim())
                        pruneDisconnectedPeer(peerId)
                        com.scmessenger.android.service.MeshEventBus.emitPeerEvent(
                            com.scmessenger.android.service.PeerEvent.Disconnected(peerId)
                        )
                    }
                }

                override fun onMessageReceived(
                    senderId: String,
                    senderPublicKeyHex: String,
                    messageId: String,
                    senderTimestamp: ULong,
                    data: ByteArray
                ) {
                    Timber.i("Message from $senderId: $messageId")
                    try {
                        // Check if relay/messaging is enabled (bidirectional control)
                        // Treat null/missing settings as disabled (fail-safe)
                        // Cache settings value to avoid race condition during check
                        val currentSettings = settingsManager?.load()
                        if (!Companion.isMeshParticipationEnabled(currentSettings)) {
                            Timber.w("Dropping received message - mesh participation is disabled or settings unavailable")
                            return
                        }

                        val normalizedSenderKey = normalizePublicKey(senderPublicKeyHex)
                        val rawContent = data.toString(Charsets.UTF_8)
                        val decodedPayload = decodeMessageWithIdentityHints(rawContent)
                        val hintedIdentity = decodedPayload.hints
                        val hintedKey = normalizePublicKey(hintedIdentity?.publicKey)
                        val verifiedHints = if (
                            hintedKey != null &&
                                normalizedSenderKey != null &&
                                hintedKey != normalizedSenderKey
                        ) {
                            Timber.w(
                                "Ignoring forged message identity hint for $senderId: key mismatch " +
                                    "(hint=${hintedKey.take(8)}..., envelope=${normalizedSenderKey.take(8)}...)"
                            )
                            null
                        } else {
                            hintedIdentity
                        }

                        var canonicalPeerId = resolveCanonicalPeerId(senderId, senderPublicKeyHex)
                        canonicalPeerId = resolveCanonicalPeerIdFromMessageHints(
                            resolvedCanonicalPeerId = canonicalPeerId,
                            senderId = senderId,
                            senderPublicKeyHex = senderPublicKeyHex,
                            hintedIdentityId = verifiedHints?.identityId
                        )

                        val hintedRoutePeerId = verifiedHints?.libp2pPeerId
                            ?.trim()
                            ?.takeIf { isLibp2pPeerId(it) }
                        val routePeerId = senderId.takeIf { isLibp2pPeerId(it) } ?: hintedRoutePeerId
                        val hintedAddresses = (
                            verifiedHints?.listeners.orEmpty() +
                                verifiedHints?.externalAddresses.orEmpty() +
                                verifiedHints?.connectionHints.orEmpty()
                            )
                        val hintedDialCandidates = buildDialCandidatesForPeer(
                            routePeerId = routePeerId,
                            rawAddresses = hintedAddresses,
                            includeRelayCircuits = true
                        )
                        val knownNickname = selectAuthoritativeNickname(
                            verifiedHints?.nickname,
                            resolveKnownPeerNickname(
                                canonicalPeerId = canonicalPeerId,
                                routePeerId = routePeerId,
                                publicKey = normalizedSenderKey
                            )
                        )
                        if (canonicalPeerId != senderId) {
                            Timber.i("Canonicalized sender $senderId -> $canonicalPeerId using public key match")
                        }

                        if (isBootstrapRelayPeer(canonicalPeerId)) {
                            Timber.i("Ignoring payload attributed to bootstrap relay peer $canonicalPeerId")
                            return
                        }

                        // Auto-upsert contact: senderPublicKeyHex is guaranteed valid Ed25519 key
                        // (Rust only fires this callback after successful decryption)
                        val existingContact = try { contactManager?.get(canonicalPeerId) } catch (e: Exception) { null }
                        if (existingContact == null && normalizedSenderKey != null) {
                            var routeNotes = if (!routePeerId.isNullOrBlank()) {
                                appendRoutingHint(notes = null, key = "libp2p_peer_id", value = routePeerId)
                            } else {
                                null
                            }
                            routeNotes = upsertRoutingListeners(
                                routeNotes,
                                normalizeOutboundListenerHints(hintedDialCandidates)
                            )
                            val autoContact = uniffi.api.Contact(
                                peerId = canonicalPeerId,
                                nickname = knownNickname,
                                localNickname = null,
                                publicKey = normalizedSenderKey,
                                addedAt = (System.currentTimeMillis() / 1000).toULong(),
                                lastSeen = (System.currentTimeMillis() / 1000).toULong(),
                                notes = routeNotes
                            )
                            try {
                                contactManager?.add(autoContact)
                                Timber.i("Auto-created contact from received message: ${canonicalPeerId.take(8)} key: ${senderPublicKeyHex.take(8)}...")
                            } catch (e: Exception) {
                                Timber.w("Auto-create contact failed for ${canonicalPeerId.take(8)}: ${e.message}")
                            }
                        } else if (existingContact != null) {
                            try { contactManager?.updateLastSeen(canonicalPeerId) } catch (e: Exception) {
                                Timber.d("updateLastSeen failed: ${e.message}")
                            }

                            if (existingContact.nickname.isNullOrBlank() && !knownNickname.isNullOrBlank()) {
                                val updatedContact = uniffi.api.Contact(
                                    peerId = existingContact.peerId,
                                    nickname = knownNickname,
                                    localNickname = existingContact.localNickname,
                                    publicKey = existingContact.publicKey,
                                    addedAt = existingContact.addedAt,
                                    lastSeen = existingContact.lastSeen,
                                    notes = existingContact.notes
                                )
                                try {
                                    contactManager?.add(updatedContact)
                                } catch (e: Exception) {
                                    Timber.d("Failed to persist nickname hint for ${existingContact.peerId}: ${e.message}")
                                }
                            }

                            // Persist explicit libp2p alias mapping when known so identity/libp2p IDs
                            // stay canonicalized to one conversation thread.
                            if (!routePeerId.isNullOrBlank() &&
                                normalizedSenderKey != null &&
                                normalizePublicKey(existingContact.publicKey) == normalizedSenderKey &&
                                parseRoutingHints(existingContact.notes).libp2pPeerId.isNullOrBlank()
                            ) {
                                val updatedNotes = appendRoutingHint(existingContact.notes, "libp2p_peer_id", routePeerId)
                                val updatedNotesWithListeners = upsertRoutingListeners(
                                    updatedNotes,
                                    normalizeOutboundListenerHints(hintedDialCandidates)
                                )
                                val updatedContact = uniffi.api.Contact(
                                    peerId = existingContact.peerId,
                                    nickname = existingContact.nickname,
                                    localNickname = existingContact.localNickname,
                                    publicKey = existingContact.publicKey,
                                    addedAt = existingContact.addedAt,
                                    lastSeen = existingContact.lastSeen,
                                    notes = updatedNotesWithListeners
                                )
                                try {
                                    contactManager?.add(updatedContact)
                                } catch (e: Exception) {
                                    Timber.d("Failed to persist libp2p alias hint for ${existingContact.peerId}: ${e.message}")
                                }
                            }
                        }

                        if (normalizedSenderKey != null) {
                            upsertFederatedContact(
                                canonicalPeerId = canonicalPeerId,
                                publicKey = normalizedSenderKey,
                                nickname = knownNickname,
                                libp2pPeerId = routePeerId,
                                listeners = hintedDialCandidates,
                                createIfMissing = false
                            )
                            val discoveredNickname = prepopulateDiscoveryNickname(
                                nickname = knownNickname,
                                peerId = canonicalPeerId,
                                publicKey = normalizedSenderKey
                            )
                            val discoveryInfo = PeerDiscoveryInfo(
                                peerId = canonicalPeerId,
                                publicKey = normalizedSenderKey,
                                nickname = discoveredNickname,
                                localNickname = existingContact?.localNickname,
                                transport = com.scmessenger.android.service.TransportType.INTERNET,
                                isFull = true,
                                lastSeen = (System.currentTimeMillis() / 1000).toULong()
                            )
                            updateDiscoveredPeer(canonicalPeerId, discoveryInfo)
                            if (!routePeerId.isNullOrBlank() && routePeerId != canonicalPeerId) {
                                updateDiscoveredPeer(routePeerId, discoveryInfo)
                            }
                            val listeners = (
                                routePeerId?.let(::getDialHintsForRoutePeer).orEmpty() +
                                    hintedDialCandidates
                                ).distinct()
                            repoScope.launch {
                                emitIdentityDiscoveredIfChanged(
                                    peerId = canonicalPeerId,
                                    publicKey = normalizedSenderKey,
                                    nickname = discoveredNickname,
                                    libp2pPeerId = routePeerId,
                                    listeners = listeners
                                )
                            }
                            annotateIdentityInLedger(
                                routePeerId = routePeerId,
                                listeners = listeners,
                                publicKey = normalizedSenderKey,
                                nickname = discoveredNickname
                            )
                        }

                        val messageKind = decodedPayload.kind.trim().lowercase()
                        if (messageKind == "identity_sync") {
                            Timber.d("Processed identity sync from $canonicalPeerId (route=$routePeerId)")
                            sendDeliveryReceiptAsync(senderPublicKeyHex, messageId, senderId)
                            return
                        }

                        val existingRecord = try {
                            historyManager?.get(messageId)
                        } catch (_: Exception) {
                            null
                        }
                        if (existingRecord?.direction == uniffi.api.MessageDirection.RECEIVED) {
                            Timber.d("Duplicate inbound message $messageId from $senderId; acknowledging without re-emitting UI")
                            sendDeliveryReceiptAsync(senderPublicKeyHex, messageId, senderId)
                            return
                        }

                        val content = decodedPayload.text
                        val fallbackNow = (System.currentTimeMillis() / 1000).toULong()
                        val canonicalTimestamp = if (senderTimestamp > 0uL) senderTimestamp else fallbackNow
                        val record = uniffi.api.MessageRecord(
                            id = messageId,
                            direction = uniffi.api.MessageDirection.RECEIVED,
                            peerId = canonicalPeerId,
                            content = content,
                            timestamp = canonicalTimestamp,
                            delivered = true
                        )
                        historyManager?.add(record)

                        // Emit for notifications and UI updates
                        repoScope.launch {
                            _messageUpdates.emit(record)
                        }

                        // Send delivery receipt ACK back to sender.
                        sendDeliveryReceiptAsync(senderPublicKeyHex, messageId, canonicalPeerId)
                    } catch (e: Exception) {
                        Timber.e(e, "Failed to process received message")
                    }
                }

                override fun onReceiptReceived(messageId: String, status: String) {
                    Timber.d("Receipt for $messageId: $status")
                    val normalized = status.trim().lowercase()
                    if (normalized != "delivered" && normalized != "read") {
                        return
                    }
                    historyManager?.markDelivered(messageId)
                    ironCore?.markMessageSent(messageId)
                    removePendingOutbound(messageId)
                    // Bridge to ChatViewModel: emit Delivered so UI delivery indicator updates
                    repoScope.launch {
                        com.scmessenger.android.service.MeshEventBus.emitMessageEvent(
                            com.scmessenger.android.service.MessageEvent.Delivered(messageId)
                        )
                    }
                }
            }
            ironCore?.setDelegate(coreDelegate)

            // 4. Start Android transports. Individual transport failures should
            // not abort the entire mesh core lifecycle.
            kotlin.runCatching { initializeAndStartBle() }
                .onFailure { Timber.w(it, "BLE transport failed to initialize; continuing with remaining transports") }
            kotlin.runCatching { initializeAndStartWifi() }
                .onFailure { Timber.w(it, "WiFi transport failed to initialize; continuing with remaining transports") }
            kotlin.runCatching { initializeAndStartSwarm() }
                .onFailure { Timber.w(it, "Swarm transport failed to initialize; core service remains active") }
            ensurePendingOutboxRetryLoop()
            ensureCoverTrafficLoop()
            repoScope.launch { flushPendingOutbox("service_started") }

            // 5. Update State
            _serviceState.value = meshService?.getState() ?: uniffi.api.ServiceState.STOPPED
            if (_serviceState.value != uniffi.api.ServiceState.RUNNING) {
                throw IllegalStateException("MeshService did not reach RUNNING state")
            }
            updateStats()
            startPeriodicStatsUpdate()

            Timber.i("Mesh service started successfully")
        } catch (e: Exception) {
            Timber.e(e, "Failed to start mesh service")
            stopMeshService()
            throw IllegalStateException("Mesh service startup failed", e)
        }
    }

    private fun sendDeliveryReceiptAsync(senderPublicKeyHex: String, messageId: String, senderId: String) {
        repoScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            try {
                val receiptBytes = ironCore?.prepareReceipt(senderPublicKeyHex, messageId)
                if (receiptBytes != null) {
                    val contact = try { contactManager?.get(senderId) } catch (_: Exception) { null }
                    val hints = parseRoutingHints(contact?.notes)
                    val routeCandidates = buildRoutePeerCandidates(
                        peerId = senderId,
                        cachedRoutePeerId = hints.libp2pPeerId,
                        notes = contact?.notes
                    )
                    attemptDirectSwarmDelivery(
                        routePeerCandidates = routeCandidates,
                        listeners = hints.listeners,
                        encryptedData = receiptBytes,
                        blePeerId = hints.blePeerId
                    )
                    Timber.d("Targeted delivery receipt sent for $messageId to $senderId")
                }
            } catch (e: Exception) {
                Timber.d("Failed to send delivery receipt for $messageId: ${e.message}")
            }
        }
    }

    private fun sendIdentitySyncIfNeeded(routePeerId: String, knownPublicKey: String? = null) {
        val normalizedRoute = routePeerId.trim()
        if (normalizedRoute.isEmpty() || !isLibp2pPeerId(normalizedRoute) || isBootstrapRelayPeer(normalizedRoute)) return
        val shouldSend = identitySyncSentPeers.add(normalizedRoute)
        if (!shouldSend) return

        repoScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            try {
                val recipientPublicKey = normalizePublicKey(knownPublicKey)
                    ?: normalizePublicKey(
                        try {
                            ironCore?.extractPublicKeyFromPeerId(normalizedRoute)
                        } catch (_: Exception) {
                            null
                        }
                    )
                if (recipientPublicKey == null) {
                    identitySyncSentPeers.remove(normalizedRoute)
                    return@launch
                }

                val payload = encodeIdentitySyncPayload()
                val prepared = ironCore?.prepareMessageWithId(recipientPublicKey, payload)
                if (prepared == null) {
                    identitySyncSentPeers.remove(normalizedRoute)
                    return@launch
                }
                
                // Use targeted delivery (swarm + BLE fallback) for identity sync
                val contact = contactManager?.list()?.firstOrNull { 
                    it.peerId == normalizedRoute || parseRoutingHints(it.notes).libp2pPeerId == normalizedRoute 
                }
                val hints = parseRoutingHints(contact?.notes)
                val routeCandidates = buildRoutePeerCandidates(
                    peerId = contact?.peerId ?: normalizedRoute,
                    cachedRoutePeerId = normalizedRoute,
                    notes = contact?.notes
                )

                attemptDirectSwarmDelivery(
                    routePeerCandidates = routeCandidates,
                    listeners = hints.listeners,
                    encryptedData = prepared.envelopeData,
                    blePeerId = hints.blePeerId
                )
                Timber.d("Identity sync sent to $normalizedRoute")
            } catch (e: Exception) {
                identitySyncSentPeers.remove(normalizedRoute)
                Timber.d("Failed to send identity sync to $normalizedRoute: ${e.message}")
            }
        }
    }

    private fun initializeAndStartBle() {
        val settings = loadSettings()
        if (!settings.bleEnabled) {
            Timber.d("BLE disabled in settings")
            return
        }
        if (!hasAllPermissions(Permissions.bluetooth)) {
            Timber.w("Skipping BLE transport start: missing Bluetooth permissions")
            return
        }

        // BLE GATT Client: must exist before scanner callbacks to avoid missing first identity reads.
        if (bleGattClient == null) {
            bleGattClient = com.scmessenger.android.transport.ble.BleGattClient(
                context,
                onIdentityReceived = { peerId, data ->
                    onPeerIdentityRead(peerId, data)
                },
                onDataReceived = { peerId, data ->
                    meshService?.onDataReceived(peerId, data)
                }
            )
        }

        // BLE Scanner: Feeds discovered peers to MeshService and handles GATT connections
        if (bleScanner == null) {
            bleScanner = com.scmessenger.android.transport.ble.BleScanner(
                context,
                onPeerDiscovered = { peerId ->
                    meshService?.onPeerDiscovered(peerId)
                    // Connect via GATT client to read identity if needed.
                    kotlin.runCatching { bleGattClient?.connect(peerId) }
                        .onFailure { Timber.w(it, "BLE GATT connect failed for $peerId") }
                },
                onDataReceived = { peerId, data ->
                    meshService?.onDataReceived(peerId, data)
                }
            )
        }
        bleScanner?.startScanning()

        // BLE Advertiser: Broadcasts our presence
        if (bleAdvertiser == null) {
            bleAdvertiser = com.scmessenger.android.transport.ble.BleAdvertiser(context)
        }
        bleAdvertiser?.startAdvertising()

        // BLE GATT Server: Serves our identity and accepts writes from nearby peers
        if (bleGattServer == null) {
            bleGattServer = com.scmessenger.android.transport.ble.BleGattServer(
                context,
                onDataReceived = { peerId, data ->
                    meshService?.onDataReceived(peerId, data)
                }
            )
        }
        bleGattServer?.start()

        // Set identity beacon on BLE GATT server so nearby peers can read our Ed25519 public key
        updateBleIdentityBeacon()
    }

    private fun updateBleIdentityBeacon() {
        val identity = ironCore?.getIdentityInfo()
        val publicKeyHex = identity?.publicKeyHex
        if (!publicKeyHex.isNullOrEmpty()) {
            repoScope.launch(kotlinx.coroutines.Dispatchers.IO) {
                var listeners = getListeningAddresses()
                var attempts = 0
                while (listeners.isEmpty() && attempts < 10) {
                    kotlinx.coroutines.delay(500)
                    listeners = getListeningAddresses()
                    attempts++
                }
                try {
                    // Keep BLE identity beacons compact; oversized values can fail GATT reads
                    // on peer platforms and drop nickname propagation.
                    var resolvedListeners = normalizeOutboundListenerHints(listeners).take(2)
                    var resolvedExternal = normalizeExternalAddressHints(getExternalAddresses()).take(2)
                    val nickname = (identity.nickname ?: "").take(32)
                    fun buildBeacon(): org.json.JSONObject {
                        val connectionHints = (resolvedListeners + resolvedExternal).distinct()
                        return org.json.JSONObject()
                            .put("identity_id", identity.identityId ?: "")
                            .put("public_key", publicKeyHex)
                            .put("nickname", nickname)
                            .put("libp2p_peer_id", identity.libp2pPeerId ?: "")
                            .put("listeners", org.json.JSONArray(resolvedListeners))
                            .put("external_addresses", org.json.JSONArray(resolvedExternal))
                            .put("connection_hints", org.json.JSONArray(connectionHints))
                    }

                    var beaconJsonObject = buildBeacon()
                    var beaconJson = beaconJsonObject.toString().toByteArray(Charsets.UTF_8)

                    if (beaconJson.size > 480) {
                        resolvedListeners = resolvedListeners.take(1)
                        resolvedExternal = resolvedExternal.take(1)
                        beaconJsonObject = buildBeacon()
                        beaconJson = beaconJsonObject.toString().toByteArray(Charsets.UTF_8)
                    }
                    if (beaconJson.size > 480) {
                        resolvedListeners = emptyList()
                        resolvedExternal = emptyList()
                        beaconJsonObject = buildBeacon()
                        beaconJson = beaconJsonObject.toString().toByteArray(Charsets.UTF_8)
                    }
                    if (beaconJson.size > 480) {
                        beaconJsonObject = org.json.JSONObject()
                            .put("identity_id", identity.identityId ?: "")
                            .put("public_key", publicKeyHex)
                            .put("nickname", nickname)
                            .put("libp2p_peer_id", identity.libp2pPeerId ?: "")
                            .put("listeners", org.json.JSONArray())
                            .put("external_addresses", org.json.JSONArray())
                            .put("connection_hints", org.json.JSONArray())
                        beaconJson = beaconJsonObject.toString().toByteArray(Charsets.UTF_8)
                    }
                    bleGattServer?.setIdentityData(beaconJson)
                    Timber.i("BLE GATT identity beacon set: ${publicKeyHex.take(8)}... (${beaconJson.size} bytes)")
                } catch (e: Exception) {
                    Timber.w("Failed to set BLE GATT identity beacon: ${e.message}")
                }
            }
        }
    }

    /**
     * Called when BLE identity beacon is read from a peer.
     * Extracts identity info and attempts to dial the peer via libp2p if possible.
     */
    private fun onPeerIdentityRead(blePeerId: String, data: ByteArray) {
        try {
            val json = org.json.JSONObject(data.toString(Charsets.UTF_8))
            val publicKeyHexRaw = json.getString("public_key")
            val publicKeyHex = normalizePublicKey(publicKeyHexRaw)
                ?: run {
                    Timber.w("Ignoring BLE identity from $blePeerId: invalid public key")
                    return
                }
            val identityId = json.optString("identity_id", "")
                .trim()
                .takeIf { it.isNotBlank() }
                ?: blePeerId
            val rawNickname = (
                json.optString("nickname", "")
                    .ifBlank { json.optString("name", "") }
                ).trim()
            val libp2pPeerId = json.optString("libp2p_peer_id")
            val listeners = json.optJSONArray("listeners")
            val externalAddresses = json.optJSONArray("external_addresses")
            val connectionHints = json.optJSONArray("connection_hints")
            val normalizedLibp2p = libp2pPeerId.takeIf { !it.isNullOrBlank() }?.trim()

            val discoveredNickname = prepopulateDiscoveryNickname(
                nickname = rawNickname,
                peerId = identityId,
                publicKey = publicKeyHex
            )

            Timber.i(
                "Peer identity read from $blePeerId: ${publicKeyHex.take(8)}... " +
                    "identity=${identityId.take(12)} nickname='${discoveredNickname?.take(24) ?: ""}'"
            )
            val selfIdentity = ironCore?.getIdentityInfo()
            val selfKey = normalizePublicKey(selfIdentity?.publicKeyHex)
            val selfIdentityId = selfIdentity?.identityId?.trim().orEmpty()
            val selfLibp2pPeerId = selfIdentity?.libp2pPeerId?.trim().orEmpty()
            if ((selfKey != null && selfKey == publicKeyHex) ||
                (selfIdentityId.isNotEmpty() && selfIdentityId == identityId) ||
                (selfLibp2pPeerId.isNotEmpty() && selfLibp2pPeerId == normalizedLibp2p)
            ) {
                Timber.d("Ignoring self BLE identity beacon from $blePeerId")
                return
            }

            // Update discovery map
            val discoveryInfo = PeerDiscoveryInfo(
                peerId = identityId,
                publicKey = publicKeyHex,
                nickname = discoveredNickname,
                localNickname = try { contactManager?.get(identityId)?.localNickname } catch (_: Exception) { null },
                transport = com.scmessenger.android.service.TransportType.BLE,
                isFull = true,
                lastSeen = System.currentTimeMillis().toULong() / 1000u
            )
            updateDiscoveredPeer(identityId, discoveryInfo)
            if (!normalizedLibp2p.isNullOrBlank()) {
                updateDiscoveredPeer(normalizedLibp2p, discoveryInfo)
            }

            // Emit identity to nearby peers bus — UI will show peer in Nearby section for user to add
            val rawHints = mutableListOf<String>()
            for (i in 0 until (listeners?.length() ?: 0)) {
                rawHints.add(listeners!!.getString(i))
            }
            for (i in 0 until (externalAddresses?.length() ?: 0)) {
                rawHints.add(externalAddresses!!.getString(i))
            }
            for (i in 0 until (connectionHints?.length() ?: 0)) {
                rawHints.add(connectionHints!!.getString(i))
            }

            val routePeerId = normalizedLibp2p
            val listenersStrings = buildDialCandidatesForPeer(
                routePeerId = routePeerId,
                rawAddresses = rawHints,
                includeRelayCircuits = true
            )
            repoScope.launch {
                emitIdentityDiscoveredIfChanged(
                    peerId = identityId,
                    publicKey = publicKeyHex,
                    nickname = discoveredNickname,
                    libp2pPeerId = routePeerId,
                    listeners = listenersStrings,
                    blePeerId = blePeerId
                )
            }
            annotateIdentityInLedger(
                routePeerId = routePeerId,
                listeners = listenersStrings,
                publicKey = publicKeyHex,
                nickname = discoveredNickname
            )
            Timber.i("Emitted IdentityDiscovered for $blePeerId: ${publicKeyHex.take(8)}...")
            // Update lastSeen if already a saved contact
            try { contactManager?.updateLastSeen(blePeerId) } catch (_: Exception) { }
            try { contactManager?.updateLastSeen(identityId) } catch (_: Exception) { }
            routePeerId?.let {
                try { contactManager?.updateLastSeen(it) } catch (_: Exception) { }
            }
            upsertFederatedContact(
                canonicalPeerId = identityId,
                publicKey = publicKeyHex,
                nickname = rawNickname.takeIf { it.isNotBlank() },
                libp2pPeerId = routePeerId,
                listeners = listenersStrings,
                blePeerId = blePeerId,
                createIfMissing = false
            )

            // Attempt to dial via Swarm if we have libp2p info
            if (!routePeerId.isNullOrEmpty() && listenersStrings.isNotEmpty()) {
                connectToPeer(routePeerId, listenersStrings)
            }
        } catch (e: Exception) {
            Timber.w("Failed to parse peer identity read: ${e.message}")
        }
    }

    private fun updateDiscoveredPeer(key: String, info: PeerDiscoveryInfo) {
        _discoveredPeers.update { current ->
            val existing = current[key]
            if (existing != null && existing.isFull && !info.isFull && (info.lastSeen - existing.lastSeen < 300u)) {
                // Don't downgrade a full identity to headless if we've seen it recently.
                current
            } else {
                val merged = if (existing == null) {
                    info
                } else {
                    info.copy(
                        peerId = selectCanonicalPeerId(info.peerId, existing.peerId),
                        publicKey = info.publicKey ?: existing.publicKey,
                        nickname = selectAuthoritativeNickname(info.nickname, existing.nickname),
                        localNickname = normalizeNickname(info.localNickname) ?: normalizeNickname(existing.localNickname),
                        transport = if (
                            info.transport == com.scmessenger.android.service.TransportType.INTERNET ||
                                existing.transport == com.scmessenger.android.service.TransportType.INTERNET
                        ) {
                            com.scmessenger.android.service.TransportType.INTERNET
                        } else {
                            info.transport
                        },
                        isFull = info.isFull || existing.isFull,
                        lastSeen = maxOf(info.lastSeen, existing.lastSeen)
                    )
                }
                val canonicalPeerId = merged.peerId.trim().ifEmpty { key.trim() }
                val canonicalPublicKey = normalizePublicKey(merged.publicKey)

                val withCanonical = current + (canonicalPeerId to merged)
                withCanonical.filterNot { (mapKey, candidate) ->
                    if (mapKey == canonicalPeerId) return@filterNot false

                    val sameCanonicalPeerId = candidate.peerId.trim() == canonicalPeerId
                    val samePublicKey = canonicalPublicKey != null &&
                        normalizePublicKey(candidate.publicKey) == canonicalPublicKey
                    sameCanonicalPeerId || samePublicKey
                }
            }
        }
    }

    private fun pruneDisconnectedPeer(peerId: String) {
        val normalizedPeerId = peerId.trim()
        if (normalizedPeerId.isEmpty()) return

        _discoveredPeers.update { current ->
            if (current.isEmpty()) return@update current

            val disconnectedPublicKey = normalizePublicKey(current[normalizedPeerId]?.publicKey)
            val keysToRemove = current
                .filter { (key, info) ->
                    key == normalizedPeerId ||
                        info.peerId == normalizedPeerId ||
                        (
                            disconnectedPublicKey != null &&
                                normalizePublicKey(info.publicKey) == disconnectedPublicKey
                            )
                }
                .keys

            if (keysToRemove.isEmpty()) current else current - keysToRemove
        }
    }

    private fun initializeAndStartWifi() {
        val settings = loadSettings()
        if (!settings.wifiAwareEnabled && !settings.wifiDirectEnabled) {
            Timber.d("WiFi Transports disabled in settings")
            // Note: WifiTransportManager manages both. Need granular control?
            // For now, if either is enabled, we start it, let it handle internals?
            // But WifiTransportManager likely starts both.
            // Assuming strict check:
             return
        }
        val wifiPermissions = Permissions.location + Permissions.nearbyWifi
        if (!hasAllPermissions(wifiPermissions)) {
            Timber.w("Skipping WiFi transport start: missing Location/Nearby WiFi permissions")
            return
        }

        if (wifiTransportManager == null) {
            wifiTransportManager = com.scmessenger.android.transport.WifiTransportManager(
                context,
                onPeerDiscovered = { peerId ->
                    meshService?.onPeerDiscovered(peerId)
                },
                onDataReceived = { peerId, data ->
                    meshService?.onDataReceived(peerId, data)
                }
            )
        }
        wifiTransportManager?.initialize()
        wifiTransportManager?.startDiscovery()
    }

    private fun initializeAndStartSwarm() {
        val settings = loadSettings()
        if (!settings.internetEnabled) {
            Timber.d("Swarm/Internet disabled in settings")
            return
        }

        try {
            ensureLocalIdentityFederation()

            if (isIdentityInitialized() == true) {
                // Configure bootstrap nodes for NAT traversal
                meshService?.setBootstrapNodes(DEFAULT_BOOTSTRAP_NODES)
                // Initiate swarm in Rust core
                meshService?.startSwarm("/ip4/0.0.0.0/tcp/0")

                // Obtain the SwarmBridge managed by Rust MeshService
                swarmBridge = meshService?.getSwarmBridge()
                updateBleIdentityBeacon()

                Timber.i("✓ Internet transport (Swarm) initiated and bridge wired")
            } else {
                Timber.w("Postponing Swarm start: Identity not ready")
            }
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize Swarm transport")
        }
    }

    private fun ensureLocalIdentityFederation() {
        val core = ironCore ?: return
        try {
            var info = core.getIdentityInfo()
            if (!info.initialized) {
                val restored = restoreIdentityFromBackup(core)
                if (restored) {
                    info = core.getIdentityInfo()
                }
            }
            if (!info.initialized) {
                Timber.i("Identity not initialized; onboarding required")
                return
            }

            val nickname = info.nickname?.trim().orEmpty()
            if (nickname.isNotEmpty()) {
                persistIdentityBackup(core)
            }
        } catch (e: Exception) {
            Timber.w("Failed to ensure local identity federation: ${e.message}")
        }
    }

    private fun restoreIdentityFromBackup(core: uniffi.api.IronCore): Boolean {
        val backup = identityBackupPrefs.getString(IDENTITY_BACKUP_KEY, null)
        if (backup.isNullOrBlank()) {
            return false
        }
        return try {
            core.importIdentityBackup(backup)
            Timber.i("Restored identity from Android backup payload")
            true
        } catch (e: Exception) {
            Timber.w("Identity backup restore failed; fallback to new identity: ${e.message}")
            false
        }
    }

    private fun persistIdentityBackup(core: uniffi.api.IronCore?) {
        val activeCore = core ?: return
        try {
            val backup = activeCore.exportIdentityBackup()
            identityBackupPrefs.edit().putString(IDENTITY_BACKUP_KEY, backup).apply()
        } catch (e: Exception) {
            Timber.w("Failed to persist identity backup payload: ${e.message}")
        }
    }

    fun setPlatformBridge(bridge: uniffi.api.PlatformBridge) {
        meshService?.setPlatformBridge(bridge)
    }

    /**
     * Stop the mesh service and all transports.
     */
    @Synchronized
    fun stopMeshService() {
        pendingOutboxRetryJob?.cancel()
        pendingOutboxRetryJob = null
        coverTrafficJob?.cancel()
        coverTrafficJob = null

        kotlin.runCatching { bleScanner?.stopScanning() }
            .onFailure { Timber.w(it, "Failed to stop BLE scanner") }
        kotlin.runCatching { bleAdvertiser?.stopAdvertising() }
            .onFailure { Timber.w(it, "Failed to stop BLE advertiser") }
        kotlin.runCatching { bleGattServer?.stop() }
            .onFailure { Timber.w(it, "Failed to stop BLE GATT server") }
        kotlin.runCatching { bleGattClient?.cleanup() }
            .onFailure { Timber.w(it, "Failed to cleanup BLE GATT client") }

        kotlin.runCatching { wifiTransportManager?.stopDiscovery() }
            .onFailure { Timber.w(it, "Failed to stop WiFi transport") }

        kotlin.runCatching { swarmBridge?.shutdown() }
            .onFailure { Timber.w(it, "Failed to shutdown swarm bridge") }

        kotlin.runCatching { meshService?.stop() }
            .onFailure { Timber.w(it, "Failed to stop Rust mesh service") }
        identitySyncSentPeers.clear()
        identityEmissionCache.clear()
        connectedEmissionCache.clear()

        // Clear references to avoid stale lifecycle state on next start.
        coreDelegate = null
        swarmBridge = null
        ironCore = null
        meshService = null
        bleScanner = null
        bleAdvertiser = null
        bleGattServer = null
        bleGattClient = null
        wifiTransportManager = null

        _serviceState.value = uniffi.api.ServiceState.STOPPED
        _serviceStats.value = null

        Timber.i("Mesh service stopped")
    }

    /**
     * Pause the mesh service (reduced activity).
     */
    fun pauseMeshService() {
        meshService?.pause()
        Timber.d("Mesh service paused")
    }

    /**
     * Resume the mesh service (full activity).
     */
    fun resumeMeshService() {
        meshService?.resume()
        Timber.d("Mesh service resumed")
    }

    /**
     * Get current service state.
     */
    fun getServiceState(): uniffi.api.ServiceState {
        return meshService?.getState() ?: uniffi.api.ServiceState.STOPPED
    }

    /**
     * Update and emit service stats.
     */
    private fun updateStats() {
        try {
            val stats = meshService?.getStats()
            if (stats != null) {
                // Combine swarm stats with our own discovery map
                val discovered = _discoveredPeers.value
                val fullCount = discovered.values.count { it.isFull }
                val headlessCount = discovered.values.count { !it.isFull }
                
                // We use a custom stats object or just update the one from core
                // For now, let's keep the core one but maybe log the detailed count
                Timber.d("Mesh Stats: ${stats.peersDiscovered} peers (Core), $fullCount full, $headlessCount headless (Repo)")
                
                _serviceStats.value = stats
                
                // Emit event for UI
                repoScope.launch {
                    com.scmessenger.android.service.MeshEventBus.emitStatusEvent(
                        com.scmessenger.android.service.StatusEvent.StatsUpdated(stats)
                    )
                }
            }
        } catch (e: Exception) {
            Timber.e(e, "Failed to get service stats")
        }
    }

    private fun startPeriodicStatsUpdate() {
        repoScope.launch {
            while (isActive) {
                kotlinx.coroutines.delay(5000) // Update every 5 seconds
                updateStats()
            }
        }
    }

    // ========================================================================
    // CONTACTS
    // ========================================================================

    fun addContact(contact: uniffi.api.Contact) {
        contactManager?.add(contact)
        val routing = parseRoutingHints(contact.notes)
        annotateIdentityInLedger(
            routePeerId = routing.libp2pPeerId,
            listeners = routing.listeners,
            publicKey = contact.publicKey,
            nickname = contact.nickname
        )
        Timber.d("Contact added: ${contact.peerId}")
    }

    fun getContact(peerId: String): uniffi.api.Contact? {
        return contactManager?.get(peerId)
    }

    fun removeContact(peerId: String) {
        contactManager?.remove(peerId)
        try {
            historyManager?.removeConversation(peerId)
        } catch (e: Exception) {
            Timber.w("Failed to remove conversation history for $peerId: ${e.message}")
        }
        Timber.d("Contact removed: $peerId and their message history")
    }

    fun listContacts(): List<uniffi.api.Contact> {
        return contactManager?.list() ?: emptyList()
    }

    fun searchContacts(query: String): List<uniffi.api.Contact> {
        return contactManager?.search(query) ?: emptyList()
    }

    fun setContactNickname(peerId: String, nickname: String?) {
        contactManager?.setNickname(peerId, nickname)
        Timber.d("Contact nickname updated: $peerId -> $nickname")
    }

    fun getContactCount(): UInt {
        return contactManager?.count() ?: 0u
    }

    // ========================================================================
    // MESSAGE HISTORY
    // ========================================================================

    fun getIdentityInfo(): uniffi.api.IdentityInfo? {
        ensureServiceInitialized()
        kotlin.runCatching { ensureLocalIdentityFederation() }
            .onFailure { Timber.w(it, "Failed to hydrate identity before getIdentityInfo") }
        return ironCore?.getIdentityInfo()
    }

    fun setNickname(nickname: String) {
        val trimmed = nickname.trim()
        if (trimmed.isEmpty()) {
            Timber.w("Refusing to set blank nickname")
            return
        }
        ironCore?.setNickname(trimmed)
        Timber.i("Nickname set to: $trimmed")
        persistIdentityBackup(ironCore)
        // If swarm start was postponed before identity/nickname was ready, resume now.
        initializeAndStartSwarm()
        updateBleIdentityBeacon()
        identitySyncSentPeers.clear()
        val connectedPeers = try {
            swarmBridge?.getPeers().orEmpty()
        } catch (e: Exception) {
            Timber.d("Unable to enumerate peers for nickname sync: ${e.message}")
            emptyList()
        }
        connectedPeers.forEach { routePeerId ->
            sendIdentitySyncIfNeeded(routePeerId = routePeerId)
        }
    }

    fun setLocalNickname(peerId: String, nickname: String?) {
        try {
            contactManager?.setLocalNickname(peerId, nickname)
            Timber.i("Local nickname for $peerId set to: $nickname")
            // Refresh discovery map if this peer is in it
                _discoveredPeers.update { current ->
                    val existing = current[peerId]
                    if (existing != null) {
                        current + (peerId to existing.copy(localNickname = nickname))
                    } else {
                        current
                    }
                }
        } catch (e: Exception) {
            Timber.e(e, "Failed to set local nickname for $peerId")
        }
    }

    suspend fun sendMessage(peerId: String, content: String) {
        kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
            try {
                // Check if relay/messaging is enabled (bidirectional control)
                // Treat null/missing settings as disabled (fail-safe)
                // Cache settings value to avoid race condition during check
                val currentSettings = settingsManager?.load()
                Companion.requireMeshParticipationEnabled(currentSettings)

                // 1. Get recipient's public key
                val contact = contactManager?.get(peerId)
                    ?: throw IllegalStateException("Contact not found for peer: $peerId")

                val publicKey = contact.publicKey.trim()

                // Pre-validate public key to provide descriptive errors
                if (publicKey.isEmpty()) {
                    throw IllegalStateException("Contact $peerId has no public key. Please re-add this contact with a valid public key.")
                }
                if (publicKey.length != 64) {
                    throw IllegalStateException("Contact $peerId has invalid public key (length: ${publicKey.length}, expected 64 hex chars). Please re-add this contact.")
                }
                if (!publicKey.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' }) {
                    throw IllegalStateException("Contact $peerId has invalid public key (non-hex characters). Please re-add this contact.")
                }

                Timber.d("Preparing message for $peerId with key: ${publicKey.take(8)}...")
                val routingHints = parseRoutingHints(contact.notes)
                val routePeerCandidates = buildRoutePeerCandidates(
                    peerId = peerId,
                    cachedRoutePeerId = routingHints.libp2pPeerId,
                    notes = contact.notes
                )
                if (routePeerCandidates.any { isBootstrapRelayPeer(it) } || isBootstrapRelayPeer(peerId)) {
                    throw IllegalStateException("Refusing to use bootstrap relay identity as a chat recipient: $peerId")
                }
                val preferredRoutePeerId = routePeerCandidates.firstOrNull()
                // 2. Encrypt/Prepare message (use trimmed key)
                val outboundContent = encodeMessageWithIdentityHints(content)
                val prepared = ironCore?.prepareMessageWithId(publicKey, outboundContent)
                    ?: throw IllegalStateException("Failed to prepare message: IronCore not initialized")
                val messageId = prepared.messageId.trim()
                if (messageId.isBlank()) {
                    throw IllegalStateException("Failed to prepare message: core returned empty message ID")
                }
                val encryptedData = prepared.envelopeData

                // 3. Save to history first so content survives transient route failures.
                val record = uniffi.api.MessageRecord(
                    id = messageId,
                    peerId = peerId,
                    direction = uniffi.api.MessageDirection.SENT,
                    content = content,
                    timestamp = (System.currentTimeMillis() / 1000).toULong(),
                    delivered = false // Will be updated on direct delivery ACK or receipt
                )
                historyManager?.add(record)

                // Emit for UI updates (e.g., chat list)
                repoScope.launch {
                    _messageUpdates.emit(record)
                }

                // 4. Send over core-selected swarm route only.
                // Mobile apps provide identity/routing hints; Rust core owns path selection.
                val delivery = attemptDirectSwarmDelivery(
                    routePeerCandidates = routePeerCandidates,
                    listeners = routingHints.listeners,
                    encryptedData = encryptedData,
                    blePeerId = routingHints.blePeerId
                )
                val selectedRoutePeerId = delivery.routePeerId ?: preferredRoutePeerId

                if (delivery.acked) {
                    enqueuePendingOutbound(
                        historyRecordId = messageId,
                        peerId = peerId,
                        routePeerId = selectedRoutePeerId,
                        listeners = routingHints.listeners,
                        encryptedData = encryptedData,
                        initialAttemptCount = 1,
                        initialDelaySec = receiptAwaitSeconds
                    )
                } else {
                    enqueuePendingOutbound(
                        historyRecordId = messageId,
                        peerId = peerId,
                        routePeerId = selectedRoutePeerId,
                        listeners = routingHints.listeners,
                        encryptedData = encryptedData,
                        initialAttemptCount = 1,
                        initialDelaySec = 0
                    )
                }

                Timber.i("Message sent (encrypted) to $peerId")
            } catch (e: Exception) {
                Timber.e(e, "Failed to send message")
                throw e
            }
        }
    }

    suspend fun dial(multiaddr: String) {
        kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
            try {
                // Attempt Swarm Dial
                swarmBridge?.dial(multiaddr)
                Timber.i("Dialed $multiaddr via SwarmBridge")
            } catch (e: Exception) {
                Timber.e(e, "Failed to dial $multiaddr")
                throw e
            }
        }
    }

    suspend fun dialPeer(multiaddr: String) = dial(multiaddr)

    // Identity Management
    fun isIdentityInitialized(): Boolean {
        ensureServiceInitialized()
        return ironCore?.getIdentityInfo()?.initialized == true
    }

    fun hasRequiredRuntimePermissions(): Boolean = hasAllPermissions(Permissions.required)

    @Synchronized
    fun onRuntimePermissionsGranted() {
        if (meshService?.getState() != uniffi.api.ServiceState.RUNNING) {
            Timber.d("Permission refresh skipped: mesh service is not running")
            return
        }
        kotlin.runCatching { initializeAndStartBle() }
            .onFailure { Timber.w(it, "BLE transport failed to start after permission grant") }
        kotlin.runCatching { initializeAndStartWifi() }
            .onFailure { Timber.w(it, "WiFi transport failed to start after permission grant") }
        kotlin.runCatching { initializeAndStartSwarm() }
            .onFailure { Timber.w(it, "Swarm transport failed to refresh after permission grant") }
    }

    suspend fun createIdentity() {
        kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
            try {
                ensureServiceInitialized()
                if (ironCore == null) {
                    Timber.e("IronCore is null after ensureServiceInitialized! Cannot create identity.")
                    throw IllegalStateException("Mesh service initialization failed")
                }
                Timber.d("Calling ironCore.initializeIdentity()...")
                ironCore?.initializeIdentity()
                ensureLocalIdentityFederation()
                persistIdentityBackup(ironCore)
                Timber.i("Identity created successfully")
                // Identity is now available; bring up internet transport if it was deferred.
                initializeAndStartSwarm()
                updateBleIdentityBeacon()
            } catch (e: Exception) {
                Timber.e(e, "Failed to create identity")
                throw e
            }
        }
    }

    // Helper to ensure service is initialized lazily
    private fun ensureServiceInitialized() {
        if (meshService == null || meshService?.getState() != uniffi.api.ServiceState.RUNNING) {
            Timber.d("Lazy starting MeshService for Identity access...")
            try {
                // If service not running, launch it properly
                val settings = loadSettings()
                val config = uniffi.api.MeshServiceConfig(
                    discoveryIntervalMs = 30000u,
                    batteryFloorPct = settings.batteryFloor
                )
                startMeshService(config)

                Timber.d("MeshService started lazily")
            } catch (e: Exception) {
                Timber.e(e, "Failed to start MeshService lazily")
            }
        }

        // Refresh ironCore reference just in case
        if (ironCore == null) {
            ironCore = meshService?.getCore()
            Timber.d("IronCore reference refreshed: ${ironCore != null}")
        }
        ensureLocalIdentityFederation()
    }

    private fun hasAllPermissions(permissions: List<String>): Boolean =
        permissions.all { permission ->
            ContextCompat.checkSelfPermission(context, permission) == PackageManager.PERMISSION_GRANTED
        }

    // Keep legacy addMessage for receiving side or manual adds
    fun addMessage(record: uniffi.api.MessageRecord) {
        historyManager?.add(record)
    }

    fun getMessage(id: String): uniffi.api.MessageRecord? {
        return historyManager?.get(id)
    }

    fun getRecentMessages(peerFilter: String? = null, limit: UInt = 50u): List<uniffi.api.MessageRecord> {
        return historyManager?.recent(peerFilter, limit) ?: emptyList()
    }

    fun getConversation(peerId: String, limit: UInt = 100u): List<uniffi.api.MessageRecord> {
        return historyManager?.conversation(peerId, limit) ?: emptyList()
    }

    fun searchMessages(query: String, limit: UInt = 50u): List<uniffi.api.MessageRecord> {
        return historyManager?.search(query, limit) ?: emptyList()
    }

    fun markMessageDelivered(id: String) {
        historyManager?.markDelivered(id)
        removePendingOutbound(id)
    }

    fun clearHistory() {
        historyManager?.clear()
        Timber.i("Message history cleared")
    }

    fun clearConversation(peerId: String) {
        historyManager?.clearConversation(peerId)
        Timber.i("Conversation cleared: $peerId")
    }

    fun getHistoryStats(): uniffi.api.HistoryStats? {
        return historyManager?.stats()
    }

    fun getMessageCount(): UInt {
        return historyManager?.count() ?: 0u
    }

    /**
     * Resets all application data, including identity, contacts, history, and preferences.
     * WARNING: This is destructive and permanent.
     */
    fun resetAllData() {
        Timber.w("RESETTING ALL APPLICATION DATA")

        // 1. Stop all active services and release UniFFI objects
        swarmBridge?.shutdown()
        swarmBridge = null

        meshService?.stop()
        meshService = null

        ironCore?.stop()
        ironCore = null

        contactManager = null
        historyManager = null
        ledgerManager = null
        settingsManager = null
        autoAdjustEngine = null

        // 2. Clear identity backup SharedPreferences
        identityBackupPrefs.edit().clear().apply()

        // 3. Delete all files in storage path (Rust DBs, keys, etc)
        val files = context.filesDir.listFiles()
        files?.forEach { file ->
            if (file.isDirectory) {
                file.deleteRecursively()
            } else {
                file.delete()
            }
        }

        Timber.i("All application data reset successfully")
    }

    // ========================================================================
    // LEDGER
    // ========================================================================

    fun recordConnection(multiaddr: String, peerId: String) {
        ledgerManager?.recordConnection(multiaddr, peerId)
    }

    fun recordConnectionFailure(multiaddr: String) {
        ledgerManager?.recordFailure(multiaddr)
    }

    fun getDialableAddresses(): List<uniffi.api.LedgerEntry> {
        return ledgerManager?.dialableAddresses() ?: emptyList()
    }

    fun getDialHintsForRoutePeer(routePeerId: String): List<String> {
        val normalizedRoute = routePeerId.trim()
        if (!isLibp2pPeerId(normalizedRoute)) return emptyList()
        val fromLedger = ledgerManager
            ?.dialableAddresses()
            ?.asSequence()
            ?.filter { it.peerId == normalizedRoute }
            ?.map { it.multiaddr }
            ?.toList()
            .orEmpty()
        return buildDialCandidatesForPeer(
            routePeerId = normalizedRoute,
            rawAddresses = fromLedger,
            includeRelayCircuits = true
        )
    }

    fun replayDiscoveredPeerEvents() {
        repoScope.launch {
            val snapshot = _discoveredPeers.value
            if (snapshot.isEmpty()) return@launch

            val aggregates = linkedMapOf<String, ReplayDiscoveredIdentity>()

            snapshot.forEach { (mapKey, info) ->
                val canonicalPeerId = info.peerId.trim().ifEmpty { mapKey.trim() }
                if (canonicalPeerId.isEmpty()) return@forEach

                val normalizedKey = normalizePublicKey(info.publicKey)
                val aggregateKey = normalizedKey ?: canonicalPeerId
                val routeCandidate = when {
                    isLibp2pPeerId(mapKey) -> mapKey
                    isLibp2pPeerId(canonicalPeerId) -> canonicalPeerId
                    else -> null
                }
                val discoveredNickname = prepopulateDiscoveryNickname(
                    nickname = info.nickname,
                    peerId = canonicalPeerId,
                    publicKey = normalizedKey
                )

                val existing = aggregates[aggregateKey]
                if (existing == null) {
                    aggregates[aggregateKey] = ReplayDiscoveredIdentity(
                        canonicalPeerId = canonicalPeerId,
                        publicKey = normalizedKey,
                        nickname = discoveredNickname,
                        localNickname = info.localNickname,
                        routePeerId = routeCandidate,
                        transport = info.transport
                    )
                } else {
                    if (existing.publicKey.isNullOrBlank() && !normalizedKey.isNullOrBlank()) {
                        existing.publicKey = normalizedKey
                    }
                    existing.canonicalPeerId = selectCanonicalPeerId(canonicalPeerId, existing.canonicalPeerId)
                    existing.nickname = selectAuthoritativeNickname(discoveredNickname, existing.nickname)
                    if (existing.localNickname.isNullOrBlank() && !info.localNickname.isNullOrBlank()) {
                        existing.localNickname = info.localNickname
                    }
                    if (existing.routePeerId.isNullOrBlank() && !routeCandidate.isNullOrBlank()) {
                        existing.routePeerId = routeCandidate
                    }
                    if (existing.transport != com.scmessenger.android.service.TransportType.INTERNET &&
                        info.transport == com.scmessenger.android.service.TransportType.INTERNET
                    ) {
                        existing.transport = info.transport
                    }
                }
            }

            aggregates.values.forEach { peer ->
                val listeners = peer.routePeerId?.let(::getDialHintsForRoutePeer).orEmpty()
                val publicKey = peer.publicKey
                if (!publicKey.isNullOrBlank()) {
                    emitIdentityDiscoveredIfChanged(
                        peerId = peer.canonicalPeerId,
                        publicKey = publicKey,
                        nickname = peer.nickname,
                        libp2pPeerId = peer.routePeerId,
                        listeners = listeners
                    )
                } else {
                    com.scmessenger.android.service.MeshEventBus.emitPeerEvent(
                        com.scmessenger.android.service.PeerEvent.Discovered(
                            peerId = peer.canonicalPeerId,
                            transport = peer.transport
                        )
                    )
                }
            }
        }
    }

    fun getAllKnownTopics(): List<String> {
        return ledgerManager?.allKnownTopics() ?: emptyList()
    }

    fun getLedgerSummary(): String {
        return ledgerManager?.summary() ?: "Ledger not available"
    }

    fun getConnectionPathState(): uniffi.api.ConnectionPathState {
        return try {
            meshService?.getConnectionPathState() ?: uniffi.api.ConnectionPathState.DISCONNECTED
        } catch (e: Exception) {
            Timber.w(e, "Failed to read connection path state")
            uniffi.api.ConnectionPathState.DISCONNECTED
        }
    }

    fun getNatStatus(): String {
        return try {
            meshService?.getNatStatus() ?: "unknown"
        } catch (e: Exception) {
            Timber.w(e, "Failed to read NAT status")
            "unknown"
        }
    }

    fun exportDiagnostics(): String {
        val coreDiagnostics = try {
            meshService?.exportDiagnostics()
        } catch (e: Exception) {
            Timber.w(e, "Failed to export core diagnostics")
            null
        }

        if (!coreDiagnostics.isNullOrBlank()) {
            return coreDiagnostics
        }

        val fallback = org.json.JSONObject()
            .put("service_state", meshService?.getState()?.name ?: "STOPPED")
            .put("connection_path_state", getConnectionPathState().name)
            .put("nat_status", getNatStatus())
            .put("discovered_peers", _discoveredPeers.value.size)
            .put("pending_outbox", loadPendingOutbox().size)
            .put("generated_at_ms", System.currentTimeMillis())
        return fallback.toString()
    }

    fun saveLedger() {
        ledgerManager?.save()
    }

    private val identityEmissionCache = java.util.concurrent.ConcurrentHashMap<String, Pair<IdentityEmissionSignature, Long>>()
    private val identityReemitIntervalMs = 15_000L
    private val connectedEmissionCache = java.util.concurrent.ConcurrentHashMap<String, Long>()
    private val connectedReemitIntervalMs = 15_000L

    private suspend fun emitIdentityDiscoveredIfChanged(
        peerId: String,
        publicKey: String,
        nickname: String?,
        libp2pPeerId: String?,
        listeners: List<String>,
        blePeerId: String? = null
    ) {
        val canonicalPeerId = peerId.trim()
        val normalizedKey = normalizePublicKey(publicKey)
        if (canonicalPeerId.isEmpty() || normalizedKey.isNullOrBlank()) {
            return
        }

        val normalizedRoute = libp2pPeerId?.trim()?.takeIf { it.isNotEmpty() }
        val normalizedBle = blePeerId?.trim()?.takeIf { it.isNotEmpty() }
        val normalizedNickname = normalizeNickname(nickname)
        val normalizedListeners = listeners
            .asSequence()
            .map { it.trim() }
            .filter { it.isNotEmpty() }
            .distinct()
            .sorted()
            .toList()
        val signature = IdentityEmissionSignature(
            canonicalPeerId = canonicalPeerId,
            publicKey = normalizedKey,
            nickname = normalizedNickname,
            libp2pPeerId = normalizedRoute,
            blePeerId = normalizedBle
        )
        val cacheKey = "$canonicalPeerId|$normalizedKey"
        val now = System.currentTimeMillis()
        val previous = identityEmissionCache[cacheKey]
        if (previous != null && previous.first == signature && (now - previous.second) < identityReemitIntervalMs) {
            return
        }
        identityEmissionCache[cacheKey] = signature to now

        com.scmessenger.android.service.MeshEventBus.emitPeerEvent(
            com.scmessenger.android.service.PeerEvent.IdentityDiscovered(
                peerId = canonicalPeerId,
                publicKey = normalizedKey,
                nickname = normalizedNickname,
                libp2pPeerId = normalizedRoute,
                listeners = normalizedListeners,
                blePeerId = normalizedBle
            )
        )
    }

    private suspend fun emitConnectedIfChanged(
        peerId: String,
        transport: com.scmessenger.android.service.TransportType
    ) {
        val normalizedPeerId = peerId.trim()
        if (normalizedPeerId.isEmpty()) return

        val now = System.currentTimeMillis()
        val lastEmitted = connectedEmissionCache[normalizedPeerId]
        if (lastEmitted != null && (now - lastEmitted) < connectedReemitIntervalMs) {
            return
        }
        connectedEmissionCache[normalizedPeerId] = now
        com.scmessenger.android.service.MeshEventBus.emitPeerEvent(
            com.scmessenger.android.service.PeerEvent.Connected(
                normalizedPeerId,
                transport
            )
        )
    }

    // ========================================================================
    // SETTINGS
    // ========================================================================

    fun loadSettings(): uniffi.api.MeshSettings {
        val loaded = try {
            settingsManager?.load()
        } catch (e: Exception) {
            Timber.w("Settings load failed (likely first run), using defaults: ${e.message}")
            null
        }

        return loaded ?: settingsManager?.defaultSettings()
            ?: uniffi.api.MeshSettings(
                relayEnabled = true,
                maxRelayBudget = 200u,
                batteryFloor = 20u,
                bleEnabled = true,
                wifiAwareEnabled = true,
                wifiDirectEnabled = true,
                internetEnabled = true,
                discoveryMode = uniffi.api.DiscoveryMode.NORMAL,
                onionRouting = false,
                coverTrafficEnabled = false,
                messagePaddingEnabled = false,
                timingObfuscationEnabled = false
            )
    }

    fun saveSettings(settings: uniffi.api.MeshSettings) {
        settingsManager?.save(settings)
        Timber.i("Settings saved")
    }

    fun validateSettings(settings: uniffi.api.MeshSettings): Boolean {
        return try {
            settingsManager?.validate(settings)
            true
        } catch (e: Exception) {
            Timber.w(e, "Settings validation failed")
            false
        }
    }

    // ========================================================================
    // AUTO-ADJUST ENGINE
    // ========================================================================

    fun computeAdjustmentProfile(deviceProfile: uniffi.api.DeviceProfile): uniffi.api.AdjustmentProfile {
        return autoAdjustEngine?.computeProfile(deviceProfile)
            ?: uniffi.api.AdjustmentProfile.STANDARD
    }

    fun computeBleAdjustment(profile: uniffi.api.AdjustmentProfile): uniffi.api.BleAdjustment {
        return autoAdjustEngine?.computeBleAdjustment(profile)
            ?: uniffi.api.BleAdjustment(
                scanIntervalMs = 2000u,
                advertiseIntervalMs = 500u,
                txPowerDbm = -4
            )
    }

    fun computeRelayAdjustment(profile: uniffi.api.AdjustmentProfile): uniffi.api.RelayAdjustment {
        return autoAdjustEngine?.computeRelayAdjustment(profile)
            ?: uniffi.api.RelayAdjustment(
                maxPerHour = 200u,
                priorityThreshold = 100u,
                maxPayloadBytes = 16384u
            )
    }

    fun overrideBleInterval(intervalMs: UInt) {
        autoAdjustEngine?.overrideBleScanInterval(intervalMs)
    }

    fun setRelayBudget(messagesPerHour: UInt) {
        meshService?.setRelayBudget(messagesPerHour)
    }

    fun updateDeviceState(profile: uniffi.api.DeviceProfile) {
        meshService?.updateDeviceState(profile)
    }

    fun overrideRelayMax(max: UInt) {
        autoAdjustEngine?.overrideRelayMaxPerHour(max)
    }

    fun clearAdjustmentOverrides() {
        autoAdjustEngine?.clearOverrides()
    }

    /**
     * Connect to a peer using provided addresses.
     */
    // ========================================================================
    // SWARM BRIDGE DELEGATIONS
    // ========================================================================

    /**
     * Get list of currently subscribed gossipsub topics from SwarmBridge.
     */
    fun getTopics(): List<String> = swarmBridge?.getTopics() ?: emptyList()

    /**
     * Subscribe to a gossipsub topic via SwarmBridge.
     */
    fun subscribeTopic(topic: String) {
        try {
            swarmBridge?.subscribeTopic(topic)
        } catch (e: Exception) {
            Timber.w("subscribeTopic failed for $topic: ${e.message}")
        }
    }

    fun unsubscribeTopic(topic: String) {
        try {
            swarmBridge?.unsubscribeTopic(topic)
        } catch (e: Exception) {
            Timber.w("unsubscribeTopic failed for $topic: ${e.message}")
        }
    }

    fun publishTopic(topic: String, data: ByteArray) {
        try {
            swarmBridge?.publishTopic(topic, data)
        } catch (e: Exception) {
            Timber.w("publishTopic failed for $topic: ${e.message}")
        }
    }

    /**
     * Broadcast data to all connected peers via SwarmBridge.
     */
    fun sendToAllPeers(data: ByteArray) {
        try {
            swarmBridge?.sendToAllPeers(data)
        } catch (e: Exception) {
            Timber.w("sendToAllPeers failed: ${e.message}")
        }
    }

    fun connectToPeer(peerId: String, addresses: List<String>) {
        val dialCandidates = buildDialCandidatesForPeer(
            routePeerId = peerId,
            rawAddresses = addresses,
            includeRelayCircuits = false
        )
        dialCandidates.forEach { addr ->
            try {
                // Only append /p2p/ component if peerId is a valid libp2p PeerId format
                // (base58btc multihash, starts with "12D3Koo" or "Qm").
                // Blake3 hex identity_ids (64 hex chars) are NOT valid libp2p PeerIds.
                val finalAddr = if (isLibp2pPeerId(peerId) && !addr.contains("/p2p/")) {
                    "$addr/p2p/$peerId"
                } else {
                    addr
                }
                if (shouldAttemptDial(finalAddr)) {
                    swarmBridge?.dial(finalAddr)
                    Timber.d("Dialing $finalAddr")
                }
            } catch (e: Exception) {
                Timber.e(e, "Failed to dial $addr")
            }
        }
    }

    private fun ensurePendingOutboxRetryLoop() {
        if (pendingOutboxRetryJob?.isActive == true) return

        pendingOutboxRetryJob = repoScope.launch {
            while (true) {
                try {
                    flushPendingOutbox("periodic")
                    kotlinx.coroutines.delay(5000)
                } catch (e: kotlinx.coroutines.CancellationException) {
                    throw e
                } catch (e: Exception) {
                    Timber.w(e, "Pending outbox retry loop error")
                    kotlinx.coroutines.delay(5000)
                }
            }
        }
    }

    /** Periodically broadcasts cover traffic when `coverTrafficEnabled` is true in settings. */
    private fun ensureCoverTrafficLoop() {
        if (coverTrafficJob?.isActive == true) return

        coverTrafficJob = repoScope.launch {
            while (true) {
                try {
                    kotlinx.coroutines.delay(30_000)
                    val enabled = try { settingsManager?.load()?.coverTrafficEnabled == true } catch (_: Exception) { false }
                    if (enabled) {
                        val core = ironCore
                        val bridge = swarmBridge
                        if (core != null && bridge != null) {
                            try {
                                val payload = core.prepareCoverTraffic(256u)
                                bridge.sendToAllPeers(payload)
                            } catch (e: Exception) {
                                Timber.d("Cover traffic send skipped: ${e.message}")
                            }
                        }
                    }
                } catch (e: kotlinx.coroutines.CancellationException) {
                    throw e
                } catch (e: Exception) {
                    Timber.w(e, "Cover traffic loop error")
                    kotlinx.coroutines.delay(30_000)
                }
            }
        }
    }

    private suspend fun attemptDirectSwarmDelivery(
        routePeerCandidates: List<String>,
        listeners: List<String>,
        encryptedData: ByteArray,
        blePeerId: String? = null
    ): DeliveryAttemptResult {
        val bridge = swarmBridge
            ?: return DeliveryAttemptResult(acked = false, routePeerId = routePeerCandidates.firstOrNull())
        val sanitizedCandidates = routePeerCandidates
            .map { it.trim() }
            .filter { it.isNotEmpty() && isLibp2pPeerId(it) && !isBootstrapRelayPeer(it) }
            .distinct()

        if (sanitizedCandidates.isEmpty()) {
            return DeliveryAttemptResult(acked = false, routePeerId = routePeerCandidates.firstOrNull())
        }

        primeRelayBootstrapConnections()

        for (routePeerId in sanitizedCandidates) {
            val dialCandidates = buildDialCandidatesForPeer(
                routePeerId = routePeerId,
                rawAddresses = listeners,
                includeRelayCircuits = true
            )
            if (dialCandidates.isNotEmpty()) {
                connectToPeer(routePeerId, dialCandidates)
                awaitPeerConnection(routePeerId)
            }

            try {
                bridge.sendMessage(routePeerId, encryptedData)
                Timber.i("✓ Direct delivery ACK from $routePeerId")
                return DeliveryAttemptResult(acked = true, routePeerId = routePeerId)
            } catch (e: Exception) {
                Timber.w("Core-routed delivery failed for $routePeerId: ${e.message}; trying alternative transports")
                
                // Fallback to BLE before trying explicit relay retry loop if available
                val bleAddr = blePeerId?.trim()?.takeIf { it.isNotEmpty() }
                if (bleAddr != null) {
                    val bleGatt = bleGattClient
                    if (bleGatt != null) {
                        try {
                            if (bleGatt.sendData(bleAddr, encryptedData)) {
                                Timber.i("✓ Delivery via BLE for $routePeerId (target=$bleAddr)")
                                return DeliveryAttemptResult(acked = true, routePeerId = routePeerId)
                            }
                        } catch (bleEx: Exception) {
                            Timber.d("BLE send failed: ${bleEx.message}")
                        }
                    }
                }
            }

            val relayOnlyCandidates = relayCircuitAddressesForPeer(routePeerId)
            if (relayOnlyCandidates.isNotEmpty()) {
                connectToPeer(routePeerId, relayOnlyCandidates)
                awaitPeerConnection(routePeerId)
                kotlinx.coroutines.delay(250)
                try {
                    bridge.sendMessage(routePeerId, encryptedData)
                    Timber.i("✓ Delivery ACK from $routePeerId after relay-circuit retry")
                    return DeliveryAttemptResult(acked = true, routePeerId = routePeerId)
                } catch (e: Exception) {
                    Timber.w("Relay-circuit retry failed for $routePeerId: ${e.message}")
                }
            }
        }
        return DeliveryAttemptResult(acked = false, routePeerId = sanitizedCandidates.firstOrNull())
    }

    private suspend fun awaitPeerConnection(peerId: String, timeoutMs: Long = 1200L): Boolean {
        val bridge = swarmBridge ?: return false
        val deadline = System.currentTimeMillis() + timeoutMs
        while (System.currentTimeMillis() < deadline) {
            val connected = try {
                bridge.getPeers().any { it == peerId }
            } catch (_: Exception) {
                false
            }
            if (connected) return true
            kotlinx.coroutines.delay(100)
        }
        return false
    }

    private suspend fun flushPendingOutbox(reason: String) {
        val now = System.currentTimeMillis() / 1000
        val queue = loadPendingOutbox().toMutableList()
        if (queue.isEmpty()) return
        Timber.d("Flushing pending outbox (${queue.size} item(s)); reason=$reason")

        var updated = false
        val iterator = queue.listIterator()
        while (iterator.hasNext()) {
            val item = iterator.next()
            if (item.nextAttemptAtEpochSec > now) continue
            val existing = historyManager?.get(item.historyRecordId)
            if (existing?.delivered == true) {
                iterator.remove()
                updated = true
                continue
            }

            val envelope = try {
                android.util.Base64.decode(item.envelopeBase64, android.util.Base64.NO_WRAP)
            } catch (_: Exception) {
                Timber.w("Dropping corrupt pending envelope ${item.queueId}")
                iterator.remove()
                updated = true
                continue
            }

            val contact = contactManager?.get(item.peerId)
            val latestRouting = parseRoutingHints(contact?.notes)
            val routePeerCandidates = buildRoutePeerCandidates(
                peerId = item.peerId,
                cachedRoutePeerId = item.routePeerId,
                notes = contact?.notes
            )
            val resolvedRoutePeerId = routePeerCandidates.firstOrNull()
            val resolvedListeners = buildDialCandidatesForPeer(
                routePeerId = resolvedRoutePeerId,
                rawAddresses = item.listeners + latestRouting.listeners,
                includeRelayCircuits = true
            )

            val delivery = attemptDirectSwarmDelivery(
                routePeerCandidates = routePeerCandidates,
                listeners = resolvedListeners,
                encryptedData = envelope
            )
            val selectedRoutePeerId = delivery.routePeerId ?: resolvedRoutePeerId

            if (delivery.acked) {
                iterator.set(
                    item.copy(
                        routePeerId = selectedRoutePeerId,
                        listeners = resolvedListeners,
                        attemptCount = item.attemptCount + 1,
                        nextAttemptAtEpochSec = now + receiptAwaitSeconds
                    )
                )
                updated = true
                continue
            }

            val nextAttemptCount = item.attemptCount + 1
            val backoffSecs = minOf(60L, 1L shl minOf(nextAttemptCount, 6))
            iterator.set(
                item.copy(
                    routePeerId = selectedRoutePeerId,
                    listeners = resolvedListeners,
                    attemptCount = nextAttemptCount,
                    nextAttemptAtEpochSec = now + backoffSecs
                )
            )
            updated = true
        }

        if (updated) {
            savePendingOutbox(queue)
        }
    }

    private fun enqueuePendingOutbound(
        historyRecordId: String,
        peerId: String,
        routePeerId: String?,
        listeners: List<String>,
        encryptedData: ByteArray,
        initialAttemptCount: Int = 0,
        initialDelaySec: Long = 0
    ) {
        val now = System.currentTimeMillis() / 1000
        val queue = loadPendingOutbox().toMutableList()
        queue.removeAll { it.historyRecordId == historyRecordId }
        queue.add(
            PendingOutboundEnvelope(
                queueId = java.util.UUID.randomUUID().toString(),
                historyRecordId = historyRecordId,
                peerId = peerId,
                routePeerId = routePeerId,
                listeners = listeners,
                envelopeBase64 = android.util.Base64.encodeToString(
                    encryptedData,
                    android.util.Base64.NO_WRAP
                ),
                createdAtEpochSec = now,
                attemptCount = initialAttemptCount,
                nextAttemptAtEpochSec = now + initialDelaySec
            )
        )
        savePendingOutbox(queue)
        repoScope.launch { flushPendingOutbox("enqueue") }
    }

    @Synchronized
    private fun loadPendingOutbox(): List<PendingOutboundEnvelope> {
        if (!pendingOutboxFile.exists()) return emptyList()
        return try {
            val raw = pendingOutboxFile.readText()
            if (raw.isBlank()) return emptyList()
            val arr = org.json.JSONArray(raw)
            buildList {
                for (i in 0 until arr.length()) {
                    val obj = arr.optJSONObject(i) ?: continue
                    val routePeerId = obj.optString("route_peer_id").ifBlank { null }
                    val listenersJson = obj.optJSONArray("listeners")
                    val listeners = buildList {
                        for (idx in 0 until (listenersJson?.length() ?: 0)) {
                            val value = listenersJson?.optString(idx).orEmpty().trim()
                            if (value.isNotEmpty()) add(value)
                        }
                    }
                    add(
                        PendingOutboundEnvelope(
                            queueId = obj.optString("queue_id").ifBlank { java.util.UUID.randomUUID().toString() },
                            historyRecordId = obj.optString("history_record_id"),
                            peerId = obj.optString("peer_id"),
                            routePeerId = routePeerId,
                            listeners = listeners,
                            envelopeBase64 = obj.optString("envelope_b64"),
                            createdAtEpochSec = obj.optLong("created_at", System.currentTimeMillis() / 1000),
                            attemptCount = obj.optInt("attempt_count", 0),
                            nextAttemptAtEpochSec = obj.optLong("next_attempt_at", 0)
                        )
                    )
                }
            }
        } catch (e: Exception) {
            Timber.w(e, "Failed to parse pending outbox")
            emptyList()
        }
    }

    @Synchronized
    private fun savePendingOutbox(queue: List<PendingOutboundEnvelope>) {
        try {
            val arr = org.json.JSONArray()
            queue.forEach { item ->
                val listeners = org.json.JSONArray()
                item.listeners.forEach { listeners.put(it) }
                arr.put(
                    org.json.JSONObject()
                        .put("queue_id", item.queueId)
                        .put("history_record_id", item.historyRecordId)
                        .put("peer_id", item.peerId)
                        .put("route_peer_id", item.routePeerId ?: "")
                        .put("listeners", listeners)
                        .put("envelope_b64", item.envelopeBase64)
                        .put("created_at", item.createdAtEpochSec)
                        .put("attempt_count", item.attemptCount)
                        .put("next_attempt_at", item.nextAttemptAtEpochSec)
                )
            }
            pendingOutboxFile.writeText(arr.toString())
        } catch (e: Exception) {
            Timber.w(e, "Failed to persist pending outbox")
        }
    }

    // ========================================================================
    // ROUTING HELPERS
    // ========================================================================

    private fun resolveCanonicalPeerId(senderId: String, senderPublicKeyHex: String): String {
        val normalizedIncomingKey = normalizePublicKey(senderPublicKeyHex) ?: return senderId
        val contacts = try {
            contactManager?.list().orEmpty()
        } catch (e: Exception) {
            Timber.d("Unable to resolve canonical sender ID: ${e.message}")
            return senderId
        }

        // Never rewrite an exact sender match.
        val exactMatch = contacts.any {
            it.peerId == senderId && normalizePublicKey(it.publicKey) == normalizedIncomingKey
        }
        if (exactMatch) return senderId

        // Prefer one stable canonical identity per public key whenever unique.
        val keyedContacts = contacts.filter {
            normalizePublicKey(it.publicKey) == normalizedIncomingKey
        }
        if (keyedContacts.size == 1) {
            return keyedContacts.first().peerId
        }
        if (keyedContacts.size > 1) {
            Timber.w(
                "Ambiguous canonical sender mapping for key ${normalizedIncomingKey.take(8)}... (matched ${keyedContacts.size} contacts); trying route-hint fallback"
            )
        }

        // Alias libp2p sender IDs only when there is explicit linkage via notes:
        // "libp2p_peer_id:<senderId>" and the key matches.
        if (isLibp2pPeerId(senderId)) {
            val linkedIdentityContacts = contacts.filter {
                normalizePublicKey(it.publicKey) == normalizedIncomingKey &&
                    parseRoutingHints(it.notes).libp2pPeerId == senderId &&
                    it.peerId != senderId
            }

            return when (linkedIdentityContacts.size) {
                1 -> linkedIdentityContacts.first().peerId
                0 -> senderId
                else -> {
                    Timber.w(
                        "Ambiguous canonical sender mapping for $senderId (matched ${linkedIdentityContacts.size} contacts); keeping raw sender ID"
                    )
                    senderId
                }
            }
        }

        // Identity IDs (Blake3 hex) can represent the same peer that was
        // previously saved under a libp2p contact ID; map only when unique.
        if (!isIdentityId(senderId)) return senderId
        val keyedRoutedContacts = contacts.filter {
            normalizePublicKey(it.publicKey) == normalizedIncomingKey &&
                it.peerId != senderId &&
                (
                    !parseRoutingHints(it.notes).libp2pPeerId.isNullOrBlank() ||
                    isLibp2pPeerId(it.peerId)
                )
        }
        return when (keyedRoutedContacts.size) {
            1 -> keyedRoutedContacts.first().peerId
            0 -> senderId
            else -> {
                Timber.w(
                    "Ambiguous identity sender mapping for $senderId (matched ${keyedRoutedContacts.size} contacts); keeping raw sender ID"
                )
                senderId
            }
        }
    }

    private fun resolveCanonicalPeerIdFromMessageHints(
        resolvedCanonicalPeerId: String,
        senderId: String,
        senderPublicKeyHex: String,
        hintedIdentityId: String?
    ): String {
        val normalizedHint = hintedIdentityId?.trim()?.takeIf { isIdentityId(it) } ?: return resolvedCanonicalPeerId
        if (normalizedHint == resolvedCanonicalPeerId) return resolvedCanonicalPeerId
        if (isBootstrapRelayPeer(normalizedHint)) return resolvedCanonicalPeerId

        val normalizedSenderKey = normalizePublicKey(senderPublicKeyHex)
        val contacts = try {
            contactManager?.list().orEmpty()
        } catch (_: Exception) {
            emptyList()
        }

        if (normalizedSenderKey != null) {
            val hintedContact = contacts.firstOrNull { it.peerId == normalizedHint }
            if (hintedContact != null && normalizePublicKey(hintedContact.publicKey) == normalizedSenderKey) {
                return normalizedHint
            }

            val keyMatches = contacts.filter { normalizePublicKey(it.publicKey) == normalizedSenderKey }
            if (keyMatches.size == 1) return keyMatches.first().peerId
            if (keyMatches.isNotEmpty()) return resolvedCanonicalPeerId
        }

        return if (resolvedCanonicalPeerId == senderId || isLibp2pPeerId(resolvedCanonicalPeerId)) {
            normalizedHint
        } else {
            resolvedCanonicalPeerId
        }
    }

    private fun encodeMessageWithIdentityHints(content: String): String {
        return encodeMeshMessagePayload(content = content, kind = "text")
    }

    private fun encodeIdentitySyncPayload(): String {
        return encodeMeshMessagePayload(content = "", kind = "identity_sync")
    }

    private fun encodeMeshMessagePayload(content: String, kind: String): String {
        val identity = ironCore?.getIdentityInfo() ?: return content
        val publicKeyHex = normalizePublicKey(identity.publicKeyHex) ?: return content

        val identityId = identity.identityId?.trim().orEmpty()
        val nickname = normalizeNickname(identity.nickname)?.take(64).orEmpty()
        val libp2pPeerId = identity.libp2pPeerId?.trim().orEmpty()
        val listeners = normalizeOutboundListenerHints(getListeningAddresses()).take(3)
        val externalAddresses = normalizeExternalAddressHints(getExternalAddresses()).take(3)
        val connectionHints = (listeners + externalAddresses).distinct().take(6)

        return try {
            org.json.JSONObject()
                .put("schema", "scm.message.identity.v1")
                .put("kind", kind)
                .put("text", content)
                .put(
                    "sender",
                    org.json.JSONObject()
                        .put("identity_id", identityId)
                        .put("public_key", publicKeyHex)
                        .put("nickname", nickname)
                        .put("libp2p_peer_id", libp2pPeerId)
                        .put("listeners", org.json.JSONArray(listeners))
                        .put("external_addresses", org.json.JSONArray(externalAddresses))
                        .put("connection_hints", org.json.JSONArray(connectionHints))
                )
                .toString()
        } catch (_: Exception) {
            content
        }
    }

    private fun decodeMessageWithIdentityHints(raw: String): DecodedMessagePayload {
        val trimmed = raw.trim()
        if (!trimmed.startsWith("{")) {
            return DecodedMessagePayload(kind = "text", text = raw, hints = null)
        }

        return try {
            val json = org.json.JSONObject(trimmed)
            if (json.optString("schema") != "scm.message.identity.v1") {
                return DecodedMessagePayload(kind = "text", text = raw, hints = null)
            }

            val sender = json.optJSONObject("sender")
            val hints = sender?.let {
                MessageIdentityHints(
                    identityId = it.optString("identity_id", "").trim().takeIf { value -> value.isNotBlank() },
                    publicKey = normalizePublicKey(it.optString("public_key", "")),
                    nickname = normalizeNickname(it.optString("nickname", "")),
                    libp2pPeerId = it.optString("libp2p_peer_id", "").trim().takeIf { value ->
                        value.isNotBlank() && isLibp2pPeerId(value)
                    },
                    listeners = jsonArrayToStringList(it.optJSONArray("listeners")),
                    externalAddresses = jsonArrayToStringList(it.optJSONArray("external_addresses")),
                    connectionHints = jsonArrayToStringList(it.optJSONArray("connection_hints"))
                )
            }

            val text = json.optString("text", raw)
            val kind = json.optString("kind", "text").trim().ifEmpty { "text" }
            DecodedMessagePayload(
                kind = kind,
                text = text,
                hints = hints
            )
        } catch (_: Exception) {
            DecodedMessagePayload(kind = "text", text = raw, hints = null)
        }
    }

    private fun jsonArrayToStringList(array: org.json.JSONArray?): List<String> {
        if (array == null) return emptyList()
        return buildList {
            for (i in 0 until array.length()) {
                val value = array.optString(i).trim()
                if (value.isNotEmpty()) add(value)
            }
        }.distinct()
    }

    private fun normalizePublicKey(value: String?): String? {
        val trimmed = value?.trim() ?: return null
        if (trimmed.length != 64) return null
        if (!trimmed.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' }) return null
        return trimmed.lowercase()
    }

    private fun normalizeNickname(value: String?): String? {
        return value?.trim()?.takeIf { it.isNotEmpty() }
    }

    private fun isSyntheticFallbackNickname(value: String?): Boolean {
        val normalized = normalizeNickname(value)?.lowercase() ?: return false
        // "peer-xxxxxx" is our receiver-side placeholder. It should never
        // overwrite a sender-provided nickname.
        return normalized.startsWith("peer-")
    }

    private fun selectAuthoritativeNickname(incoming: String?, existing: String?): String? {
        val incomingNormalized = normalizeNickname(incoming)
        val existingNormalized = normalizeNickname(existing)

        val incomingSynthetic = isSyntheticFallbackNickname(incomingNormalized)
        val existingSynthetic = isSyntheticFallbackNickname(existingNormalized)

        return when {
            incomingNormalized == null && existingSynthetic -> null
            incomingNormalized == null -> existingNormalized
            incomingSynthetic && existingNormalized == null -> null
            incomingSynthetic && existingSynthetic -> null
            incomingSynthetic -> existingNormalized
            existingSynthetic -> incomingNormalized
            else -> incomingNormalized
        }
    }

    private fun isBlePeerId(value: String): Boolean {
        return kotlin.runCatching { java.util.UUID.fromString(value.trim()) }.isSuccess
    }

    private fun selectCanonicalPeerId(incomingPeerId: String, existingPeerId: String): String {
        val incoming = incomingPeerId.trim()
        val existing = existingPeerId.trim()
        if (incoming.isEmpty()) return existing
        if (existing.isEmpty() || existing == incoming) return incoming

        val incomingIsLibp2p = isLibp2pPeerId(incoming)
        val existingIsLibp2p = isLibp2pPeerId(existing)
        val incomingIsIdentity = isIdentityId(incoming)
        val existingIsIdentity = isIdentityId(existing)
        val incomingIsBle = isBlePeerId(incoming)
        val existingIsBle = isBlePeerId(existing)

        return when {
            existingIsIdentity && incomingIsLibp2p -> existing
            incomingIsIdentity && existingIsLibp2p -> incoming
            existingIsBle && !incomingIsBle -> incoming
            !existingIsBle && incomingIsBle -> existing
            else -> incoming
        }
    }

    private fun prepopulateDiscoveryNickname(
        nickname: String?,
        peerId: String,
        publicKey: String?
    ): String? {
        val incomingNickname = normalizeNickname(nickname)

        val normalizedKey = normalizePublicKey(publicKey)
        val contacts = try {
            contactManager?.list().orEmpty()
        } catch (_: Exception) {
            emptyList()
        }

        val fromContact = contacts.firstOrNull {
            it.peerId == peerId ||
                (
                    normalizedKey != null &&
                        normalizePublicKey(it.publicKey) == normalizedKey
                    )
        }?.nickname
        return selectAuthoritativeNickname(incomingNickname, fromContact)
    }

    private fun resolveKnownPeerNickname(
        canonicalPeerId: String,
        routePeerId: String?,
        publicKey: String?
    ): String? {
        val normalizedKey = normalizePublicKey(publicKey)
        val routeCandidate = routePeerId?.trim()?.takeIf { it.isNotBlank() }

        val fromDiscovery = _discoveredPeers.value.values
            .asSequence()
            .filter { info ->
                info.peerId == canonicalPeerId ||
                    (!routeCandidate.isNullOrBlank() && info.peerId == routeCandidate) ||
                    (
                        normalizedKey != null &&
                            normalizePublicKey(info.publicKey) == normalizedKey
                        )
            }
            .mapNotNull { normalizeNickname(it.nickname) }
            .firstOrNull()

        val fromLedger = ledgerManager?.dialableAddresses()
            ?.asSequence()
            ?.filter { entry ->
                entry.peerId == canonicalPeerId ||
                    (!routeCandidate.isNullOrBlank() && entry.peerId == routeCandidate) ||
                    (
                        normalizedKey != null &&
                            normalizePublicKey(entry.publicKey) == normalizedKey
                        )
            }
            ?.mapNotNull { normalizeNickname(it.nickname) }
            ?.firstOrNull()

        val fromContact = try {
            contactManager?.list()
                ?.asSequence()
                ?.filter { contact ->
                    contact.peerId == canonicalPeerId ||
                        (!routeCandidate.isNullOrBlank() && contact.peerId == routeCandidate) ||
                        (
                            normalizedKey != null &&
                                normalizePublicKey(contact.publicKey) == normalizedKey
                            )
                }
                ?.mapNotNull { normalizeNickname(it.nickname) }
                ?.firstOrNull()
        } catch (_: Exception) {
            null
        }

        val discoveryOrLedger = selectAuthoritativeNickname(fromDiscovery, fromLedger)
        return selectAuthoritativeNickname(discoveryOrLedger, fromContact)
    }

    private fun annotateIdentityInLedger(
        routePeerId: String?,
        listeners: List<String>,
        publicKey: String?,
        nickname: String?
    ) {
        val normalizedRoute = routePeerId?.trim().orEmpty()
        if (normalizedRoute.isEmpty() || !isLibp2pPeerId(normalizedRoute)) return

        val dialHints = buildDialCandidatesForPeer(
            routePeerId = normalizedRoute,
            rawAddresses = listeners,
            includeRelayCircuits = true
        )
        if (dialHints.isEmpty()) return

        val normalizedKey = normalizePublicKey(publicKey)
        val normalizedNickname = nickname?.trim()?.takeIf { it.isNotEmpty() }
        dialHints.forEach { multiaddr ->
            kotlin.runCatching {
                ledgerManager?.annotateIdentity(
                    multiaddr,
                    normalizedRoute,
                    normalizedKey,
                    normalizedNickname
                )
            }.onFailure {
                Timber.d("Failed to annotate identity for ledger entry $multiaddr: ${it.message}")
            }
        }
    }

    private fun appendRoutingHint(notes: String?, key: String, value: String?): String? {
        val normalizedValue = value?.trim().orEmpty()
        if (normalizedValue.isEmpty()) return notes

        val existing = notes?.trim().orEmpty()
        val components = if (existing.isEmpty()) {
            emptyList()
        } else {
            existing.split(';', '\n').map { it.trim() }
        }
        val alreadyPresent = components.any { it.startsWith("$key:") && it.removePrefix("$key:").trim() == normalizedValue }
        if (alreadyPresent) return notes

        return listOfNotNull(
            existing.takeIf { it.isNotEmpty() },
            "$key:$normalizedValue"
        ).joinToString(";")
    }

    private fun resolveTransportIdentity(libp2pPeerId: String): TransportIdentityResolution? {
        if (!isLibp2pPeerId(libp2pPeerId)) return null

        val extractedKey = try {
            ironCore?.extractPublicKeyFromPeerId(libp2pPeerId)
        } catch (e: Exception) {
            Timber.d("Failed to extract public key from peer $libp2pPeerId: ${e.message}")
            null
        }
        val normalizedKey = normalizePublicKey(extractedKey) ?: return null

        val selfKey = normalizePublicKey(ironCore?.getIdentityInfo()?.publicKeyHex)
        if (selfKey != null && selfKey == normalizedKey) return null

        val contacts = try {
            contactManager?.list().orEmpty()
        } catch (e: Exception) {
            Timber.d("Unable to resolve transport identity for $libp2pPeerId: ${e.message}")
            emptyList()
        }

        val keyMatches = contacts.filter { normalizePublicKey(it.publicKey) == normalizedKey }
        if (keyMatches.size > 1) {
            Timber.w(
                "Multiple contacts share transport key ${normalizedKey.take(8)}...; using explicit route match where possible"
            )
        }

        val routeLinked = keyMatches.firstOrNull { contact ->
            contact.peerId == libp2pPeerId || parseRoutingHints(contact.notes).libp2pPeerId == libp2pPeerId
        }
        val canonicalContact = routeLinked ?: keyMatches.firstOrNull()

        return TransportIdentityResolution(
            canonicalPeerId = canonicalContact?.peerId ?: libp2pPeerId,
            publicKey = normalizedKey,
            nickname = canonicalContact?.nickname?.takeIf { it.isNotBlank() },
            localNickname = canonicalContact?.localNickname?.takeIf { it.isNotBlank() }
        )
    }

    private fun persistRouteHintsForTransportPeer(
        libp2pPeerId: String,
        listeners: List<String>,
        knownPublicKey: String? = null
    ) {
        val normalizedListeners = normalizeOutboundListenerHints(listeners)
        if (libp2pPeerId.isBlank()) return

        val normalizedTransportKey = knownPublicKey
            ?: normalizePublicKey(
                try {
                    ironCore?.extractPublicKeyFromPeerId(libp2pPeerId)
                } catch (e: Exception) {
                    Timber.d("Unable to derive public key for transport peer $libp2pPeerId: ${e.message}")
                    null
                }
            )

        val contacts = try {
            contactManager?.list().orEmpty()
        } catch (e: Exception) {
            Timber.d("Unable to update route hints for $libp2pPeerId: ${e.message}")
            return
        }

        contacts.forEach { contact ->
            val routing = parseRoutingHints(contact.notes)
            val isMatch = contact.peerId == libp2pPeerId ||
                routing.libp2pPeerId == libp2pPeerId ||
                (
                    normalizedTransportKey != null &&
                        normalizePublicKey(contact.publicKey) == normalizedTransportKey
                    )
            if (!isMatch) return@forEach

            val withPeerId = appendRoutingHint(contact.notes, "libp2p_peer_id", libp2pPeerId)
            val withListeners = upsertRoutingListeners(withPeerId, normalizedListeners)
            if (withListeners == contact.notes) return@forEach

            val updated = uniffi.api.Contact(
                peerId = contact.peerId,
                nickname = contact.nickname,
                localNickname = contact.localNickname,
                publicKey = contact.publicKey,
                addedAt = contact.addedAt,
                lastSeen = contact.lastSeen,
                notes = withListeners
            )
            try {
                contactManager?.add(updated)
            } catch (e: Exception) {
                Timber.d("Failed to persist route hints for ${contact.peerId}: ${e.message}")
            }
        }
    }

    private fun upsertFederatedContact(
        canonicalPeerId: String,
        publicKey: String,
        nickname: String?,
        libp2pPeerId: String?,
        listeners: List<String>,
        blePeerId: String? = null,
        createIfMissing: Boolean = true
    ) {
        val normalizedPeerId = canonicalPeerId.trim()
        val normalizedKey = normalizePublicKey(publicKey) ?: return
        if (normalizedPeerId.isEmpty()) return

        val routePeer = libp2pPeerId?.trim()?.takeIf { it.isNotEmpty() }
        if (!routePeer.isNullOrBlank() && isBootstrapRelayPeer(routePeer)) return

        val contacts = try {
            contactManager?.list().orEmpty()
        } catch (e: Exception) {
            Timber.d("Failed to list contacts for federation upsert: ${e.message}")
            return
        }

        val existingByKey = contacts.firstOrNull { normalizePublicKey(it.publicKey) == normalizedKey }
        val existingById = contacts.firstOrNull { it.peerId == normalizedPeerId }

        // Auth guard: for an existing canonical peerId, only accept federated updates
        // when the source public key is consistent with the stored contact key.
        if (existingById != null && normalizePublicKey(existingById.publicKey) != normalizedKey) {
            Timber.w(
                "Rejected federated nickname update for $normalizedPeerId: key mismatch " +
                    "(stored=${existingById.publicKey.take(8)}..., incoming=${normalizedKey.take(8)}...)"
            )
            return
        }
       
        // If we found them by key but they have a different PeerID now,
        // we should merge them to prevent duplicates.
        if (existingByKey != null && existingById != null && existingByKey.peerId != existingById.peerId) {
            Timber.i("Merging duplicate identities for key ${normalizedKey.take(8)}...: ${existingById.peerId} -> ${existingByKey.peerId}")
            try { contactManager?.remove(existingById.peerId) } catch (_: Exception) {}
        }
       
        val existing = existingByKey ?: existingById
        if (existing == null && !createIfMissing) {
            return
        }

        var notes = existing?.notes
        if (!routePeer.isNullOrBlank()) {
            notes = appendRoutingHint(notes = notes, key = "libp2p_peer_id", value = routePeer)
        }
        val normalizedBle = blePeerId?.trim()?.takeIf { it.isNotEmpty() }
        if (!normalizedBle.isNullOrBlank()) {
            notes = appendRoutingHint(notes = notes, key = "ble_peer_id", value = normalizedBle)
        }
        notes = upsertRoutingListeners(notes, normalizeOutboundListenerHints(listeners))

        val now = (System.currentTimeMillis() / 1000).toULong()
        val resolvedPeerId = existing?.peerId ?: normalizedPeerId
        val resolvedPublicKey = existingByKey?.publicKey ?: normalizedKey
        val incomingNickname = normalizeNickname(nickname)
        val resolvedNickname = selectAuthoritativeNickname(incomingNickname, existing?.nickname)
        val resolvedLocalNickname = normalizeNickname(existing?.localNickname)

        val updated = uniffi.api.Contact(
            peerId = resolvedPeerId,
            nickname = resolvedNickname,
            localNickname = resolvedLocalNickname,
            publicKey = resolvedPublicKey,
            addedAt = existing?.addedAt ?: now,
            lastSeen = now,
            notes = notes
        )

        try {
            contactManager?.add(updated)
        } catch (e: Exception) {
            Timber.d("Failed to upsert federated contact for $resolvedPeerId: ${e.message}")
        }

        annotateIdentityInLedger(
            routePeerId = routePeer,
            listeners = listeners,
            publicKey = resolvedPublicKey,
            nickname = resolvedNickname
        )
    }

    private fun upsertRoutingListeners(notes: String?, listeners: List<String>): String? {
        if (listeners.isEmpty()) return notes
        val base = notes?.trim().orEmpty()
        val filtered = base
            .split(';', '\n')
            .map { it.trim() }
            .filter { it.isNotEmpty() && !it.startsWith("listeners:") }
        return (filtered + "listeners:${listeners.joinToString(",")}").joinToString(";")
    }

    @Synchronized
    private fun removePendingOutbound(historyRecordId: String) {
        if (historyRecordId.isBlank()) return
        val queue = loadPendingOutbox().toMutableList()
        val removed = queue.removeAll { it.historyRecordId == historyRecordId }
        if (removed) savePendingOutbox(queue)
    }

    private fun parseRoutingHints(notes: String?): RoutingHints {
        if (notes.isNullOrEmpty()) {
            return RoutingHints(blePeerId = null, libp2pPeerId = null, listeners = emptyList())
        }

        var blePeerId: String? = null
        var peerId: String? = null
        var listeners: List<String> = emptyList()

        for (component in notes.split(';', '\n')) {
            val kv = component.trim()
            if (kv.startsWith("ble_peer_id:")) {
                val value = kv.removePrefix("ble_peer_id:").trim()
                blePeerId = value.ifEmpty { null }
            } else if (kv.startsWith("libp2p_peer_id:")) {
                val value = kv.removePrefix("libp2p_peer_id:").trim()
                peerId = value.ifEmpty { null }
            } else if (kv.startsWith("listeners:")) {
                val value = kv.removePrefix("listeners:").trim()
                if (value.isNotEmpty()) {
                    listeners = value
                        .split(",")
                        .map { it.trim() }
                        .filter { it.isNotEmpty() }
                }
            }
        }

        return RoutingHints(
            blePeerId = blePeerId,
            libp2pPeerId = peerId,
            listeners = listeners
        )
    }

    private fun parseAllRoutingPeerIds(notes: String?): List<String> {
        if (notes.isNullOrBlank()) return emptyList()
        val out = mutableListOf<String>()
        for (component in notes.split(';', '\n')) {
            val kv = component.trim()
            if (!kv.startsWith("libp2p_peer_id:")) continue
            val value = kv.removePrefix("libp2p_peer_id:").trim()
            if (value.isNotEmpty() && isLibp2pPeerId(value)) {
                out.add(value)
            }
        }
        return out.distinct()
    }

    private fun buildRoutePeerCandidates(
        peerId: String,
        cachedRoutePeerId: String?,
        notes: String?
    ): List<String> {
        val candidates = mutableListOf<String>()
        val notedPeerIds = parseAllRoutingPeerIds(notes)
        val newestHint = notedPeerIds.lastOrNull()
        if (!newestHint.isNullOrBlank()) candidates.add(newestHint)
        for (hint in notedPeerIds.asReversed()) candidates.add(hint)
        cachedRoutePeerId?.trim()?.takeIf { it.isNotEmpty() }?.let { candidates.add(it) }
        if (isLibp2pPeerId(peerId)) candidates.add(peerId)
        return candidates
            .map { it.trim() }
            .filter { it.isNotEmpty() && isLibp2pPeerId(it) }
            .distinct()
    }

    private fun isLibp2pPeerId(value: String): Boolean {
        return value.startsWith("12D3Koo") || value.startsWith("Qm")
    }

    private fun isIdentityId(value: String): Boolean {
        return value.length == 64 && value.all {
            (it in '0'..'9') || (it in 'a'..'f') || (it in 'A'..'F')
        }
    }

    private fun buildDialCandidatesForPeer(
        routePeerId: String?,
        rawAddresses: List<String>,
        includeRelayCircuits: Boolean
    ): List<String> {
        val normalized = rawAddresses
            .mapNotNull { normalizeAddressHint(it) }
            .distinct()
        val prioritized = prioritizeAddressesForCurrentNetwork(normalized)
        val relayCircuits = if (includeRelayCircuits && !routePeerId.isNullOrBlank() && isLibp2pPeerId(routePeerId)) {
            relayCircuitAddressesForPeer(routePeerId)
        } else {
            emptyList()
        }
        return (prioritized + relayCircuits).distinct()
    }

    private fun normalizeOutboundListenerHints(rawAddresses: List<String>): List<String> {
        return rawAddresses
            .mapNotNull { normalizeAddressHint(it) }
            .distinct()
    }

    private fun normalizeExternalAddressHints(rawAddresses: List<String>): List<String> {
        return rawAddresses
            .mapNotNull { normalizeAddressHint(it) }
            .distinct()
    }

    private fun normalizeAddressHint(raw: String): String? {
        val trimmed = raw.trim()
        if (trimmed.isEmpty()) return null

        val normalizedZeroAddr = if (trimmed.contains("/ip4/0.0.0.0/")) {
            val localIp = getLocalIpAddress() ?: return null
            trimmed.replace("/ip4/0.0.0.0/", "/ip4/$localIp/")
        } else {
            trimmed
        }

        val asMultiaddr = if (normalizedZeroAddr.startsWith("/")) {
            normalizedZeroAddr
        } else {
            toMultiaddrFromSocketAddress(normalizedZeroAddr) ?: return null
        }

        if (!isDialableAddress(asMultiaddr)) return null
        return asMultiaddr
    }

    private fun toMultiaddrFromSocketAddress(value: String): String? {
        val trimmed = value.trim()
        if (trimmed.isEmpty()) return null
        if (trimmed.startsWith("/")) return trimmed

        val separatorIdx = trimmed.lastIndexOf(':')
        if (separatorIdx <= 0 || separatorIdx >= trimmed.lastIndex) return null

        val host = trimmed.substring(0, separatorIdx).trim().removePrefix("[").removeSuffix("]")
        val port = trimmed.substring(separatorIdx + 1).toIntOrNull() ?: return null
        if (port !in 1..65535 || host.isEmpty()) return null

        val ipv4Regex = Regex("^\\d{1,3}(\\.\\d{1,3}){3}$")
        return when {
            host.contains(":") -> "/ip6/$host/tcp/$port"
            ipv4Regex.matches(host) -> "/ip4/$host/tcp/$port"
            else -> "/dns4/$host/tcp/$port"
        }
    }

    private fun isDialableAddress(multiaddr: String): Boolean {
        if (multiaddr.contains("/p2p-circuit")) return true

        val ip = extractIpv4FromMultiaddr(multiaddr) ?: return true
        if (ip == "0.0.0.0") return false
        if (ip.startsWith("127.")) return false
        if (ip.startsWith("169.254.")) return false

        return if (isPrivateIpv4(ip)) {
            isSameLanAddress(multiaddr)
        } else {
            true
        }
    }

    private fun isPrivateIpv4(ip: String): Boolean {
        val octets = ip.split('.').mapNotNull { it.toIntOrNull() }
        if (octets.size != 4) return false
        return (octets[0] == 10) ||
            (octets[0] == 172 && octets[1] in 16..31) ||
            (octets[0] == 192 && octets[1] == 168)
    }

    private fun relayCircuitAddressesForPeer(targetPeerId: String): List<String> {
        if (!isLibp2pPeerId(targetPeerId)) return emptyList()
        return DEFAULT_BOOTSTRAP_NODES.mapNotNull { bootstrap ->
            val relayInfo = parseBootstrapRelay(bootstrap) ?: return@mapNotNull null
            val (relayTransportAddr, relayPeerId) = relayInfo
            "$relayTransportAddr/p2p/$relayPeerId/p2p-circuit/p2p/$targetPeerId"
        }
    }

    private fun parseBootstrapRelay(multiaddr: String): Pair<String, String>? {
        val marker = "/p2p/"
        val idx = multiaddr.lastIndexOf(marker)
        if (idx <= 0) return null
        val transportAddr = multiaddr.substring(0, idx).trimEnd('/')
        val relayPeerId = multiaddr.substring(idx + marker.length).trim()
        if (transportAddr.isEmpty() || relayPeerId.isEmpty()) return null
        return transportAddr to relayPeerId
    }

    private fun isBootstrapRelayPeer(peerId: String): Boolean {
        if (peerId.isBlank()) return false
        return DEFAULT_BOOTSTRAP_NODES.any { addr ->
            parseBootstrapRelay(addr)?.second == peerId
        }
    }

    private fun primeRelayBootstrapConnections() {
        val bridge = swarmBridge ?: return
        val nowMs = System.currentTimeMillis()
        if (nowMs - lastRelayBootstrapDialMs < 10_000L) return
        lastRelayBootstrapDialMs = nowMs

        DEFAULT_BOOTSTRAP_NODES.forEach { addr ->
            try {
                if (!shouldAttemptDial(addr)) return@forEach
                bridge.dial(addr)
            } catch (e: Exception) {
                Timber.d("Relay bootstrap dial skipped for $addr: ${e.message}")
            }
        }
    }

    private fun prioritizeAddressesForCurrentNetwork(addresses: List<String>): List<String> {
        if (addresses.size <= 1) return addresses
        val lan = addresses.filter { isSameLanAddress(it) }
        if (lan.isEmpty()) return addresses
        return (lan + addresses.filterNot { it in lan }).distinct()
    }

    private fun isSameLanAddress(multiaddr: String): Boolean {
        val targetIp = extractIpv4FromMultiaddr(multiaddr) ?: return false
        val localIp = getLocalIpAddress() ?: return false
        return sameSubnet24(localIp, targetIp)
    }

    private fun extractIpv4FromMultiaddr(multiaddr: String): String? {
        val marker = "/ip4/"
        val start = multiaddr.indexOf(marker)
        if (start < 0) return null
        val rest = multiaddr.substring(start + marker.length)
        val end = rest.indexOf('/')
        return if (end >= 0) rest.substring(0, end) else rest
    }

    private fun sameSubnet24(ipA: String, ipB: String): Boolean {
        val a = ipA.split(".")
        val b = ipB.split(".")
        if (a.size != 4 || b.size != 4) return false
        return a[0] == b[0] && a[1] == b[1] && a[2] == b[2]
    }

    private fun shouldAttemptDial(multiaddr: String): Boolean {
        val key = multiaddr.trim()
        if (key.isEmpty()) return false

        val now = System.currentTimeMillis()
        val (attempts, nextAllowedMs) = dialThrottleState[key] ?: (0 to 0L)
        if (now < nextAllowedMs) {
            return false
        }

        val nextAttempts = (attempts + 1).coerceAtMost(8)
        val backoffMs = when (nextAttempts) {
            1 -> 500L
            2 -> 1_500L
            3 -> 3_000L
            4 -> 6_000L
            5 -> 10_000L
            else -> 15_000L
        }
        dialThrottleState[key] = nextAttempts to (now + backoffMs)
        return true
    }

    // ========================================================================
    // IDENTITY EXPORT HELPERS
    // ========================================================================    // MARK: - Identity Helpers

    fun getPreferredRelay(): String? {
        val relays = ledgerManager?.getPreferredRelays(1u)
        return relays?.firstOrNull()?.peerId
    }

    /**
     * Returns external NAT-mapped addresses observed by peer nodes on the mesh.
     * Uses relay/peer-confirmed observations (identify + reflection consensus).
     */
    fun getExternalAddresses(): List<String> {
        return swarmBridge?.getExternalAddresses() ?: emptyList()
    }

    /**
     * Returns local listener addresses (bound TCP ports on LAN interfaces).
     * External NAT-mapped addresses are intentionally excluded — they are observed
     * outbound ports, not stable inbound addresses, and including them causes remote
     * peers to attempt unreachable dials.
     */
    fun getListeningAddresses(): List<String> {
        return swarmBridge?.getListeners() ?: emptyList()
    }

    fun getLocalIpAddress(): String? {
        try {
            val interfaces = java.net.NetworkInterface.getNetworkInterfaces()
            while (interfaces.hasMoreElements()) {
                val networkInterface = interfaces.nextElement()
                val addresses = networkInterface.inetAddresses
                while (addresses.hasMoreElements()) {
                    val address = addresses.nextElement()
                    if (!address.isLoopbackAddress && address is java.net.Inet4Address) {
                        return address.hostAddress
                    }
                }
            }
        } catch (e: Exception) {
            timber.log.Timber.e(e, "Failed to get local IP")
        }
        return null
    }

    fun getIdentityExportString(): String {
        val identity = getIdentityInfo() ?: return "{}"
        var listeners = normalizeOutboundListenerHints(getListeningAddresses()).toMutableList()
        val externalAddresses = normalizeExternalAddressHints(getExternalAddresses())
        val relay = getPreferredRelay()
        val localIp = getLocalIpAddress()

        if (localIp != null) {
            listeners = listeners.map { addr ->
                if (addr.contains("0.0.0.0")) addr.replace("0.0.0.0", localIp) else addr
            }.toMutableList()
        }

        val payload = org.json.JSONObject()
            .put("identity_id", identity.identityId ?: "")
            .put("nickname", identity.nickname ?: "")
            .put("public_key", identity.publicKeyHex ?: "")
            .put("libp2p_peer_id", identity.libp2pPeerId ?: "")
            .put("listeners", org.json.JSONArray(listeners))
            .put("external_addresses", org.json.JSONArray(externalAddresses))
            .put("connection_hints", org.json.JSONArray((listeners + externalAddresses).distinct()))
            .put("relay", relay ?: "None")

        return payload.toString()
    }

    // ========================================================================
    // OBSERVABLES FOR UI (NEW)
    // ========================================================================

    /**
     * Observe incoming messages from MeshEventBus.
     */
    fun observeIncomingMessages(): kotlinx.coroutines.flow.Flow<com.scmessenger.android.service.MessageEvent> {
        return com.scmessenger.android.service.MeshEventBus.messageEvents
    }

    /**
     * Observe peer events from MeshEventBus.
     */
    fun observePeers(): kotlinx.coroutines.flow.Flow<List<String>> {
        return kotlinx.coroutines.flow.flow {
            com.scmessenger.android.service.MeshEventBus.peerEvents.collect { _ ->
                // Convert peer events to peer list
                kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
                    val peers = ledgerManager?.dialableAddresses()?.mapNotNull { it.peerId }?.distinct() ?: emptyList()
                    emit(peers)
                }
            }
        }
    }

    /**
     * Observe network stats with periodic refresh.
     */
    fun observeNetworkStats(): kotlinx.coroutines.flow.Flow<uniffi.api.ServiceStats> {
        return kotlinx.coroutines.flow.flow {
            while (true) {
                kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
                    val stats = meshService?.getStats()
                    if (stats != null) {
                        emit(stats)
                    }
                }
                kotlinx.coroutines.delay(2000) // Refresh every 2 seconds
            }
        }
    }

    // ========================================================================
    // CLEANUP
    // ========================================================================

    fun cleanup() {
        try {
            stopMeshService()
            saveLedger()
            pendingOutboxRetryJob?.cancel()
            pendingOutboxRetryJob = null
            coverTrafficJob?.cancel()
            coverTrafficJob = null

            // Clear references
            meshService = null
            contactManager = null
            historyManager = null
            ledgerManager = null
            settingsManager = null
            autoAdjustEngine = null

            Timber.i("MeshRepository cleaned up")
        } catch (e: Exception) {
            Timber.e(e, "Error during cleanup")
        }
    }
}
