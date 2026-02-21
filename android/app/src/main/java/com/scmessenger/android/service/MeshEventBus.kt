package com.scmessenger.android.service

import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import timber.log.Timber

/**
 * Central event dispatcher for mesh network events.
 *
 * Maps UniFFI callback data to Kotlin sealed classes and dispatches them
 * via SharedFlow for consumption by ViewModels and UI components.
 *
 * All flows have replay=1 for late subscribers to receive the latest event.
 * Thread-safe for concurrent access.
 */
object MeshEventBus {

    // Peer discovery/disconnection events
    private val _peerEvents = MutableSharedFlow<PeerEvent>(replay = 1)
    val peerEvents: SharedFlow<PeerEvent> = _peerEvents.asSharedFlow()

    // Message send/receive events
    private val _messageEvents = MutableSharedFlow<MessageEvent>(replay = 1)
    val messageEvents: SharedFlow<MessageEvent> = _messageEvents.asSharedFlow()

    // Service status changes
    private val _statusEvents = MutableSharedFlow<StatusEvent>(replay = 1)
    val statusEvents: SharedFlow<StatusEvent> = _statusEvents.asSharedFlow()

    // Network/transport events
    private val _networkEvents = MutableSharedFlow<NetworkEvent>(replay = 1)
    val networkEvents: SharedFlow<NetworkEvent> = _networkEvents.asSharedFlow()

    /**
     * Emit a peer event (discovery, disconnection, status change).
     */
    suspend fun emitPeerEvent(event: PeerEvent) {
        try {
            _peerEvents.emit(event)
            Timber.d("PeerEvent emitted: $event")
        } catch (e: Exception) {
            Timber.e(e, "Failed to emit peer event")
        }
    }

    /**
     * Emit a message event (received, sent, delivered).
     */
    suspend fun emitMessageEvent(event: MessageEvent) {
        try {
            _messageEvents.emit(event)
            Timber.d("MessageEvent emitted: ${event.javaClass.simpleName}")
        } catch (e: Exception) {
            Timber.e(e, "Failed to emit message event")
        }
    }

    /**
     * Emit a status event (service state change, profile change).
     */
    suspend fun emitStatusEvent(event: StatusEvent) {
        try {
            _statusEvents.emit(event)
            Timber.d("StatusEvent emitted: $event")
        } catch (e: Exception) {
            Timber.e(e, "Failed to emit status event")
        }
    }

    /**
     * Emit a network event (transport change, connection quality).
     */
    suspend fun emitNetworkEvent(event: NetworkEvent) {
        try {
            _networkEvents.emit(event)
            Timber.d("NetworkEvent emitted: $event")
        } catch (e: Exception) {
            Timber.e(e, "Failed to emit network event")
        }
    }
}

// ============================================================================
// Event Types
// ============================================================================

/**
 * Peer-related events.
 */
sealed class PeerEvent {
    data class Discovered(val peerId: String, val transport: TransportType) : PeerEvent()
    data class IdentityDiscovered(
        val peerId: String,
        val publicKey: String,
        val nickname: String?,
        val libp2pPeerId: String?,
        val listeners: List<String>,
        val blePeerId: String? = null
    ) : PeerEvent()
    data class Connected(val peerId: String, val transport: TransportType) : PeerEvent()
    data class Disconnected(val peerId: String) : PeerEvent()
    data class StatusChanged(val peerId: String, val online: Boolean) : PeerEvent()
}

/**
 * Message-related events.
 */
sealed class MessageEvent {
    data class Received(val messageRecord: uniffi.api.MessageRecord) : MessageEvent()
    data class Sent(val messageId: String, val peerId: String) : MessageEvent()
    data class Delivered(val messageId: String) : MessageEvent()
    data class Failed(val messageId: String, val error: String) : MessageEvent()
}

/**
 * Service status events.
 */
sealed class StatusEvent {
    data class ServiceStateChanged(val state: uniffi.api.ServiceState) : StatusEvent()
    data class ProfileChanged(val profile: uniffi.api.AdjustmentProfile) : StatusEvent()
    data class StatsUpdated(val stats: uniffi.api.ServiceStats) : StatusEvent()
}

/**
 * Network/transport events.
 */
sealed class NetworkEvent {
    data class TransportEnabled(val transport: TransportType) : NetworkEvent()
    data class TransportDisabled(val transport: TransportType) : NetworkEvent()
    data class ConnectionQualityChanged(val peerId: String, val quality: ConnectionQuality) : NetworkEvent()
    data class BatteryStateChanged(val level: Int, val isCharging: Boolean) : NetworkEvent()
}

/**
 * Transport types.
 */
enum class TransportType {
    BLE,
    WIFI_AWARE,
    WIFI_DIRECT,
    INTERNET
}

/**
 * Connection quality levels.
 */
enum class ConnectionQuality {
    EXCELLENT,
    GOOD,
    FAIR,
    POOR,
    UNKNOWN
}
