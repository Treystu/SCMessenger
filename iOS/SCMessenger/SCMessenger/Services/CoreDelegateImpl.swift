//
//  CoreDelegateImpl.swift
//  SCMessenger
//
//  Implements Rust CoreDelegate callback interface
//  Receives events FROM Rust core and publishes to MeshEventBus
//

import Combine
import Foundation
import os

/// Implements Rust CoreDelegate callback interface
///
/// This class receives callbacks FROM the Rust core (UniFFI)
/// and publishes them to the MeshEventBus for Swift/SwiftUI consumption.
///
/// Flow: Rust Core → CoreDelegate → MeshEventBus → SwiftUI Views
final class CoreDelegateImpl: CoreDelegate {
    private let logger = Logger(subsystem: "com.scmessenger", category: "CoreDelegate")
    private let eventBus = MeshEventBus.shared
    private weak var meshRepository: MeshRepository?
    
    init(meshRepository: MeshRepository?) {
        self.meshRepository = meshRepository
    }
    
    // MARK: - CoreDelegate Protocol (called FROM Rust)
    
    func onPeerDiscovered(peerId: String) {
        logger.info("Peer discovered: \(peerId)")
        eventBus.peerEvents.send(.discovered(peerId: peerId))
    }

    func onPeerConnected(peerId: String) {
        logger.info("Peer connected: \(peerId)")
        eventBus.peerEvents.send(.connected(peerId: peerId))
    }

    func onPeerDisconnected(peerId: String) {
        logger.info("Peer disconnected: \(peerId)")
        eventBus.peerEvents.send(.disconnected(peerId: peerId))
    }
    
    func onMessageReceived(senderId: String, messageId: String, data: Data) {
        logger.info("Message received: \(messageId) from \(senderId) (\(data.count) bytes)")

        // UniFFI callbacks arrive on a Rust thread; MeshRepository is @MainActor.
        // Capture values before the dispatch to avoid capturing self or mutable state.
        let repo = meshRepository
        DispatchQueue.main.async {
            repo?.onMessageReceived(senderId: senderId, messageId: messageId, data: data)
        }

        // Publish event (PassthroughSubject is thread-safe for send())
        eventBus.messageEvents.send(.received(
            senderId: senderId,
            messageId: messageId,
            data: data
        ))
    }
    
    func onMessageSent(messageId: String) {
        logger.info("Message sent: \(messageId)")
        eventBus.messageEvents.send(.sent(messageId: messageId))
    }

    func onMessageDelivered(messageId: String) {
        logger.info("Message delivered: \(messageId)")
        eventBus.messageEvents.send(.delivered(messageId: messageId))
    }

    func onMessageFailed(messageId: String, error: String) {
        logger.error("Message failed: \(messageId) - \(error)")
        eventBus.messageEvents.send(.failed(messageId: messageId, error: error))
    }

    func onReceiptReceived(messageId: String, status: String) {
        logger.info("Receipt received: \(messageId) status=\(status)")

        // Map receipt status to message events
        switch status.lowercased() {
        case "delivered":
            eventBus.messageEvents.send(.delivered(messageId: messageId))
        case "failed":
            eventBus.messageEvents.send(.failed(messageId: messageId, error: "Receipt indicated failure"))
        default:
            logger.debug("Unknown receipt status: \(status)")
        }
    }

    func onServiceStateChanged(state: ServiceState) {
        logger.info("Service state changed: \(String(describing: state))")
        eventBus.statusEvents.send(.serviceStateChanged(state))
    }

    func onStatsUpdated(stats: ServiceStats) {
        logger.debug("Stats updated: \(stats.peersDiscovered) peers, \(stats.messagesRelayed) messages")
        eventBus.statusEvents.send(.statsUpdated(stats))
    }
}

// MARK: - ServiceState Description

extension ServiceState: CustomStringConvertible {
    public var description: String {
        switch self {
        case .stopped: return "stopped"
        case .starting: return "starting"
        case .running: return "running"
        case .stopping: return "stopping"
        }
    }
}
