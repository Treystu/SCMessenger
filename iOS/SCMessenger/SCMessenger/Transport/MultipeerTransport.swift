//
//  MultipeerTransport.swift
//  SCMessenger
//
//  WiFi Direct equivalent using Apple's Multipeer Connectivity
//  Transport priority: Multipeer > BLE > Internet
//

import MultipeerConnectivity
import os

/// WiFi Direct equivalent using Apple's Multipeer Connectivity
///
/// Multipeer Connectivity provides peer-to-peer WiFi and Bluetooth connectivity
/// without requiring internet infrastructure. Preferred over BLE for speed.
final class MultipeerTransport: NSObject {
    private let logger = Logger(subsystem: "com.scmessenger", category: "Multipeer")
    private weak var meshRepository: MeshRepository?
    
    // Multipeer components
    private var peerID: MCPeerID!
    private var session: MCSession!
    private var advertiser: MCNearbyServiceAdvertiser?
    private var browser: MCNearbyServiceBrowser?
    
    // Service type (must be ≤15 chars, lowercase, no special chars)
    private let serviceType = "scmesh"
    
    // Connection state
    private var connectedPeers: Set<MCPeerID> = []
    private var isAdvertising = false
    private var isBrowsing = false
    
    init(meshRepository: MeshRepository) {
        self.meshRepository = meshRepository
        super.init()
        setupPeerID()
        setupSession()
    }
    
    // MARK: - Setup
    
    private func setupPeerID() {
        // Use identity snippet as display name
        let displayName = meshRepository?.getIdentitySnippet() ?? "SCMesh"
        peerID = MCPeerID(displayName: displayName)
        logger.info("Multipeer peer ID: \(displayName)")
    }
    
    private func setupSession() {
        session = MCSession(
            peer: peerID,
            securityIdentity: nil,
            encryptionPreference: .required
        )
        session.delegate = self
    }
    
    // MARK: - Public API
    
    func startAdvertising() {
        guard !isAdvertising else {
            logger.debug("Already advertising")
            return
        }
        
        advertiser = MCNearbyServiceAdvertiser(
            peer: peerID,
            discoveryInfo: nil,
            serviceType: serviceType
        )
        advertiser?.delegate = self
        advertiser?.startAdvertisingPeer()
        isAdvertising = true
        logger.info("Started Multipeer advertising")
    }
    
    func stopAdvertising() {
        advertiser?.stopAdvertisingPeer()
        advertiser = nil
        isAdvertising = false
        logger.info("Stopped Multipeer advertising")
    }
    
    func startBrowsing() {
        guard !isBrowsing else {
            logger.debug("Already browsing")
            return
        }
        
        browser = MCNearbyServiceBrowser(peer: peerID, serviceType: serviceType)
        browser?.delegate = self
        browser?.startBrowsingForPeers()
        isBrowsing = true
        logger.info("Started Multipeer browsing")
    }
    
    func stopBrowsing() {
        browser?.stopBrowsingForPeers()
        browser = nil
        isBrowsing = false
        logger.info("Stopped Multipeer browsing")
    }
    
    func sendData(to peer: MCPeerID, data: Data) throws {
        guard connectedPeers.contains(peer) else {
            logger.error("Peer \(peer.displayName) not connected")
            throw MultipeerError.notConnected
        }
        
        try session.send(data, toPeers: [peer], with: .reliable)
        logger.debug("Sent \(data.count) bytes to \(peer.displayName)")
    }
    
    func sendDataToAll(_ data: Data) {
        let peers = Array(connectedPeers)
        guard !peers.isEmpty else {
            logger.warning("No connected peers")
            return
        }
        
        do {
            try session.send(data, toPeers: peers, with: .reliable)
            logger.debug("Sent \(data.count) bytes to \(peers.count) peers")
        } catch {
            logger.error("Failed to send data: \(error.localizedDescription)")
        }
    }
    
    func disconnect() {
        session.disconnect()
        stopAdvertising()
        stopBrowsing()
        connectedPeers.removeAll()
        logger.info("Disconnected from Multipeer session")
    }
    
    func hasConnection(_ peerId: String) -> Bool {
        connectedPeers.contains(where: { $0.displayName == peerId })
    }
}

// MARK: - MCSessionDelegate

extension MultipeerTransport: MCSessionDelegate {
    func session(_ session: MCSession, peer peerID: MCPeerID, didChange state: MCSessionState) {
        logger.info("Peer \(peerID.displayName) state changed: \(state.rawValue)")
        
        switch state {
        case .connected:
            connectedPeers.insert(peerID)
            MeshEventBus.shared.peerEvents.send(.connected(peerId: peerID.displayName))
            
        case .connecting:
            logger.debug("Connecting to \(peerID.displayName)")
            
        case .notConnected:
            connectedPeers.remove(peerID)
            MeshEventBus.shared.peerEvents.send(.disconnected(peerId: peerID.displayName))
            
        @unknown default:
            logger.warning("Unknown session state")
        }
    }
    
    func session(_ session: MCSession, didReceive data: Data, fromPeer peerID: MCPeerID) {
        logger.debug("Received \(data.count) bytes from \(peerID.displayName)")
        meshRepository?.onBleDataReceived(peerId: peerID.displayName, data: data)
    }
    
    func session(_ session: MCSession, didReceive stream: InputStream, withName streamName: String, fromPeer peerID: MCPeerID) {
        logger.debug("Received stream from \(peerID.displayName)")
        // Handle stream if needed
    }
    
    func session(_ session: MCSession, didStartReceivingResourceWithName resourceName: String, fromPeer peerID: MCPeerID, with progress: Progress) {
        logger.debug("Receiving resource from \(peerID.displayName)")
    }
    
    func session(_ session: MCSession, didFinishReceivingResourceWithName resourceName: String, fromPeer peerID: MCPeerID, at localURL: URL?, withError error: Error?) {
        if let error = error {
            logger.error("Resource receive error: \(error.localizedDescription)")
        }
    }
}

// MARK: - MCNearbyServiceAdvertiserDelegate

extension MultipeerTransport: MCNearbyServiceAdvertiserDelegate {
    func advertiser(_ advertiser: MCNearbyServiceAdvertiser, didReceiveInvitationFromPeer peerID: MCPeerID, withContext context: Data?, invitationHandler: @escaping (Bool, MCSession?) -> Void) {
        logger.info("Received invitation from \(peerID.displayName)")
        
        // Auto-accept invitations (mesh network)
        invitationHandler(true, session)
    }
    
    func advertiser(_ advertiser: MCNearbyServiceAdvertiser, didNotStartAdvertisingPeer error: Error) {
        logger.error("Failed to start advertising: \(error.localizedDescription)")
        isAdvertising = false
    }
}

// MARK: - MCNearbyServiceBrowserDelegate

extension MultipeerTransport: MCNearbyServiceBrowserDelegate {
    func browser(_ browser: MCNearbyServiceBrowser, foundPeer peerID: MCPeerID, withDiscoveryInfo info: [String: String]?) {
        logger.info("Found peer: \(peerID.displayName)")
        
        // Auto-invite found peers (mesh network)
        browser.invitePeer(peerID, to: session, withContext: nil, timeout: 10)
        
        MeshEventBus.shared.peerEvents.send(.discovered(peerId: peerID.displayName))
    }
    
    func browser(_ browser: MCNearbyServiceBrowser, lostPeer peerID: MCPeerID) {
        logger.info("Lost peer: \(peerID.displayName)")
    }
    
    func browser(_ browser: MCNearbyServiceBrowser, didNotStartBrowsingForPeers error: Error) {
        logger.error("Failed to start browsing: \(error.localizedDescription)")
        isBrowsing = false
    }
}

// MARK: - Error Types

enum MultipeerError: LocalizedError {
    case notConnected
    case sendFailed
    
    var errorDescription: String? {
        switch self {
        case .notConnected:
            return "Peer not connected"
        case .sendFailed:
            return "Failed to send data"
        }
    }
}
