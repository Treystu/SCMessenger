//
//  mDNSServiceDiscovery.swift
//  SCMessenger
//
//  mDNS/DNS-SD service discovery for cross-platform LAN discovery
//  Mirrors: android/.../transport/WifiDirectTransport.kt DNS-SD implementation
//  Service type: _scmessenger._tcp (matches Android)
//

import Foundation
import Network
import os
import Combine

/// mDNS/DNS-SD service discovery for cross-platform LAN discovery
///
/// This implements the same DNS-SD service type as Android's WiFi Direct transport,
/// allowing iOS devices to discover Android devices on the same local network.
///
/// Service type: _scmessenger._tcp (matches Android's WifiDirectTransport.SERVICE_TYPE)
final class mDNSServiceDiscovery: NSObject {
    private let logger = Logger(subsystem: "com.scmessenger", category: "mDNS")
    private weak var meshRepository: MeshRepository?

    // Service discovery
    private var netServiceBrowser: NetServiceBrowser?
    private var discoveredServices: [String: NetService] = [:]
    private var isBrowsing = false

    // Service advertisement
    private var localService: NetService?
    private var isAdvertising = false

    // Service type (must match Android's WifiDirectTransport.SERVICE_TYPE)
    private let serviceType = "_scmessenger._tcp"
    private let serviceName = "SCMessenger"

    /// Callback when a LAN peer's address is resolved (host + port).
    /// The caller can construct a multiaddr and dial via SwarmBridge.
    var onLanPeerResolved: ((String, Int32) -> Void)?

    init(meshRepository: MeshRepository?) {
        self.meshRepository = meshRepository
        super.init()
    }

    // MARK: - Public API

    func startBrowsing() {
        guard !isBrowsing else {
            logger.debug("Already browsing for mDNS services")
            return
        }

        logger.info("Starting mDNS browsing for \(self.serviceType)")
        netServiceBrowser = NetServiceBrowser()
        netServiceBrowser?.delegate = self
        netServiceBrowser?.searchForServices(ofType: serviceType, inDomain: "local.")
        isBrowsing = true
    }

    func stopBrowsing() {
        guard isBrowsing else { return }
        logger.info("Stopping mDNS browsing")
        netServiceBrowser?.stop()
        netServiceBrowser = nil
        discoveredServices.removeAll()
        isBrowsing = false
    }

    func startAdvertising(port: Int32) {
        guard !isAdvertising else {
            logger.debug("Already advertising mDNS service")
            return
        }

        logger.info("Starting mDNS advertising for \(self.serviceName) on port \(port)")
        localService = NetService(
            domain: "local.",
            type: serviceType,
            name: serviceName,
            port: port
        )
        localService?.delegate = self
        localService?.publish()
        isAdvertising = true
    }

    func stopAdvertising() {
        guard isAdvertising else { return }
        logger.info("Stopping mDNS advertising")
        localService?.stop()
        localService = nil
        isAdvertising = false
    }

    func cleanup() {
        stopBrowsing()
        stopAdvertising()
    }
}

// MARK: - NetServiceBrowserDelegate

extension mDNSServiceDiscovery: NetServiceBrowserDelegate {
    func netServiceBrowser(_ browser: NetServiceBrowser, didFind service: NetService, moreComing: Bool) {
        let serviceKey = "\(service.name):\(service.type)"
        logger.info("mDNS service found: \(service.name) type: \(service.type)")

        // Resolve the service to get the address
        service.delegate = self
        service.resolve(withTimeout: 5.0)
        discoveredServices[serviceKey] = service
    }

    func netServiceBrowser(_ browser: NetServiceBrowser, didRemove service: NetService, moreComing: Bool) {
        let serviceKey = "\(service.name):\(service.type)"
        logger.info("mDNS service removed: \(service.name)")
        discoveredServices.removeValue(forKey: serviceKey)
    }

    func netServiceBrowserDidStopSearch(_ browser: NetServiceBrowser) {
        logger.info("mDNS browser stopped")
        isBrowsing = false
    }

    func netServiceBrowser(_ browser: NetServiceBrowser, didNotSearch error: Error) {
        logger.error("mDNS browser failed: \(error.localizedDescription)")
        isBrowsing = false
    }
}

// MARK: - NetServiceDelegate

extension mDNSServiceDiscovery: NetServiceDelegate {
    func netServiceDidResolveAddress(_ sender: NetService) {
        guard let addresses = sender.addresses, !addresses.isEmpty else {
            logger.warning("mDNS service resolved but no addresses: \(sender.name)")
            return
        }

        // Use the first address and convert to string
        let address = addresses[0]
        var host = "unknown"
        var port = Int32(0)
        
        // Convert sockaddr to string representation
        address.withUnsafeBytes { ptr in
            let sockaddrPtr = ptr.bindMemory(to: sockaddr.self)
            guard let firstSockaddr = sockaddrPtr.first else { return }
            var buffer = [CChar](repeating: 0, count: Int(INET6_ADDRSTRLEN))
            if firstSockaddr.sa_family == sa_family_t(AF_INET) {
                var sin = address.withUnsafeBytes { $0.load(as: sockaddr_in.self) }
                inet_ntop(AF_INET, &sin.sin_addr, &buffer, socklen_t(INET_ADDRSTRLEN))
                host = String(cString: buffer)
                port = Int32(sin.sin_port)
            } else if firstSockaddr.sa_family == sa_family_t(AF_INET6) {
                var sin6 = address.withUnsafeBytes { $0.load(as: sockaddr_in6.self) }
                inet_ntop(AF_INET6, &sin6.sin6_addr, &buffer, socklen_t(INET6_ADDRSTRLEN))
                host = String(cString: buffer)
                port = Int32(sin6.sin6_port)
            }
        }

        logger.info("mDNS service resolved: \(sender.name) at \(host):\(port)")

        // Create a peer ID from the service name (matches Android's device.deviceAddress pattern)
        let peerId = "mdns-\(sender.name)"

        // Notify discovery
        let repo = meshRepository
        DispatchQueue.main.async {
            repo?.handleTransportPeerDiscovered(peerId: peerId)
            // Also send to event bus for UI
            MeshEventBus.shared.peerEvents.send(.discovered(peerId: peerId))
        }

        // TCP/mDNS parity: Notify the resolved LAN address so the caller
        // can generate a libp2p multiaddr and dial via SwarmBridge.
        if host != "unknown" && port > 0 {
            logger.info("mDNS: LAN peer resolved at \(host):\(port) — notifying for SwarmBridge dial")
            onLanPeerResolved?(host, port)
        }
    }

    func netService(_ sender: NetService, didNotResolve errorDict: [String: NSNumber]) {
        logger.error("mDNS service failed to resolve: \(sender.name)")
    }
}
