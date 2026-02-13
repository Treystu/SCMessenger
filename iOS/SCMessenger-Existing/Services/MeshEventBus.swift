//
//  MeshEventBus.swift
//  SCMessenger
//
//  Central event dispatch for mesh network events
//  Mirrors: android/.../service/MeshEventBus.kt
//

import Combine
import Foundation

/// Central event dispatch for mesh network events
///
/// Uses Combine PassthroughSubjects (equivalent to Android SharedFlow)
/// for publishing events throughout the app.
///
/// All mesh-related events flow through this central bus:
/// - Peer discovery and connection events
/// - Message send/receive/delivery events
/// - Service status changes
/// - Network and battery changes
final class MeshEventBus {
    static let shared = MeshEventBus()
    
    private init() {}
    
    // MARK: - Event Streams
    
    /// Peer lifecycle events
    let peerEvents = PassthroughSubject<PeerEvent, Never>()
    
    /// Message lifecycle events
    let messageEvents = PassthroughSubject<MessageEvent, Never>()
    
    /// Service status events
    let statusEvents = PassthroughSubject<StatusEvent, Never>()
    
    /// Network and transport events
    let networkEvents = PassthroughSubject<NetworkEvent, Never>()
    
    // MARK: - Event Types
    
    enum PeerEvent: Equatable {
        case discovered(peerId: String)
        case connected(peerId: String)
        case disconnected(peerId: String)
        case connectionFailed(peerId: String, error: String)
    }
    
    enum MessageEvent: Equatable {
        case received(senderId: String, messageId: String, data: Data)
        case sent(messageId: String)
        case delivered(messageId: String)
        case failed(messageId: String, error: String)
    }
    
    enum StatusEvent: Equatable {
        case serviceStateChanged(ServiceState)
        case statsUpdated(ServiceStats)
    }
    
    enum NetworkEvent: Equatable {
        case transportEnabled(TransportType)
        case transportDisabled(TransportType)
        case batteryChanged(pct: UInt8, charging: Bool)
        case networkChanged(wifi: Bool, cellular: Bool)
    }
    
    enum TransportType: String, Equatable {
        case ble = "BLE"
        case multipeer = "Multipeer"
        case internet = "Internet"
    }
}

// MARK: - ServiceStats Equatable Conformance

extension ServiceStats: Equatable {
    public static func == (lhs: ServiceStats, rhs: ServiceStats) -> Bool {
        lhs.peersDiscovered == rhs.peersDiscovered &&
        lhs.messagesRelayed == rhs.messagesRelayed &&
        lhs.bytesTransferred == rhs.bytesTransferred &&
        lhs.uptimeSecs == rhs.uptimeSecs
    }
}

// MARK: - ServiceState Equatable Conformance

extension ServiceState: Equatable {
    public static func == (lhs: ServiceState, rhs: ServiceState) -> Bool {
        switch (lhs, rhs) {
        case (.stopped, .stopped): return true
        case (.starting, .starting): return true
        case (.running, .running): return true
        case (.stopping, .stopping): return true
        default: return false
        }
    }
}
