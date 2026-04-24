package com.scmessenger.android.data

import android.content.Context
import android.content.pm.PackageManager
import android.content.SharedPreferences
import androidx.core.content.ContextCompat
import com.scmessenger.android.transport.NetworkDetector
import com.scmessenger.android.utils.Permissions
import com.scmessenger.android.utils.CircuitBreaker
import com.scmessenger.android.utils.NetworkFailureMetrics
import com.scmessenger.android.utils.PeerIdValidator
import com.scmessenger.android.utils.PeerKeyUtils
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.async
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicBoolean
import timber.log.Timber
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.cancel
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
open class MeshRepository(private val context: Context) {
    private val storagePath = context.filesDir.absolutePath
    private val networkFailureMetrics = NetworkFailureMetrics()
    private val transportHealthMonitor = com.scmessenger.android.transport.TransportHealthMonitor()
    private val retryBackoff = com.scmessenger.android.utils.BackoffStrategy()


    companion object {
        private const val IDENTITY_BACKUP_PREFS = "identity_backup_prefs"
        private const val IDENTITY_BACKUP_KEY = "identity_backup_v1"
        /** Static fallback bootstrap nodes for NAT traversal and internet roaming.
         *  These are used if env override and remote fetch both fail/are absent.
         *  Priority order: QUIC/UDP (cellular-friendly) → TCP (WiFi/enterprise).
         *
         *  QUIC is prioritized for cellular NAT traversal because many carriers
         *  block TCP on non-standard ports but allow UDP. The swarm automatically
         *  binds both TCP and QUIC listeners, so we advertise both endpoints.
         */
        private val STATIC_BOOTSTRAP_NODES: List<String> = listOf(
            // GCP relay — QUIC/UDP (cellular-friendly, primary)
            "/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw",
            // GCP relay — TCP (fallback for WiFi/enterprise networks)
            "/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw",
            // OSX home relay — QUIC/UDP (cellular-friendly)
            "/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9",
            // OSX home relay — TCP (fallback)
            "/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9"
        )

        /**
         * P0_NETWORK_001: WebSocket fallback bootstrap nodes on standard ports.
         * These bypass carrier-level port filtering on cellular networks
         * that block non-standard ports like 9001/9010.
         */
        private val WEBSOCKET_FALLBACK_NODES: List<String> = listOf(
            // WebSocket on standard HTTPS port (most likely to bypass filtering)
            "/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw",
            // WebSocket on standard HTTP port
            "/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw"
        )

        /**
         * P0_ANDROID_017: Cached static bootstrap nodes to prevent blocking network I/O on Settings ANR.
         * Lazy initialization of DEFAULT_BOOTSTRAP_NODES was causing UI thread blocking because:
         * 1. BootstrapResolver.resolve() calls out to network on first access
         * 2. RemoteConfigBootstrapSource does blocking HTTP on calling thread
         * Solution: Pre-populate with static fallback immediately, defer remote fetch to background.
         */
        private val cachedBootstrapNodes: List<String> by lazy {
            Timber.d("Pre-populating static bootstrap nodes (no network I/O)")
            STATIC_BOOTSTRAP_NODES
        }

        /**
         * Resolve bootstrap nodes using the core BootstrapResolver.
         * Priority: SC_BOOTSTRAP_NODES env var → remote URL → static fallback.
         * Returns cached static nodes immediately; remote fetch happens in background.
         */
        val DEFAULT_BOOTSTRAP_NODES: List<String>
            get() = cachedBootstrapNodes

        /**
         * P0_ANDROID_007: Implement diverse bootstrap source strategies.
         * Resolves nodes from environment, remote config, and static fallback.
         * DEPRECATED: Use cachedBootstrapNodes for Settings screen to avoid blocking I/O.
         */
        @Deprecated("Causes ANR - use cachedBootstrapNodes instead")
        internal suspend fun resolveAllBootstrapSourcesAsync(): List<String> = kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
            val sources = listOf(
                EnvironmentBootstrapSource(),
                StaticBootstrapSource()
                // REMOTE CONFIG REMOVED - causes blocking network I/O on calling thread
                // RemoteConfigBootstrapSource("https://bootstrap.scmessenger.net/nodes.json")
            )

            for (source in sources) {
                try {
                    val nodes = source.getBootstrapNodes()
                    if (nodes.isNotEmpty()) {
                        Timber.i("Using bootstrap source: ${source.name} with ${nodes.size} nodes")
                        return@withContext nodes
                    }
                } catch (e: Exception) {
                    Timber.w("Bootstrap source ${source.name} failed: ${e.message}")
                }
            }
            STATIC_BOOTSTRAP_NODES
        }

        /**
         * ANR FIX: Get bootstrap nodes synchronously without network I/O.
         * Used by Settings screen to avoid UI thread blocking.
         */
        fun getBootstrapNodesForSettings(): List<String> = cachedBootstrapNodes

        interface BootstrapSource {
            val name: String
            fun getBootstrapNodes(): List<String>
        }

        class EnvironmentBootstrapSource : BootstrapSource {
            override val name = "Environment"
            override fun getBootstrapNodes(): List<String> {
                val env = System.getenv("SC_BOOTSTRAP_NODES") ?: return emptyList()
                return env.split(",").map { it.trim() }.filter { it.isNotEmpty() }
            }
        }

        /**
         * ANR FIX: StaticBootstrapSource returns immediately without any network I/O.
         */
        class StaticBootstrapSource : BootstrapSource {
            override val name = "StaticFallback"
            override fun getBootstrapNodes(): List<String> = STATIC_BOOTSTRAP_NODES
        }

        internal fun isMeshParticipationEnabled(settings: uniffi.api.MeshSettings?): Boolean {
            // Default to ENABLED when settings unavailable (matches Rust default: relay_enabled=true)
            return settings?.relayEnabled ?: true
        }

        internal fun requireMeshParticipationEnabled(settings: uniffi.api.MeshSettings?) {
            if (!isMeshParticipationEnabled(settings)) {
                throw IllegalStateException(
                    "Cannot send messages: mesh participation is disabled. Enable mesh participation in settings to send and receive messages."
                )
            }
        }

        internal fun isEnabledFlag(raw: String?): Boolean {
            return when (raw?.trim()?.lowercase()) {
                "1", "true", "yes", "on" -> true
                else -> false
            }
        }

        internal data class LocalTransportFallbackResult(
            val wifiAttempted: Boolean,
            val wifiAcked: Boolean,
            val bleAttempted: Boolean,
            val bleAcked: Boolean
        ) {
            val acked: Boolean
                get() = wifiAcked || bleAcked
        }

        internal fun attemptWifiThenBleFallback(
            wifiPeerId: String?,
            blePeerId: String?,
            tryWifi: (String) -> Boolean,
            tryBle: (String) -> Boolean
        ): LocalTransportFallbackResult {
            val normalizedWifi = wifiPeerId?.trim()?.takeIf { it.isNotEmpty() }
            if (normalizedWifi != null) {
                if (tryWifi(normalizedWifi)) {
                    return LocalTransportFallbackResult(
                        wifiAttempted = true,
                        wifiAcked = true,
                        bleAttempted = false,
                        bleAcked = false
                    )
                }
            }

            val normalizedBle = blePeerId?.trim()?.takeIf { it.isNotEmpty() }
            if (normalizedBle != null) {
                val bleAcked = tryBle(normalizedBle)
                return LocalTransportFallbackResult(
                    wifiAttempted = normalizedWifi != null,
                    wifiAcked = false,
                    bleAttempted = true,
                    bleAcked = bleAcked
                )
            }

            return LocalTransportFallbackResult(
                wifiAttempted = normalizedWifi != null,
                wifiAcked = false,
                bleAttempted = false,
                bleAcked = false
            )
        }
    }

    /** Available internal storage in MB. used for low-storage warnings. */
    fun getAvailableStorageMB(): Long {
        return com.scmessenger.android.utils.StorageManager.getAvailableStorageMB(context)
    }

    private suspend fun enhanceNetworkErrorLogging(exception: Exception, node: String) {
        val errorDetails = classifyBootstrapError(exception, node)
        Timber.w("Bootstrap failed for $node - $errorDetails")
        trackNetworkFailure(node, errorDetails, exception)
    }

    private suspend fun trackNetworkFailure(nodeId: String, reason: String, exception: Exception) {
        networkFailureMetrics.recordFailure(nodeId, reason, exception)

        // P0_ANDROID_010: Prevent recursive fallback triggering (StackOverflowError)
        // Atomic check-and-set ensures only one concurrent fallback attempt
        if (inFallbackProtocol.get()) {
            Timber.w("Skipping recursive fallback for $nodeId - already in fallback protocol")
            return
        }

        // Trigger fallback if node is marked unreachable
        if (networkFailureMetrics.isNodeUnreachable(nodeId)) {
            triggerFallbackProtocol(nodeId)
        }
    }

    /**
     * P0_ANDROID_007: When a bootstrap node becomes unreachable, attempt alternative sources.
     * Falls back through: environment override → remote config → static fallback → WebSocket on standard ports.
     */
    private suspend fun triggerFallbackProtocol(failedNodeId: String) {
        // P0_ANDROID_010: Atomic compareAnd-set ensures race-free recursion guard
        if (!inFallbackProtocol.compareAndSet(false, true)) {
            Timber.w("Skipping re-entrant fallback for $failedNodeId - already in progress")
            return
        }
        try {
            Timber.w("Node $failedNodeId marked unreachable, triggering fallback protocol")
            val fallbackNodes = resolveAllBootstrapSourcesAsync()
            if (fallbackNodes.isNotEmpty()) {
                Timber.i("Fallback protocol: ${fallbackNodes.size} alternative bootstrap nodes available")
                val bridge = swarmBridge ?: return
                for (addr in fallbackNodes) {
                    if (addr != failedNodeId && relayCircuitBreaker.allowRequest(addr)) {
                        try {
                            bridge.dial(addr)
                            Timber.i("Fallback bootstrap dial: %s", addr)
                            return
                        } catch (e: Exception) {
                            // Log fallback failure directly — do NOT call enhanceNetworkErrorLogging
                            // to prevent infinite recursion (trackNetworkFailure → triggerFallbackProtocol loop)
                            Timber.w(e, "Fallback bootstrap dial failed for $addr")
                        }
                    }
                }
            }
        } finally {
            inFallbackProtocol.set(false)
        }
    }

    private val identityBackupPrefs: SharedPreferences by lazy {
        context.getSharedPreferences(IDENTITY_BACKUP_PREFS, Context.MODE_PRIVATE)
    }

    // Mesh service instance (lazy init)
    @Volatile private var meshService: uniffi.api.MeshService? = null

    // Managers (lazy init)
    @Volatile private var contactManager: uniffi.api.ContactManager? = null
    @Volatile private var historyManager: uniffi.api.HistoryManager? = null
    @Volatile private var ledgerManager: uniffi.api.LedgerManager? = null
    @Volatile private var settingsManager: uniffi.api.MeshSettingsManager? = null
    @Volatile private var autoAdjustEngine: uniffi.api.AutoAdjustEngine? = null

    // P0_NETWORK_001: Circuit breaker for relay failure tracking
    private val relayCircuitBreaker = CircuitBreaker()
    // P0_NETWORK_001: Network detector for cellular-aware transport selection
    private val networkDetector = NetworkDetector(context)
    // P0_ANDROID_007: Diagnostics reporter for connectivity analysis
    private val diagnosticsReporter = com.scmessenger.android.network.DiagnosticsReporter(
        context,
        com.scmessenger.android.network.NetworkDiagnostics(context),
        com.scmessenger.android.network.NetworkTypeDetector(context),
        networkFailureMetrics
    )

    // Core & Network (lazy init)
    @Volatile private var ironCore: uniffi.api.IronCore? = null
    // Swarm Bridge (Internet/Libp2p)
    @Volatile private var swarmBridge: uniffi.api.SwarmBridge? = null

    // Wifi Transport
    @Volatile private var wifiTransportManager: com.scmessenger.android.transport.WifiTransportManager? = null

    // Smart Transport Router (500ms timeout fallback + health tracking)
    @Volatile private var smartTransportRouter: com.scmessenger.android.transport.SmartTransportRouter? = null

    // Service state
    private val _serviceState = MutableStateFlow(uniffi.api.ServiceState.STOPPED)
    open val serviceState: StateFlow<uniffi.api.ServiceState> = _serviceState.asStateFlow()

    // Service stats
    private val _serviceStats = MutableStateFlow<uniffi.api.ServiceStats?>(null)
    open val serviceStats: StateFlow<uniffi.api.ServiceStats?> = _serviceStats.asStateFlow()

    // Message updates flow (both sent and received) used for UI updates
    private val _messageUpdates = kotlinx.coroutines.flow.MutableSharedFlow<uniffi.api.MessageRecord>(replay = 0, extraBufferCapacity = 1, onBufferOverflow = kotlinx.coroutines.channels.BufferOverflow.DROP_OLDEST)
    open val messageUpdates = _messageUpdates.asSharedFlow()

    // Compatibility for notifications (incoming only)
    val incomingMessages = messageUpdates.filter { it.direction == uniffi.api.MessageDirection.RECEIVED }

    private val repoScope = kotlinx.coroutines.CoroutineScope(kotlinx.coroutines.Dispatchers.IO + kotlinx.coroutines.SupervisorJob())
    private var pendingOutboxRetryJob: kotlinx.coroutines.Job? = null
    private var coverTrafficJob: kotlinx.coroutines.Job? = null
    private var maintenanceJob: kotlinx.coroutines.Job? = null
    private val pendingOutboxFile = File(storagePath, "pending_outbox.json")
    private val pendingOutboxFlushMutex = kotlinx.coroutines.sync.Mutex()
    // P1: Mutex to synchronize contact upsert operations and prevent duplicate contact creation
    // during concurrent peer identification callbacks (AND-CONTACT-DUP-001)
    private val contactUpsertMutex = kotlinx.coroutines.sync.Mutex()
    private val receiptAwaitSeconds: Long = 8L
    private val pendingOutboxMaxAttempts: Int = 720
    private val pendingOutboxMaxAgeSeconds: Long = 7L * 24L * 60L * 60L
    private val historySyncSentPeers = java.util.concurrent.ConcurrentHashMap<String, Long>()
    private val HISTORY_SYNC_COOLDOWN_MS = 60_000L
    private val identitySyncSentPeers = java.util.Collections.synchronizedSet(mutableSetOf<String>())
    private val deliveredReceiptCache = java.util.concurrent.ConcurrentHashMap<String, Long>()
    private val deliveredReceiptCacheTtlMs = 2L * 60L * 60L * 1000L
    private val pendingReceiptSendJobs = java.util.concurrent.ConcurrentHashMap<String, kotlinx.coroutines.Job>()
    private val receiptSendMaxAttempts = 6
    @Volatile
    private var serviceStartedAtEpochSec: Long = 0L
    @Volatile
    private var lastRelayBootstrapDialMs: Long = 0L
    // P1_ANDROID_013: Track consecutive full-bootstrap failures for exponential backoff
    @Volatile
    private var consecutiveBootstrapFailures: Int = 0
    @Volatile
    private var nextBootstrapAttemptMs: Long = 0L
    private val dialThrottleState = java.util.concurrent.ConcurrentHashMap<String, Pair<Int, Long>>()

    // Reinstall detection: true when SharedPreferences has identity backup but contacts.db
    // is missing from filesDir. Triggers post-start aggressive identity beacon.
    @Volatile private var isReinstallWithMissingData = false

    // P0: Peer tracking to prevent ghost connections (700+ peers bug)
    private val transportToCanonicalMap = ConcurrentHashMap<String, String>() // libp2pPeerId -> canonicalPeerId
    private val activeSessions = ConcurrentHashMap<String, Long>() // libp2pPeerId -> lastActive
    // P0: Dedup cache — suppress redundant peer-identified callbacks for the same peer
    // within a 30-second window. The Rust core fires identify per-substream.
    private val peerIdentifiedDedupCache =
        java.util.concurrent.ConcurrentHashMap<String, Pair<String, Long>>()
    private val peerIdentifiedDedupIntervalMs = 30_000L

    // P1: Dedup cache — suppress duplicate disconnect callbacks for the same peer
    // within a 1-second window. The Rust core fires one disconnect per-substream.
    private val peerDisconnectDedupCache = java.util.concurrent.ConcurrentHashMap<String, Long>()
    private val peerDisconnectDedupIntervalMs = 1_000L

    // P0_ANDROID_005: Message ID tracking cache with corruption detection and recovery
    // Thread-safe ConcurrentHashMap for storing message delivery tracking information
    private val messageTrackingCache = java.util.concurrent.ConcurrentHashMap<String, MessageTracking>()

    // P0_ANDROID_005: Retry lock for thread-safe attempt count updates
    private val retryLock = kotlinx.coroutines.sync.Mutex()

    // P0_ANDROID_010: Atomic recursion guard for fallback protocol
    // Prevents enhanceNetworkErrorLogging → trackNetworkFailure → triggerFallbackProtocol
    // from re-entering concurrently. AtomicBoolean ensures check-and-set is race-free.
    private val inFallbackProtocol = AtomicBoolean(false)

    // P4: Dedup cache — suppress redundant dial-throttle log lines
    // for the same address within a 5-minute window.
    private val dialThrottleLogCache = java.util.concurrent.ConcurrentHashMap<String, Long>()
    private val dialThrottleLogIntervalMs = 300_000L

    // AND-SEND-BTN-001: In-memory cache for identity ID resolution.
    // canonicalContactId() calls ironCore?.resolveIdentity() via synchronous FFI on every
    // recomposition when invoked from Composable code. This cache eliminates repeated FFI calls
    // for the same peer ID, preventing UI thread freezes.
    private val identityIdCache = java.util.concurrent.ConcurrentHashMap<String, String>()

    // TCP/mDNS transport parity: Track peers discovered on LAN via libp2p mDNS.
    // Key = libp2p PeerId, Value = set of LAN multiaddresses for direct TCP delivery.
    private val mdnsLanPeers = ConcurrentHashMap<String, List<String>>()

    // Core Delegate reference to prevent GC
    @Volatile private var coreDelegate: uniffi.api.CoreDelegate? = null

    // BLE Components
    @Volatile private var bleScanner: com.scmessenger.android.transport.ble.BleScanner? = null
    @Volatile private var bleAdvertiser: com.scmessenger.android.transport.ble.BleAdvertiser? = null
    @Volatile private var bleGattServer: com.scmessenger.android.transport.ble.BleGattServer? = null
    @Volatile private var bleGattClient: com.scmessenger.android.transport.ble.BleGattClient? = null
    @Volatile private var lastBleBeaconUpdateMillis: Long = 0
    private var lastBleBeaconPayload: ByteArray = byteArrayOf()
    @Volatile private var lastBleBeaconPayloadPublishedAtMillis: Long = 0

    private data class RoutingHints(
        val wifiPeerId: String?,
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

    internal data class PendingOutboundEnvelope(
        val queueId: String,
        val historyRecordId: String,
        val peerId: String,
        val routePeerId: String?,
        val listeners: List<String>,
        val envelopeBase64: String,
        val createdAtEpochSec: Long,
        val attemptCount: Int,
        val nextAttemptAtEpochSec: Long,
        val strictBleOnlyMode: Boolean? = null,
        val recipientIdentityId: String? = null,
        val intendedDeviceId: String? = null,
        val terminalFailureCode: String? = null
    )

    private data class MessageIdentityHints(
        val identityId: String?,
        val publicKey: String?,
        val deviceId: String?,
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
        val libp2pPeerId: String? = null, // Libp2p peer ID for routing
        val transport: com.scmessenger.android.service.TransportType,
        val isFull: Boolean,         // True if peer identity is authenticated (non-relay)
        val isRelay: Boolean = false,
        val lastSeen: ULong = System.currentTimeMillis().toULong() / 1000u
    )

    private data class DeliveryAttemptResult(
        val acked: Boolean,
        val routePeerId: String?,
        val terminalFailureCode: String? = null
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

    private data class BleRouteObservation(
        val address: String,
        val lastSeenMs: Long,
        val source: String
    )

    /**
     * P0_ANDROID_005: Message tracking data structure.
     *
     * Tracks message delivery status, attempt counts, and corruption indicators
     * for robust message retry handling.
     */
    private data class MessageTracking(
        val messageId: String,
        var attemptCount: Int = 0,
        var lastAttemptAt: Long = 0L,
        var lastDeliveryAttemptAt: Long = 0L,
        var lastDeliveryOutcome: String? = null,
        var deliveryStatus: DeliveryStatus = DeliveryStatus.PENDING,
        var lastErrorCode: String? = null,
        var corruptionDetected: Boolean = false,
        var recoveredAt: Long? = null
    ) {
        /**
         * Check if this tracking entry is corrupted and needs recovery.
         * Criteria: (1) explicitly flagged as corrupted, or (2) retry overflow (> MAX_RETRY_ATTEMPTS).
         * The attemptCount check is a safety net since recordFailure() also sets corruptionDetected.
         */
        fun isCorrupted(): Boolean {
            return corruptionDetected || attemptCount > MAX_RETRY_ATTEMPTS
        }

        /**
         * Mark this tracking as corrupted.
         */
        fun markCorrupted() {
            this.corruptionDetected = true
        }

        /**
         * Record a successful delivery attempt.
         */
        fun recordSuccess() {
            this.attemptCount = 0
            this.deliveryStatus = DeliveryStatus.DELIVERED
            this.lastDeliveryOutcome = "success"
            this.lastDeliveryAttemptAt = System.currentTimeMillis()
            this.corruptionDetected = false
            this.recoveredAt = null
        }

        /**
         * Record a failed delivery attempt.
         */
        fun recordFailure(errorCode: String?) {
            this.attemptCount++
            this.lastAttemptAt = System.currentTimeMillis()
            this.lastErrorCode = errorCode
            this.deliveryStatus = DeliveryStatus.PENDING
            this.corruptionDetected = attemptCount > MAX_RETRY_ATTEMPTS
        }

        companion object {
            const val MAX_RETRY_ATTEMPTS = 12

            /**
             * Create a new tracking entry for a message.
             */
            fun forMessage(messageId: String): MessageTracking {
                return MessageTracking(messageId = messageId)
            }

            /**
             * Recover from corruption by resetting tracking state.
             */
            fun recoverFromCorruption(tracking: MessageTracking): MessageTracking {
                return MessageTracking(
                    messageId = tracking.messageId,
                    attemptCount = 0,
                    corruptionDetected = false,
                    recoveredAt = System.currentTimeMillis()
                )
            }
        }
    }

    /**
     * P0_ANDROID_005: Message delivery status enum.
     */
    private enum class DeliveryStatus {
        PENDING,
        DELIVERED,
        FAILED,
        EXPIRED
    }

    // Maximum retry attempts before marking message as failed
    // Defined here for external references; also in MessageTracking.Companion for inner class access
    private val MAX_RETRY_ATTEMPTS: Int = 12

    private val strictBleOnlyValidation = isEnabledFlag(System.getenv("SC_BLE_ONLY_VALIDATION"))
    private val bleRouteObservations = ConcurrentHashMap<String, BleRouteObservation>()
    // Extended from 2 minutes to 5 minutes to fix TRANSPORT-001: BLE hint staleness
    // When iOS app crashes, BLE hints become stale. Extended TTL makes fallback more resilient.
    private val bleRouteFreshnessTtlMs = 300_000L
    // Stale hint grace period: allow slightly stale hints to be used for fallback
    private val bleRouteStaleGraceMs = 600_000L  // 10 minutes

    private fun isTerminalIdentityFailure(errorCode: String?): Boolean {
        return when (errorCode?.trim()) {
            "identity_device_mismatch",
            "identity_abandoned" -> true
            else -> false
        }
    }

    private fun terminalIdentityFailureMessage(errorCode: String?): String {
        return when (errorCode?.trim()) {
            "identity_device_mismatch" ->
                "This contact's identity has been recycled onto another device. Refresh their contact details before retrying."
            "identity_abandoned" ->
                "This contact abandoned the identity you tried to reach. Re-verify the contact before sending again."
            else -> "This message was rejected because the recipient identity is no longer valid."
        }
    }

    // ========================================
    // P0_ANDROID_005: Message ID Tracking Functions
    // ========================================

    /**
     * Get or create message tracking for a message ID.
     * This function gracefully handles missing tracking entries by creating new ones.
     */
    private fun getMessageIdTracking(messageId: String): MessageTracking {
        return messageTrackingCache[messageId] ?: run {
            // If tracking is missing, create a fresh entry
            val tracking = MessageTracking.forMessage(messageId)
            messageTrackingCache[messageId] = tracking
            Timber.w("Message ID tracking recreated for $messageId (was missing)")
            tracking
        }
    }

    /**
     * Detect and recover from corrupted message tracking entries.
     * This prevents tracking corruption from causing retry storms.
     */
    private fun detectAndRecoverMessageTracking() {
        val corruptedIds = mutableListOf<String>()
        messageTrackingCache.forEach { (messageId, tracking) ->
            if (tracking.isCorrupted()) {
                Timber.w("Corrupted message tracking detected for $messageId, recovering")
                corruptedIds.add(messageId)
            }
        }

        // Recover corrupted entries
        for (messageId in corruptedIds) {
            val tracking = messageTrackingCache[messageId]
            if (tracking != null) {
                val recovered = MessageTracking.recoverFromCorruption(tracking)
                messageTrackingCache[messageId] = recovered
                Timber.i("Message tracking recovered for $messageId (was corrupted)")
            }
        }

        // Log if we found corrupted entries
        if (corruptedIds.isNotEmpty()) {
            Timber.w("Message tracking corruption recovery: ${corruptedIds.size} entries recovered")
        }
    }

    /**
     * Safely increment the attempt count for a message with proper synchronization.
     */
    private suspend fun incrementAttemptCount(messageId: String) {
        retryLock.withLock {
            val tracking = getMessageIdTracking(messageId)
            tracking.recordFailure(null)
        }
    }

    /**
     * Get the next retry delay for a message based on exponential backoff.
     */
    private fun getRetryDelay(attemptCount: Int): Long {
        return when (attemptCount) {
            0 -> 1000L
            1 -> 2000L
            2 -> 4000L
            3 -> 8000L
            4 -> 16000L
            else -> 30000L // Cap at 30 seconds
        }
    }

    /**
     * Check if a message should be retried based on attempt count.
     */
    private fun shouldRetryMessage(messageId: String): Boolean {
        val tracking = getMessageIdTracking(messageId)
        return tracking.attemptCount < MAX_RETRY_ATTEMPTS
    }

    /**
     * Log message delivery attempt for monitoring and debugging.
     */
    private fun logMessageDeliveryAttempt(messageId: String, attempt: Int, outcome: String) {
        Timber.d("Message delivery: id=$messageId, attempt=$attempt, outcome=$outcome")
    }

    /**
     * Detect and log retry storms (messages with >5 attempts).
     */
    private fun logRetryStormDetection() {
        val highRetryMessages = messageTrackingCache.values
            .filter { it.attemptCount > 5 }
            .count()

        if (highRetryMessages > 10) {
            Timber.w("Retry storm detected: $highRetryMessages messages with >5 attempts")
        }
    }

    init {
        Timber.d("MeshRepository initialized with storage: $storagePath")
        if (strictBleOnlyValidation) {
            Timber.w("Strict BLE-only validation mode is enabled (SC_BLE_ONLY_VALIDATION)")
        }
        checkReinstallState()
        initializeManagers()
    }

    /**
     * Deferred repository initialization for heavy operations.
     * Call from a background thread (e.g., lifecycleScope on Dispatchers.IO)
     * to avoid blocking the main thread during Activity.onCreate().
     */
    fun initializeRepository() {
        try {
            startStorageMaintenance()
            Timber.i("Repository background initialization completed")
        } catch (e: Exception) {
            Timber.w(e, "Repository background initialization failed")
        }
    }

    private fun checkReinstallState() {
        // Detect reinstall: SharedPreferences identity backup exists but contacts.db is gone.
        // Triggers post-start aggressive identity beacon to recover contact info from peers.
        val hasIdentityBackup = identityBackupPrefs.contains(IDENTITY_BACKUP_KEY)
        val contactsOnDisk = java.io.File(storagePath, "contacts.db").exists()
        val historyOnDisk = java.io.File(storagePath, "history.db").exists()

        Timber.d("AND-CONTACTS-WIPE-001: Reinstall check - hasIdentityBackup=$hasIdentityBackup, contactsOnDisk=$contactsOnDisk, historyOnDisk=$historyOnDisk")

        if (hasIdentityBackup && (!contactsOnDisk || !historyOnDisk)) {
            isReinstallWithMissingData = true
            Timber.i("Reinstall detected: identity backup present but contacts=$contactsOnDisk history=$historyOnDisk")
            // AND-CONTACTS-WIPE-001: Log additional details for data recovery
            if (!contactsOnDisk) {
                Timber.w("AND-CONTACTS-WIPE-001: CONTACT DATA MISSING - Will attempt peer recovery")
            }
            if (!historyOnDisk) {
                Timber.w("AND-CONTACTS-WIPE-001: HISTORY DATA MISSING")
            }
        } else if (hasIdentityBackup && contactsOnDisk && historyOnDisk) {
            Timber.d("AND-CONTACTS-WIPE-001: Normal startup - all data present")
        } else if (!hasIdentityBackup) {
            Timber.d("AND-CONTACTS-WIPE-001: Fresh install - no identity backup found")
        }
    }

    private fun initializeManagers() {
        try {
            // REGRESSION FIX (AND-CONTACTS-WIPE-001): Migrate contacts BEFORE ContactManager
            // opens the new database. Previously the migration ran after construction, which
            // meant sled had the file locked and the copied data was invisible to the open handle.
            migrateContactsFromOldLocation()

            // Initialize Data Managers
            settingsManager = uniffi.api.MeshSettingsManager(storagePath)
            historyManager = uniffi.api.HistoryManager(storagePath)
            // P0_SECURITY_001: Bounded retention enforcement at startup.
            // Prune messages older than 90 days and enforce a 50k message cap.
            // The periodic maintenance loop (startStorageMaintenance) also enforces this
            // every 15 minutes, but we run it at init for immediate cleanup.
            try {
                val prunedByAge = historyManager?.pruneBefore(
                    (System.currentTimeMillis() / 1000 - 90L * 24 * 3600).toULong()
                )
                if (prunedByAge != null && prunedByAge > 0u) {
                    Timber.i("P0_SECURITY_001: Pruned $prunedByAge messages older than 90 days at startup")
                }
                val count = historyManager?.count() ?: 0u
                if (count > 50_000u) {
                    val prunedByCap = historyManager?.enforceRetention(50_000u)
                    if (prunedByCap != null && prunedByCap > 0u) {
                        Timber.i("P0_SECURITY_001: Pruned $prunedByCap messages exceeding 50k cap at startup")
                    }
                }
            } catch (e: Exception) {
                Timber.w(e, "P0_SECURITY_001: Retention enforcement at startup failed")
            }
            contactManager = uniffi.api.ContactManager(storagePath)
            ledgerManager = uniffi.api.LedgerManager(storagePath)
            autoAdjustEngine = uniffi.api.AutoAdjustEngine()

            // Pre-load data where applicable
            ledgerManager?.load()

            Timber.i("all_managers_init_success")
            Timber.i("All managers initialized successfully")

            // One-time migration: clear stale routing hints inherited from
            // pre-fix builds.  The old appendRoutingHint accumulated duplicate
            // BLE MACs and stale libp2p_peer_id entries that now cause endless
            // retry loops to unreachable peers.
            migrateStaleRoutingHints()

            // One-time migration: repair contacts with truncated/invalid public keys
            // that were stored before validation was added (March 2026).
            migrateTruncatedPublicKeys()

            // AND-CONTACTS-WIPE-001: Verify contact data integrity after initialization
            verifyContactDataIntegrity()

            // P0_ANDROID_001: Emergency contact corruption detection and recovery
            // Run corruption detection after managers are initialized
            repoScope.launch {
                try {
                    val corruptionDetected = detectAndRepairCorruption()
                    if (corruptionDetected) {
                        Timber.i("P0_ANDROID_001: Database corruption detected and repaired at startup")
                    }
                } catch (e: Exception) {
                    Timber.w(e, "P0_ANDROID_001: Corruption detection failed at startup")
                }
            }
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize managers")
        }
    }

    /**
     * AND-CONTACTS-WIPE-001: Verify contact data integrity after initialization.
     * Helps detect and log any potential data loss issues for diagnostics.
     */
    private fun verifyContactDataIntegrity() {
        try {
            val contacts = contactManager?.list().orEmpty()
            Timber.d("AND-CONTACTS-WIPE-001: Contact data verification - Found ${contacts.size} contacts")

            if (contacts.isEmpty()) {
                Timber.w("AND-CONTACTS-WIPE-001: WARNING - No contacts found. If this is unexpected, data recovery may be needed.")

                // Check if there are any messages - if yes, but no contacts, that's a red flag
                val messageCount = try {
                    getMessageCount()
                } catch (e: Exception) {
                    0u
                }

                if (messageCount > 0u) {
                    Timber.e("AND-CONTACTS-WIPE-001: CRITICAL - Messages exist (${messageCount} total) but no contacts found. Possible data loss scenario.")
                }
            } else {
                Timber.i("AND-CONTACTS-WIPE-001: Contact data verification successful - ${contacts.size} contacts available")

                // Log sample contact info for diagnostics (without exposing sensitive data)
                val sampleSize = minOf(5, contacts.size)
                Timber.d("AND-CONTACTS-WIPE-001: Sample contacts (${sampleSize} of ${contacts.size}):")
                contacts.take(sampleSize).forEach { contact ->
                    val peerIdPreview = contact.peerId.take(12) + if (contact.peerId.length > 12) "..." else ""
                    val hasValidKey = !contact.publicKey.isNullOrEmpty() && contact.publicKey.length >= 64
                    Timber.d("  - $peerIdPreview (added: ${contact.addedAt}, hasKey: $hasValidKey)")
                }
            }
        } catch (e: Exception) {
            Timber.e(e, "AND-CONTACTS-WIPE-001: Failed to verify contact data integrity")
        }
    }

    /**
     * REGRESSION FIX (AND-CONTACTS-WIPE-001): Migrate contacts from old storage location.
     *
     * Issue: UniFFI contract update changed ContactManager to use "contacts.db/"
     * instead of "contacts/", causing contacts to disappear after app update.
     *
     * Solution: Copy sled database files from old to new location at file system level.
     * Must run BEFORE ContactManager construction so the new DB isn't locked.
     */
    private fun migrateContactsFromOldLocation() {
        try {
            val prefs = context.getSharedPreferences("mesh_migrations", android.content.Context.MODE_PRIVATE)
            if (prefs.getBoolean("v2_contacts_db_migration", false)) {
                Timber.d("Contacts migration already completed, skipping")
                return
            }

            Timber.i("AND-CONTACTS-WIPE-001: Starting contacts migration process")

            val oldDir = java.io.File(storagePath, "contacts")
            val newDir = java.io.File(storagePath, "contacts.db")

            Timber.d("AND-CONTACTS-WIPE-001: Checking old directory existence: ${oldDir.exists()}, isDirectory: ${oldDir.isDirectory}")

            if (!oldDir.exists() || !oldDir.isDirectory) {
                Timber.d("AND-CONTACTS-WIPE-001: Old contacts directory not found, skipping migration")
                prefs.edit().putBoolean("v2_contacts_db_migration", true).apply()
                return
            }

            val oldDb = java.io.File(oldDir, "db")
            Timber.d("AND-CONTACTS-WIPE-001: Old DB exists: ${oldDb.exists()}, size: ${oldDb.length()} bytes")

            if (!oldDb.exists() || oldDb.length() == 0L) {
                Timber.d("AND-CONTACTS-WIPE-001: Old contacts database empty or missing, skipping migration")
                prefs.edit().putBoolean("v2_contacts_db_migration", true).apply()
                return
            }

            // Create new directory if it doesn't exist
            if (!newDir.exists()) {
                Timber.d("AND-CONTACTS-WIPE-001: Creating new contacts directory")
                newDir.mkdirs()
            }

            // Check if new database has meaningful data (not just sled's initial empty tree)
            val newDb = java.io.File(newDir, "db")
            val newDbSize = if (newDb.exists()) newDb.length() else 0L
            Timber.d("AND-CONTACTS-WIPE-001: New DB size: $newDbSize bytes")

            // Migrate if old DB has more data than new DB, or new DB is absent/tiny.
            // Sled creates a ~4KB empty tree on open, so we use a low threshold.
            if (oldDb.length() > newDbSize || newDbSize < 4096) {
                Timber.i("AND-CONTACTS-WIPE-001: MIGRATION NEEDED - Copying contacts from old location (${oldDb.length()} bytes -> new ${newDbSize} bytes)")

                var copySuccess = true
                var copiedFiles = 0
                var totalBytes = 0L

                // Copy all sled files from old to new directory
                oldDir.listFiles()?.forEach { file ->
                    if (file.isFile) {
                        val dest = java.io.File(newDir, file.name)
                        try {
                            file.copyTo(dest, overwrite = true)
                            copiedFiles++
                            totalBytes += file.length()
                            Timber.d("AND-CONTACTS-WIPE-001: Copied ${file.name} (${file.length()} bytes)")
                        } catch (e: Exception) {
                            Timber.e(e, "AND-CONTACTS-WIPE-001: Failed to copy ${file.name}")
                            copySuccess = false
                        }
                    }
                }

                Timber.i("AND-CONTACTS-WIPE-001: Copy operation completed. Files: $copiedFiles, Total bytes: $totalBytes, Success: $copySuccess")

                if (copySuccess) {
                    Timber.i("AND-CONTACTS-WIPE-001: MIGRATION COMPLETED SUCCESSFULLY")
                    prefs.edit().putBoolean("v2_contacts_db_migration", true).apply()
                } else {
                    // Don't mark migration as complete — allow retry on next app start
                    Timber.w("AND-CONTACTS-WIPE-001: Some files failed to copy, will retry on next launch")
                }
            } else {
                Timber.d("AND-CONTACTS-WIPE-001: New database already has sufficient data ($newDbSize bytes), skipping migration")
                prefs.edit().putBoolean("v2_contacts_db_migration", true).apply()
            }

        } catch (e: Exception) {
            Timber.e(e, "AND-CONTACTS-WIPE-001: Contacts migration failed — NOT marking complete to allow retry")
            // AND-CONTACTS-WIPE-001: Do NOT mark migration as complete on failure.
            // Previously this set the flag to true on failure, preventing any retry.
        }
    }

    private fun migrateStaleRoutingHints() {
        try {
            val prefs = context.getSharedPreferences("mesh_migrations", android.content.Context.MODE_PRIVATE)
            if (prefs.getBoolean("v1_routing_hint_cleanup", false)) return

            val contacts = contactManager?.list().orEmpty()
            var cleaned = 0
            for (contact in contacts) {
                val notes = contact.notes ?: continue
                if (!notes.contains("libp2p_peer_id:") && !notes.contains("ble_peer_id:")) continue
                // Strip stale routing entries — fresh discovery will repopulate them.
                val stripped = notes.split(';', '\n')
                    .map { it.trim() }
                    .filter { segment ->
                        !segment.startsWith("libp2p_peer_id:") &&
                            !segment.startsWith("ble_peer_id:")
                    }
                    .joinToString(";")
                val updatedNotes = stripped.ifEmpty { null }
                if (updatedNotes != notes) {
                    contactManager?.add(uniffi.api.Contact(
                        peerId = contact.peerId,
                        nickname = contact.nickname,
                        localNickname = contact.localNickname,
                        publicKey = contact.publicKey,
                        addedAt = contact.addedAt,
                        lastSeen = contact.lastSeen,
                        notes = updatedNotes,
                        lastKnownDeviceId = null
                    ))
                    cleaned++
                }
            }
            prefs.edit().putBoolean("v1_routing_hint_cleanup", true).apply()
            if (cleaned > 0) {
                Timber.i("Routing hint migration: cleaned $cleaned contact(s) with stale routing entries")
            }
        } catch (e: Exception) {
            Timber.w(e, "Routing hint migration failed (non-fatal)")
        }
    }

    /**
     * Migration: Repair contacts with truncated/invalid public keys.
     *
     * Before March 2026, addContact() had no validation, so contacts could be stored
     * with truncated keys (e.g., 8 chars like "f669fb0f" instead of 64-char hex).
     * This caused BLE decryption failures when receiving messages from those peers.
     *
     * This migration:
     * 1. Finds contacts with invalid public keys
     * 2. Attempts to repair them from discovered peers or BLE identity beacons
     * 3. If repair fails, keeps the contact but logs a warning (user can re-pair)
     */
    private fun migrateTruncatedPublicKeys() {
        try {
            val prefs = context.getSharedPreferences("mesh_migrations", android.content.Context.MODE_PRIVATE)
            if (prefs.getBoolean("v2_truncated_key_migration", false)) return

            val contacts = contactManager?.list().orEmpty()
            var repaired = 0
            var unrepaired = 0

            for (contact in contacts) {
                val rawKey = contact.publicKey ?: continue
                val trimmedKey = rawKey.trim()

                // Skip contacts that already have valid keys
                if (trimmedKey.length == 64 && trimmedKey.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' }) {
                    continue
                }

                Timber.w("Migration: Found contact with invalid key (${trimmedKey.length} chars): ${contact.peerId.take(8)}...")

                // Try to find a matching discovered peer with a valid key
                val discoveredPeer = _discoveredPeers.value.entries.firstOrNull { (key, info) ->
                    // Match by peerId or by partial key match
                    info.peerId == contact.peerId ||
                    key.startsWith(trimmedKey) ||
                    trimmedKey.startsWith(key.take(8))
                }

                val discoveredKey = discoveredPeer?.value?.publicKey?.trim()
                if (discoveredKey != null && discoveredKey.length == 64 && discoveredKey.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' }) {
                    // Repair the contact with the valid key
                    Timber.i("Migration: Repairing contact ${contact.peerId.take(8)} with full key from discovered peer")
                    contactManager?.add(uniffi.api.Contact(
                        peerId = contact.peerId,
                        nickname = contact.nickname,
                        localNickname = contact.localNickname,
                        publicKey = discoveredKey,  // Use the repaired key
                        addedAt = contact.addedAt,
                        lastSeen = contact.lastSeen,
                        notes = contact.notes,
                        lastKnownDeviceId = contact.lastKnownDeviceId
                    ))
                    repaired++
                } else {
                    // Can't repair - keep contact but log warning
                    // User will need to re-pair with this peer to get the full key
                    Timber.w("Migration: Cannot repair contact ${contact.peerId.take(8)} - no discovered peer with valid key. User must re-pair.")
                    unrepaired++
                }
            }

            prefs.edit().putBoolean("v2_truncated_key_migration", true).apply()
            if (repaired > 0 || unrepaired > 0) {
                Timber.i("Truncated key migration: repaired=$repaired unrepaired=$unrepaired")
            }
        } catch (e: Exception) {
            Timber.w(e, "Truncated key migration failed (non-fatal)")
        }
    }

    // ========================================================================
    // MESH SERVICE LIFECYCLE
    // ========================================================================

    // ANR FIX: Add network connectivity test to diagnose ledger relay failures
    private fun testLedgerRelayConnectivity(): Boolean {
        return try {
            // Test connectivity to addresses from ledger instead of static bootstrap
            val ledgerAddresses = ledgerManager?.getPreferredRelays(3u) ?: emptyList()
            if (ledgerAddresses.isEmpty()) {
                Timber.w("Network connectivity test: No preferred relays in ledger")
                return false
            }
            
            ledgerAddresses.any { relay ->
                try {
                    // Extract IP and port from multiaddr
                    val multiaddr = relay.multiaddr ?: return@any false
                    val parts = multiaddr.split("/")
                    val ipIndex = parts.indexOf("ip4")
                    val tcpIndex = parts.indexOf("tcp")
                    if (ipIndex < 0 || tcpIndex < 0 || ipIndex + 1 >= parts.size || tcpIndex + 1 >= parts.size) {
                        return@any false
                    }
                    
                    val ip = parts[ipIndex + 1]
                    val port = parts[tcpIndex + 1].toIntOrNull() ?: return@any false
                    
                    val socket = java.net.Socket()
                    socket.connect(java.net.InetSocketAddress(ip, port), 3000)
                    socket.close()
                    Timber.d("Network connectivity test: $ip:$port reachable (ledger relay)")
                    true
                } catch (e: Exception) {
                    Timber.w("Network connectivity test: ${relay.multiaddr} unreachable - ${e.message}")
                    false
                }
            }
        } catch (e: Exception) {
            Timber.w("Network connectivity test failed: ${e.message}")
            false
        }
    }

    /**
     * Start the mesh service with the given configuration.
     * This initializes the Rust core, starts BLE transport, and wires up events.
     */
    @Synchronized
    fun startMeshService(config: uniffi.api.MeshServiceConfig) {
        Timber.i("service_start_requested")
        if (meshService?.getState() == uniffi.api.ServiceState.RUNNING) {
            _serviceState.value = uniffi.api.ServiceState.RUNNING
            if (serviceStartedAtEpochSec == 0L) {
                serviceStartedAtEpochSec = System.currentTimeMillis() / 1000
            }
            Timber.d("MeshService is already running")
            return
        }

        try {
            Timber.d("Starting MeshService...")
            if (meshService == null) {
                // Recreate service instance after stop/failure so start is always clean.
                val logsDir = context.filesDir.absolutePath + "/logs"
                meshService = uniffi.api.MeshService.withStorageAndLogs(config, storagePath, logsDir)
            }

            // 1. Start the Rust Core service
            meshService?.start()

            // 2. Obtain shared IronCore instance
            ironCore = meshService?.getCore()
            if (ironCore == null) {
                Timber.e("Failed to obtain IronCore from MeshService")
                _serviceState.value = uniffi.api.ServiceState.STOPPED
                return
            }

            // Initialize SmartTransportRouter for intelligent transport selection
            smartTransportRouter = com.scmessenger.android.transport.SmartTransportRouter()
            Timber.i("SmartTransportRouter initialized for intelligent transport selection")

            // P0_NETWORK_001: Start network detection for cellular-aware fallback
            networkDetector.startMonitoring()
            Timber.i("NetworkDetector started — cellular-aware transport fallback active")

            // P0_NETWORK_001: Watch for network type changes and re-bootstrap
            startNetworkChangeWatch()

            // P0: One-time migration to unify legacy IDs (libp2p, etc) into canonical Identity IDs (hash)
            migrateToCanonicalIds()

            // WS12.41: Inject IronCore into FileLoggingTree for summarized logging
            timber.log.Timber.forest().forEach { tree ->
                if (tree is com.scmessenger.android.utils.FileLoggingTree) {
                    tree.setIronCore(ironCore)
                }
            }

            // WS12.41: Start storage maintenance loop
            startStorageMaintenance()

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
                            libp2pPeerId = peerId,
                            transport = com.scmessenger.android.service.TransportType.INTERNET, // Default for swarm
                            isFull = !isRelay && (
                                transportIdentity != null ||
                                    !extractedKey.isNullOrBlank()
                                ),
                            isRelay = isRelay,
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
                            // Don't auto-create contacts for relay peers - they are infrastructure, not user contacts
                            if (!isRelay) {
                                upsertFederatedContact(
                                    canonicalPeerId = transportIdentity.canonicalPeerId,
                                    publicKey = transportIdentity.publicKey,
                                    nickname = transportIdentity.nickname,
                                    libp2pPeerId = peerId,
                                    listeners = relayHints,
                                    createIfMissing = false
                                )
                                Timber.d("Auto-created/updated contact for discovered peer: ${transportIdentity.canonicalPeerId}")
                            } else {
                                Timber.d("Skipping contact creation for relay peer in onPeerDiscovered: $peerId")
                            }
                            try { contactManager?.updateLastSeen(transportIdentity.canonicalPeerId) } catch (_: Exception) { }
                            try { contactManager?.updateLastSeen(peerId) } catch (_: Exception) { }
                            if (!isRelay && relayHints.isNotEmpty()) {
                                connectToPeer(peerId, relayHints)
                            }
                        }
                    }
                }

	                override fun onPeerIdentified(peerId: String, agentVersion: String, listenAddrs: List<String>) {
                    // P0: Deduplicate peer-identified events — Rust core fires one per substream
                    val trimmedPeerId = peerId.trim()
                    val identifySignature = (
                        listOf(agentVersion.trim()) +
                            listenAddrs.map { it.trim() }.filter { it.isNotEmpty() }.sorted()
                        ).joinToString("|")
                    val now = System.currentTimeMillis()
                    val lastIdentified = peerIdentifiedDedupCache[trimmedPeerId]
                    if (
                        lastIdentified != null &&
                        lastIdentified.first == identifySignature &&
                        (now - lastIdentified.second) < peerIdentifiedDedupIntervalMs
                    ) {
                        return
                    }
                    peerIdentifiedDedupCache[trimmedPeerId] = identifySignature to now

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

                        // TCP/mDNS parity: Detect LAN addresses (RFC1918) from listen_addrs.
                        // If any private-network TCP/QUIC address is present, this peer
                        // was discovered on the local network (typically via libp2p mDNS).
                        val lanAddrs = listenAddrs.filter { addr ->
                            val a = addr.trim()
                            val isPrivateIp = a.startsWith("/ip4/192.168.") ||
                             a.startsWith("/ip4/10.") ||
                             (a.startsWith("/ip4/172.") && run {
                                 val parts = a.removePrefix("/ip4/").split(".")
                                 val secondOctet = parts.getOrNull(1)?.toIntOrNull() ?: 0
                                 secondOctet in 16..31
                             })
                            isPrivateIp && (a.contains("/tcp/") || a.contains("/udp/"))
                        }
                        if (lanAddrs.isNotEmpty()) {
                            mdnsLanPeers[trimmedPeerId] = lanAddrs
                            Timber.i("TCP/mDNS: LAN peer detected $trimmedPeerId with ${lanAddrs.size} local addresses")
                        } else {
                            mdnsLanPeers.remove(trimmedPeerId)
                        }

	                        val dialCandidates = buildDialCandidatesForPeer(
	                            routePeerId = peerId,
	                            rawAddresses = listenAddrs,
	                            includeRelayCircuits = true
	                        )

	                        val syncPeerIds = linkedSetOf(peerId.trim())
	                        val isHeadless = agentVersion.contains("/headless/")
	                        val transportIdentity = resolveTransportIdentity(peerId)
	                        val shouldTreatAsHeadless = isBootstrapRelayPeer(peerId) || (isHeadless && transportIdentity == null)
	                        if (shouldTreatAsHeadless) {
	                            Timber.i("Headless/Relay transport node identified: $peerId (agent: $agentVersion)")
	                            emitConnectedIfChanged(
	                                peerId = peerId,
	                                transport = com.scmessenger.android.service.TransportType.INTERNET
	                            )
	                            transportToCanonicalMap[peerId] = peerId // Relay counts as its own canonical
	                            activeSessions[peerId] = System.currentTimeMillis()
	                        } else {
	                            if (isHeadless && transportIdentity != null) {
	                                Timber.i("Promoting peer $peerId to full node: identity resolved despite headless agent $agentVersion")
	                            }
	                            if (transportIdentity != null) {
	                                syncPeerIds.add(transportIdentity.canonicalPeerId)
	                        }
                            val canonicalId = transportIdentity?.canonicalPeerId ?: peerId
                            transportToCanonicalMap[peerId] = canonicalId
                            activeSessions[peerId] = System.currentTimeMillis()

                            val discoveredNickname = prepopulateDiscoveryNickname(
                                nickname = transportIdentity?.nickname,
                                peerId = transportIdentity?.canonicalPeerId ?: peerId,
                                publicKey = transportIdentity?.publicKey
                            )

                            // Update discovery map
                            val peerTransportType = if (mdnsLanPeers.containsKey(trimmedPeerId))
                                com.scmessenger.android.service.TransportType.TCP_MDNS
                            else
                                com.scmessenger.android.service.TransportType.INTERNET
                            val discoveryInfo = PeerDiscoveryInfo(
                                peerId = transportIdentity?.canonicalPeerId ?: peerId,
                                publicKey = transportIdentity?.publicKey,
                                nickname = discoveredNickname,
                                localNickname = transportIdentity?.localNickname,
                                libp2pPeerId = peerId,
                                transport = peerTransportType,
                                isFull = transportIdentity != null,
                                isRelay = isBootstrapRelayPeer(peerId),
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
                                // FIX: Auto-create contact even when transportIdentity is null
                                // This happens on fresh install when no contact exists yet
                                if (!isBootstrapRelayPeer(peerId) && !isHeadless) {
                                    val extractedKey = try {
                                        ironCore?.extractPublicKeyFromPeerId(peerId)
                                    } catch (_: Exception) { null }

                                    if (extractedKey != null) {
                                        val normalizedKey = normalizePublicKey(extractedKey)
                                        if (normalizedKey != null) {
                                            // ID-STANDARDIZATION-002: Check for existing contact before auto-creating
                                            val existingContact = try {
                                                contactManager?.list()?.firstOrNull {
                                                    normalizePublicKey(it.publicKey) == normalizedKey
                                                }
                                            } catch (e: Exception) {
                                                null
                                            }
                                            
                                            if (existingContact != null) {
                                                Timber.i("Contact already exists for public key ${normalizedKey.take(8)}... (peerId=${existingContact.peerId}), skipping auto-creation")
                                                // Update last seen for existing contact
                                                try { contactManager?.updateLastSeen(existingContact.peerId) } catch (_: Exception) {}
                                            } else {
                                                // ID-STANDARDIZATION-003: Use proper canonical ID resolution
                                                val canonicalId = try {
                                                    validateAndStandardizeId(peerId, "auto-contact-creation")
                                                } catch (e: Exception) {
                                                    Timber.w("Using fallback peerId for contact creation: ${e.message}")
                                                    PeerIdValidator.normalize(peerId)
                                                }
                                                
                                                Timber.i("Auto-creating contact for newly discovered peer: $peerId -> $canonicalId (extracted key: ${normalizedKey.take(8)}...)")
                                                repoScope.launch {
                                                    upsertFederatedContact(
                                                        canonicalPeerId = canonicalId,
                                                        publicKey = normalizedKey,
                                                        nickname = null,
                                                        libp2pPeerId = peerId,
                                                        listeners = dialCandidates,
                                                        createIfMissing = true
                                                    )
                                                }
                                            }
                                        }
                                    } else {
                                        Timber.w("Could not extract public key from peer $peerId for auto-contact creation")
                                    }
                                }
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
                                // Don't auto-create contacts for relay peers - they are infrastructure, not user contacts
                                if (!isBootstrapRelayPeer(peerId)) {
                                    upsertFederatedContact(
                                        canonicalPeerId = transportIdentity.canonicalPeerId,
                                        publicKey = transportIdentity.publicKey,
                                        nickname = transportIdentity.nickname,
                                        libp2pPeerId = peerId,
                                        listeners = dialCandidates,
                                        createIfMissing = true  // AUTO-CREATE contacts for all discovered peers
                                    )
                                    Timber.i("Auto-created/updated contact for peer: ${transportIdentity.canonicalPeerId} (nickname: ${transportIdentity.nickname})")
                                } else {
                                    Timber.d("Skipping contact creation for relay peer: $peerId")
                                }
                            }
                            sendIdentitySyncIfNeeded(
                                routePeerId = peerId,
                                knownPublicKey = transportIdentity?.publicKey
                            )
                            sendHistorySyncIfNeeded(
                                routePeerId = peerId,
                                knownPublicKey = transportIdentity?.publicKey
                            )
	                    }

                        // Identified implies an active session exists; avoid immediate re-dial loops.
                        syncPeerIds
                            .map { it.trim() }
                            .filter { it.isNotEmpty() }
                            .forEach { promotePendingOutboundForPeer(peerId = it) }
                        flushPendingOutbox("peer_identified:$peerId")
                        updateBleIdentityBeacon()
                    }
                }

                override fun onPeerDisconnected(peerId: String) {
                    // P0: Deduplicate disconnect events — libp2p may fire multiple per session
                    val trimmedPeerId = peerId.trim()
                    val now = System.currentTimeMillis()
                    val lastDisconnected = peerDisconnectDedupCache[trimmedPeerId]
                    if (lastDisconnected != null && (now - lastDisconnected) < peerDisconnectDedupIntervalMs) {
                        return
                    }
                    peerDisconnectDedupCache[trimmedPeerId] = now

                    Timber.d("Core notified disconnected: $peerId")
                    repoScope.launch {
                        val canonicalId = transportToCanonicalMap.remove(peerId) ?: peerId
                        activeSessions.remove(peerId)

                        // Remove disconnected aliases (peerId + canonical + same-key aliases).
                        pruneDisconnectedPeer(peerId)
                        if (canonicalId != peerId) {
                            pruneDisconnectedPeer(canonicalId)
                        }
                        mdnsLanPeers.remove(trimmedPeerId)

                        emitDisconnectedIfChanged(
                            peerId = peerId
                        )
                        if (canonicalId != peerId) {
                            emitDisconnectedIfChanged(
                                peerId = canonicalId
                            )
                        }
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
                    logDeliveryAttempt(
                        messageId = messageId,
                        medium = "core",
                        phase = "rx",
                        outcome = "received",
                        detail = "sender=$senderId",
                        callerContext = "onMessageReceived"
                    )
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
                            ?.takeIf { PeerIdValidator.isLibp2pPeerId(it) }
                        val routeWifiPeerId = senderId.takeIf { isWifiPeerId(it) }
                        val routePeerId = senderId.takeIf { PeerIdValidator.isLibp2pPeerId(it) } ?: hintedRoutePeerId
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
                        val messageKind = decodedPayload.kind.trim().lowercase()
                        val isChatEvent = messageKind == "text" || messageKind.isEmpty()

                        val existingContact = try { contactManager?.get(canonicalPeerId) } catch (e: Exception) { null }
                        if (existingContact == null && normalizedSenderKey != null && isChatEvent) {
                            var routeNotes = if (!routePeerId.isNullOrBlank()) {
                                appendRoutingHint(notes = null, key = "libp2p_peer_id", value = routePeerId)
                            } else {
                                null
                            }
                            routeNotes = appendRoutingHint(
                                notes = routeNotes,
                                key = "wifi_peer_id",
                                value = routeWifiPeerId
                            )
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
                                notes = routeNotes,
                                lastKnownDeviceId = verifiedHints?.deviceId
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
                                    notes = existingContact.notes,
                                    lastKnownDeviceId = verifiedHints?.deviceId ?: existingContact.lastKnownDeviceId
                                )
                                try {
                                    contactManager?.add(updatedContact)
                                } catch (e: Exception) {
                                    Timber.d("Failed to persist nickname hint for ${existingContact.peerId}: ${e.message}")
                                }
                            }

                            val currentRouting = parseRoutingHints(existingContact.notes)
                            val normalizedRoutePeerId = routePeerId?.trim()?.takeIf { it.isNotEmpty() }
                            val normalizedRouteWifiPeerId = routeWifiPeerId?.trim()?.takeIf { it.isNotEmpty() }

                            // Persist updated libp2p alias mapping when known so identity/libp2p IDs
                            // stay canonicalized to one conversation thread across peer-id rotations.
                            if (!normalizedRoutePeerId.isNullOrBlank() &&
                                normalizedSenderKey != null &&
                                normalizePublicKey(existingContact.publicKey) == normalizedSenderKey &&
                                currentRouting.libp2pPeerId?.trim() != normalizedRoutePeerId
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
                                    notes = updatedNotesWithListeners,
                                    lastKnownDeviceId = verifiedHints?.deviceId ?: existingContact.lastKnownDeviceId
                                )
                                try {
                                    contactManager?.add(updatedContact)
                                } catch (e: Exception) {
                                    Timber.d("Failed to persist libp2p alias hint for ${existingContact.peerId}: ${e.message}")
                                }
                            }

                            if (!normalizedRouteWifiPeerId.isNullOrBlank() &&
                                normalizedSenderKey != null &&
                                normalizePublicKey(existingContact.publicKey) == normalizedSenderKey &&
                                currentRouting.wifiPeerId?.trim() != normalizedRouteWifiPeerId
                            ) {
                                val updatedNotes = appendRoutingHint(existingContact.notes, "wifi_peer_id", routeWifiPeerId)
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
                                    notes = updatedNotesWithListeners,
                                    lastKnownDeviceId = verifiedHints?.deviceId ?: existingContact.lastKnownDeviceId
                                )
                                try {
                                    contactManager?.add(updatedContact)
                                } catch (e: Exception) {
                                    Timber.d("Failed to persist WiFi alias hint for ${existingContact.peerId}: ${e.message}")
                                }
                            }
                        }

                        if (normalizedSenderKey != null) {
                            repoScope.launch {
                                upsertFederatedContact(
                                    canonicalPeerId = canonicalPeerId,
                                    publicKey = normalizedSenderKey,
                                    nickname = knownNickname,
                                    libp2pPeerId = routePeerId,
                                    wifiPeerId = routeWifiPeerId,
                                    listeners = hintedDialCandidates,
                                    deviceId = verifiedHints?.deviceId,
                                    createIfMissing = false
                                )
                            }
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
                                libp2pPeerId = routePeerId,
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

                        if (messageKind == "identity_sync") {
                            Timber.d("Processed identity sync from $canonicalPeerId (route=$routePeerId)")
                            sendDeliveryReceiptAsync(
                                senderPublicKeyHex = senderPublicKeyHex,
                                messageId = messageId,
                                senderId = canonicalPeerId,
                                preferredRoutePeerId = routePeerId,
                                preferredWifiPeerId = routeWifiPeerId,
                                preferredListenerHints = hintedDialCandidates
                            )
                            return
                        }

                        if (messageKind == "history_sync") {
                            Timber.d("Processed history sync request from $canonicalPeerId")
                            sendHistorySyncDataIfNeeded(canonicalPeerId, routePeerId, senderPublicKeyHex, hintedDialCandidates, routeWifiPeerId)
                            sendDeliveryReceiptAsync(senderPublicKeyHex, messageId, canonicalPeerId, routePeerId, routeWifiPeerId, preferredListenerHints = hintedDialCandidates)
                            return
                        }
                        if (messageKind == "history_sync_data") {
                            Timber.d("Processed history sync data from $canonicalPeerId")
                            try {
                                val arr = org.json.JSONArray(decodedPayload.text)
                                for (i in 0 until arr.length()) {
                                    val obj = arr.getJSONObject(i)
                                    val msgId = obj.getString("id")
                                    val existing = historyManager?.get(msgId)
                                    if (existing == null) {
                                        val record = uniffi.api.MessageRecord(
                                            id = msgId,
                                            direction = if (obj.getString("dir") == "sent") uniffi.api.MessageDirection.RECEIVED else uniffi.api.MessageDirection.SENT,
                                            peerId = canonicalPeerId,
                                            content = obj.getString("txt"),
                                            timestamp = obj.getLong("ts").toULong(),
                                            senderTimestamp = obj.getLong("sts").toULong(),
                                            delivered = obj.getBoolean("del"),
                                            hidden = false
                                        )
                                        historyManager?.add(record)
                                        repoScope.launch { _messageUpdates.emit(record) }
                                    } else {
                                        val dirStr = obj.optString("dir")
                                        val isPeerReflectingRecv = dirStr == "recv"
                                        if (existing.direction == uniffi.api.MessageDirection.SENT && !existing.delivered && isPeerReflectingRecv) {
                                            historyManager?.markDelivered(msgId)
                                            val updated = historyManager?.get(msgId)
                                            if (updated != null) {
                                                repoScope.launch { _messageUpdates.emit(updated) }
                                                repoScope.launch {
                                                    com.scmessenger.android.service.MeshEventBus.emitMessageEvent(
                                                        com.scmessenger.android.service.MessageEvent.Delivered(msgId)
                                                    )
                                                }
                                            }
                                            removePendingOutbound(msgId)
                                        }
                                    }
                                }
                                historyManager?.flush()
                            } catch (e: Exception) { Timber.e(e, "Failed to parse history_sync_data") }
                            sendDeliveryReceiptAsync(senderPublicKeyHex, messageId, canonicalPeerId, routePeerId, routeWifiPeerId, preferredListenerHints = hintedDialCandidates)
                            return
                        }

                        val existingRecord = try {
                            historyManager?.get(messageId)
                        } catch (_: Exception) {
                            null
                        }
                        if (existingRecord?.direction == uniffi.api.MessageDirection.RECEIVED) {
                            Timber.d("Duplicate inbound message $messageId from $senderId; acknowledging without re-emitting UI")
                            sendDeliveryReceiptAsync(
                                senderPublicKeyHex = senderPublicKeyHex,
                                messageId = messageId,
                                senderId = canonicalPeerId,
                                preferredRoutePeerId = routePeerId,
                                preferredWifiPeerId = routeWifiPeerId,
                                preferredListenerHints = hintedDialCandidates
                            )
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
                            senderTimestamp = senderTimestamp,
                            delivered = true,
                            hidden = false
                        )
                        historyManager?.add(record)
                        logDeliveryAttempt(
                            messageId = messageId,
                            medium = "core",
                            phase = "rx",
                            outcome = "processed",
                            detail = "stored_in_history sender=$senderId",
                            callerContext = "onMessageReceived_processing"
                        )

                        // Emit for notifications and UI updates
                        repoScope.launch {
                            _messageUpdates.emit(record)
                        }

                        // Send delivery receipt ACK back to sender.
                        sendDeliveryReceiptAsync(
                            senderPublicKeyHex = senderPublicKeyHex,
                            messageId = messageId,
                            senderId = canonicalPeerId,
                            preferredRoutePeerId = routePeerId,
                            preferredWifiPeerId = routeWifiPeerId,
                            preferredListenerHints = hintedDialCandidates
                        )
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
                    val existingRecord = try {
                        historyManager?.get(messageId)
                    } catch (_: Exception) {
                        null
                    }
                    val hasPendingForReceipt = loadPendingOutbox().any { it.historyRecordId == messageId }
                    if (existingRecord == null || existingRecord.direction != uniffi.api.MessageDirection.SENT) {
                        if (hasPendingForReceipt) {
                            removePendingOutbound(messageId)
                            logDeliveryState(
                                messageId = messageId,
                                state = "delivered",
                                detail = "delivery_receipt_recovered_without_history status=$normalized direction=${existingRecord?.direction ?: "missing"}"
                            )
                            return
                        }
                        logDeliveryState(
                            messageId = messageId,
                            state = "pending",
                            detail = "delivery_receipt_ignored_non_outbound status=$normalized direction=${existingRecord?.direction ?: "missing"}"
                        )
                        return
                    }
                    val wasAlreadyDelivered = existingRecord.delivered
                    val firstReceiptSeen = markDeliveredReceiptSeen(messageId)
                    if (!firstReceiptSeen && wasAlreadyDelivered) {
                        removePendingOutbound(messageId)
                        logDeliveryState(
                            messageId = messageId,
                            state = "delivered",
                            detail = "delivery_receipt_duplicate_status=$normalized"
                        )
                        return
                    }
                    if (!wasAlreadyDelivered) {
                        historyManager?.markDelivered(messageId)
                        historyManager?.flush()
                    }
                    removePendingOutbound(messageId)
                    val refreshedRecord = try {
                        historyManager?.get(messageId)
                    } catch (_: Exception) {
                        null
                    }
                    if (refreshedRecord != null) {
                        repoScope.launch {
                            _messageUpdates.emit(refreshedRecord)
                        }
                    }
                    if (wasAlreadyDelivered) return
                    ironCore?.markMessageSent(messageId)
                    // Bridge to ChatViewModel: emit Delivered so UI delivery indicator updates
                    repoScope.launch {
                        com.scmessenger.android.service.MeshEventBus.emitMessageEvent(
                            com.scmessenger.android.service.MessageEvent.Delivered(messageId)
                        )
                    }
                    logDeliveryState(
                        messageId = messageId,
                        state = "delivered",
                        detail = "delivery_receipt_status=$normalized"
                    )
                }
            }
            ironCore?.setDelegate(coreDelegate)

            // 4. Start Android transports. Individual transport failures should
            // not abort the entire mesh core lifecycle.
            repoScope.launch {
                try {
                    initializeAndStartBle()
                } catch (e: Exception) {
                    Timber.w(e, "BLE transport failed to initialize; continuing with remaining transports")
                }
                try {
                    initializeAndStartWifi()
                } catch (e: Exception) {
                    Timber.w(e, "WiFi transport failed to initialize; continuing with remaining transports")
                }
                try {
                    initializeAndStartSwarm()
                } catch (e: Exception) {
                    Timber.w(e, "Swarm transport failed to initialize; core service remains active")
                }
                ensurePendingOutboxRetryLoop()
                ensureCoverTrafficLoop()
                flushPendingOutbox("service_started")
            }

            // 5. Update State
            _serviceState.value = meshService?.getState() ?: uniffi.api.ServiceState.STOPPED
            if (_serviceState.value != uniffi.api.ServiceState.RUNNING) {
                throw IllegalStateException("MeshService did not reach RUNNING state")
            }
            serviceStartedAtEpochSec = System.currentTimeMillis() / 1000
            updateStats()
            startPeriodicStatsUpdate()

            // On reinstall with missing data: broadcast identity beacon after swarm connects
            // so nearby and relay-connected peers can re-send identity info.
            if (isReinstallWithMissingData) {
                isReinstallWithMissingData = false
                Timber.i("Reinstall recovery: scheduling post-start identity beacon")
                repoScope.launch {
                    kotlinx.coroutines.delay(4_000L)
                    updateBleIdentityBeacon()
                    Timber.i("Reinstall recovery beacon sent")
                }
            }

            val info = ironCore?.getIdentityInfo()
            Timber.i("SC_IDENTITY_OWN p2p_id=${info?.libp2pPeerId ?: "unknown"} pk=${info?.publicKeyHex ?: "unknown"}")
            Timber.i("Mesh service started successfully")
        } catch (e: Exception) {
            Timber.e(e, "Failed to start mesh service")
            stopMeshService()
            throw IllegalStateException("Mesh service startup failed", e)
        }
    }

    private fun sendDeliveryReceiptAsync(
        senderPublicKeyHex: String,
        messageId: String,
        senderId: String,
        preferredRoutePeerId: String? = null,
        preferredWifiPeerId: String? = null,
        preferredBlePeerId: String? = null,
        preferredListenerHints: List<String> = emptyList()
    ) {
        // ✅ BLOCKING: Skip receipt if sender is blocked (relay unaffected)
        if (isBlocked(senderId)) {
            Timber.i("📛 Blocking: Skipping receipt for blocked peer $senderId (relay unaffected)")
            return
        }

        val normalizedMessageId = messageId.trim()
        if (normalizedMessageId.isEmpty()) return

        val candidateJob = repoScope.launch(
            kotlinx.coroutines.Dispatchers.IO,
            start = kotlinx.coroutines.CoroutineStart.LAZY
        ) {
            try {
                for (attempt in 1..receiptSendMaxAttempts) {
                    val receiptBytes = ironCore?.prepareReceipt(senderPublicKeyHex, normalizedMessageId)
                    if (receiptBytes == null) {
                        Timber.d("Skipping delivery receipt for $normalizedMessageId: prepareReceipt returned null")
                        return@launch
                    }
                    val contact = try { contactManager?.get(senderId) } catch (_: Exception) { null }
                    val hints = parseRoutingHints(contact?.notes)
                    val routeCandidates = buildRoutePeerCandidates(
                        peerId = senderId,
                        cachedRoutePeerId = preferredRoutePeerId ?: hints.libp2pPeerId,
                        notes = contact?.notes,
                        recipientPublicKey = senderPublicKeyHex
                    )
                    val delivery = attemptDirectSwarmDelivery(
                        routePeerCandidates = routeCandidates,
                        listeners = (preferredListenerHints + hints.listeners).distinct(),
                        encryptedData = receiptBytes,
                        wifiPeerId = preferredWifiPeerId ?: hints.wifiPeerId,
                        blePeerId = preferredBlePeerId ?: hints.blePeerId,
                        traceMessageId = normalizedMessageId,
                        attemptContext = "receipt_send",
                    )
                    if (delivery.acked) {
                        logDeliveryAttempt(
                            messageId = normalizedMessageId,
                            medium = "receipt",
                            phase = "aggregate",
                            outcome = "acked",
                            detail = "ctx=receipt_send sender=$senderId attempt=$attempt"
                        )
                        Timber.d("Targeted delivery receipt sent for $normalizedMessageId to $senderId")
                        return@launch
                    }

                    if (attempt < receiptSendMaxAttempts) {
                        val delaySec = receiptRetryDelaySec(attempt)
                        logDeliveryAttempt(
                            messageId = normalizedMessageId,
                            medium = "receipt",
                            phase = "retry",
                            outcome = "scheduled",
                            detail = "ctx=receipt_send sender=$senderId attempt=$attempt delay_sec=$delaySec"
                        )
                        kotlinx.coroutines.delay(delaySec * 1000L)
                    } else {
                        logDeliveryAttempt(
                            messageId = normalizedMessageId,
                            medium = "receipt",
                            phase = "aggregate",
                            outcome = "exhausted",
                            detail = "ctx=receipt_send sender=$senderId attempts=$receiptSendMaxAttempts"
                        )
                    }
                }
            } catch (e: kotlinx.coroutines.CancellationException) {
                throw e
            } catch (e: Exception) {
                Timber.d("Failed to send delivery receipt for $normalizedMessageId: ${e.message}")
            } finally {
                pendingReceiptSendJobs.remove(normalizedMessageId)
            }
        }

        val existingJob = pendingReceiptSendJobs.putIfAbsent(normalizedMessageId, candidateJob)
        if (existingJob != null && existingJob.isActive) {
            candidateJob.cancel()
            logDeliveryAttempt(
                messageId = normalizedMessageId,
                medium = "receipt",
                phase = "dedupe",
                outcome = "skipped_active",
                detail = "ctx=receipt_send sender=$senderId"
            )
            return
        }
        if (existingJob != null) {
            pendingReceiptSendJobs[normalizedMessageId] = candidateJob
        }
        candidateJob.invokeOnCompletion {
            pendingReceiptSendJobs.remove(normalizedMessageId, candidateJob)
        }
        candidateJob.start()
    }

    private fun receiptRetryDelaySec(attempt: Int): Long {
        val exponent = (attempt - 1).coerceIn(0, 3)
        return 1L shl exponent
    }

    private fun sendIdentitySyncIfNeeded(routePeerId: String, knownPublicKey: String? = null) {
        val normalizedRoute = routePeerId.trim()
        if (normalizedRoute.isEmpty() || isBootstrapRelayPeer(normalizedRoute)) return

        // Check if core is initialized before attempting identity sync
        if (ironCore == null) {
            Timber.d("sendIdentitySyncIfNeeded: IronCore not initialized, skipping for $normalizedRoute")
            return
        }

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
                val prepared = ironCore?.prepareMessageWithId(recipientPublicKey, payload, null)
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
                    notes = contact?.notes,
                    recipientPublicKey = recipientPublicKey
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


    private fun sendHistorySyncIfNeeded(routePeerId: String, knownPublicKey: String? = null) {
        val normalizedRoute = routePeerId.trim()
        Timber.w("sendHistorySyncIfNeeded called for $normalizedRoute")
        if (normalizedRoute.isEmpty() || isBootstrapRelayPeer(normalizedRoute)) return

        // Gate on both IronCore instance availability AND identity readiness.
        // A non-null IronCore with an un-initialized identity will throw
        // IronCoreException.NotInitialized from prepareMessageWithId, producing
        // noisy false-positive error logs on fresh-install startup.
        val core = ironCore ?: run {
            Timber.w("sendHistorySyncIfNeeded: IronCore not initialized, skipping for $normalizedRoute")
            return
        }
        val identityReady = try {
            core.getIdentityInfo().identityId != null
        } catch (_: uniffi.api.IronCoreException.NotInitialized) {
            false
        } catch (e: Exception) {
            Timber.w(e, "sendHistorySyncIfNeeded: unexpected error checking identity readiness for $normalizedRoute")
            false
        }
        if (!identityReady) {
            Timber.w("sendHistorySyncIfNeeded: identity not ready yet, skipping for $normalizedRoute")
            return
        }

        val now = System.currentTimeMillis()
        val lastSent = historySyncSentPeers[normalizedRoute] ?: 0L
        val shouldSend = (now - lastSent) > HISTORY_SYNC_COOLDOWN_MS
        Timber.w("sendHistorySyncIfNeeded shouldSend=$shouldSend for $normalizedRoute (age=${now - lastSent}ms)")
        if (!shouldSend) return
        historySyncSentPeers[normalizedRoute] = now

        repoScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            try {
                val extractedPublicKey = try { ironCore?.extractPublicKeyFromPeerId(normalizedRoute) } catch (_: Exception) { null }
                val recipientPublicKey = normalizePublicKey(knownPublicKey) ?: normalizePublicKey(extractedPublicKey)
                if (recipientPublicKey == null) {
                    Timber.e("sendHistorySyncIfNeeded: missing recipientPublicKey for $normalizedRoute (known=$knownPublicKey, extracted=$extractedPublicKey)")
                    historySyncSentPeers.remove(normalizedRoute)
                    return@launch
                }

                val payload = encodeMeshMessagePayload(content = "", kind = "history_sync")
                val prepared = try { ironCore?.prepareMessageWithId(recipientPublicKey, payload, null) } catch (e: Exception) { Timber.e(e, "prepareMessageWithId failed in history_sync"); null }
                if (prepared == null) {
                    Timber.e("sendHistorySyncIfNeeded: prepared is null for $normalizedRoute")
                    historySyncSentPeers.remove(normalizedRoute)
                    return@launch
                }

                val contact = contactManager?.list()?.firstOrNull { it.peerId == normalizedRoute || parseRoutingHints(it.notes).libp2pPeerId == normalizedRoute }
                val hints = parseRoutingHints(contact?.notes)
                val routeCandidates = buildRoutePeerCandidates(contact?.peerId ?: normalizedRoute, normalizedRoute, contact?.notes, recipientPublicKey)

                attemptDirectSwarmDelivery(routeCandidates, hints.listeners, prepared.envelopeData, hints.wifiPeerId, hints.blePeerId)
                Timber.w("History sync request sent to $normalizedRoute")
            } catch (e: Exception) {
                Timber.e(e, "History sync error for $normalizedRoute")
                historySyncSentPeers.remove(normalizedRoute)
            }
        }
    }

    private val historySyncDataInProgress = java.util.concurrent.ConcurrentHashMap<String, Boolean>()

    private fun sendHistorySyncDataIfNeeded(canonicalPeerId: String, routePeerId: String?, recipientPublicKey: String, listeners: List<String>, wifiPeerId: String?) {
        if (historySyncDataInProgress.putIfAbsent(canonicalPeerId, true) != null) {
            Timber.d("sendHistorySyncDataIfNeeded: already in progress for $canonicalPeerId")
            return
        }
        repoScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            try {
                val recentMsgs = historyManager?.conversation(canonicalPeerId, 400u)?.sortedBy { it.timestamp } ?: emptyList()
                if (recentMsgs.isEmpty()) {
                    Timber.w("sendHistorySyncDataIfNeeded: no recent msgs for $canonicalPeerId")
                    return@launch
                }
                Timber.w("sendHistorySyncDataIfNeeded: compiling ${recentMsgs.size} msgs for $canonicalPeerId")

                val hints = parseRoutingHints(contactManager?.get(canonicalPeerId)?.notes)
                val routeCandidates = buildRoutePeerCandidates(canonicalPeerId, routePeerId ?: hints.libp2pPeerId, contactManager?.get(canonicalPeerId)?.notes, recipientPublicKey)
                val allListeners = (listeners + hints.listeners).distinct()

                // Chunk into batches of 20 to stay within encryption payload limits
                val batchSize = 20
                val batches = recentMsgs.chunked(batchSize)
                var sentBatches = 0

                for ((batchIndex, batch) in batches.withIndex()) {
                    val arr = org.json.JSONArray()
                    batch.forEach { msg ->
                        val obj = org.json.JSONObject()
                        obj.put("id", msg.id)
                        obj.put("dir", if (msg.direction == uniffi.api.MessageDirection.SENT) "sent" else "recv")
                        obj.put("pid", msg.peerId)
                        obj.put("txt", msg.content)
                        obj.put("ts", msg.timestamp.toLong())
                        obj.put("sts", msg.senderTimestamp.toLong())
                        obj.put("del", msg.delivered)
                        arr.put(obj)
                    }
                    val payload = encodeMeshMessagePayload(content = arr.toString(), kind = "history_sync_data")
                    val prepared = try { ironCore?.prepareMessageWithId(recipientPublicKey, payload, null) } catch (e: Exception) {
                        Timber.e(e, "prepareMessage failed in sync_data batch $batchIndex (${batch.size} msgs)")
                        null
                    }
                    if (prepared == null) {
                        Timber.e("sendHistorySyncDataIfNeeded: prepared is null for batch $batchIndex of $canonicalPeerId")
                        continue
                    }

                    attemptDirectSwarmDelivery(routeCandidates, allListeners, prepared.envelopeData, wifiPeerId ?: hints.wifiPeerId, hints.blePeerId)
                    sentBatches++
                    // Small delay between batches to avoid overwhelming BLE
                    if (batchIndex < batches.size - 1) {
                        kotlinx.coroutines.delay(200)
                    }
                }
                Timber.w("History sync data sent to $canonicalPeerId ($sentBatches/${batches.size} batches, ${recentMsgs.size} items total)")
            } catch (e: Exception) {
                Timber.e(e, "sendHistorySyncDataIfNeeded error for $canonicalPeerId")
            } finally {
                historySyncDataInProgress.remove(canonicalPeerId)
            }
        }
    }
    private suspend fun initializeAndStartBle() {
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
                    noteBleRouteObservation(peerId = peerId, bleAddress = peerId, source = "scan")
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
        repoScope.launch {
            bleScanner?.startScanning()
        }

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

        // Identity beacon data served to BLE scanners via IDENTITY_CHAR_UUID reads.
        // Initialize with a valid empty JSON to avoid platform parsing errors during startup.
        bleGattServer?.setIdentityData(identityData)

        // Set identity beacon on BLE GATT server so nearby peers can read our Ed25519 public key
        updateBleIdentityBeacon()
    }

    // Identity beacon data served to BLE scanners via IDENTITY_CHAR_UUID reads.
    // Initialize with a valid empty JSON to avoid platform parsing errors during startup.
    private var identityData: ByteArray = "{}".toByteArray()

    private fun updateBleIdentityBeacon() {
        val identity = ironCore?.getIdentityInfo()
        val publicKeyHex = identity?.publicKeyHex
        if (!publicKeyHex.isNullOrEmpty()) {
            val now = System.currentTimeMillis()
            if (now - lastBleBeaconUpdateMillis < 5000) {
                // Throttled: wait for the next call or periodic update
                return
            }
            lastBleBeaconUpdateMillis = now

            // 1. Immediate update with just identity (no listeners yet)
            // This allows peers to "see" us in the UI immediately while we wait for swarm listeners to bind.
            setIdentityBeaconInternal(identity, emptyList())

            // 2. Delayed update with full connection hints
            repoScope.launch(kotlinx.coroutines.Dispatchers.IO) {
                var listeners = getListeningAddresses()
                var attempts = 0
                while (listeners.isEmpty() && attempts < 10 && isActive) {
                    kotlinx.coroutines.delay(500)
                    listeners = getListeningAddresses()
                    attempts++
                }
                if (isActive) {
                    setIdentityBeaconInternal(identity, listeners)
                }
            }
        }
    }

    private fun setIdentityBeaconInternal(identity: uniffi.api.IdentityInfo, listeners: List<String>) {
        val publicKeyHex = identity.publicKeyHex ?: return
        try {
            // Keep BLE identity beacons compact
            var resolvedListeners = normalizeOutboundListenerHints(listeners).take(2)
            var resolvedExternal = normalizeExternalAddressHints(getExternalAddresses()).take(2)
            val nickname = (identity.nickname ?: "").take(32)

            fun buildBeacon(): org.json.JSONObject {
                val connectionHints = (resolvedListeners + resolvedExternal).distinct()
                // UNIFIED ID FIX: BLE beacon primary ID is libp2p_peer_id (network routable)
                return org.json.JSONObject()
                    .put("peer_id", identity.libp2pPeerId ?: "")           // PRIMARY: libp2p Peer ID
                    .put("public_key", publicKeyHex)                         // Canonical identity key
                    .put("device_id", identity.deviceId ?: "")             // Multi-device routing
                    .put("identity_id", identity.identityId ?: "")         // SECONDARY: Blake3 hash
                    .put("nickname", nickname)
                    .put("libp2p_peer_id", identity.libp2pPeerId ?: "")    // Backward compatibility
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
                    .put("peer_id", identity.libp2pPeerId ?: "")           // PRIMARY
                    .put("public_key", publicKeyHex)
                    .put("device_id", identity.deviceId ?: "")
                    .put("identity_id", identity.identityId ?: "")
                    .put("nickname", nickname)
                    .put("libp2p_peer_id", identity.libp2pPeerId ?: "")
                    .put("listeners", org.json.JSONArray())
                    .put("external_addresses", org.json.JSONArray())
                    .put("connection_hints", org.json.JSONArray())
                beaconJson = beaconJsonObject.toString().toByteArray(Charsets.UTF_8)
            }
            val publishedAt = System.currentTimeMillis()
            if (
                beaconJson.contentEquals(lastBleBeaconPayload) &&
                (publishedAt - lastBleBeaconPayloadPublishedAtMillis) < 5000
            ) {
                return
            }
            lastBleBeaconPayload = beaconJson.copyOf()
            lastBleBeaconPayloadPublishedAtMillis = publishedAt
            bleAdvertiser?.updateIdentityBeacon(beaconJson)
            bleGattServer?.setIdentityData(beaconJson)
            identityData = beaconJson // Store for immediate use by GATT server
            Timber.i(
                "BLE GATT identity beacon updated: ${publicKeyHex.take(8)}... " +
                    "(${beaconJson.size} bytes, listeners=${resolvedListeners.size}, external=${resolvedExternal.size}) " +
                    "p2p_id=${identity.libp2pPeerId ?: "unknown"}"
            )
        } catch (e: Exception) {
            Timber.w("Failed to set BLE GATT identity beacon: ${e.message}")
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
            // UNIFIED ID FIX: BLE beacon primary ID is "peer_id" (libp2p Peer ID)
            val identityId = json.optString("identity_id", "")
                .trim()
                .takeIf { it.isNotBlank() }
                ?: blePeerId
            val rawNickname = (
                json.optString("nickname", "")
                    .ifBlank { json.optString("name", "") }
                ).trim()
            val libp2pPeerId = json.optString("peer_id", "")
                .trim()
                .takeIf { it.isNotBlank() }
                ?: json.optString("libp2p_peer_id", "")
                    .trim()
                    .takeIf { it.isNotBlank() }
                    ?: ""
            val listeners = json.optJSONArray("listeners")
            val externalAddresses = json.optJSONArray("external_addresses")
            val connectionHints = json.optJSONArray("connection_hints")
            val normalizedLibp2p = libp2pPeerId.takeIf { !it.isNullOrBlank() }?.trim()
            noteBleRouteObservation(peerId = blePeerId, bleAddress = blePeerId, source = "identity_self")
            noteBleRouteObservation(peerId = identityId, bleAddress = blePeerId, source = "identity_canonical")
            noteBleRouteObservation(peerId = normalizedLibp2p, bleAddress = blePeerId, source = "identity_route")

            val discoveredNickname = prepopulateDiscoveryNickname(
                nickname = rawNickname,
                peerId = identityId,
                publicKey = publicKeyHex
            )

            Timber.i(
                "Peer identity read from $blePeerId: ${publicKeyHex.take(8)}... " +
                    "identity=$identityId nickname='${discoveredNickname?.take(24) ?: ""}'"
            )

            // Persist BLE -> Identity mapping in contact notes so it survives restarts
            // and helps routing even when BLE is the only transport initially.
            repoScope.launch {
                try {
                    val contact = contactManager?.get(identityId)
                    if (contact != null) {
                        val updatedNotes = appendRoutingHint(contact.notes, "ble_peer_id", blePeerId)
                        if (updatedNotes != contact.notes) {
                            contactManager?.add(contact.copy(notes = updatedNotes))
                            Timber.d("Updated persistent BLE routing for $identityId: $blePeerId")
                        }
                    }
                } catch (e: Exception) {
                    Timber.w(e, "Failed to persist BLE routing for $identityId")
                }
            }

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

            repoScope.launch {
                listOfNotNull(identityId, blePeerId, normalizedLibp2p)
                    .map { it.trim() }
                    .filter { it.isNotEmpty() }
                    .forEach { promotePendingOutboundForPeer(peerId = it) }
                flushPendingOutbox("peer_discovered")
            }

            // Update discovery map
            val discoveryInfo = PeerDiscoveryInfo(
                peerId = identityId,
                publicKey = publicKeyHex,
                nickname = discoveredNickname,
                localNickname = try { contactManager?.get(identityId)?.localNickname } catch (_: Exception) { null },
                libp2pPeerId = normalizedLibp2p,
                transport = com.scmessenger.android.service.TransportType.BLE,
                isFull = true,
                lastSeen = System.currentTimeMillis().toULong() / 1000u
            )
            updateDiscoveredPeer(identityId, discoveryInfo)
            if (!normalizedLibp2p.isNullOrBlank()) {
                updateDiscoveredPeer(normalizedLibp2p, discoveryInfo)
            }
            // Remove the preliminary BLE-UUID entry (isFull=false) that was created when
            // the connection was first established before identity was read. Now that we
            // have the real identityId, the BLE UUID key is an duplicate we no longer need.
            if (blePeerId != identityId && blePeerId != normalizedLibp2p) {
                _discoveredPeers.update { current ->
                    current.filterKeys { key ->
                        key != blePeerId &&
                            // Also remove any entry whose peerId field matches the BLE UUID
                            current[key]?.peerId != blePeerId
                    }
                }
                Timber.d("Removed preliminary BLE entry $blePeerId → promoted to identity $identityId")
            }

            // Emit identity to nearby peers bus — UI will show peer in Nearby section for user to add
            val rawHints = mutableListOf<String>()
            listeners?.let { arr ->
                for (i in 0 until arr.length()) {
                    rawHints.add(arr.getString(i))
                }
            }
            externalAddresses?.let { arr ->
                for (i in 0 until arr.length()) {
                    rawHints.add(arr.getString(i))
                }
            }
            connectionHints?.let { arr ->
                for (i in 0 until arr.length()) {
                    rawHints.add(arr.getString(i))
                }
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
            // Trigger history sync over BLE when we discover a peer's identity
            if (!routePeerId.isNullOrEmpty()) {
                sendHistorySyncIfNeeded(routePeerId, publicKeyHex)
            } else {
                sendHistorySyncIfNeeded(identityId, publicKeyHex)
            }
            // Update lastSeen if already a saved contact
            try { contactManager?.updateLastSeen(blePeerId) } catch (_: Exception) { }
            try { contactManager?.updateLastSeen(identityId) } catch (_: Exception) { }
            routePeerId?.let {
                try { contactManager?.updateLastSeen(it) } catch (_: Exception) { }
            }
            repoScope.launch {
                upsertFederatedContact(
                    canonicalPeerId = publicKeyHex,       // UNIFIED ID FIX: canonical = public_key_hex
                    publicKey = publicKeyHex,
                    nickname = rawNickname.takeIf { it.isNotBlank() },
                    libp2pPeerId = routePeerId,
                    listeners = listenersStrings,
                    blePeerId = blePeerId,
                    createIfMissing = false
                )
            }

            if (!routePeerId.isNullOrEmpty() && listenersStrings.isNotEmpty()) {
                connectToPeer(routePeerId, listenersStrings)
            }
        } catch (e: Exception) {
            Timber.w("Failed to parse peer identity read: ${e.message}")
        }
    }

    private fun updateDiscoveredPeer(key: String, info: PeerDiscoveryInfo) {
        val normalizedKey = PeerIdValidator.normalize(key)
        _discoveredPeers.update { current ->
            val existing = current[normalizedKey]
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
                        libp2pPeerId = info.libp2pPeerId ?: existing.libp2pPeerId,
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
                val canonicalPeerId = PeerIdValidator.normalize(merged.peerId.ifEmpty { normalizedKey })
                val canonicalPublicKey = normalizePublicKey(merged.publicKey)

                val withCanonical = current + (canonicalPeerId to merged)
                withCanonical.filterNot { (mapKey, candidate) ->
                    // Cleanup duplicate entries for the same identity if they were added under different keys
                    mapKey != canonicalPeerId && PeerIdValidator.isSame(mapKey, canonicalPeerId)
                }
            }
        }
    }

    private fun noteBleRouteObservation(peerId: String?, bleAddress: String?, source: String) {
        val normalizedPeerId = peerId?.trim().orEmpty()
        val normalizedBleAddress = bleAddress?.trim().orEmpty()
        if (normalizedPeerId.isEmpty() || normalizedBleAddress.isEmpty()) return

        val observation = BleRouteObservation(
            address = normalizedBleAddress,
            lastSeenMs = System.currentTimeMillis(),
            source = source
        )
        bleRouteObservations[normalizedPeerId] = observation
        bleRouteObservations[normalizedBleAddress] = observation
    }

    private fun resolveFreshBlePeerId(candidates: List<String>): BleRouteObservation? {
        if (candidates.isEmpty()) return null

        val now = System.currentTimeMillis()
        return candidates
            .asSequence()
            .map { it.trim() }
            .filter { it.isNotEmpty() }
            .mapNotNull { candidate ->
                val observation = bleRouteObservations[candidate] ?: return@mapNotNull null
                val ageMs = now - observation.lastSeenMs
                
                // Extended freshness check for TRANSPORT-001: BLE hint staleness fix
                // 1. Within fresh TTL: use normally
                // 2. Within stale grace period: use for fallback (slightly stale but better than nothing)
                // 3. Beyond grace period: remove and skip
                when {
                    ageMs <= bleRouteFreshnessTtlMs -> observation  // Fresh
                    ageMs <= bleRouteStaleGraceMs -> {
                        // Stale but within grace period - use for fallback
                        Timber.d("Using stale BLE hint for $candidate (age=${ageMs}ms, grace=${bleRouteStaleGraceMs}ms)")
                        observation
                    }
                    else -> {
                        // Too stale, remove
                        bleRouteObservations.remove(candidate, observation)
                        null
                    }
                }
            }
            .maxByOrNull { it.lastSeenMs }
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
            // Configure bootstrap nodes for NAT traversal
            meshService?.setBootstrapNodes(DEFAULT_BOOTSTRAP_NODES)
            // Initiate swarm in Rust core.
            // Core auto-selects headless mode when identity is absent and upgrades when identity appears.
            // P0_TRANSPORT_001: Use static port 9001 for LAN connectivity with CLI daemon.
            // This ensures both sides can dial each other using predictable addresses.
            meshService?.startSwarm("/ip4/0.0.0.0/tcp/9001")

            // Obtain the SwarmBridge managed by Rust MeshService
            swarmBridge = meshService?.getSwarmBridge()
            updateBleIdentityBeacon()

            Timber.i("✓ Internet transport (Swarm) initiated and bridge wired")
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
                    // P0_ANDROID_010: Grant consent after successful backup restore.
                    // The Rust core starts with consent_granted=false, but if we have a
                    // persisted identity backup, the user already consented in a prior session.
                    core.grantConsent()
                    info = core.getIdentityInfo()
                }
            }
            if (!info.initialized) {
                Timber.i("Identity not initialized; onboarding required")
                return
            }

            // P0_ANDROID_010: Ensure consent is granted whenever identity is initialized.
            // This handles the case where identity was loaded from sled but consent wasn't set
            // (e.g., after a process restart where consent_granted resets to false).
            try {
                if (!core.isConsentGranted()) {
                    core.grantConsent()
                }
            } catch (_: Exception) {
                // isConsentGranted may not exist in all versions; grant unconditionally
                core.grantConsent()
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
            core.importIdentityBackup(backup, "")
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
            val backup = activeCore.exportIdentityBackup("")
            // P0_ANDROID_010: Use commit() for synchronous write.
            // apply() is async and can lose the backup if the process is killed
            // before the disk write completes (e.g., during an ANR crash).
            val committed = identityBackupPrefs.edit().putString(IDENTITY_BACKUP_KEY, backup).commit()
            if (!committed) {
                Timber.e("Failed to commit identity backup to SharedPreferences")
            }
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
        stopNetworkChangeWatch()
        networkDetector.stopMonitoring()
        pendingOutboxRetryJob?.cancel()
        pendingOutboxRetryJob = null
        coverTrafficJob?.cancel()
        coverTrafficJob = null
        pendingReceiptSendJobs.values.forEach { it.cancel() }
        pendingReceiptSendJobs.clear()

        try {
            kotlinx.coroutines.runBlocking { bleScanner?.stopScanning() }
        } catch (e: Exception) {
            Timber.w(e, "Failed to stop BLE scanner")
        }
        try {
            bleAdvertiser?.stopAdvertising()
        } catch (e: Exception) {
            Timber.w(e, "Failed to stop BLE advertiser")
        }
        try {
            bleGattServer?.stop()
        } catch (e: Exception) {
            Timber.w(e, "Failed to stop BLE GATT server")
        }
        try {
            bleGattClient?.cleanup()
        } catch (e: Exception) {
            Timber.w(e, "Failed to cleanup BLE GATT client")
        }

        kotlin.runCatching { wifiTransportManager?.stopDiscovery() }
            .onFailure { Timber.w(it, "Failed to stop WiFi transport") }

        kotlin.runCatching { swarmBridge?.shutdown() }
            .onFailure { Timber.w(it, "Failed to shutdown swarm bridge") }

        kotlin.runCatching { meshService?.stop() }
            .onFailure { Timber.w(it, "Failed to stop Rust mesh service") }
        
        identitySyncSentPeers.clear()
        historySyncSentPeers.clear()
        identityEmissionCache.clear()
        connectedEmissionCache.clear()
        mdnsLanPeers.clear()
        
        // Clear discovered peers from UI on service stop
        _discoveredPeers.value = emptyMap()
        Timber.i("Cleared all discovered peers on mesh service stop")

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
        serviceStartedAtEpochSec = 0L

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
     * Reset service runtime counters for a fresh diagnostics window.
     */
    fun resetServiceStats() {
        meshService?.resetStats()
        _serviceStats.value = meshService?.getStats()
        Timber.d("Mesh service stats reset")
    }

    /**
     * Called when WiFi connectivity is recovered. Immediately flushes
     * the pending outbox and re-primes relay connections to maximize
     * delivery opportunity.
     */
    fun notifyNetworkRecovered() {
        Timber.i("WiFi recovered — flushing pending outbox immediately")
        Timber.i("network_recovery wifi=true flush_triggered=true")
        repoScope.launch {
            primeRelayBootstrapConnections()
            flushPendingOutbox("wifi_recovered")
        }
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
                val fallbackUptimeSecs = if (serviceStartedAtEpochSec > 0L) {
                    ((System.currentTimeMillis() / 1000) - serviceStartedAtEpochSec).coerceAtLeast(0L).toULong()
                } else {
                    0uL
                }
                val normalizedStats = if (stats.uptimeSecs == 0uL && fallbackUptimeSecs > 0uL) {
                    uniffi.api.ServiceStats(
                        peersDiscovered = stats.peersDiscovered,
                        messagesRelayed = stats.messagesRelayed,
                        bytesTransferred = stats.bytesTransferred,
                        uptimeSecs = fallbackUptimeSecs
                    )
                } else {
                    stats
                }

                // We use a custom stats object or just update the one from core
                // For now, let's keep the core one but maybe log the detailed count
                Timber.d("Mesh Stats: ${normalizedStats.peersDiscovered} peers (Core), $fullCount full, $headlessCount headless (Repo)")

                _serviceStats.value = normalizedStats

                // Emit event for UI
                repoScope.launch {
                    com.scmessenger.android.service.MeshEventBus.emitStatusEvent(
                        com.scmessenger.android.service.StatusEvent.StatsUpdated(normalizedStats)
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

    /**
     * Canonical contact ID normalization function.
     * Maps all ID variants (SHA-256(publicKey), Hash(pubkey+identity), LibP2P Peer ID)
     * to a single canonical form using public key as the primary key.
     *
     * This ensures consistent contact identification across all ID schemes:
     * - Identity/Discovery: SHA-256(publicKey) → canonical
     * - Storage/Database: Hash(pubkey+identity) → canonical
     * - Transport: LibP2P Peer ID → canonical
     *
     * @param id Any ID format (public_key_hex, identity_id, or libp2p_peer_id)
     * @return Canonical identity_id (Blake3 hash of public key)
     */
    /**
     * ID-STANDARDIZATION-001: Comprehensive ID sanity check
     * Validates that an ID is consistent and can be properly resolved
     */
    private fun validateAndStandardizeId(id: String, operation: String): String {
        if (id.isBlank()) {
            throw IllegalArgumentException("ID cannot be blank for operation: $operation")
        }
        
        val trimmed = id.trim()
        val canonicalId = canonicalContactId(trimmed)
        
        // Additional validation: Check if this ID maps to multiple contacts (ambiguity check)
        try {
            val contacts = contactManager?.list().orEmpty()
            val matchingContacts = contacts.filter {
                val contactId = canonicalContactId(it.peerId)
                PeerIdValidator.isSame(contactId, canonicalId)
            }
            
            if (matchingContacts.size > 1) {
                Timber.w("ID_AMBIGUITY: Operation '$operation' on ID '$trimmed' matches ${matchingContacts.size} contacts:")
                matchingContacts.forEach { contact ->
                    Timber.w("  - Contact: ${contact.peerId} (key=${contact.publicKey?.take(8)})")
                }
                // In case of ambiguity, prefer the first one but log the issue
            }
        } catch (e: Exception) {
            Timber.w("ID_VALIDATION: Could not check contact ambiguity for '$trimmed': ${e.message}")
        }
        
        return canonicalId
    }

    private fun canonicalContactId(id: String): String {
        val trimmed = id.trim()
        if (trimmed.isEmpty()) return trimmed

        // AND-SEND-BTN-001: Check in-memory cache first to avoid synchronous FFI on UI thread.
        // Identity resolution is deterministic — once resolved, the mapping never changes.
        identityIdCache[trimmed]?.let { return it }

        // ID-STANDARDIZATION-001: Comprehensive ID resolution with sanity checks
        Timber.d("ID_RESOLUTION: Input ID '$trimmed' (length=${trimmed.length})")

        // UNIFIED ID FIX: Never short-circuit on identity_id shape.
        // resolveIdentity() handles all formats (public_key_hex, identity_id, libp2p_peer_id)
        // and always returns public_key_hex (canonical). The 64-hex short-circuit was
        // allowing identity_id to be stored as Contact.peerId, causing duplicate contacts.

        // Try to resolve to canonical public_key_hex via IronCore
        try {
            val resolvedPk = ironCore?.resolveIdentity(trimmed)
            if (resolvedPk != null) {
                val normalized = PeerIdValidator.normalize(resolvedPk)
                identityIdCache[trimmed] = normalized
                Timber.d("ID_RESOLUTION: Resolved '$trimmed' -> public_key_hex: ${normalized.take(8)}...")
                return normalized
            } else {
                Timber.w("ID_RESOLUTION: IronCore returned null for '$trimmed'")
            }
        } catch (e: Exception) {
            Timber.w("ID_RESOLUTION: Failed to resolve ID '$trimmed' to public_key_hex: ${e.message}")
        }

        // Fallback: Normalize the input ID (libp2p peer ID or other format)
        val normalizedFallback = PeerIdValidator.normalize(trimmed)
        identityIdCache[trimmed] = normalizedFallback
        Timber.d("ID_RESOLUTION: Fallback to normalized input: ${normalizedFallback.take(16)}...")
        return normalizedFallback
    }
    
    /**
     * Legacy canonicalId function - kept for backward compatibility.
     * @deprecated Use canonicalContactId() for new code.
     */
    private fun canonicalId(id: String): String {
        return canonicalContactId(id)
    }

    fun addContact(contact: uniffi.api.Contact) {
        // CRITICAL: Validate public key before storing
        val trimmedKey = contact.publicKey?.trim()
        if (trimmedKey.isNullOrEmpty()) {
            Timber.e("addContact rejected: public key is empty for peer ${contact.peerId}")
            return
        }
        if (trimmedKey.length != 64) {
            Timber.e("addContact rejected: public key has invalid length ${trimmedKey.length} (expected 64) for peer ${contact.peerId}")
            return
        }
        if (!trimmedKey.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' }) {
            Timber.e("addContact rejected: public key contains invalid characters for peer ${contact.peerId}")
            return
        }

        val canonical = canonicalId(contact.peerId)
        val finalContact = if (canonical != contact.peerId) {
            uniffi.api.Contact(
                peerId = canonical,
                nickname = contact.nickname,
                localNickname = contact.localNickname,
                publicKey = contact.publicKey,
                addedAt = contact.addedAt,
                lastSeen = contact.lastSeen,
                notes = contact.notes,
                lastKnownDeviceId = contact.lastKnownDeviceId
            )
        } else {
            contact
        }

        contactManager?.add(finalContact)
        val routing = parseRoutingHints(finalContact.notes)
        annotateIdentityInLedger(
            routePeerId = routing.libp2pPeerId,
            listeners = routing.listeners,
            publicKey = finalContact.publicKey,
            nickname = finalContact.nickname
        )
        Timber.d("Contact added: $canonical")
    }

    fun getContact(peerId: String): uniffi.api.Contact? {
        return contactManager?.get(canonicalId(peerId))
    }

    /**
     * WS14: Check if a conversation exists with the given peer.
     * Used for notification classification (DM vs DM Request).
     */
    fun hasConversationWith(peerId: String): Boolean {
        val canonical = canonicalId(peerId)
        return try {
            // Check if we have any message history with this peer
            historyManager?.conversation(canonical, 1u)?.isNotEmpty() == true
        } catch (e: Exception) {
            false
        }
    }

    fun removeContact(peerId: String) {
        val canonical = canonicalId(peerId)
        contactManager?.remove(canonical)
        try {
            historyManager?.removeConversation(canonical)
        } catch (e: Exception) {
            Timber.w("Failed to remove conversation history for $canonical: ${e.message}")
        }
        
        // Clear in-memory caches to prevent stale contact from showing (CONTACT-STALE-001)
        // 1. Remove from discovered peers cache
        _discoveredPeers.update { current ->
            val keysToRemove = current.filter { (key, info) ->
                key == canonical ||
                    info.peerId == canonical ||
                    PeerIdValidator.isSame(key, canonical) ||
                    PeerIdValidator.isSame(info.peerId, canonical)
            }.keys
            
            if (keysToRemove.isEmpty()) current else current - keysToRemove
        }
        
        // 2. Remove from BLE route observations cache
        val keysToRemove = bleRouteObservations.keys.filter { key ->
            key == canonical || PeerIdValidator.isSame(key, canonical)
        }
        keysToRemove.forEach { bleRouteObservations.remove(it) }
        
        Timber.d("Contact removed: $canonical and their message history, caches cleared")
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
    // BLOCKING
    // ========================================================================

    fun blockPeer(peerId: String, deviceId: String? = null, reason: String? = null) {
        ensureServiceInitialized()
        try {
            ironCore?.blockPeer(peerId, deviceId, reason)
            Timber.i("Blocked peer: $peerId (device: $deviceId, reason: $reason)")
        } catch (e: Exception) {
            Timber.e(e, "Failed to block peer: $peerId")
        }
    }

    fun unblockPeer(peerId: String, deviceId: String? = null) {
        ensureServiceInitialized()
        try {
            ironCore?.unblockPeer(peerId, deviceId)
            Timber.i("Unblocked peer: $peerId (device: $deviceId)")
        } catch (e: Exception) {
            Timber.e(e, "Failed to unblock peer: $peerId")
        }
    }

    /**
     * Block a peer AND delete all their stored messages (cascade purge).
     * Future payloads from this peer are dropped at the ingress layer.
     */
    fun blockAndDeletePeer(peerId: String, deviceId: String? = null, reason: String? = null) {
        ensureServiceInitialized()
        try {
            ironCore?.blockAndDeletePeer(peerId, deviceId, reason)
            Timber.i("Blocked and deleted peer: $peerId (device: $deviceId, reason: $reason)")
        } catch (e: Exception) {
            Timber.e(e, "Failed to block and delete peer: $peerId")
        }
    }

    fun isBlocked(peerId: String, deviceId: String? = null): Boolean {
        ensureServiceInitialized()
        return try {
            ironCore?.isPeerBlocked(peerId, deviceId) ?: false
        } catch (e: Exception) {
            Timber.w(e, "Failed to check if peer blocked: $peerId")
            false
        }
    }

    fun listBlockedPeers(): List<uniffi.api.BlockedIdentity> {
        ensureServiceInitialized()
        return try {
            ironCore?.listBlockedPeers() ?: emptyList()
        } catch (e: Exception) {
            Timber.w(e, "Failed to list blocked peers")
            emptyList()
        }
    }

    fun getBlockedCount(): UInt {
        ensureServiceInitialized()
        return try {
            ironCore?.blockedCount() ?: 0u
        } catch (e: Exception) {
            Timber.w(e, "Failed to get blocked count")
            0u
        }
    }

    // ========================================================================
    // CRYPTO UTILITIES
    // ========================================================================

    fun signData(data: ByteArray): uniffi.api.SignatureResult? {
        ensureServiceInitialized()
        return try {
            ironCore?.signData(data)
        } catch (e: Exception) {
            Timber.e(e, "Failed to sign data")
            null
        }
    }

    fun verifySignature(data: ByteArray, signature: ByteArray, publicKeyHex: String): Boolean {
        ensureServiceInitialized()
        return try {
            ironCore?.verifySignature(data, signature, publicKeyHex) ?: false
        } catch (e: Exception) {
            Timber.e(e, "Failed to verify signature")
            false
        }
    }

    // ========================================================================
    // WS13 DEVICE MANAGEMENT
    // ========================================================================

    fun getDeviceId(): String? {
        return ironCore?.getDeviceId()
    }

    fun getSeniorityTimestamp(): ULong? {
        return ironCore?.getSeniorityTimestamp()
    }

    fun getRegistrationState(identityId: String): uniffi.api.RegistrationStateInfo? {
        return ironCore?.getRegistrationState(identityId)
    }

    // ========================================================================
    // LOGGING
    // ========================================================================

    fun exportLogs(): String? {
        return try {
            ironCore?.exportLogs()
        } catch (e: Exception) {
            Timber.w(e, "Failed to export logs")
            null
        }
    }

    // ========================================================================
    // QUEUE COUNTS
    // ========================================================================

    fun getInboxCount(): UInt {
        return ironCore?.inboxCount() ?: 0u
    }

    // ========================================================================
    // CONTACT DEVICE ID (WS13)
    // ========================================================================

    fun updateContactDeviceId(peerId: String, deviceId: String?) {
        try {
            contactManager?.updateDeviceId(peerId, deviceId)
            Timber.i("Updated device ID for $peerId: $deviceId")
        } catch (e: Exception) {
            Timber.w(e, "Failed to update device ID for $peerId")
        }
    }

    // ========================================================================
    // MESSAGE HISTORY
    // ========================================================================

    /**
     * Get identity info without blocking.
     * Returns null if the service is not yet initialized; call again after a delay if needed.
     * This is non-blocking and safe to call from the main thread during UI composition.
     */
    fun getIdentityInfoNonBlocking(): uniffi.api.IdentityInfo? {
        val state = meshService?.getState()
        if (state != uniffi.api.ServiceState.RUNNING) {
            // Service not running, return null - identity will be populated later
            return null
        }
        // Service is running, now safe to call getIdentityInfo
        return getIdentityInfo()
    }

    fun getIdentityInfo(): uniffi.api.IdentityInfo? {
        ensureServiceInitialized()
        kotlin.runCatching { ensureLocalIdentityFederation() }
            .onFailure { Timber.w(it, "Failed to hydrate identity before getIdentityInfo") }
        val result = ironCore?.getIdentityInfo()
        Timber.d(
            "getIdentityInfo: result=%s, initialized=%s, nickname=%s",
            result?.identityId,
            result?.initialized,
            result?.nickname
        )
        return result
    }

    fun setNickname(nickname: String) {
        val trimmed = nickname.trim()
        if (trimmed.isEmpty()) {
            Timber.w("Refusing to set blank nickname")
            return
        }
        Timber.d("setNickname: Requested nickname='%s', trimmed='%s'", nickname, trimmed)
        val core = ironCore
            ?: throw IllegalStateException("Cannot set nickname: IronCore is not initialized")
        try {
            Timber.d("setNickname: Calling core.setNickname()")
            core.setNickname(trimmed)
            Timber.i("Nickname set to: $trimmed")
            // Verify the nickname was persisted
            val info = ironCore?.getIdentityInfo()
            Timber.d(
                "setNickname: Verification - nickname=%s, initialized=%s",
                info?.nickname,
                info?.initialized
            )
        } catch (e: Exception) {
            Timber.e(e, "Rust core failed to set nickname")
            throw IllegalStateException("Failed to persist nickname: ${e.message}", e)
        }
        persistIdentityBackup(core)
        // If swarm start was postponed before identity/nickname was ready, resume now.
        initializeAndStartSwarm()
        updateBleIdentityBeacon()
        identitySyncSentPeers.clear()
        historySyncSentPeers.clear()
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
            val routingPeerId = PeerIdValidator.normalize(peerId)
            val initialMessageId = java.util.UUID.randomUUID().toString()
            val now = (System.currentTimeMillis() / 1000).toULong()

            // 1. Resolve public key FIRST to determine canonical ID
            var publicKey: String? = try {
                ironCore?.resolveIdentity(routingPeerId)
            } catch (e: Exception) {
                null
            }

            // Fallback: Try contact manager
            val contact = contactManager?.get(routingPeerId)
            if (publicKey == null && contact != null && !contact.publicKey.isNullOrEmpty()) {
                publicKey = contact.publicKey.trim()
                
                // CRITICAL: Validate public key length
                if (publicKey.length != 64) {
                    Timber.e("SEND_MSG_BUG: Contact has invalid public key length: ${publicKey.length} (expected 64)")
                    Timber.e("SEND_MSG_BUG: Contact peer_id: ${contact.peerId}")
                    Timber.e("SEND_MSG_BUG: Routing peer_id: $routingPeerId")
                    Timber.e("SEND_MSG_BUG: Public key value: '$publicKey'")
                    
                    // Try to recover from discovered peers
                    val discoveredPeer = _discoveredPeers.value.entries.find {
                        PeerIdValidator.isSame(it.key, routingPeerId) ||
                        PeerIdValidator.isSame(it.key, contact.peerId) ||
                        (it.value.publicKey == contact.publicKey)
                    }?.value
                    
                    if (discoveredPeer != null && !discoveredPeer.publicKey.isNullOrEmpty() && 
                        discoveredPeer.publicKey.trim().length == 64) {
                        publicKey = discoveredPeer.publicKey.trim()
                        Timber.w("SEND_MSG_RECOVER: Using public key from discovered peers: ${publicKey?.take(8)}")
                    } else {
                        throw IllegalStateException("Contact public key is invalid (${publicKey.length} chars) and no discovered peer available for recovery")
                    }
                } else {
                    Timber.d("SEND_MSG: Resolved from contact: key=${publicKey?.take(8)}")
                }
            }

            // Fallback: Try discovered peers
            if (publicKey == null) {
                val discoveredPeer = _discoveredPeers.value.entries.find {
                    PeerIdValidator.isSame(it.key, routingPeerId)
                }?.value
                if (discoveredPeer != null && !discoveredPeer.publicKey.isNullOrEmpty()) {
                    publicKey = discoveredPeer.publicKey.trim()
                    Timber.d("SEND_MSG: Resolved from discovered peers: key=${publicKey?.take(8)}")
                }
            }

            // Fallback: Try extracting public key directly from libp2p peer ID
            if (publicKey == null && PeerIdValidator.isLibp2pPeerId(routingPeerId)) {
                val transportIdentity = resolveTransportIdentity(routingPeerId)
                if (transportIdentity != null) {
                    publicKey = transportIdentity.publicKey
                    Timber.d("SEND_MSG: Resolved via transport identity: key=${publicKey?.take(8)}, canonical=${transportIdentity.canonicalPeerId}")
                }

                if (publicKey == null) {
                    try {
                        val extractedKey = ironCore?.extractPublicKeyFromPeerId(routingPeerId)
                        if (!extractedKey.isNullOrEmpty()) {
                            publicKey = normalizePublicKey(extractedKey)
                            Timber.d("SEND_MSG: Extracted public key from peer ID: ${publicKey?.take(8)}")
                        }
                    } catch (e: Exception) {
                        Timber.d("SEND_MSG: Failed to extract public key from peer ID: ${e.message}")
                    }
                }
            }

            // Use public key as canonical peer ID for history storage
            val normalizedPeerId = if (publicKey != null) {
                normalizePublicKey(publicKey) ?: routingPeerId
            } else {
                routingPeerId
            }

            // 1. Save to history IMMEDIATELY.
            val initialRecord = uniffi.api.MessageRecord(
                id = initialMessageId,
                peerId = normalizedPeerId,
                direction = uniffi.api.MessageDirection.SENT,
                content = content,
                timestamp = now,
                senderTimestamp = now,
                delivered = false,
                hidden = false
            )

            try {
                historyManager?.add(initialRecord)
                historyManager?.flush()

                // Emit for UI update
                repoScope.launch {
                    _messageUpdates.emit(initialRecord)
                }

                Timber.d("SEND_MSG_START: original='$peerId', normalized='$normalizedPeerId', routing='$routingPeerId', publicKey='${publicKey?.take(8)}', initialId=$initialMessageId")

                // Check if relay/messaging is enabled (bidirectional control)
                val currentSettings = settingsManager?.load()
                Companion.requireMeshParticipationEnabled(currentSettings)

            if (publicKey == null) {
                    // 3. Queue with placeholder encrypted data (will re-encrypt when peer discovered)
                    Timber.w("SEND_MSG_QUEUE: Peer not found - will retry when discovered: $normalizedPeerId")

                    logDeliveryState(
                        messageId = initialMessageId,
                        state = "queued",
                        detail = "peer_not_discovered_yet awaiting_public_key"
                    )

                    enqueuePendingOutbound(
                        historyRecordId = initialMessageId,
                        peerId = normalizedPeerId,
                        routePeerId = null,
                        listeners = emptyList(),
                        encryptedData = content.toByteArray(), // Plaintext for now, encrypt on flush
                        initialAttemptCount = 0,
                        initialDelaySec = 5, // Retry in 5 seconds
                        strictBleOnlyMode = false
                    )

                    Timber.i("Message queued for $normalizedPeerId - will send when peer discovered")
                    return@withContext
                }

                val finalPublicKey = publicKey!!
                // Pre-validate public key
                if (finalPublicKey.length != 64 || !finalPublicKey.all { it.isDigit() || it.lowercaseChar() in 'a'..'f' }) {
                    throw IllegalStateException("Invalid public key format for $normalizedPeerId: $finalPublicKey")
                }

                Timber.d("Preparing message for $normalizedPeerId with key: ${finalPublicKey.take(8)}...")

                val routePeerCandidates = buildRoutePeerCandidates(
                    peerId = routingPeerId,
                    cachedRoutePeerId = null,
                    notes = contact?.notes ?: "",
                    recipientPublicKey = finalPublicKey
                )

                if (isKnownRelay(normalizedPeerId) || isBootstrapRelayPeer(normalizedPeerId)) {
                    throw IllegalStateException("Refusing to use headless relay identity as a chat recipient: $normalizedPeerId")
                }

                val preferredRoutePeerId = routePeerCandidates.firstOrNull()

                // 4. Encrypt/Prepare message
                val outboundContent = encodeMessageWithIdentityHints(content)
                val prepared = ironCore?.prepareMessageWithId(finalPublicKey, outboundContent, null)
                    ?: throw IllegalStateException("Failed to prepare message: IronCore not initialized")

                val realMessageId = prepared.messageId.trim()
                if (realMessageId.isBlank()) {
                    throw IllegalStateException("Failed to prepare message: core returned empty message ID")
                }
                val encryptedData = prepared.envelopeData

                // 5. Reconcile message IDs: update the initial history record to use
                // Core's wire message ID. Delivery receipts and markMessageSent are
                // keyed by realMessageId, so our local record must match.
                if (realMessageId != initialMessageId) {
                    try {
                        historyManager?.delete(initialMessageId)
                        val reconciledRecord = uniffi.api.MessageRecord(
                            id = realMessageId,
                            peerId = normalizedPeerId,
                            direction = uniffi.api.MessageDirection.SENT,
                            content = content,
                            timestamp = now,
                            senderTimestamp = now,
                            delivered = false,
                            hidden = false
                        )
                        historyManager?.add(reconciledRecord)
                        historyManager?.flush()
                        repoScope.launch { _messageUpdates.emit(reconciledRecord) }
                        Timber.d("SEND_MSG: Reconciled message ID: $initialMessageId -> $realMessageId")
                        
                        // AND-MSG-DISAPPEAR-001: Verify message was actually stored
                        val storedMessage = try {
                            historyManager?.get(realMessageId)
                        } catch (e: Exception) {
                            null
                        }
                        if (storedMessage == null) {
                            Timber.e("CRITICAL: Message $realMessageId was not found in history after storage!")
                        } else {
                            Timber.d("SEND_MSG: Verified message stored in history: ${realMessageId.take(8)}...")
                        }
                    } catch (e: Exception) {
                        Timber.w(e, "Failed to reconcile message ID; delivery tracking may be unreliable")
                    }
                }

                logDeliveryState(
                    messageId = realMessageId,
                    state = "pending",
                    detail = "message_prepared_local_history_written"
                )

                // 6. Send over core-selected swarm route only.
                val delivery = attemptDirectSwarmDelivery(
                    routePeerCandidates = routePeerCandidates,
                    listeners = emptyList(),
                    encryptedData = encryptedData,
                    wifiPeerId = preferredRoutePeerId,
                    blePeerId = null,
                    traceMessageId = realMessageId,
                    attemptContext = "initial_send",
                    recipientIdentityId = finalPublicKey,
                    intendedDeviceId = contact?.lastKnownDeviceId
                )

                val selectedRoutePeerId = delivery.routePeerId ?: preferredRoutePeerId

                if (delivery.acked) {
                    promotePendingOutboundForPeer(peerId = normalizedPeerId, excludingMessageId = realMessageId)
                }

                val receiptAwaitSeconds = 30L
                val strictBleOnlyValidation = false

                if (isMessageDeliveredLocally(realMessageId)) {
                    removePendingOutbound(realMessageId)
                    logDeliveryState(
                        messageId = realMessageId,
                        state = "delivered",
                        detail = "delivery_receipt_arrived_before_enqueue"
                    )
                } else if (delivery.terminalFailureCode != null) {
                    enqueuePendingOutbound(
                        historyRecordId = realMessageId,
                        peerId = normalizedPeerId,
                        routePeerId = selectedRoutePeerId,
                        listeners = emptyList(),
                        encryptedData = encryptedData,
                        initialAttemptCount = 1,
                        initialDelaySec = 0,
                        strictBleOnlyMode = strictBleOnlyValidation,
                        recipientIdentityId = finalPublicKey,
                        intendedDeviceId = contact?.lastKnownDeviceId,
                        terminalFailureCode = delivery.terminalFailureCode
                    )
                    logDeliveryState(
                        messageId = realMessageId,
                        state = "rejected",
                        detail = "terminal_failure_code=${delivery.terminalFailureCode}"
                    )
                    throw IllegalStateException(
                        terminalIdentityFailureMessage(delivery.terminalFailureCode)
                    )
                } else {
                    enqueuePendingOutbound(
                        historyRecordId = realMessageId,
                        peerId = normalizedPeerId,
                        routePeerId = selectedRoutePeerId,
                        listeners = emptyList(),
                        encryptedData = encryptedData,
                        initialAttemptCount = 1,
                        initialDelaySec = if (delivery.acked) receiptAwaitSeconds else 0,
                        strictBleOnlyMode = strictBleOnlyValidation,
                        recipientIdentityId = finalPublicKey,
                        intendedDeviceId = contact?.lastKnownDeviceId
                    )
                }

                Timber.i("Message sent (encrypted) to $normalizedPeerId (id=$realMessageId)")
            } catch (e: Exception) {
                Timber.e(e, "Failed to send message to $normalizedPeerId")
                // Even on error, the message is in history (initialRecord),
                // but we should probably ensure it's also in the pending outbox for retry if it's a transient error.
                // However, most exceptions here (invalid key, etc.) are non-transient.
                // Transient transport errors are handled within attemptDirectSwarmDelivery.
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
        // Check if we have an identity backup first (fast path)
        if (identityBackupPrefs.contains(IDENTITY_BACKUP_KEY)) {
            // P0_ANDROID_010: Verify Rust core also has the identity.
            // The backup may exist in SharedPreferences but the Rust core may have
            // lost its sled database (e.g., after an unclean shutdown). If the service
            // is running and the core reports uninitialized, trigger a restore.
            if (meshService?.getState() == uniffi.api.ServiceState.RUNNING && ironCore != null) {
                val coreInitialized = ironCore?.getIdentityInfo()?.initialized == true
                if (!coreInitialized) {
                    Timber.w("Backup exists but Rust core identity is lost — attempting restore")
                    try {
                        val restored = restoreIdentityFromBackup(ironCore!!)
                        if (restored) {
                            ironCore?.grantConsent()
                            Timber.i("Identity restored from backup during isIdentityInitialized check")
                        }
                    } catch (e: Exception) {
                        Timber.w(e, "Failed to restore identity during isIdentityInitialized check")
                    }
                }
            }
            return true
        }

        // Only start service if we already have it running or explicitly need it
        if (meshService?.getState() == uniffi.api.ServiceState.RUNNING && ironCore != null) {
            return ironCore?.getIdentityInfo()?.initialized == true
        }

        // If service is not running, check if identity files exist on disk
        val identityFile = File(storagePath, "identity.db")
        return identityFile.exists()
    }

    /**
     * Grant explicit user consent for cryptographic identity generation.
     * Must be called BEFORE initializeIdentity() or createIdentity().
     * Persists consent in the Rust core (sled-backed).
     */
    fun grantConsent() {
        try {
            ensureServiceInitialized()
            ironCore?.grantConsent()
            Timber.i("Consent granted in Rust core")
        } catch (e: Exception) {
            Timber.e(e, "Failed to grant consent")
        }
    }

    fun hasRequiredRuntimePermissions(): Boolean = hasAllPermissions(Permissions.required)

    @Synchronized
    fun onRuntimePermissionsGranted() {
        if (meshService?.getState() != uniffi.api.ServiceState.RUNNING) {
            Timber.d("Permission refresh skipped: mesh service is not running")
            return
        }
        repoScope.launch {
            try {
                initializeAndStartBle()
            } catch (e: Exception) {
                Timber.w(e, "BLE transport failed to start after permission grant")
            }
        }
        repoScope.launch {
            try {
                initializeAndStartWifi()
            } catch (e: Exception) {
                Timber.w(e, "WiFi transport failed to start after permission grant")
            }
        }
        repoScope.launch {
            try {
                initializeAndStartSwarm()
            } catch (e: Exception) {
                Timber.w(e, "Swarm transport failed to refresh after permission grant")
            }
        }
    }

    suspend fun createIdentity() {
        kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
            try {
                ensureServiceInitializedBlocking()
                if (ironCore == null) {
                    Timber.e("IronCore is null after ensureServiceInitialized! Cannot create identity.")
                    throw IllegalStateException("Mesh service initialization failed")
                }
                // P0_ANDROID_010: Grant consent before identity initialization.
                // The Rust core requires consent_granted=true for initialize_identity().
                // OnboardingScreen's consent checkbox gates the UI; this propagates consent to the core.
                ironCore?.grantConsent()
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
    @Volatile
    private var isServiceStarting = false

    /**
     * ANR FIX (P0_ANDROID_017): Defer service initialization to background thread without blocking.
     * The original implementation called loadSettings() synchronously inside the coroutine,
     * which blocked because settingsManager?.load() waits for the Rust core to initialize.
     * This caused a deadlock: UI thread wanted identity info → called ensureServiceInitializedDeferred()
     * → startMeshService() → loadSettings() → Rust core not ready → block.
     *
     * Solution: Use default settings (already cached) for initial config, then reload settings
     * asynchronously after service starts. This allows Settings screen to load immediately.
     */
    private fun ensureServiceInitializedDeferred() {
        repoScope.launch {
            val state = meshService?.getState()
            if (state == uniffi.api.ServiceState.RUNNING) {
                return@launch
            }

            // Skip if already starting (prevents pile-up on @Synchronized startMeshService)
            if (state == uniffi.api.ServiceState.STARTING || isServiceStarting) {
                Timber.d("MeshService is already starting, skipping redundant init")
                return@launch
            }

            Timber.d("Lazy starting MeshService (async settings reload)...")
            isServiceStarting = true
            try {
                // Use default settings for initial config - no blocking I/O
                val defaultSettings = uniffi.api.MeshSettings(
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
                    timingObfuscationEnabled = false,
                    notificationsEnabled = true,
                    notifyDmEnabled = true,
                    notifyDmRequestEnabled = true,
                    notifyDmInForeground = false,
                    notifyDmRequestInForeground = true,
                    soundEnabled = true,
                    badgeEnabled = true
                )
                val config = uniffi.api.MeshServiceConfig(
                    discoveryIntervalMs = 30000u,
                    batteryFloorPct = defaultSettings.batteryFloor
                )
                startMeshService(config)
                Timber.d("MeshService started lazily (async settings reload)")

                // Async reload of settings after service started - doesn't block
                repoScope.launch {
                    try {
                        val loaded = loadSettings()
                        Timber.d("Settings reloaded asynchronously after service startup")
                    } catch (e: Exception) {
                        Timber.w(e, "Async settings reload failed")
                    }
                }
            } catch (e: Exception) {
                Timber.e(e, "Failed to start MeshService lazily")
            } finally {
                isServiceStarting = false
            }

            // Refresh ironCore reference just in case
            if (ironCore == null) {
                ironCore = meshService?.getCore()
                Timber.d("IronCore reference refreshed: ${ironCore != null}")
            }
            ensureLocalIdentityFederation()
        }
    }

    private fun ensureServiceInitialized() {
        // Fire-and-forget for non-critical paths (prevents UI thread blocking)
        ensureServiceInitializedDeferred()
    }

    /**
     * Blocking variant for suspend functions that require the service to be running.
     * Waits up to 10 seconds for MeshService to reach RUNNING state.
     */
    private suspend fun ensureServiceInitializedBlocking() {
        // Fast path: already running
        if (meshService?.getState() == uniffi.api.ServiceState.RUNNING && ironCore != null) {
            return
        }

        // Start initialization if needed
        ensureServiceInitializedDeferred()

        // Wait for service to actually start (with timeout)
        val startTime = System.currentTimeMillis()
        val timeoutMs = 10_000L
        while (System.currentTimeMillis() - startTime < timeoutMs) {
            if (meshService?.getState() == uniffi.api.ServiceState.RUNNING && ironCore != null) {
                return
            }
            kotlinx.coroutines.delay(100)
        }
        Timber.w("ensureServiceInitializedBlocking timed out after ${timeoutMs}ms")
    }

    private fun hasAllPermissions(permissions: List<String>): Boolean =
        permissions.all { permission ->
            ContextCompat.checkSelfPermission(context, permission) == PackageManager.PERMISSION_GRANTED
        }

    // Keep legacy addMessage for receiving side or manual adds
    fun addMessage(record: uniffi.api.MessageRecord) {
        historyManager?.add(record)
        historyManager?.flush()
    }

    fun getMessage(id: String): uniffi.api.MessageRecord? {
        return historyManager?.get(id)
    }

    fun getRecentMessages(peerFilter: String? = null, limit: UInt = 50u): List<uniffi.api.MessageRecord> {
        val filter = peerFilter?.let { canonicalId(it) }
        return historyManager?.recent(filter, limit) ?: emptyList()
    }

    fun getConversation(peerId: String, limit: UInt = 100u): List<uniffi.api.MessageRecord> {
        return historyManager?.conversation(canonicalId(peerId), limit) ?: emptyList()
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
        // ID-STANDARDIZATION-001: Use comprehensive ID validation for deletion operations
        try {
            val validatedId = validateAndStandardizeId(peerId, "clearConversation")
            historyManager?.clearConversation(validatedId)
            Timber.i("Conversation cleared: $validatedId")
        } catch (e: Exception) {
            Timber.e(e, "Failed to clear conversation due to ID validation error")
            throw e
        }
    }

    fun getHistoryStats(): uniffi.api.HistoryStats? {
        return historyManager?.stats()
    }

    fun getMessageCount(): UInt {
        return historyManager?.count() ?: 0u
    }

    // ── History Retention ────────────────────────────────────────────────

    /**
     * Enforce message retention by keeping only the newest [maxMessages] messages.
     * Returns the number of messages pruned.
     */
    fun enforceRetention(maxMessages: UInt): UInt {
        return try {
            historyManager?.enforceRetention(maxMessages = maxMessages) ?: 0u
        } catch (e: Exception) {
            Timber.e(e, "Failed to enforce retention")
            0u
        }
    }

    /**
     * Prune messages older than the given Unix timestamp (seconds).
     * Returns the number of messages pruned.
     */
    fun pruneBefore(beforeTimestamp: ULong): UInt {
        return try {
            historyManager?.pruneBefore(beforeTimestamp = beforeTimestamp) ?: 0u
        } catch (e: Exception) {
            Timber.e(e, "Failed to prune history")
            0u
        }
    }

    /**
     * Resets all application data, including identity, contacts, history, and preferences.
     * WARNING: This is destructive and permanent.
     */
    fun resetAllData() {
        Timber.w("RESETTING ALL APPLICATION DATA")

        pendingReceiptSendJobs.values.forEach { it.cancel() }
        pendingReceiptSendJobs.clear()

        // 1. Stop all active services and release UniFFI objects
        swarmBridge?.shutdown()
        swarmBridge = null

        meshService?.stop()
        meshService = null

        ironCore?.stop()
        try {
            contactManager?.flush()
            historyManager?.flush()
        } catch (e: Exception) {
            Timber.w("Failed to flush managers during shutdown: ${e.message}")
        }
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
                    PeerIdValidator.isLibp2pPeerId(mapKey) -> mapKey
                    PeerIdValidator.isLibp2pPeerId(canonicalPeerId) -> canonicalPeerId
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
                        transport = com.scmessenger.android.service.TransportType.INTERNET
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

    fun getServiceStateName(): String {
        return meshService?.getState()?.name ?: "STOPPED"
    }

    fun getDiscoveredPeerCount(): Int {
        return _discoveredPeers.value.size
    }

    fun getPendingOutboxCount(): Int {
        return loadPendingOutbox().size
    }

    fun getPendingDeliverySnapshot(messageId: String): Pair<Int, Long>? {
        if (messageId.isBlank()) return null
        val pending = loadPendingOutbox().firstOrNull { it.historyRecordId == messageId } ?: return null
        return pending.attemptCount to pending.nextAttemptAtEpochSec
    }

    fun getPendingTerminalFailureCode(messageId: String): String? {
        if (messageId.isBlank()) return null
        return loadPendingOutbox()
            .firstOrNull { it.historyRecordId == messageId }
            ?.terminalFailureCode
    }

    fun getMissingRuntimePermissions(): List<String> {
        return Permissions.required.filter { permission ->
            ContextCompat.checkSelfPermission(context, permission) != PackageManager.PERMISSION_GRANTED
        }
    }

    /**
     * ANR FIX (P0_ANDROID_017): Export diagnostics asynchronously to avoid Main thread I/O.
     * The original implementation blocked on:
     * 1. meshService?.exportDiagnostics() - Rust FFI call that can hang
     * 2. loadPendingOutbox() - File I/O operation
     *
     * Solution: Run diagnostics export on IO dispatcher and cache results.
     */
    suspend fun exportDiagnosticsAsync(): String = kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
        try {
            exportDiagnosticsInternal()
        } catch (e: Exception) {
            Timber.w(e, "Failed to export diagnostics")
            // Return minimal fallback JSON to avoid crash
            org.json.JSONObject()
                .put("error", "Diagnostics export failed: ${e.message}")
                .put("service_state", meshService?.getState()?.name ?: "UNKNOWN")
                .put("generated_at_ms", System.currentTimeMillis())
                .toString()
        }
    }

    /**
     * Original diagnostics export - now only called from IO dispatcher.
     */
    fun exportDiagnostics(): String = exportDiagnosticsInternal()

    private fun exportDiagnosticsInternal(): String {
        val coreDiagnostics = try {
            meshService?.exportDiagnostics()
        } catch (e: Exception) {
            Timber.w(e, "Failed to export core diagnostics")
            null
        }

        val bleDiscovery = bleScanner?.getDiscoveryStats()
        val bleClient = bleGattClient?.getClientStats()

        if (!coreDiagnostics.isNullOrBlank()) {
            try {
                val obj = org.json.JSONObject(coreDiagnostics)
                bleDiscovery?.let { stats ->
                    obj.put("ble_advertisements_seen", stats.advertisementsSeen)
                    obj.put("ble_peers_discovered", stats.peersDiscovered)
                    obj.put("ble_scan_failures", stats.scanFailures)
                    obj.put("ble_peer_cache_size", stats.peerCacheSize)
                }
                bleClient?.let { stats ->
                    obj.put("ble_gatt_connect_attempts", stats.connectAttempts)
                    obj.put("ble_gatt_connect_initiated", stats.connectInitiated)
                    obj.put("ble_gatt_connect_failures", stats.connectFailures)
                    obj.put("ble_gatt_connect_state_successes", stats.connectStateSuccesses)
                    obj.put("ble_gatt_disconnects", stats.disconnects)
                    obj.put("ble_duplicate_permit_releases_ignored", stats.duplicatePermitReleasesIgnored)
                    obj.put("ble_semaphore_release_overflows", stats.semaphoreReleaseOverflows)
                    obj.put("ble_no_response_callbacks_ignored", stats.noResponseCallbacksIgnored)
                    obj.put("ble_address_type_mismatch_signals", stats.addressTypeMismatchSignals)
                    obj.put("ble_active_connections", stats.activeConnections)
                }
                obj.put("strict_ble_only_validation", strictBleOnlyValidation)
                return obj.toString()
            } catch (_: Exception) {
                return coreDiagnostics
            }
        }

        // ANR FIX: Use async pending outbox loader
        val pendingOutboxSize = try {
            loadPendingOutbox().size
        } catch (e: Exception) {
            Timber.w(e, "Failed to load pending outbox for diagnostics")
            0
        }

        val fallback = org.json.JSONObject()
            .put("service_state", meshService?.getState()?.name ?: "STOPPED")
            .put("connection_path_state", getConnectionPathState().name)
            .put("nat_status", getNatStatus())
            .put("discovered_peers", _discoveredPeers.value.size)
            .put("pending_outbox", pendingOutboxSize)
            .put("strict_ble_only_validation", strictBleOnlyValidation)
            .put("generated_at_ms", System.currentTimeMillis())
        bleDiscovery?.let { stats ->
            fallback.put("ble_advertisements_seen", stats.advertisementsSeen)
            fallback.put("ble_peers_discovered", stats.peersDiscovered)
            fallback.put("ble_scan_failures", stats.scanFailures)
            fallback.put("ble_peer_cache_size", stats.peerCacheSize)
        }
        bleClient?.let { stats ->
            fallback.put("ble_gatt_connect_attempts", stats.connectAttempts)
            fallback.put("ble_gatt_connect_initiated", stats.connectInitiated)
            fallback.put("ble_gatt_connect_failures", stats.connectFailures)
            fallback.put("ble_gatt_connect_state_successes", stats.connectStateSuccesses)
            fallback.put("ble_gatt_disconnects", stats.disconnects)
            fallback.put("ble_duplicate_permit_releases_ignored", stats.duplicatePermitReleasesIgnored)
            fallback.put("ble_semaphore_release_overflows", stats.semaphoreReleaseOverflows)
            fallback.put("ble_no_response_callbacks_ignored", stats.noResponseCallbacksIgnored)
            fallback.put("ble_address_type_mismatch_signals", stats.addressTypeMismatchSignals)
            fallback.put("ble_active_connections", stats.activeConnections)
        }
        return fallback.toString()
    }

    fun saveLedger() {
        ledgerManager?.save()
    }

    private val identityEmissionCache = java.util.concurrent.ConcurrentHashMap<String, Pair<IdentityEmissionSignature, Long>>()
    private val identityReemitIntervalMs = 15_000L
    private val connectedEmissionCache = java.util.concurrent.ConcurrentHashMap<String, Long>()
    private val connectedReemitIntervalMs = 15_000L
    private val disconnectEmissionCache = java.util.concurrent.ConcurrentHashMap<String, Long>()

    private suspend fun emitIdentityDiscoveredIfChanged(
        peerId: String,
        publicKey: String,
        nickname: String?,
        libp2pPeerId: String?,
        listeners: List<String>,
        blePeerId: String? = null
    ) {
        val canonicalPeerId = PeerIdValidator.normalize(peerId)
        val normalizedKey = normalizePublicKey(publicKey)
        if (canonicalPeerId.isEmpty() || normalizedKey.isNullOrBlank()) {
            return
        }

        val normalizedRoute = libp2pPeerId?.let { PeerIdValidator.normalize(it) }?.takeIf { it.isNotEmpty() }
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
        val normalizedPeerId = PeerIdValidator.normalize(peerId)
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
    private suspend fun emitDisconnectedIfChanged(
        peerId: String
    ) {
        val normalizedPeerId = PeerIdValidator.normalize(peerId)
        if (normalizedPeerId.isEmpty()) return

        val now = System.currentTimeMillis()
        val lastEmitted = disconnectEmissionCache[normalizedPeerId]
        if (lastEmitted != null && (now - lastEmitted) < peerDisconnectDedupIntervalMs) {
            return
        }
        disconnectEmissionCache[normalizedPeerId] = now
        com.scmessenger.android.service.MeshEventBus.emitPeerEvent(
            com.scmessenger.android.service.PeerEvent.Disconnected(
                normalizedPeerId
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
                timingObfuscationEnabled = false,
                notificationsEnabled = true,
                notifyDmEnabled = true,
                notifyDmRequestEnabled = true,
                notifyDmInForeground = false,
                notifyDmRequestInForeground = true,
                soundEnabled = true,
                badgeEnabled = true
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
                scanIntervalMs = 30000u,
                advertiseIntervalMs = 30000u,
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
                val finalAddr = if (PeerIdValidator.isLibp2pPeerId(peerId) && !addr.contains("/p2p/")) {
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
                    // Proactively ensure we stay connected to relays
                    primeRelayBootstrapConnections()

                    flushPendingOutbox("periodic")
                    kotlinx.coroutines.delay(8000)
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
        wifiPeerId: String? = null,
        blePeerId: String? = null,
        traceMessageId: String? = null,
        attemptContext: String = "send",
        strictBleOnlyOverride: Boolean? = null,
        recipientIdentityId: String? = null,
        intendedDeviceId: String? = null
    ): DeliveryAttemptResult {
        val strictBleOnly = strictBleOnlyOverride ?: strictBleOnlyValidation
        val routePeerFallback = routePeerCandidates.firstOrNull() ?: "unknown_route_${System.currentTimeMillis()}"
        if (strictBleOnly) {
            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "ble-only",
                phase = "mode",
                outcome = "enabled",
                detail = "ctx=$attemptContext route_candidates=${routePeerCandidates.size}"
            )
            if (!wifiPeerId.isNullOrBlank()) {
                logDeliveryAttempt(
                    messageId = traceMessageId,
                    medium = "wifi-direct",
                    phase = "ble_only",
                    outcome = "blocked",
                    detail = "ctx=$attemptContext reason=strict_ble_only_mode"
                )
            }
            if (routePeerCandidates.isNotEmpty() || listeners.isNotEmpty()) {
                logDeliveryAttempt(
                    messageId = traceMessageId,
                    medium = "core",
                    phase = "ble_only",
                    outcome = "blocked",
                    detail = "ctx=$attemptContext reason=strict_ble_only_mode"
                )
            }
        }

        val connectedBleDevices = kotlin.runCatching { bleGattServer?.getConnectedDeviceAddresses().orEmpty() }
            .getOrDefault(emptyList())
            .mapNotNull { it.trim().takeIf { value -> value.isNotEmpty() } }
            .distinct()
        val requestedBlePeerId = blePeerId?.trim()?.takeIf { it.isNotEmpty() }
        val freshBleObservation = resolveFreshBlePeerId(
            buildList {
                addAll(routePeerCandidates)
                requestedBlePeerId?.let(::add)
                recipientIdentityId?.trim()?.takeIf { it.isNotEmpty() }?.let(::add)
                intendedDeviceId?.trim()?.takeIf { it.isNotEmpty() }?.let(::add)
            }
        )
        val effectiveBlePeerId = connectedBleDevices.firstOrNull()
            ?: freshBleObservation?.address
            ?: requestedBlePeerId?.takeIf {
                resolveFreshBlePeerId(listOf(it))?.address == it
            }

        if (requestedBlePeerId == null && effectiveBlePeerId != null) {
            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "ble",
                phase = "local_fallback",
                outcome = "target_fallback",
                detail = "ctx=$attemptContext target=$effectiveBlePeerId reason=ble_peer_missing_connected_device_available"
            )
        } else if (
            freshBleObservation != null &&
            effectiveBlePeerId != null &&
            requestedBlePeerId != null &&
            effectiveBlePeerId != requestedBlePeerId
        ) {
            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "ble",
                phase = "local_fallback",
                outcome = "target_fallback",
                detail = "ctx=$attemptContext target=$effectiveBlePeerId requested_target=$requestedBlePeerId reason=fresh_ble_observation source=${freshBleObservation.source}"
            )
        } else if (
            requestedBlePeerId != null &&
            effectiveBlePeerId != null &&
            requestedBlePeerId != effectiveBlePeerId
        ) {
            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "ble",
                phase = "local_fallback",
                outcome = "target_fallback",
                detail = "ctx=$attemptContext target=$effectiveBlePeerId requested_target=$requestedBlePeerId reason=prefer_connected_device"
            )
        } else if (
            requestedBlePeerId != null &&
            effectiveBlePeerId == null &&
            connectedBleDevices.isEmpty()
        ) {
            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "ble",
                phase = "local_fallback",
                outcome = "skipped",
                detail = "ctx=$attemptContext requested_target=$requestedBlePeerId reason=stale_ble_hint_no_fresh_observation"
            )
        }

        // Use SmartTransportRouter for intelligent transport selection with 500ms timeout fallback
        val smartResult = if (smartTransportRouter != null) {
            smartTransportRouter!!.attemptDelivery(
                peerId = routePeerFallback,
                envelopeData = encryptedData,
                wifiPeerId = if (strictBleOnly) null else wifiPeerId,
                blePeerId = effectiveBlePeerId,
                tcpMdnsPeerId = routePeerCandidates
                    .firstOrNull { candidate ->
                        val trimmed = candidate.trim()
                        mdnsLanPeers[trimmed]?.isNotEmpty() == true
                    }
                    ?.trim(),
                routePeerCandidates = routePeerCandidates,
                listeners = listeners,
                traceMessageId = traceMessageId,
                attemptContext = attemptContext,
                tryWifi = { wifiId ->
                    val wifi = wifiTransportManager ?: run {
                        Timber.d("tryWifiDelivery: wifiTransportManager is null, returning false")
                        return@attemptDelivery false
                    }
                    try {
                        if (wifi.sendData(wifiId, encryptedData)) {
                            Timber.i("✓ Delivery via WiFi Direct (target=$wifiId)")
                            logDeliveryAttempt(
                                messageId = traceMessageId,
                                medium = "wifi-direct",
                                phase = "smart_router",
                                outcome = "success",
                                detail = "ctx=$attemptContext target=$wifiId"
                            )
                            true
                        } else {
                            Timber.d("tryWifiDelivery: wifiTransportManager.sendData returned false for $wifiId")
                            logDeliveryAttempt(
                                messageId = traceMessageId,
                                medium = "wifi-direct",
                                phase = "smart_router",
                                outcome = "failed",
                                detail = "ctx=$attemptContext target=$wifiId reason=sendData_false"
                            )
                            false
                        }
                    } catch (wifiEx: Exception) {
                        Timber.w(wifiEx, "WiFi send failed for $wifiId")
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "wifi-direct",
                            phase = "smart_router",
                            outcome = "failed",
                            detail = "ctx=$attemptContext target=$wifiId reason=${wifiEx.message ?: "exception"}"
                        )
                        false
                    }
                },
                tryBle = { bleAddr ->
                    val bleClient = bleGattClient
                    val bleServer = bleGattServer
                    if (bleClient == null && bleServer == null) {
                        Timber.d("tryBleDelivery: both bleGattClient and bleGattServer are null, returning false")
                        return@attemptDelivery false
                    }

                    // Fast-skip: if no BLE devices are connected and the requested
                    // address is a cached hint (not actively connected), skip entirely
                    // to avoid wasting time on stale MACs.
                    if (bleAddr.isNullOrBlank() && connectedBleDevices.isEmpty()) {
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "ble",
                            phase = "smart_router",
                            outcome = "skipped",
                            detail = "ctx=$attemptContext no_hint_and_no_connected_devices"
                        )
                        return@attemptDelivery false
                    }

                    val sendTargets = linkedSetOf(bleAddr).apply { addAll(connectedBleDevices) }
                    var lastFailureReason = "no_target_attempted"

                    for (target in sendTargets) {
                        if (bleClient != null) {
                            try {
                                if (bleClient.sendData(target, encryptedData)) {
                                    Timber.i("✓ Delivery via BLE client (target=$target)")
                                    logDeliveryAttempt(
                                        messageId = traceMessageId,
                                        medium = "ble",
                                        phase = "smart_router",
                                        outcome = "accepted",
                                        detail = "ctx=$attemptContext role=central requested_target=$bleAddr target=$target"
                                    )
                                    return@attemptDelivery true
                                }
                                lastFailureReason = "client_sendData_false:$target"
                            } catch (bleClientEx: Exception) {
                                Timber.w(bleClientEx, "BLE client send failed for $target")
                                lastFailureReason = "client_exception:${bleClientEx.message ?: "unknown"}"
                            }
                        }

                        if (bleServer != null) {
                            try {
                                if (bleServer.sendData(target, encryptedData)) {
                                    Timber.i("✓ Delivery via BLE server notify (target=$target)")
                                    logDeliveryAttempt(
                                        messageId = traceMessageId,
                                        medium = "ble",
                                        phase = "smart_router",
                                        outcome = "accepted",
                                        detail = "ctx=$attemptContext role=peripheral requested_target=$bleAddr target=$target"
                                    )
                                    return@attemptDelivery true
                                }
                                lastFailureReason = "server_sendData_false:$target"
                            } catch (bleServerEx: Exception) {
                                Timber.w(bleServerEx, "BLE server send failed for $target")
                                lastFailureReason = "server_exception:${bleServerEx.message ?: "unknown"}"
                            }
                        }
                    }
                    false
                },
                tryTcpMdns = { lanPeerId ->
                    // TCP/mDNS transport: Direct LAN delivery via libp2p TCP.
                    // This peer was discovered via mDNS and has LAN addresses — skip relay.
                    val bridge = swarmBridge ?: run {
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "tcp_mdns",
                            phase = "smart_router",
                            outcome = "failed",
                            detail = "ctx=$attemptContext reason=swarm_bridge_unavailable"
                        )
                        return@attemptDelivery false
                    }

                    // Dial LAN addresses directly (no relay circuits)
                    val lanAddrs = mdnsLanPeers[lanPeerId] ?: emptyList()
                    if (lanAddrs.isNotEmpty()) {
                        val dialCandidates = buildDialCandidatesForPeer(
                            routePeerId = lanPeerId,
                            rawAddresses = lanAddrs,
                            includeRelayCircuits = false
                        )
                        if (dialCandidates.isNotEmpty()) {
                            connectToPeer(lanPeerId, dialCandidates)
                            awaitPeerConnection(lanPeerId, timeoutMs = 1000L)
                        }
                    }

                    val directError = bridge.sendMessageStatus(
                        lanPeerId,
                        encryptedData,
                        recipientIdentityId,
                        intendedDeviceId
                    )

                    if (directError == null) {
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "tcp_mdns",
                            phase = "smart_router",
                            outcome = "success",
                            detail = "ctx=$attemptContext route=$lanPeerId lan_addrs=${lanAddrs.size}"
                        )
                        true
                    } else {
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "tcp_mdns",
                            phase = "smart_router",
                            outcome = "failed",
                            detail = "ctx=$attemptContext route=$lanPeerId reason=$directError"
                        )
                        false
                    }
                },
                tryCore = { corePeerId ->
                    // Core transport attempt (libp2p/internet relay)
                    val bridge = swarmBridge ?: run {
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "core",
                            phase = "smart_router",
                            outcome = "failed",
                            detail = "ctx=$attemptContext reason=swarm_bridge_unavailable"
                        )
                        return@attemptDelivery false
                    }
                    
                    val liveRouteHints = getDialHintsForRoutePeer(corePeerId)
                    val dialCandidates = buildDialCandidatesForPeer(
                        routePeerId = corePeerId,
                        rawAddresses = listeners + liveRouteHints,
                        includeRelayCircuits = true
                    )
                    if (dialCandidates.isNotEmpty()) {
                        connectToPeer(corePeerId, dialCandidates)
                        val connected = awaitPeerConnection(corePeerId, timeoutMs = 2000L)
                        Timber.d("🔀 Transport: route=$corePeerId connected=$connected timeout=2000ms")
                    }
                    
                    val attemptStart = System.currentTimeMillis()
                    val directError = bridge.sendMessageStatus(
                        corePeerId,
                        encryptedData,
                        recipientIdentityId,
                        intendedDeviceId
                    )
                    
                    if (directError == null) {
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "core",
                            phase = "smart_router",
                            outcome = "success",
                            detail = "ctx=$attemptContext route=$corePeerId"
                        )
                        true
                    } else {
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "core",
                            phase = "smart_router",
                            outcome = "failed",
                            detail = "ctx=$attemptContext route=$corePeerId reason=$directError"
                        )
                        false
                    }
                }
            )
        } else {
            // Fallback to legacy LocalTransportFallback if router not available
            val localFallback = Companion.attemptWifiThenBleFallback(
                wifiPeerId = if (strictBleOnly) null else wifiPeerId,
                blePeerId = effectiveBlePeerId,
                tryWifi = { wifiId ->
                    val wifi = wifiTransportManager ?: run {
                        Timber.d("tryWifiDelivery: wifiTransportManager is null, returning false")
                        return@attemptWifiThenBleFallback false
                    }
                    try {
                        if (wifi.sendData(wifiId, encryptedData)) {
                            Timber.i("✓ Delivery via WiFi Direct (target=$wifiId)")
                            logDeliveryAttempt(
                                messageId = traceMessageId,
                                medium = "wifi-direct",
                                phase = "local_fallback",
                                outcome = "success",
                                detail = "ctx=$attemptContext target=$wifiId"
                            )
                            true
                        } else {
                            Timber.d("tryWifiDelivery: wifiTransportManager.sendData returned false for $wifiId")
                            logDeliveryAttempt(
                                messageId = traceMessageId,
                                medium = "wifi-direct",
                                phase = "local_fallback",
                                outcome = "failed",
                                detail = "ctx=$attemptContext target=$wifiId reason=sendData_false"
                            )
                            false
                        }
                    } catch (wifiEx: Exception) {
                        Timber.w(wifiEx, "WiFi send failed for $wifiId")
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "wifi-direct",
                            phase = "local_fallback",
                            outcome = "failed",
                            detail = "ctx=$attemptContext target=$wifiId reason=${wifiEx.message ?: "exception"}"
                        )
                        false
                    }
                },
                tryBle = { bleAddr ->
                    val bleClient = bleGattClient
                    val bleServer = bleGattServer
                    if (bleClient == null && bleServer == null) {
                        Timber.d("tryBleDelivery: both bleGattClient and bleGattServer are null, returning false")
                        return@attemptWifiThenBleFallback false
                    }

                    // Fast-skip: if no BLE devices are connected and the requested
                    // address is a cached hint (not actively connected), skip entirely
                    // to avoid wasting time on stale MACs.
                    if (bleAddr.isNullOrBlank() && connectedBleDevices.isEmpty()) {
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "ble",
                            phase = "local_fallback",
                            outcome = "skipped",
                            detail = "ctx=$attemptContext no_hint_and_no_connected_devices"
                        )
                        return@attemptWifiThenBleFallback false
                    }

                    val sendTargets = linkedSetOf(bleAddr).apply { addAll(connectedBleDevices) }
                    var lastFailureReason = "no_target_attempted"

                    for (target in sendTargets) {
                        if (bleClient != null) {
                            try {
                                if (bleClient.sendData(target, encryptedData)) {
                                    Timber.i("✓ Delivery via BLE client (target=$target)")
                                    logDeliveryAttempt(
                                        messageId = traceMessageId,
                                        medium = "ble",
                                        phase = "local_fallback",
                                        outcome = "accepted",
                                        detail = "ctx=$attemptContext role=central requested_target=$bleAddr target=$target"
                                    )
                                    return@attemptWifiThenBleFallback true
                                }
                                lastFailureReason = "client_sendData_false:$target"
                            } catch (bleClientEx: Exception) {
                                Timber.w(bleClientEx, "BLE client send failed for $target")
                                lastFailureReason = "client_exception:${bleClientEx.message ?: "unknown"}"
                            }
                        }

                        if (bleServer != null) {
                            try {
                                if (bleServer.sendData(target, encryptedData)) {
                                    Timber.i("✓ Delivery via BLE server notify (target=$target)")
                                    logDeliveryAttempt(
                                        messageId = traceMessageId,
                                        medium = "ble",
                                        phase = "local_fallback",
                                        outcome = "accepted",
                                        detail = "ctx=$attemptContext role=peripheral requested_target=$bleAddr target=$target"
                                    )
                                    return@attemptWifiThenBleFallback true
                                }
                                lastFailureReason = "server_sendData_false:$target"
                            } catch (bleServerEx: Exception) {
                                Timber.w(bleServerEx, "BLE server send failed for $target")
                                lastFailureReason = "server_exception:${bleServerEx.message ?: "unknown"}"
                            }
                        }
                    }
                    false
                }
            )
            com.scmessenger.android.transport.SmartTransportRouter.TransportDeliveryResult(
                transport = if (localFallback.acked) {
                    if (localFallback.wifiAcked) com.scmessenger.android.transport.SmartTransportRouter.TransportType.WIFI_DIRECT
                    else com.scmessenger.android.transport.SmartTransportRouter.TransportType.BLE
                } else com.scmessenger.android.transport.SmartTransportRouter.TransportType.CORE,
                success = localFallback.acked,
                latencyMs = 0,
                error = if (localFallback.acked) null else "legacy_fallback_failed"
            )
        }
        
        val localAcked = smartResult.success

        if (strictBleOnly) {
            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "ble-only",
                phase = "aggregate",
                outcome = if (localAcked) "accepted" else "failed",
                detail = "ctx=$attemptContext route_fallback=$blePeerId"
            )
            // BLE-only mode: accept local ACK as delivery
            return DeliveryAttemptResult(acked = localAcked, routePeerId = blePeerId)
        }

        val bridge = swarmBridge ?: run {
            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "core",
                phase = "direct",
                outcome = "failed",
                detail = "ctx=$attemptContext reason=swarm_bridge_unavailable route_fallback=$wifiPeerId ble_only=${localAcked}"
            )
            // Core unavailable - don't mark as delivered even if BLE succeeded
            return DeliveryAttemptResult(acked = false, routePeerId = wifiPeerId)
        }
        val sanitizedCandidates = routePeerCandidates
            .map { it.trim() }
            .filter { it.isNotEmpty() && PeerIdValidator.isLibp2pPeerId(it) }
            .distinct()

        if (sanitizedCandidates.isEmpty()) {
            // AND-NO-ROUTE-001: Add diagnostic context for empty route candidates
            val diagnosticDetails = buildString {
                append("ctx=$attemptContext ")
                append("route_fallback=$wifiPeerId ")
                append("ble_only=${localAcked} ")
                
                // Analyze why candidates might be empty (using available parameters)
                val discoveryCount = discoverRoutePeersForPublicKey(recipientIdentityId).size
                val candidatesCount = routePeerCandidates.size
                val listenersCount = listeners.size
                
                append("discovery=$discoveryCount ")
                append("input_candidates=$candidatesCount ")
                append("listeners=$listenersCount ")
                append("recipient_id=${recipientIdentityId?.take(8) ?: "null"}")
                append("intended_device=${intendedDeviceId?.take(8) ?: "null"}")
            }
            
            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "core",
                phase = "direct",
                outcome = "failed",
                detail = "reason=no_route_candidates $diagnosticDetails",
                callerContext = "attemptDirectSwarmDelivery"
            )
            // No core routes - don't mark as delivered even if BLE succeeded
            return DeliveryAttemptResult(acked = false, routePeerId = wifiPeerId)
        }

        primeRelayBootstrapConnections()

        for (routePeerId in sanitizedCandidates) {
            val liveRouteHints = getDialHintsForRoutePeer(routePeerId)
            val dialCandidates = buildDialCandidatesForPeer(
                routePeerId = routePeerId,
                rawAddresses = listeners + liveRouteHints,
                includeRelayCircuits = true
            )
            Timber.d("🔀 Transport: route=$routePeerId dialCandidates=${dialCandidates.size} (${dialCandidates.joinToString { it.substringBefore("/") }})")
            if (dialCandidates.isNotEmpty()) {
                connectToPeer(routePeerId, dialCandidates)
                val connected = awaitPeerConnection(routePeerId, timeoutMs = 2000L)
                Timber.d("🔀 Transport: route=$routePeerId connected=$connected timeout=2000ms")
            }

            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "core",
                phase = "direct",
                outcome = "attempt",
                detail = "ctx=$attemptContext route=$routePeerId transports=${dialCandidates.size}"
            )
            val attemptStart = System.currentTimeMillis()
            val directError = bridge.sendMessageStatus(
                routePeerId,
                encryptedData,
                recipientIdentityId,
                intendedDeviceId
            )
            if (directError == null) {
                val latencyMs = System.currentTimeMillis() - attemptStart
                Timber.i("✓ Direct delivery ACK from $routePeerId (${latencyMs}ms)")
                logDeliveryAttempt(
                    messageId = traceMessageId,
                    medium = "core",
                    phase = "direct",
                    outcome = "success",
                    detail = "ctx=$attemptContext route=$routePeerId latency=${latencyMs}ms"
                )
                return DeliveryAttemptResult(acked = true, routePeerId = routePeerId)
            } else {
                Timber.w("Core-routed delivery failed for $routePeerId: $directError; trying alternative transports")
                logDeliveryAttempt(
                    messageId = traceMessageId,
                    medium = "core",
                    phase = "direct",
                    outcome = "failed",
                    detail = "ctx=$attemptContext route=$routePeerId reason=$directError"
                )
                if (isTerminalIdentityFailure(directError)) {
                    return DeliveryAttemptResult(
                        acked = false,
                        routePeerId = routePeerId,
                        terminalFailureCode = directError
                    )
                }
            }

            val relayOnlyCandidates = relayCircuitAddressesForPeer(routePeerId)
            if (relayOnlyCandidates.isNotEmpty()) {
                Timber.d("🔀 Transport: Attempting relay-circuit for $routePeerId (${relayOnlyCandidates.size} candidates)")
                connectToPeer(routePeerId, relayOnlyCandidates)
                val connected = awaitPeerConnection(routePeerId, timeoutMs = 1500L)
                Timber.d("🔀 Transport: relay-circuit route=$routePeerId connected=$connected timeout=1500ms")
                kotlinx.coroutines.delay(500)
                logDeliveryAttempt(
                    messageId = traceMessageId,
                    medium = "relay-circuit",
                    phase = "retry",
                    outcome = "attempt",
                    detail = "ctx=$attemptContext route=$routePeerId relays=${relayOnlyCandidates.size}"
                )
                val relayStart = System.currentTimeMillis()
                val relayError = bridge.sendMessageStatus(
                    routePeerId,
                    encryptedData,
                    recipientIdentityId,
                    intendedDeviceId
                )
                if (relayError == null) {
                    val latencyMs = System.currentTimeMillis() - relayStart
                    Timber.i("✓ Delivery ACK from $routePeerId after relay-circuit retry (${latencyMs}ms)")
                    logDeliveryAttempt(
                        messageId = traceMessageId,
                        medium = "relay-circuit",
                        phase = "retry",
                        outcome = "success",
                        detail = "ctx=$attemptContext route=$routePeerId latency=${latencyMs}ms"
                    )
                    return DeliveryAttemptResult(acked = true, routePeerId = routePeerId)
                } else {
                    Timber.w("Relay-circuit retry failed for $routePeerId: $relayError")
                    logDeliveryAttempt(
                        messageId = traceMessageId,
                        medium = "relay-circuit",
                        phase = "retry",
                        outcome = "failed",
                        detail = "ctx=$attemptContext route=$routePeerId reason=$relayError"
                    )
                    if (isTerminalIdentityFailure(relayError)) {
                        return DeliveryAttemptResult(
                            acked = false,
                            routePeerId = routePeerId,
                            terminalFailureCode = relayError
                        )
                    }
                }
            }
        }
        logDeliveryAttempt(
            messageId = traceMessageId,
            medium = "final",
            phase = "aggregate",
            outcome = "failed",
            detail = "ctx=$attemptContext reason=all_transports_failed ble_only=${localAcked}"
        )
        return DeliveryAttemptResult(acked = false, routePeerId = null)
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
        pendingOutboxFlushMutex.lock()
        try {
            // Ensure relay backbone is reachable whenever we check outbox
            primeRelayBootstrapConnections()

            val now = System.currentTimeMillis() / 1000
            val queue = loadPendingOutbox().toMutableList()
            if (queue.isEmpty()) return
            Timber.d("Flushing pending outbox (${queue.size} item(s)); reason=$reason")

            var updated = false
            val iterator = queue.listIterator()
            while (iterator.hasNext()) {
                // Yield between items to prevent CPU starvation under retry load.
                kotlinx.coroutines.yield()
                val item = iterator.next()
                val expiryReason = pendingOutboxExpiryReason(item, now)
                if (expiryReason != null) {
                    logDeliveryState(
                        messageId = item.historyRecordId,
                        state = "failed",
                        detail = "dropped_pending_outbox reason=$expiryReason attempt=${item.attemptCount}"
                    )
                    iterator.remove()
                    updated = true
                    continue
                }
                if (item.terminalFailureCode != null) {
                    continue
                }
                // AND-DELIVERY-001: Enforce maximum retry limit to prevent infinite retries
                if (item.attemptCount >= pendingOutboxMaxAttempts) {
                    Timber.w("Dropping message ${item.historyRecordId} after ${item.attemptCount} attempts (max=$pendingOutboxMaxAttempts)")
                    logDeliveryState(
                        messageId = item.historyRecordId,
                        state = "failed",
                        detail = "dropped_pending_outbox reason=max_attempts_exceeded attempt=${item.attemptCount}"
                    )
                    iterator.remove()
                    updated = true
                    continue
                }
                if (item.nextAttemptAtEpochSec > now) continue
                if (isMessageDeliveredLocally(item.historyRecordId)) {
                    iterator.remove()
                    updated = true
                    continue
                }
                logDeliveryState(
                    messageId = item.historyRecordId,
                    state = "forwarding",
                    detail = "retry_attempt=${item.attemptCount + 1}"
                )

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
                    notes = contact?.notes,
                    recipientPublicKey = contact?.publicKey
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
                    encryptedData = envelope,
                    wifiPeerId = latestRouting.wifiPeerId,
                    blePeerId = latestRouting.blePeerId,
                    traceMessageId = item.historyRecordId,
                    attemptContext = "outbox_retry",
                    strictBleOnlyOverride = item.strictBleOnlyMode,
                    recipientIdentityId = item.recipientIdentityId ?: contact?.publicKey,
                    intendedDeviceId = item.intendedDeviceId ?: contact?.lastKnownDeviceId
                )
                val selectedRoutePeerId = if (delivery.acked) {
                    delivery.routePeerId ?: resolvedRoutePeerId
                } else {
                    delivery.routePeerId
                }
                if (isMessageDeliveredLocally(item.historyRecordId)) {
                    iterator.remove()
                    updated = true
                    continue
                }

                if (delivery.terminalFailureCode != null) {
                    iterator.set(
                        item.copy(
                            routePeerId = selectedRoutePeerId,
                            listeners = resolvedListeners,
                            terminalFailureCode = delivery.terminalFailureCode
                        )
                    )
                    logDeliveryState(
                        messageId = item.historyRecordId,
                        state = "rejected",
                        detail = "terminal_failure_code=${delivery.terminalFailureCode}"
                    )
                    updated = true
                    continue
                }

                if (delivery.acked) {
                    // Adaptive post-ACK receipt wait: grows with attempt count to prevent
                    // re-delivering the same message every 8 seconds indefinitely when
                    // receipt delivery is slow or broken.
                    val adaptiveReceiptWait = when {
                        item.attemptCount <= 3 -> receiptAwaitSeconds        // 8s for first few
                        item.attemptCount <= 10 -> 30L                       // 30s for moderate retries
                        item.attemptCount <= 30 -> 60L                       // 60s for persistent retries
                        else -> 120L                                         // 2 min for very old messages
                    }
                    iterator.set(
                        item.copy(
                            routePeerId = selectedRoutePeerId,
                            listeners = resolvedListeners,
                            attemptCount = item.attemptCount + 1,
                            nextAttemptAtEpochSec = now + adaptiveReceiptWait,
                            strictBleOnlyMode = item.strictBleOnlyMode
                        )
                    )
                    logDeliveryState(
                        messageId = item.historyRecordId,
                        state = "stored",
                        detail = "awaiting_receipt_delay_sec=$adaptiveReceiptWait"
                    )
                    updated = true
                    continue
                }

                val nextAttemptCount = item.attemptCount + 1
                // Aggressive retry for transport transitions: fast initial attempts
                // Attempt 1: 0.5s, 2: 1s, 3: 2s, 4: 4s, 5: 8s, 6: 16s
                // Attempts 7-20: 60 seconds (steady retry)
                // Attempts 21+: 300 seconds (patient long-term)
                val backoffSecs = when (nextAttemptCount) {
                    1 -> 0L  // Immediate first retry
                    2 -> 1L
                    3 -> 2L
                    4 -> 4L
                    5 -> 8L
                    6 -> 16L
                    in 7..20 -> 60L
                    else -> 300L
                }
                iterator.set(
                    item.copy(
                        routePeerId = selectedRoutePeerId,
                        listeners = resolvedListeners,
                        attemptCount = nextAttemptCount,
                        nextAttemptAtEpochSec = now + backoffSecs,
                        strictBleOnlyMode = item.strictBleOnlyMode
                    )
                )
                logDeliveryState(
                    messageId = item.historyRecordId,
                    state = "stored",
                    detail = "retry_backoff_sec=$backoffSecs attempt=$nextAttemptCount"
                )
                updated = true
            }

            if (updated) {
                savePendingOutbox(queue)
                // P0_ANDROID_005: Check for retry storms after outbox state changes
                logRetryStormDetection()
            }
        } finally {
            pendingOutboxFlushMutex.unlock()
        }
    }

    private fun enqueuePendingOutbound(
        historyRecordId: String,
        peerId: String,
        routePeerId: String?,
        listeners: List<String>,
        encryptedData: ByteArray,
        initialAttemptCount: Int = 0,
        initialDelaySec: Long = 0,
        strictBleOnlyMode: Boolean = false,
        recipientIdentityId: String? = null,
        intendedDeviceId: String? = null,
        terminalFailureCode: String? = null
    ) {
        if (isMessageDeliveredLocally(historyRecordId)) {
            logDeliveryState(
                messageId = historyRecordId,
                state = "delivered",
                detail = "skip_enqueue_already_delivered"
            )
            return
        }
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
                nextAttemptAtEpochSec = now + initialDelaySec,
                strictBleOnlyMode = strictBleOnlyMode,
                recipientIdentityId = recipientIdentityId,
                intendedDeviceId = intendedDeviceId,
                terminalFailureCode = terminalFailureCode
            )
        )
        savePendingOutbox(queue)
        val initialState = if (initialDelaySec > 0) "stored" else "forwarding"
        logDeliveryState(
            messageId = historyRecordId,
            state = initialState,
            detail = "enqueued attempt=$initialAttemptCount next_attempt_delay_sec=$initialDelaySec"
        )
        repoScope.launch { flushPendingOutbox("enqueue") }
    }

    // ANR FIX: Cache for pending outbox to avoid repeated I/O
    @Volatile private var cachedPendingOutbox: List<PendingOutboundEnvelope> = emptyList()
    @Volatile private var pendingOutboxCacheTimeMs: Long = 0L
    private val pendingOutboxCacheTtlMs = 1000L  // 1 second TTL

    /**
     * ANR FIX (P0_ANDROID_017): Load pending outbox asynchronously to avoid Main thread I/O.
     * Uses cached results when available to prevent repeated file reads during diagnostics export.
     */
    private suspend fun loadPendingOutboxAsync(): List<PendingOutboundEnvelope> = kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
        // Return cached result if fresh enough
        if (cachedPendingOutbox.isNotEmpty() && System.currentTimeMillis() - pendingOutboxCacheTimeMs < pendingOutboxCacheTtlMs) {
            return@withContext cachedPendingOutbox
        }

        val result = if (!pendingOutboxFile.exists()) {
            emptyList()
        } else {
            try {
                val raw = pendingOutboxFile.readText()
                if (raw.isBlank()) return@withContext emptyList()
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
                                nextAttemptAtEpochSec = obj.optLong("next_attempt_at", 0),
                                strictBleOnlyMode = if (obj.has("strict_ble_only_mode")) obj.optBoolean("strict_ble_only_mode") else null,
                                recipientIdentityId = obj.optString("recipient_identity_id").ifBlank { null },
                                intendedDeviceId = obj.optString("intended_device_id").ifBlank { null },
                                terminalFailureCode = obj.optString("terminal_failure_code").ifBlank { null }
                            )
                        )
                    }
                }
            } catch (e: Exception) {
                Timber.w(e, "Failed to parse pending outbox")
                emptyList()
            }
        }

        // Update cache
        cachedPendingOutbox = result
        pendingOutboxCacheTimeMs = System.currentTimeMillis()
        result
    }

    /**
     * Synchronous load for internal use (must only be called from IO dispatcher).
     */
    @Synchronized
    private fun loadPendingOutboxSync(): List<PendingOutboundEnvelope> {
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
                            nextAttemptAtEpochSec = obj.optLong("next_attempt_at", 0),
                            strictBleOnlyMode = if (obj.has("strict_ble_only_mode")) obj.optBoolean("strict_ble_only_mode") else null,
                            recipientIdentityId = obj.optString("recipient_identity_id").ifBlank { null },
                            intendedDeviceId = obj.optString("intended_device_id").ifBlank { null },
                            terminalFailureCode = obj.optString("terminal_failure_code").ifBlank { null }
                        )
                    )
                }
            }
        } catch (e: Exception) {
            Timber.w(e, "Failed to parse pending outbox")
            emptyList()
        }
    }

    /**
     * Public sync version - must only be called from IO dispatcher.
     */
    internal fun loadPendingOutbox(): List<PendingOutboundEnvelope> = loadPendingOutboxSync()

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
                        .put("strict_ble_only_mode", item.strictBleOnlyMode ?: false)
                        .put("recipient_identity_id", item.recipientIdentityId ?: "")
                        .put("intended_device_id", item.intendedDeviceId ?: "")
                        .put("terminal_failure_code", item.terminalFailureCode ?: "")
                )
            }
            pendingOutboxFile.writeText(arr.toString())
        } catch (e: Exception) {
            Timber.w(e, "Failed to persist pending outbox")
        }
    }

    private fun pendingOutboxExpiryReason(item: PendingOutboundEnvelope, nowEpochSec: Long): String? {
        // PHILOSOPHY: Messages NEVER expire. Every message retries
        // until successfully delivered. No attempt limit, no age limit.
        return null
    }

    // ========================================================================
    // ROUTING HELPERS
    // ========================================================================

    private fun resolveCanonicalPeerId(senderId: String, senderPublicKeyHex: String): String {
        val normalizedIncomingKey = normalizePublicKey(senderPublicKeyHex) ?: return senderId

        // Priority 1: Use direct hash resolution from core
        val identityId = try { ironCore?.resolveIdentity(senderPublicKeyHex) } catch (e: Exception) { null }
        if (identityId != null) {
            // Check if we already have a contact pointing to ANOTHER peerId for this key
            // (e.g. legacy libp2p ID contact). If so, we merge by returning the existing ID.
            val contacts = try { contactManager?.list().orEmpty() } catch (e: Exception) { emptyList() }
            val existingContact = contacts.firstOrNull { normalizePublicKey(it.publicKey) == normalizedIncomingKey }
            return existingContact?.peerId ?: identityId
        }

        val contacts = try {
            contactManager?.list().orEmpty()
        } catch (e: Exception) {
            Timber.d("Unable to resolve canonical sender ID: ${e.message}")
            return senderId
        }

        val exactMatch = contacts.firstOrNull {
            PeerIdValidator.isSame(it.peerId, senderId) && normalizePublicKey(it.publicKey) == normalizedIncomingKey
        }
        if (exactMatch != null) return exactMatch.peerId

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
        if (PeerIdValidator.isLibp2pPeerId(senderId)) {
            val linkedIdentityContacts = contacts.filter {
                normalizePublicKey(it.publicKey) == normalizedIncomingKey &&
                    PeerIdValidator.isSame(parseRoutingHints(it.notes).libp2pPeerId.orEmpty(), senderId) &&
                    !PeerIdValidator.isSame(it.peerId, senderId)
            }

            return when (linkedIdentityContacts.size) {
                1 -> linkedIdentityContacts.first().peerId
                else -> {
                    if (linkedIdentityContacts.size > 1) {
                        Timber.w(
                            "Ambiguous canonical sender mapping for $senderId (matched ${linkedIdentityContacts.size} contacts); keeping raw sender ID"
                        )
                    }
                    PeerIdValidator.normalize(senderId)
                }
            }
        }

        // Identity IDs (Blake3 hex) can represent the same peer that was
        // previously saved under a libp2p contact ID; map only when unique.
        if (!PeerIdValidator.isIdentityId(senderId)) return senderId
        val keyedRoutedContacts = contacts.filter {
            normalizePublicKey(it.publicKey) == normalizedIncomingKey &&
                it.peerId != senderId &&
                (
                    !parseRoutingHints(it.notes).libp2pPeerId.isNullOrBlank() ||
                    PeerIdValidator.isLibp2pPeerId(it.peerId)
                )
        }
        return when (keyedRoutedContacts.size) {
            1 -> keyedRoutedContacts.first().peerId
            else -> {
                if (keyedRoutedContacts.size > 1) {
                    Timber.w(
                        "Ambiguous identity sender mapping for $senderId (matched ${keyedRoutedContacts.size} contacts); keeping raw sender ID"
                    )
                }
                PeerIdValidator.normalize(senderId)
            }
        }
    }

    private fun resolveCanonicalPeerIdFromMessageHints(
        resolvedCanonicalPeerId: String,
        senderId: String,
        senderPublicKeyHex: String,
        hintedIdentityId: String?
    ): String {
        val hint = hintedIdentityId?.trim() ?: return resolvedCanonicalPeerId
        val normalizedHint = PeerIdValidator.normalize(hint)
        if (!PeerIdValidator.isIdentityId(normalizedHint)) return resolvedCanonicalPeerId
        if (PeerIdValidator.isSame(normalizedHint, resolvedCanonicalPeerId)) return resolvedCanonicalPeerId
        if (isBootstrapRelayPeer(normalizedHint)) return resolvedCanonicalPeerId

        val normalizedSenderKey = normalizePublicKey(senderPublicKeyHex)
        val contacts = try {
            contactManager?.list().orEmpty()
        } catch (_: Exception) {
            emptyList()
        }

        if (normalizedSenderKey != null) {
            val hintedContact = contacts.firstOrNull { PeerIdValidator.isSame(it.peerId, normalizedHint) }
            if (hintedContact != null && normalizePublicKey(hintedContact.publicKey) == normalizedSenderKey) {
                return hintedContact.peerId
            }

            val keyMatches = contacts.filter { normalizePublicKey(it.publicKey) == normalizedSenderKey }
            if (keyMatches.size == 1) return keyMatches.first().peerId
            if (keyMatches.isNotEmpty()) return resolvedCanonicalPeerId
        }

        return if (PeerIdValidator.isSame(resolvedCanonicalPeerId, senderId) || PeerIdValidator.isLibp2pPeerId(resolvedCanonicalPeerId)) {
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
        val deviceId = identity.deviceId?.trim().orEmpty()
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
                        .put("device_id", deviceId)
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
                    deviceId = it.optString("device_id", "").trim().takeIf { value -> value.isNotBlank() },
                    nickname = normalizeNickname(it.optString("nickname", "")),
                    libp2pPeerId = it.optString("libp2p_peer_id", "").trim().takeIf { value ->
                        value.isNotBlank() && PeerIdValidator.isLibp2pPeerId(value)
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

    private fun isWifiPeerId(value: String): Boolean {
        val normalized = value.trim()
        if (normalized.isEmpty()) return false
        val macRegex = Regex("(?i)^([0-9a-f]{2}:){5}[0-9a-f]{2}$")
        val ipv4Regex = Regex("^\\d{1,3}(\\.\\d{1,3}){3}$")
        return macRegex.matches(normalized) || ipv4Regex.matches(normalized)
    }

    private fun selectCanonicalPeerId(incomingPeerId: String, existingPeerId: String): String {
        val incoming = incomingPeerId.trim()
        val existing = existingPeerId.trim()
        if (incoming.isEmpty()) return existing
        if (existing.isEmpty() || existing == incoming) return incoming

        val incomingIsLibp2p = PeerIdValidator.isLibp2pPeerId(incoming)
        val existingIsLibp2p = PeerIdValidator.isLibp2pPeerId(existing)
        val incomingIsIdentity = PeerIdValidator.isIdentityId(incoming)
        val existingIsIdentity = PeerIdValidator.isIdentityId(existing)
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
    ): String {
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

        val resolved = selectAuthoritativeNickname(incomingNickname, fromContact)
        if (!resolved.isNullOrBlank()) {
            return resolved
        }

        // Auto-generate nickname if none found
        val shortId = if (peerId.startsWith("12D3KooW")) {
            peerId.takeLast(8)
        } else {
            peerId.take(8)
        }
        val generated = "peer-$shortId"
        Timber.d("Generated default nickname '$generated' for peer $peerId")
        return generated
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
        if (normalizedRoute.isEmpty() || !PeerIdValidator.isLibp2pPeerId(normalizedRoute)) return

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
            mutableListOf<String>()
        } else {
            existing.split(';', '\n').map { it.trim() }.toMutableList()
        }

        // Replace existing entry for this key instead of appending a duplicate.
        // This prevents stale BLE MACs / route peer IDs from accumulating.
        val existingIndex = components.indexOfFirst { it.startsWith("$key:") }
        val newEntry = "$key:$normalizedValue"
        if (existingIndex >= 0) {
            if (components[existingIndex] == newEntry) return notes // unchanged
            Timber.d("Routing hint update: replacing old ${components[existingIndex]} with $newEntry")
            components[existingIndex] = newEntry
        } else {
            components.add(newEntry)
        }

        return components.filter { it.isNotEmpty() }.joinToString(";")
    }

    /**
     * Merge routing notes from two contacts, preserving all unique hints.
     * Used during ID coalescence to avoid losing routing information.
     */
    private fun mergeNotes(existing: String?, incoming: String?): String? {
        if (existing.isNullOrBlank()) return incoming
        if (incoming.isNullOrBlank()) return existing
        
        val existingComponents = existing.split(';', '\n').map { it.trim() }.filter { it.isNotEmpty() }
        val incomingComponents = incoming.split(';', '\n').map { it.trim() }.filter { it.isNotEmpty() }
        
        // Build a map of key:value pairs, preferring existing values
        val merged = mutableMapOf<String, String>()
        for (component in existingComponents) {
            val colonIndex = component.indexOf(':')
            if (colonIndex > 0) {
                merged[component.substring(0, colonIndex)] = component
            } else {
                merged[component] = component
            }
        }
        // Add incoming components that don't conflict
        for (component in incomingComponents) {
            val colonIndex = component.indexOf(':')
            val key = if (colonIndex > 0) component.substring(0, colonIndex) else component
            if (key !in merged) {
                merged[key] = component
            }
        }
        
        return merged.values.filter { it.isNotEmpty() }.joinToString(";").ifEmpty { null }
    }

    private fun resolveTransportIdentity(libp2pPeerId: String): TransportIdentityResolution? {
        if (!PeerIdValidator.isLibp2pPeerId(libp2pPeerId)) {
            Timber.d("Invalid libp2p peer ID format: $libp2pPeerId")
            return null
        }
        
        Timber.d("resolveTransportIdentity called for: $libp2pPeerId")
        
        // Relay peers should not have user-visible transport identities
        val isRelay = isBootstrapRelayPeer(libp2pPeerId)
        Timber.d("  isBootstrapRelayPeer check: $isRelay")
        
        if (isRelay) {
            Timber.d("  → Filtering relay peer from transport identity resolution")
            return null
        }

        val extractedKey = try {
            ironCore?.extractPublicKeyFromPeerId(libp2pPeerId)
        } catch (e: Exception) {
            Timber.d("  Failed to extract public key from peer $libp2pPeerId: ${e.message}")
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
        
        if (canonicalContact == null) {
            if (isBootstrapRelayPeerFromKey(normalizedKey)) {
                Timber.d("No existing contact for transport key ${normalizedKey.take(8)}..., treating as transient relay")
                return null
            }

            val peerId = PeerKeyUtils.extractPeerIdFromPublicKey(normalizedKey)
            if (!validatePeerBeforeContactCreation(peerId, normalizedKey)) {
                Timber.w("Peer validation failed for ${normalizedKey.take(8)}..., skipping emergency contact creation")
                return TransportIdentityResolution(
                    canonicalPeerId = normalizedKey,
                    publicKey = normalizedKey,
                    nickname = null,
                    localNickname = null
                )
            }

            Timber.w("Creating emergency contact for unknown peer: ${normalizedKey.take(8)}...")
            val emergencyContact = createEmergencyContact(normalizedKey)
            logIdentityResolutionDetails(normalizedKey, emergencyContact, createdNew = true)

            return TransportIdentityResolution(
                canonicalPeerId = normalizedKey,
                publicKey = normalizedKey,
                nickname = emergencyContact.nickname?.takeIf { it.isNotBlank() },
                localNickname = emergencyContact.localNickname?.takeIf { it.isNotBlank() }
            )
        }

        logIdentityResolutionDetails(normalizedKey, canonicalContact, createdNew = false)

        // UNIFIED ID FIX: canonicalPeerId is ALWAYS public_key_hex for contact storage.
        // libp2p_peer_id is only used for network transport, never as the contact key.
        return TransportIdentityResolution(
            canonicalPeerId = normalizedKey,
            publicKey = normalizedKey,
            nickname = canonicalContact.nickname?.takeIf { it.isNotBlank() },
            localNickname = canonicalContact.localNickname?.takeIf { it.isNotBlank() }
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
                notes = withListeners,
                lastKnownDeviceId = contact.lastKnownDeviceId
            )
            try {
                contactManager?.add(updated)
            } catch (e: Exception) {
                Timber.d("Failed to persist route hints for ${contact.peerId}: ${e.message}")
            }
        }
    }

    /**
     * Upserts a federated contact with synchronization to prevent duplicate creation.
     *
     * This function is synchronized using contactUpsertMutex to prevent race conditions
     * where concurrent peer identification callbacks could create duplicate contacts
     * for the same peer (AND-CONTACT-DUP-001).
     *
     * The function:
     * 1. Acquires the mutex lock to ensure atomic contact lookup and creation
     * 2. Checks for existing contacts by both public key and peer ID
     * 3. Merges duplicates if found by both key and ID but with different peer IDs
     * 4. Creates or updates the contact with resolved identity information
     */
    private suspend fun upsertFederatedContact(
        canonicalPeerId: String,
        publicKey: String,
        nickname: String?,
        libp2pPeerId: String?,
        wifiPeerId: String? = null,
        listeners: List<String>,
        blePeerId: String? = null,
        deviceId: String? = null,
        createIfMissing: Boolean = true
    ) {
        contactUpsertMutex.withLock {
            val normalizedPeerId = canonicalPeerId.trim()
            val normalizedKey = normalizePublicKey(publicKey) ?: return@withLock
            if (normalizedPeerId.isEmpty()) return@withLock

            val routePeer = libp2pPeerId?.trim()?.takeIf { it.isNotEmpty() }
            if (!routePeer.isNullOrBlank() && isBootstrapRelayPeer(routePeer)) return@withLock

            val contacts = try {
                contactManager?.list().orEmpty()
            } catch (e: Exception) {
                Timber.d("Failed to list contacts for federation upsert: ${e.message}")
                return@withLock
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
                return@withLock
            }

            // If we found them by key but they have a different PeerID now,
            // we should merge them to prevent duplicates.
            if (existingByKey != null && existingById != null && existingByKey.peerId != existingById.peerId) {
                Timber.i("Merging duplicate identities for key ${normalizedKey.take(8)}...: ${existingById.peerId} -> ${existingByKey.peerId}")
                try { contactManager?.remove(existingById.peerId) } catch (_: Exception) {}
            }

            val existing = existingByKey ?: existingById
            if (existing == null && !createIfMissing) {
                return@withLock
            }

            var notes = existing?.notes
            if (!routePeer.isNullOrBlank()) {
                notes = appendRoutingHint(notes = notes, key = "libp2p_peer_id", value = routePeer)
            }
            val normalizedBle = blePeerId?.trim()?.takeIf { it.isNotEmpty() }
            if (!normalizedBle.isNullOrBlank()) {
                notes = appendRoutingHint(notes = notes, key = "ble_peer_id", value = normalizedBle)
            }
            val normalizedWifi = wifiPeerId?.trim()?.takeIf { it.isNotEmpty() }
            if (!normalizedWifi.isNullOrBlank()) {
                notes = appendRoutingHint(notes = notes, key = "wifi_peer_id", value = normalizedWifi)
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
                notes = notes,
                lastKnownDeviceId = deviceId?.trim()?.takeIf { it.isNotEmpty() } ?: existing?.lastKnownDeviceId
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

    @Synchronized
    private fun promotePendingOutboundForPeer(peerId: String, excludingMessageId: String? = null) {
        val trimmedPeerId = peerId.trim()
        if (trimmedPeerId.isEmpty()) return
        val now = System.currentTimeMillis() / 1000
        val queue = loadPendingOutbox().toMutableList()
        var changed = false
        for (idx in queue.indices) {
            val item = queue[idx]
            val routePeerId = item.routePeerId?.trim()
            if (item.peerId != trimmedPeerId && routePeerId != trimmedPeerId) continue
            if (!excludingMessageId.isNullOrBlank() && item.historyRecordId == excludingMessageId) continue
            if (item.terminalFailureCode != null) continue
            if (item.nextAttemptAtEpochSec <= now) continue
            queue[idx] = item.copy(nextAttemptAtEpochSec = now)
            changed = true
        }
        if (!changed) return
        savePendingOutbox(queue)
        logDeliveryState(
            messageId = excludingMessageId ?: "unknown",
            state = "forwarding",
            detail = "peer_queue_promoted peer=$trimmedPeerId"
        )
    }

    private fun isMessageDeliveredLocally(messageId: String): Boolean {
        pruneDeliveredReceiptCache()
        if (deliveredReceiptCache.containsKey(messageId)) {
            return true
        }
        return try {
            historyManager?.get(messageId)?.delivered == true
        } catch (_: Exception) {
            false
        }
    }

    @Synchronized
    private fun markDeliveredReceiptSeen(messageId: String): Boolean {
        pruneDeliveredReceiptCache()
        val now = System.currentTimeMillis()
        return deliveredReceiptCache.putIfAbsent(messageId, now) == null
    }

    private fun pruneDeliveredReceiptCache(nowMs: Long = System.currentTimeMillis()) {
        deliveredReceiptCache.forEach { (messageId, seenAtMs) ->
            if (nowMs - seenAtMs > deliveredReceiptCacheTtlMs) {
                deliveredReceiptCache.remove(messageId, seenAtMs)
            }
        }
        if (deliveredReceiptCache.size <= 2048) {
            return
        }
        val keep = deliveredReceiptCache.entries
            .sortedByDescending { it.value }
            .take(1024)
            .associate { it.key to it.value }
        deliveredReceiptCache.clear()
        deliveredReceiptCache.putAll(keep)
    }

    private fun parseRoutingHints(notes: String?): RoutingHints {
        if (notes.isNullOrEmpty()) {
            return RoutingHints(
                wifiPeerId = null,
                blePeerId = null,
                libp2pPeerId = null,
                listeners = emptyList()
            )
        }

        var wifiPeerId: String? = null
        var blePeerId: String? = null
        var peerId: String? = null
        var listeners: List<String> = emptyList()

        for (component in notes.split(';', '\n')) {
            val kv = component.trim()
            if (kv.startsWith("wifi_peer_id:")) {
                val value = kv.removePrefix("wifi_peer_id:").trim()
                wifiPeerId = value.ifEmpty { null }
            } else if (kv.startsWith("ble_peer_id:")) {
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
            wifiPeerId = wifiPeerId,
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
            if (value.isNotEmpty() && PeerIdValidator.isLibp2pPeerId(value)) {
                out.add(value)
            }
        }
        return out.distinct()
    }

    private fun buildRoutePeerCandidates(
        peerId: String,
        cachedRoutePeerId: String?,
        notes: String?,
        recipientPublicKey: String? = null
    ): List<String> {
        val candidates = mutableListOf<String>()
        candidates.addAll(discoverRoutePeersForPublicKey(recipientPublicKey))
        val notedPeerIds = parseAllRoutingPeerIds(notes)
        val newestHint = notedPeerIds.lastOrNull()
        if (!newestHint.isNullOrBlank()) candidates.add(newestHint)
        for (hint in notedPeerIds.asReversed()) candidates.add(hint)
        cachedRoutePeerId?.trim()?.takeIf { it.isNotEmpty() }?.let { candidates.add(it) }
        if (PeerIdValidator.isLibp2pPeerId(peerId)) candidates.add(peerId)
        return candidates
            .map { it.trim() }
            .filter { candidate ->
                candidate.isNotEmpty() &&
                    PeerIdValidator.isLibp2pPeerId(candidate) &&
                    routeCandidateMatchesRecipient(candidate, recipientPublicKey)
            }
            .distinct()
    }

    private fun discoverRoutePeersForPublicKey(recipientPublicKey: String?): List<String> {
        val normalizedRecipientKey = normalizePublicKey(recipientPublicKey) ?: return emptyList()
        val fromDiscovery = _discoveredPeers.value.values
            .asSequence()
            .filter { info ->
                normalizePublicKey(info.publicKey) == normalizedRecipientKey
            }
            .mapNotNull { info ->
                // Prefer libp2pPeerId for routing if available
                val routeId = info.libp2pPeerId?.trim()?.takeIf { 
                    it.isNotEmpty() && PeerIdValidator.isLibp2pPeerId(it)
                } ?: info.peerId.trim().takeIf { 
                    it.isNotEmpty() && PeerIdValidator.isLibp2pPeerId(it) 
                }
                routeId
            }
            .toList()

        val fromLedger = (ledgerManager?.dialableAddresses() ?: emptyList())
            .asSequence()
            .mapNotNull { entry ->
                val candidate = entry.peerId?.trim().orEmpty()
                if (candidate.isEmpty() || !PeerIdValidator.isLibp2pPeerId(candidate)) {
                    return@mapNotNull null
                }
                val candidateKey = normalizePublicKey(entry.publicKey) ?: return@mapNotNull null
                if (candidateKey != normalizedRecipientKey) return@mapNotNull null
                candidate
            }
            .toList()

        return (fromDiscovery + fromLedger).distinct()
    }

    private fun routeCandidateMatchesRecipient(
        routePeerId: String,
        recipientPublicKey: String?
    ): Boolean {
        val normalizedRoute = routePeerId.trim()
        if (normalizedRoute.isEmpty() || !PeerIdValidator.isLibp2pPeerId(normalizedRoute)) return false
        if (isKnownRelay(normalizedRoute)) return false

        val normalizedRecipientKey = normalizePublicKey(recipientPublicKey) ?: return true
        val extractedKey = try {
            ironCore?.extractPublicKeyFromPeerId(normalizedRoute)
        } catch (_: Exception) {
            null
        }
        val normalizedExtractedKey = normalizePublicKey(extractedKey)
        if (normalizedExtractedKey != null) {
            return normalizedExtractedKey == normalizedRecipientKey
        }

        val discoveryMatch = _discoveredPeers.value.entries.any { (key, info) ->
            (key == normalizedRoute || info.peerId == normalizedRoute) &&
                normalizePublicKey(info.publicKey) == normalizedRecipientKey
        }
        if (discoveryMatch) return true

        val ledgerMatch = (ledgerManager?.dialableAddresses() ?: emptyList()).any { entry ->
            entry.peerId?.trim() == normalizedRoute &&
                normalizePublicKey(entry.publicKey) == normalizedRecipientKey
        }
        return ledgerMatch
    }

    // Identity validation logic centralized in PeerIdValidator

    private fun buildDialCandidatesForPeer(
        routePeerId: String?,
        rawAddresses: List<String>,
        includeRelayCircuits: Boolean
    ): List<String> {
        val normalized = rawAddresses
            .mapNotNull { normalizeAddressHint(it) }
            .distinct()
        val prioritized = prioritizeAddressesForCurrentNetwork(normalized)
        val relayCircuits = if (includeRelayCircuits && !routePeerId.isNullOrBlank() && PeerIdValidator.isLibp2pPeerId(routePeerId)) {
            relayCircuitAddressesForPeer(routePeerId)
        } else {
            emptyList()
        }
        // Cap at 6 candidates to avoid excessive dialing.
        // Priority: LAN addresses first (from prioritized), then relay circuits,
        // then remaining public addresses.
        return (prioritized + relayCircuits).distinct().take(6)
    }

    fun getDialHintsForRoutePeer(routePeerId: String): List<String> {
        if (!PeerIdValidator.isLibp2pPeerId(routePeerId)) return emptyList()
        val fromLedger = (ledgerManager?.dialableAddresses() ?: emptyList())
            .filter { it.peerId == routePeerId }
            .map { it.multiaddr }
        return buildDialCandidatesForPeer(
            routePeerId = routePeerId,
            rawAddresses = fromLedger,
            includeRelayCircuits = false // Avoid infinite recursion
        )
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
        if (isSpecialUseIpv4(ip)) return false

        return if (isPrivateIpv4(ip)) {
            isSameLanAddress(multiaddr)
        } else {
            true
        }
    }

    private fun parseIpv4Octets(ip: String): List<Int>? {
        val octets = ip.split('.').mapNotNull { it.toIntOrNull() }
        if (octets.size != 4) return null
        if (octets.any { it !in 0..255 }) return null
        return octets
    }

    private fun isPrivateIpv4(ip: String): Boolean {
        val octets = parseIpv4Octets(ip) ?: return false
        return (octets[0] == 10) ||
            (octets[0] == 172 && octets[1] in 16..31) ||
            (octets[0] == 192 && octets[1] == 168)
    }

    private fun isSpecialUseIpv4(ip: String): Boolean {
        val octets = parseIpv4Octets(ip) ?: return true
        val o0 = octets[0]
        val o1 = octets[1]
        val o2 = octets[2]

        if (o0 == 0 || o0 == 127) return true
        if (o0 == 169 && o1 == 254) return true
        if (o0 == 100 && o1 in 64..127) return true // RFC6598 CGNAT
        if (o0 == 192 && o1 == 0 && (o2 == 0 || o2 == 2)) return true
        if (o0 == 198 && ((o1 == 18) || (o1 == 19))) return true // Benchmark network
        if (o0 == 198 && o1 == 51 && o2 == 100) return true
        if (o0 == 203 && o1 == 0 && o2 == 113) return true
        if (o0 >= 224) return true // multicast/reserved/broadcast
        return false
    }

    fun isKnownRelay(peerId: String): Boolean {
        val normalized = peerId.trim()
        if (isBootstrapRelayPeer(normalized)) return true
        val info = _discoveredPeers.value.entries.firstOrNull {
            it.key.equals(normalized, ignoreCase = true)
        }?.value ?: return false
        return info.isRelay && !info.isFull
    }

    private fun relayCircuitAddressesForPeer(targetPeerId: String): List<String> {
        if (!PeerIdValidator.isLibp2pPeerId(targetPeerId)) return emptyList()
        val circuits = mutableListOf<String>()

        // 1. Static Bootstrap Relays (prioritized by network type)
        val prioritizedNodes = if (networkDetector.isCellularNetwork) {
            WEBSOCKET_FALLBACK_NODES + DEFAULT_BOOTSTRAP_NODES
        } else {
            DEFAULT_BOOTSTRAP_NODES + WEBSOCKET_FALLBACK_NODES
        }

        prioritizedNodes.forEach { bootstrap ->
            val relayInfo = parseBootstrapRelay(bootstrap)
            if (relayInfo != null) {
                val (relayTransportAddr, relayPeerId) = relayInfo
                // Skip circuit addresses for relays with open circuit breakers
                if (relayCircuitBreaker.isCircuitOpen(bootstrap)) return@forEach
                circuits.add("$relayTransportAddr/p2p/$relayPeerId/p2p-circuit/p2p/$targetPeerId")
            }
        }

        // 2. Dynamic Discovered Relays
        _discoveredPeers.value.entries.filter {
            it.value.isRelay && !it.value.isFull && it.key != targetPeerId
        }.forEach { entry ->
            val relayPeerId = entry.key
            if (PeerIdValidator.isLibp2pPeerId(relayPeerId)) {
                val directAddrs = getDialHintsForRoutePeer(relayPeerId)
                directAddrs.forEach { addr ->
                    val circuit = "$addr/p2p/$relayPeerId/p2p-circuit/p2p/$targetPeerId"
                    if (!circuits.contains(circuit)) circuits.add(circuit)
                }
            }
        }

        return circuits
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

    fun isBootstrapRelayPeer(peerId: String): Boolean {
        if (peerId.isBlank()) return false
        return DEFAULT_BOOTSTRAP_NODES.any { addr ->
            parseBootstrapRelay(addr)?.second == peerId
        }
    }

    /**
     * Check if a public key corresponds to a bootstrap relay peer.
     * This is used when resolving transport identities to determine if
     * we should create an emergency contact or treat as a transient relay.
     */
    fun isBootstrapRelayPeerFromKey(normalizedKey: String): Boolean {
        if (normalizedKey.isBlank()) return false
        // First check if we can extract a peer ID from the key
        val peerId = PeerKeyUtils.extractPeerIdFromPublicKey(normalizedKey)
        return isBootstrapRelayPeer(peerId)
    }

    /**
     * P0_ANDROID_004: Create an emergency contact for a newly discovered peer.
     *
     * This function is called when a peer is identified but no existing contact
     * is found in the database. It creates a minimal contact entry that can be
     * later enriched through federated updates.
     *
     * @param publicKey The normalized public key of the peer
     * @return The newly created Contact
     */
    private fun createEmergencyContact(publicKey: String): uniffi.api.Contact {
        // Extract peer ID from public key, or generate a fallback
        val peerId = PeerKeyUtils.extractPeerIdFromPublicKey(publicKey)
        val normalizedPeerId = PeerIdValidator.normalize(peerId)

        // Create minimal contact with discovered state
        val contact = uniffi.api.Contact(
            peerId = normalizedPeerId,
            nickname = "Unknown Peer",
            localNickname = null,
            publicKey = publicKey,
            addedAt = (System.currentTimeMillis() / 1000).toULong(),
            lastSeen = (System.currentTimeMillis() / 1000).toULong(),
            notes = "Emergency contact created during peer discovery",
            lastKnownDeviceId = null
        )

        // Save to database
        try {
            contactManager?.add(contact)
            Timber.i("Emergency contact created for peer: ${normalizedPeerId.take(8)}... (key: ${publicKey.take(8)}...)")
        } catch (e: Exception) {
            Timber.e("Failed to create emergency contact: ${e.message}")
            // Continue anyway - contact may already exist
        }

        return contact
    }

    /**
     * P0_ANDROID_004: Validate peer before contact creation.
     *
     * Comprehensive peer validation to ensure we only create contacts for
     * valid, non-relay peers.
     *
     * @param peerId The peer ID to validate
     * @param publicKey The public key to validate
     * @return true if the peer is valid for contact creation
     */
    private fun validatePeerBeforeContactCreation(peerId: String, publicKey: String): Boolean {
        return when {
            peerId.isBlank() -> {
                Timber.w("Peer validation failed: blank peer ID")
                false
            }
            publicKey.isBlank() -> {
                Timber.w("Peer validation failed: blank public key")
                false
            }
            !PeerKeyUtils.isValidPeerId(peerId) -> {
                Timber.w("Peer validation failed: invalid peer ID format: $peerId")
                false
            }
            !PeerKeyUtils.isValidPublicKey(publicKey) -> {
                Timber.w("Peer validation failed: invalid public key format: ${publicKey.take(8)}...")
                false
            }
            isBootstrapRelayPeer(peerId) -> {
                Timber.d("Peer validation: skipping relay peer")
                false
            }
            else -> true
        }
    }

    /**
     * P0_ANDROID_004: Enhanced logging for identity resolution.
     *
     * Logs detailed information about identity resolution for debugging
     * and monitoring purposes.
     */
    private fun logIdentityResolutionDetails(
        normalizedKey: String,
        canonicalContact: uniffi.api.Contact?,
        createdNew: Boolean
    ) {
        Timber.d(
            "Identity resolution: key=${normalizedKey.take(8)}..., " +
                "contact=${canonicalContact?.peerId?.take(8)}..., " +
                "createdNew=$createdNew"
        )
    }

    /**
     * P0_NETWORK_001: Bootstrap relay connections with circuit breaker and
     * WebSocket fallback for cellular networks.
     *
     * Priority order respects network type:
     * - WiFi/Ethernet: QUIC → TCP → WebSocket (full connectivity)
     * - Cellular: WebSocket(443) → TCP(443) → QUIC → TCP → WebSocket(80)
     */
    private suspend fun primeRelayBootstrapConnections() {
        val bridge = swarmBridge ?: return
        val nowMs = System.currentTimeMillis()

        // P1_ANDROID_013: Respect exponential backoff from consecutive failures
        if (nowMs < nextBootstrapAttemptMs) {
            return
        }
        if (nowMs - lastRelayBootstrapDialMs < 10_000L) return
        lastRelayBootstrapDialMs = nowMs

        // Determine transport priority based on current network
        val transportPriority = networkDetector.getTransportPriority()
        val isCellular = networkDetector.isCellularNetwork
        Timber.i("Bootstrap: network=%s, cellular=%b, priority=%s",
            networkDetector.networkType.value, isCellular, transportPriority)

        // Build address list: primary nodes + WebSocket fallback if cellular
        val addresses = if (isCellular) {
            // On cellular, prioritize WebSocket on standard ports first
            WEBSOCKET_FALLBACK_NODES + DEFAULT_BOOTSTRAP_NODES
        } else {
            DEFAULT_BOOTSTRAP_NODES + WEBSOCKET_FALLBACK_NODES
        }

        var anySuccess = false
        for (addr in addresses) {
            try {
                // Check circuit breaker before attempting
                if (!relayCircuitBreaker.allowRequest(addr)) {
                    Timber.d("Circuit breaker blocked %s, skipping", addr)
                    continue
                }
                if (!shouldAttemptDial(addr)) continue
                bridge.dial(addr)
                Timber.d("Bootstrap dial initiated: %s", addr)
                anySuccess = true
            } catch (e: Exception) {
                // P1_ANDROID_013: Record failure metrics directly without triggering
                // enhanceNetworkErrorLogging for each failure. The fallback protocol
                // is now handled by the racing bootstrap, not per-dial error logging.
                val errorDetail = classifyBootstrapError(e, addr)
                Timber.w("Bootstrap dial failed for $addr - $errorDetail")
                relayCircuitBreaker.recordFailure(addr, errorDetail)
                networkFailureMetrics.recordFailure(addr, errorDetail, e)
            }
        }

        // P1_ANDROID_013: Update consecutive failure tracking and backoff
        if (!anySuccess) {
            consecutiveBootstrapFailures++
            val backoffMs = when {
                consecutiveBootstrapFailures <= 1 -> 10_000L
                consecutiveBootstrapFailures <= 3 -> 30_000L
                else -> 60_000L.coerceAtMost(1000L * (1L shl consecutiveBootstrapFailures.coerceAtMost(6)))
            }
            nextBootstrapAttemptMs = nowMs + backoffMs
            Timber.w("Bootstrap all-failed (consecutive=%d), next attempt in %dms",
                consecutiveBootstrapFailures, backoffMs)
        } else {
            consecutiveBootstrapFailures = 0
            nextBootstrapAttemptMs = 0L
        }
    }

    /** @deprecated Use the suspend version with circuit breaker support */
    private fun primeRelayBootstrapConnectionsLegacy() {
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

    /**
     * P0_NETWORK_001: Racing bootstrap with fallback strategy.
     *
     * Races all transport multiaddrs for each relay in parallel within a
     * 500ms connectivity window. On cellular networks, WebSocket on standard
     * ports (80/443) is prioritized since carriers commonly block non-standard
     * ports like 9001/9010. Circuit breakers gate each address before dialing.
     * If all relay bootstrap fails, falls back to mDNS local discovery.
     */
    internal suspend fun racingBootstrapWithFallback(): BootstrapResult {
        val transportPriority = networkDetector.getTransportPriority()
        val diagnostics = networkDetector.getNetworkDiagnostics()
        Timber.i("Racing bootstrap: network=%s, transports=%s", diagnostics.networkType,
            transportPriority.joinToString("→") { it.scheme })

        // Reset circuit breakers on network change
        relayCircuitBreaker.resetAll()

        // Build prioritized address list based on network type
        // P0_ANDROID_007: Use resolveAllBootstrapSources() to try environment, remote, then static fallbacks
        val baseNodes = resolveAllBootstrapSourcesAsync() + WEBSOCKET_FALLBACK_NODES
        val prioritizedAddresses = if (networkDetector.isCellularNetwork) {
            // On cellular, WebSocket on standard ports first (carriers block non-standard ports)
            WEBSOCKET_FALLBACK_NODES + baseNodes.filter { it !in WEBSOCKET_FALLBACK_NODES }
        } else {
            baseNodes
        }

        // Proactively probe known relay ports to deprioritize blocked addresses
        val probeTargets = listOf(
            "34.135.34.73" to 9001, "34.135.34.73" to 443,
            "104.28.216.43" to 9010, "104.28.216.43" to 443
        )
        val portProbeResults = networkDetector.probePorts(probeTargets)

        // Filter out circuit-breaker-blocked and throttle-blocked addresses,
        // and deprioritize addresses whose host:port is confirmed blocked
        val candidateAddresses = prioritizedAddresses.filter { addr ->
            relayCircuitBreaker.allowRequest(addr) && shouldAttemptDial(addr)
        }.sortedByDescending { addr ->
            // Boost priority for addresses whose ports are confirmed reachable
            val port = extractPortFromMultiaddr(addr)
            val host = Regex("""/ip4/([^/]+)|/dns4/([^/]+)""").find(addr)?.groupValues?.let {
                if (it[1].isNotEmpty()) it[1] else (it[2] ?: "")
            } ?: ""
            val key = "$host:$port"
            portProbeResults[key] != false // true or null (unprobed) → high priority; false → low
        }

        if (candidateAddresses.isEmpty()) {
            Timber.w("No candidate addresses available (all circuit-breaker-blocked or throttled)")
            return attemptMdnsFallback()
        }

        // Race all candidate addresses in parallel with 3s timeout
        // (Individual dials may take longer; first success wins, others are cancelled)
        val result = kotlinx.coroutines.withTimeoutOrNull(3_000L) {
            kotlinx.coroutines.coroutineScope {
                val deferreds = candidateAddresses.map { addr ->
                    async(Dispatchers.IO) {
                        try {
                            val bridge = swarmBridge ?: return@async BootstrapAttempt.Failure(addr, "no bridge")
                            bridge.dial(addr)
                            relayCircuitBreaker.recordSuccess(addr)
                            Timber.i("Bootstrap connected: %s", addr)
                            BootstrapAttempt.Success(addr)
                        } catch (e: Exception) {
                            // P1_ANDROID_013: Record failure metrics directly without triggering
                            // enhanceNetworkErrorLogging, which would cascade into fallback protocol
                            val detail = classifyBootstrapError(e, addr)
                            relayCircuitBreaker.recordFailure(addr, detail)
                            networkFailureMetrics.recordFailure(addr, detail, e)
                            Timber.d("Bootstrap race attempt failed for $addr: $detail")
                            BootstrapAttempt.Failure(addr, e.message ?: "unknown")
                        }
                    }
                }

                // Take first success, or all failures
                for (deferred in deferreds) {
                    val attempt = deferred.await()
                    if (attempt is BootstrapAttempt.Success) {
                        // Cancel remaining coroutines
                        deferreds.forEach { if (!it.isCompleted) it.cancel() }
                        return@coroutineScope attempt
                    }
                }
                BootstrapAttempt.Failure("all", "all bootstrap attempts failed")
            }
        }

        return when (result) {
            is BootstrapAttempt.Success -> BootstrapResult.Connected(result.addr, transportPriority.firstOrNull())
            else -> {
                Timber.w("All bootstrap addresses failed or timed out, falling back to mDNS")
                attemptMdnsFallback()
            }
        }
    }

    /**
     * P0_NETWORK_001: mDNS fallback when all relay bootstrap fails.
     */
    private suspend fun attemptMdnsFallback(): BootstrapResult {
        Timber.i("All relay bootstrap failed, attempting mDNS local discovery")
        // mDNS discovery is handled by MdnsServiceDiscovery in the transport layer.
        // The service should already be running; wait up to 5s for a peer to resolve.
        val mdnsResult = kotlinx.coroutines.withTimeoutOrNull(5_000L) {
            // Wait for the mDNS service to discover a LAN peer
            // The onLanPeerResolved callback will trigger a SwarmBridge dial
            // via the TransportManager, so we just need to confirm connectivity
            var checks = 0
            while (checks < 10) {
                val peerCount = meshService?.getStats()?.peersDiscovered?.toInt() ?: 0
                if (peerCount > 0) {
                    Timber.i("mDNS fallback: connected to %d LAN peer(s)", peerCount)
                    return@withTimeoutOrNull BootstrapResult.MdnsFallback("lan-peer")
                }
                kotlinx.coroutines.delay(500)
                checks++
            }
            null
        }

        return if (mdnsResult != null) {
            mdnsResult
        } else {
            Timber.e("mDNS fallback: no LAN peers discovered within timeout")
            BootstrapResult.AllRelaysFailed
        }
    }

    /** Racing bootstrap attempt result */
    private sealed class BootstrapAttempt {
        data class Success(val addr: String) : BootstrapAttempt()
        data class Failure(val addr: String, val reason: String) : BootstrapAttempt()
    }

    /** Public bootstrap result for callers */
    sealed class BootstrapResult {
        data class Connected(val multiaddr: String, val transport: com.scmessenger.android.transport.FallbackTransport?) : BootstrapResult()
        data class MdnsFallback(val peerId: String) : BootstrapResult()
        data object AllRelaysFailed : BootstrapResult()
    }

    /**
     * Legacy sequential bootstrap — kept as fallback if racing is unavailable.
     * Delegates to racingBootstrapWithFallback() which subsumes this logic.
     */
    internal suspend fun bootstrapWithFallbackStrategy() {
        when (val result = racingBootstrapWithFallback()) {
            is BootstrapResult.Connected -> {
                Timber.i("Bootstrap succeeded: %s (transport: %s)", result.multiaddr, result.transport)
            }
            is BootstrapResult.MdnsFallback -> {
                Timber.i("Bootstrap fell back to mDNS: %s", result.peerId)
            }
            is BootstrapResult.AllRelaysFailed -> {
                Timber.e("All bootstrap methods failed (relay + mDNS)")
            }
        }
    }

    // P0_NETWORK_001: Watch for network type changes to trigger re-bootstrap
    private var networkWatchJob: kotlinx.coroutines.Job? = null

    private fun startNetworkChangeWatch() {
        networkWatchJob?.cancel()
        networkWatchJob = repoScope.launch {
            var previousType = networkDetector.networkType.value
            networkDetector.networkType.collect { newType ->
                if (newType != previousType && newType != com.scmessenger.android.transport.NetworkType.UNKNOWN) {
                    Timber.i("Network type changed: %s → %s, resetting circuit breakers and re-bootstrapping",
                        previousType, newType)
                    previousType = newType
                    relayCircuitBreaker.resetAll()
                    racingBootstrapWithFallback()
                }
            }
        }
    }

    private fun stopNetworkChangeWatch() {
        networkWatchJob?.cancel()
        networkWatchJob = null
    }

    /**
     * P0_NETWORK_001 / P0_ANDROID_007: Classify bootstrap connection errors for diagnostics.
     * Records failure metrics for adaptive routing and user-facing diagnostics.
     */
    private fun classifyBootstrapError(exception: Exception, nodeAddr: String? = null): String {
        val port = nodeAddr?.let { extractPortFromMultiaddr(it) }

        // P1_ANDROID_013: Distinguish "device offline" from "server unreachable" from "timeout"
        val isDeviceOffline = networkDetector.networkType.value == com.scmessenger.android.transport.NetworkType.UNKNOWN
        val detail = when {
            isDeviceOffline -> "Device offline — no active network"
            exception is java.net.UnknownHostException ->
                "DNS resolution failed for ${exception.message ?: "unknown host"}"
            exception is java.net.ConnectException -> {
                if (networkDetector.isCellularNetwork && port != null && port !in setOf(80, 443)) {
                    "Port $port blocked on cellular — carrier filtering non-standard ports"
                } else {
                    "Connection refused — port blocked or service down"
                }
            }
            exception is java.net.SocketTimeoutException -> "Connection timeout after dial period"
            exception is javax.net.ssl.SSLException -> "TLS handshake failed: ${exception.message}"
            exception is java.net.PortUnreachableException -> {
                if (networkDetector.isCellularNetwork) {
                    "UDP port unreachable on cellular — carrier blocking QUIC/UDP, try WebSocket"
                } else {
                    "Port unreachable (likely firewall-blocked)"
                }
            }
            exception is java.net.NoRouteToHostException -> "No route to host (network unreachable)"
            exception is java.net.SocketException -> "Socket error: ${exception.message}"
            exception is java.io.IOException -> "Network I/O error: ${exception.message}"
            else -> "Unknown network error: ${exception.javaClass.simpleName}: ${exception.message}"
        }
        // Record failure metrics for diagnostics reporting
        if (nodeAddr != null) {
            networkFailureMetrics.recordFailure(nodeAddr, detail, exception)
        }
        return detail
    }

    /** Extract port number from a libp2p multiaddr string. */
    private fun extractPortFromMultiaddr(multiaddr: String): Int? {
        // e.g. "/ip4/34.135.34.73/tcp/9001/p2p/..." → 9001
        // e.g. "/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/..." → 443
        val portRegex = Regex("""/tcp/(\d+)""").find(multiaddr)
            ?: Regex("""/udp/(\d+)""").find(multiaddr)
        return portRegex?.groupValues?.get(1)?.toIntOrNull()
    }

    private fun shouldAttemptDial(multiaddr: String): Boolean {
        val key = multiaddr.trim()
        if (key.isEmpty()) return false

        val now = System.currentTimeMillis()
        val (attempts, nextAllowedMs) = dialThrottleState[key] ?: (0 to 0L)
        if (now < nextAllowedMs) {
            // P4: Only log dial throttle once per address per 5-minute window
            val lastLogged = dialThrottleLogCache[key]
            if (lastLogged == null || (now - lastLogged) >= dialThrottleLogIntervalMs) {
                dialThrottleLogCache[key] = now
            }
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

    /** P0_ANDROID_007: Get network failure metrics summary for diagnostics UI */
    fun getNetworkFailureSummary(): com.scmessenger.android.utils.NetworkFailureMetrics.Summary {
        return networkFailureMetrics.getSummary()
    }

    /** P0_ANDROID_007: Get network detector diagnostics */
    fun getNetworkDiagnosticsSnapshot(): com.scmessenger.android.transport.NetworkDiagnostics {
        return networkDetector.getNetworkDiagnostics()
    }

    private fun startStorageMaintenance() {
        if (maintenanceJob?.isActive == true) return

        maintenanceJob = repoScope.launch {
            while (isActive) {
                try {
                    val stat = android.os.StatFs(context.filesDir.path)
                    val total = stat.blockCountLong * stat.blockSizeLong
                    val free = stat.availableBlocksLong * stat.blockSizeLong

                    ironCore?.updateDiskStats(total.toULong(), free.toULong())
                    ironCore?.performMaintenance()

                    // P0_ANDROID_005: Periodic message tracking health check
                    detectAndRecoverMessageTracking()
                    logRetryStormDetection()

                    // P0_ANDROID_003: Periodic BLE transport health check
                    handleBleTransportDegradation()

                    Timber.d("Storage maintenance check: free=${free / 1024 / 1024}MB / total=${total / 1024 / 1024}MB")
                } catch (e: Exception) {
                    Timber.w("Storage maintenance loop error: ${e.message}")
                }
                kotlinx.coroutines.delay(15 * 60 * 1000) // Every 15 minutes
            }
        }
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
            val interfaces = java.net.NetworkInterface.getNetworkInterfaces() ?: return null
            var bestIp: String? = null
            var bestScore = Int.MIN_VALUE

            while (interfaces.hasMoreElements()) {
                val networkInterface = interfaces.nextElement()
                if (!networkInterface.isUp || networkInterface.isLoopback || networkInterface.isVirtual) {
                    continue
                }

                val ifaceName = networkInterface.name?.lowercase().orEmpty()
                val addresses = networkInterface.inetAddresses
                while (addresses.hasMoreElements()) {
                    val address = addresses.nextElement()
                    if (address !is java.net.Inet4Address || address.isLoopbackAddress || address.isLinkLocalAddress) {
                        continue
                    }

                    val ip = address.hostAddress?.trim().orEmpty()
                    if (ip.isEmpty() || isSpecialUseIpv4(ip)) continue

                    val isPrivate = isPrivateIpv4(ip)
                    val ifaceScore = when {
                        ifaceName.startsWith("wlan") || ifaceName.startsWith("wifi") || ifaceName == "en0" -> 3
                        ifaceName.startsWith("eth") || ifaceName.startsWith("en") -> 2
                        else -> 1
                    }
                    val score = (if (isPrivate) 100 else 10) + ifaceScore
                    if (score > bestScore) {
                        bestScore = score
                        bestIp = ip
                    }
                }
            }
            return bestIp
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

        // UNIFIED ID FIX: QR code primary ID is libp2p_peer_id (network routable)
        // identity_id is secondary (human fingerprint for backup/recovery)
        val payload = org.json.JSONObject()
            .put("peer_id", identity.libp2pPeerId ?: "")           // PRIMARY: libp2p Peer ID for contact add
            .put("public_key", identity.publicKeyHex ?: "")         // Canonical identity key
            .put("device_id", identity.deviceId ?: "")              // Multi-device routing
            .put("identity_id", identity.identityId ?: "")           // SECONDARY: Blake3 hash
            .put("nickname", identity.nickname ?: "")
            .put("libp2p_peer_id", identity.libp2pPeerId ?: "")      // Backward compatibility
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
            pendingReceiptSendJobs.values.forEach { it.cancel() }
            pendingReceiptSendJobs.clear()

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

    // ========================================================================
    // DIAGNOSTICS
    // ========================================================================

    /**
     * Get path to the diagnostics log file.
     */
    fun getDiagnosticsLogPath(): String {
        return File(context.filesDir, "mesh_diagnostics.log").absolutePath
    }

    /**
     * Get recent diagnostics logs.
     */
    fun getDiagnosticsLogs(limit: Int = 500): String {
        return try {
            val logFile = File(getDiagnosticsLogPath())
            if (!logFile.exists()) return "No diagnostics recorded yet."

            val lines = logFile.readLines()
            if (lines.isEmpty()) return "No diagnostics recorded yet."

            lines.takeLast(limit).joinToString("\n")
        } catch (e: Exception) {
            "Error reading diagnostics: ${e.message}"
        }
    }

    /**
     * Clear all diagnostics logs.
     */
    fun clearDiagnosticsLogs() {
        try {
            val logFile = File(getDiagnosticsLogPath())
            if (logFile.exists()) {
                logFile.writeText("")
                Timber.i("Diagnostics logs cleared")
            }
        } catch (e: Exception) {
            Timber.e(e, "Error clearing diagnostics")
        }
    }

    private fun logDeliveryState(messageId: String, state: String, detail: String) {
        if (messageId.isBlank()) return
        Timber.i("delivery_state msg=$messageId state=$state detail=$detail")
    }

    private fun logDeliveryAttempt(
        messageId: String?,
        medium: String,
        phase: String,
        outcome: String,
        detail: String,
        callerContext: String? = null
    ) {
        val msg = messageId?.takeIf { it.isNotBlank() }
        if (msg == null) {
            val contextInfo = callerContext?.takeIf { it.isNotBlank() } ?: "unknown_caller"
            Timber.w("delivery_attempt msg=unknown (messageId was null or blank) medium=%s phase=%s outcome=%s detail=%s caller=%s",
                medium, phase, outcome, detail, contextInfo)
            // AND-DELIVERY-001: Add stack trace for debugging message ID loss (non-blocking warning only)
            if (medium != "receipt" && phase != "rx") {  // Skip for expected receipt cases
                Timber.w("Stack trace for msg=unknown (non-blocking warning)")
            }
        }
        Timber.i(
            "delivery_attempt msg=%s medium=%s phase=%s outcome=%s detail=%s",
            msg ?: "unknown",
            medium,
            phase,
            outcome,
            detail
        )
    }
    private fun migrateToCanonicalIds() {
        val iron = ironCore ?: return
        val history = historyManager ?: return
        val contacts = contactManager ?: return

        try {
            val prefs = context.getSharedPreferences("mesh_migrations", android.content.Context.MODE_PRIVATE)
            if (prefs.getBoolean("v2_id_coalescence", false)) return

            Timber.i("Starting ID Coalescence Migration...")

            // 1. Migrate Contacts
            val contactList = contacts.list()
            val idMap = mutableMapOf<String, String>() // old -> new

            for (contact in contactList) {
                val identityId = try { iron.resolveIdentity(contact.publicKey) } catch (e: Exception) { null }
                if (identityId != null && identityId != contact.peerId) {
                    Timber.i("Coalescing ID for ${contact.nickname ?: contact.peerId}: ${contact.peerId} -> $identityId")
                    idMap[contact.peerId] = identityId

                    val existingCanonical = try { contacts.get(identityId) } catch (e: Exception) { null }
                    if (existingCanonical == null) {
                        // No existing canonical contact - create one with the identity ID
                        try {
                            contacts.add(contact.copy(peerId = identityId))
                        } catch (e: Exception) {
                            Timber.w("Failed to create canonical contact $identityId")
                        }
                    } else {
                        // Canonical contact already exists - merge data from old contact
                        val merged = existingCanonical.copy(
                            nickname = existingCanonical.nickname ?: contact.nickname,
                            localNickname = existingCanonical.localNickname ?: contact.localNickname,
                            lastSeen = maxOf(existingCanonical.lastSeen ?: 0u, contact.lastSeen ?: 0u),
                            notes = mergeNotes(existingCanonical.notes, contact.notes),
                            lastKnownDeviceId = existingCanonical.lastKnownDeviceId ?: contact.lastKnownDeviceId
                        )
                        if (merged != existingCanonical) {
                            try {
                                contacts.add(merged)
                                Timber.i("Merged contact data from ${contact.peerId} into $identityId")
                            } catch (e: Exception) {
                                Timber.w("Failed to merge contact data: ${e.message}")
                            }
                        }
                    }
                    // Only remove old contact after successful merge/creation
                    try { contacts.remove(contact.peerId) } catch (e: Exception) { }
                }
            }

            // 2. Migrate History
            val stats = history.stats()
            if (stats.totalMessages > 0u) {
                val allMessages = history.recent(null, 100000u)
                var updatedMessages = 0
                for (msg in allMessages) {
                    val canonical = idMap[msg.peerId] ?: try { iron.resolveIdentity(msg.peerId) } catch (e: Exception) { null }
                    if (canonical != null && canonical != msg.peerId) {
                        try {
                            history.add(msg.copy(peerId = canonical))
                            updatedMessages++
                        } catch (e: Exception) {
                            Timber.w("Failed to update message peer_id for ${msg.id}")
                        }
                    }
                }
                if (updatedMessages > 0) {
                    Timber.i("Migrated $updatedMessages messages to canonical peer IDs")
                }
            }

            history.flush()
            contacts.flush()
            prefs.edit().putBoolean("v2_id_coalescence", true).apply()
            Timber.i("ID Coalescence Migration completed successfully")
        } catch (e: Exception) {
            Timber.e(e, "ID Coalescence Migration failed")
        }
    }

    // ============================================================================
    // Emergency Contact Recovery (P0_ANDROID_001)
    // ============================================================================

    /**
     * Emergency contact reconstruction from message history.
     * Reconstructs contacts when the contacts database appears corrupted (0 contacts but messages exist).
     */
    private suspend fun emergencyContactRecovery(): Int {
        val history = historyManager ?: run {
            Timber.e("Emergency recovery: HistoryManager not initialized")
            return 0
        }
        val contacts = contactManager ?: run {
            Timber.e("Emergency recovery: ContactManager not initialized")
            return 0
        }

        return try {
            // Get all messages to identify peer IDs
            val messages = history.recent(null, 100000u)
            val peerIds = messages.map { it.peerId }.distinct()

            var recoveredCount = 0
            for (peerId in peerIds) {
                // Check if contact exists
                val existingContact = try { contacts.get(peerId) } catch (e: Exception) { null }
                if (existingContact == null) {
                    // Create a basic contact from the peer ID
                    // Extract actual public key from libp2p PeerId for proper cryptographic operations
                    try {
                        val extractedKey = try {
                            ironCore?.extractPublicKeyFromPeerId(peerId)
                        } catch (e: Exception) {
                            Timber.w("Emergency recovery: Failed to extract public key from $peerId: ${e.message}")
                            null
                        }

                        val publicKey = if (!extractedKey.isNullOrEmpty()) {
                            extractedKey
                        } else {
                            // Fallback: use peer ID as placeholder (will be updated later via federation)
                            peerId
                        }

                        val contact = uniffi.api.Contact(
                            peerId = peerId,
                            nickname = null,
                            localNickname = null,
                            publicKey = publicKey,
                            addedAt = System.currentTimeMillis().toULong(),
                            lastSeen = null,
                            notes = "Emergency contact recovered from message history",
                            lastKnownDeviceId = null
                        )
                        contacts.add(contact)
                        recoveredCount++
                        Timber.d("Emergency recovery: Created contact for ${peerId.take(8)}... with key: ${publicKey.take(8)}...")
                    } catch (e: Exception) {
                        Timber.w("Emergency recovery: Failed to create contact for $peerId: ${e.message}")
                    }
                }
            }
            contacts.flush()
            Timber.i("Emergency contact recovery completed: $recoveredCount contacts recovered")
            recoveredCount
        } catch (e: Exception) {
            Timber.e(e, "Emergency contact recovery failed")
            0
        }
    }

    /**
     * Detect and repair data corruption in the contacts database.
     * Checks if contact count is 0 while message count is > 0, indicating corruption.
     */
    private suspend fun detectAndRepairCorruption(): Boolean {
        val history = historyManager ?: run {
            Timber.e("Corruption detection: HistoryManager not initialized")
            return false
        }
        val contacts = contactManager ?: run {
            Timber.e("Corruption detection: ContactManager not initialized")
            return false
        }

        return try {
            val contactCount = contacts.list().size
            val messageCount = history.stats().totalMessages.toInt()

            Timber.d("Corruption check: contacts=$contactCount, messages=$messageCount")

            if (contactCount == 0 && messageCount > 0) {
                Timber.e("CRITICAL: Data corruption detected - $messageCount messages but 0 contacts")

                // Backup corrupted database
                backupCorruptedDatabase()

                // Attempt emergency recovery
                val recovered = emergencyContactRecovery()

                if (recovered > 0) {
                    Timber.i("Corruption repair successful: $recovered contacts recovered")
                    true
                } else {
                    Timber.e("Corruption repair failed: No contacts could be recovered")
                    false
                }
            } else {
                Timber.d("Database integrity check passed")
                false
            }
        } catch (e: Exception) {
            Timber.e(e, "Corruption detection failed")
            false
        }
    }

    /**
     * Backup corrupted database files before repair.
     * Creates timestamped backups in the app's backup directory.
     */
    private fun backupCorruptedDatabase() {
        val backupDir = File(storagePath, "backup_corrupted_${System.currentTimeMillis()}")

        try {
            backupDir.mkdirs()
            Timber.i("Created backup directory: ${backupDir.absolutePath}")

            // Backup contacts.db if it exists
            val contactsDb = File(storagePath, "contacts.db")
            if (contactsDb.exists()) {
                val backupFile = File(backupDir, "contacts.db")
                contactsDb.copyTo(backupFile, overwrite = true)
                Timber.i("Backed up contacts.db to ${backupFile.absolutePath}")
            }

            // Backup history files if they exist
            val historyDir = File(storagePath, "history")
            if (historyDir.exists()) {
                val backupHistoryDir = File(backupDir, "history")
                backupHistoryDir.mkdirs()
                historyDir.listFiles()?.forEach { file ->
                    if (file.isFile) {
                        file.copyTo(File(backupHistoryDir, file.name), overwrite = true)
                    }
                }
                Timber.i("Backed up history directory to ${backupHistoryDir.absolutePath}")
            }

            Timber.i("Corrupted database backup completed")
        } catch (e: Exception) {
            Timber.e(e, "Failed to backup corrupted database")
        }
    }

    // ========================================
    // P0_ANDROID_003: BLE Transport Stabilization & Graceful Degradation
    // ========================================

    /**
     * Check if BLE transport is degraded and initiate fallback.
     * Called from periodic health checks when BLE failures are detected.
     */
    fun handleBleTransportDegradation() {
        if (transportHealthMonitor.isDegraded("ble")) {
            Timber.w("P0_ANDROID_003: BLE transport degraded, initiating graceful fallback")
            // Reduce BLE scan frequency by switching to background mode
            bleScanner?.setBackgroundMode(true)
            // Log current transport health for diagnostics
            val bleHealth = transportHealthMonitor.getHealth("ble")
            Timber.w("P0_ANDROID_003: BLE health — successes: ${bleHealth.successCount}, failures: ${bleHealth.failureCount}, consecutive failures: ${bleHealth.consecutiveFailures}")
        }
    }

    /**
     * Record a transport event for health monitoring.
     */
    fun recordTransportEvent(transport: String, success: Boolean, latencyMs: Long? = null) {
        if (success) {
            transportHealthMonitor.recordSuccess(transport, latencyMs)
        } else {
            transportHealthMonitor.recordFailure(transport)
            // Check if degraded after recording failure
            if (transportHealthMonitor.isDegraded(transport)) {
                handleBleTransportDegradation()
            }
        }
    }

    fun getTransportHealthSummary(): Map<String, com.scmessenger.android.transport.TransportHealthMonitor.TransportHealth> {
        return transportHealthMonitor.getSummary()
    }

}
