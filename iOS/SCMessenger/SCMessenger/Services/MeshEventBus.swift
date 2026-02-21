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
        case identityDiscovered(peerId: String, publicKey: String, nickname: String?, libp2pPeerId: String?, listeners: [String])
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
