//
//  MeshDashboardView.swift
//  SCMessenger
//
//  Mesh network dashboard
//

import SwiftUI

struct DashboardPeer: Identifiable, Equatable {
    enum Transport: String {
        case ble = "BLE"
        case internet = "Internet"
        case unknown = "Unknown"
    }

    let id: String
    var peerId: String
    var publicKey: String?
    var nickname: String?
    var localNickname: String?
    var libp2pPeerId: String?
    var blePeerId: String?
    var transport: Transport
    var isOnline: Bool
    var isRelay: Bool
    var isFull: Bool
    var lastSeen: Date

    var displayName: String {
        let local = localNickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        if !local.isEmpty { return local }
        let federated = nickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        if !federated.isEmpty { return federated }
        
        if isRelay {
            // Check if it's a known bootstrap node via libp2pPeerId or canonicalId
            return "Relay Node"
        }
        return isFull ? "Full Node" : "Headless Node"
    }

    var roleLabel: String {
        if isRelay { return "Relay" }
        return isFull ? "Full" : "Headless"
    }
}

struct MeshDashboardView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var stats: ServiceStats?
    @State private var peersByKey: [String: DashboardPeer] = [:]

    private var peers: [DashboardPeer] {
        peersByKey.values.sorted { $0.lastSeen > $1.lastSeen }
    }

    private var fullPeers: Int {
        peers.filter { $0.isFull }.count
    }

    private var headlessPeers: Int {
        peers.filter { !$0.isFull }.count
    }

    var body: some View {
        ScrollView {
            VStack(spacing: Theme.spacingLarge) {
                ServiceStatusCard(stats: stats)

                DiscoveredNodesSection(
                    peers: peers,
                    fullPeers: fullPeers,
                    headlessPeers: headlessPeers
                )

                TransportStatusSection()

                if let stats = stats {
                    RelayStatsSection(stats: stats)
                }
            }
            .padding(Theme.spacingMedium)
        }
        .navigationTitle("Mesh Dashboard")
        .task {
            loadDashboardData()
            refreshPeersFromRepository()
        }
        .onReceive(MeshEventBus.shared.statusEvents) { event in
            if case .statsUpdated(let updatedStats) = event {
                stats = updatedStats
            }
        }
        .onReceive(MeshEventBus.shared.peerEvents) { event in
            handlePeerEvent(event)
        }
    }

    private func loadDashboardData() {
        repository.updateStats()
        stats = repository.serviceStats
    }

    private func refreshPeersFromRepository() {
        let contacts = (try? repository.getContacts()) ?? []
        let contactsByPeerId = Dictionary(uniqueKeysWithValues: contacts.map { ($0.peerId, $0) })
        var contactsByRoutePeerId: [String: Contact] = [:]
        var contactsByPublicKey: [String: Contact] = [:]
        var contactsByNickname: [String: Contact] = [:]
        
        contacts.forEach { contact in
            if let routePeerId = parseRoutingLibp2pPeerId(from: contact.notes) {
                if contactsByRoutePeerId[routePeerId] == nil {
                    contactsByRoutePeerId[routePeerId] = contact
                }
            }
            let pk = contact.publicKey.trimmingCharacters(in: .whitespacesAndNewlines)
            if !pk.isEmpty {
                contactsByPublicKey[pk] = contact
            }
            if let nn = contact.nickname?.trimmingCharacters(in: .whitespacesAndNewlines), !nn.isEmpty {
                contactsByNickname[nn] = contact
            }
        }

        var merged = peersByKey
        let now = Date()

        for contact in contacts {
            let isRelay = repository.isKnownRelay(contact.peerId)
            let routePeerId = parseRoutingLibp2pPeerId(from: contact.notes)
            
            var existing = merged[contact.peerId]
            if existing == nil {
                let pk = contact.publicKey.trimmingCharacters(in: .whitespacesAndNewlines)
                if !pk.isEmpty {
                    existing = merged.values.first(where: { $0.publicKey == pk })
                }
            }
            if existing == nil, let rid = routePeerId {
                existing = merged.values.first(where: { $0.libp2pPeerId == rid || $0.peerId == rid })
            }
            if existing == nil, let nn = contact.nickname?.trimmingCharacters(in: .whitespacesAndNewlines), !nn.isEmpty {
                existing = merged.values.first(where: { $0.nickname == nn })
            }
            
            if let oldId = existing?.id, oldId != contact.peerId {
                merged.removeValue(forKey: oldId)
            }

            merged[contact.peerId] = DashboardPeer(
                id: contact.peerId,
                peerId: contact.peerId,
                publicKey: contact.publicKey,
                nickname: contact.nickname,
                localNickname: contact.localNickname,
                libp2pPeerId: routePeerId ?? existing?.libp2pPeerId,
                blePeerId: existing?.blePeerId,
                transport: existing?.transport ?? .unknown,
                isOnline: existing?.isOnline ?? isRecent(contact.lastSeen),
                isRelay: isRelay,
                isFull: classifyPeerAsFull(
                    peerId: contact.peerId,
                    publicKey: contact.publicKey,
                    nickname: contact.nickname,
                    localNickname: contact.localNickname,
                    isRelay: isRelay
                ),
                lastSeen: existing?.lastSeen ?? dateFromEpoch(contact.lastSeen) ?? now
            )
        }

        if let entries = try? repository.getDialableAddresses() {
            for entry in entries {
                guard let routePeerId = entry.peerId?.trimmingCharacters(in: .whitespacesAndNewlines),
                      !routePeerId.isEmpty else { continue }
                
                let entryPublicKey = entry.publicKey?.trimmingCharacters(in: .whitespacesAndNewlines)
                let entryNickname = entry.nickname?.trimmingCharacters(in: .whitespacesAndNewlines)
                
                let matchedContact = contactsByPeerId[routePeerId] ?? 
                                     contactsByRoutePeerId[routePeerId] ?? 
                                     (entryPublicKey.flatMap { pk in pk.isEmpty ? nil : contactsByPublicKey[pk] }) ??
                                     (entryNickname.flatMap { nn in nn.isEmpty ? nil : contactsByNickname[nn] })
                
                let canonicalPeerId = matchedContact?.peerId ?? routePeerId
                let relay = repository.isKnownRelay(routePeerId) || repository.isKnownRelay(canonicalPeerId)
                
                var existing = merged[canonicalPeerId]
                if existing == nil, let pk = matchedContact?.publicKey ?? entryPublicKey, !pk.isEmpty {
                    existing = merged.values.first(where: { $0.publicKey == pk })
                    if let oldId = existing?.id, oldId != canonicalPeerId {
                        merged.removeValue(forKey: oldId)
                    }
                }
                if existing == nil, let nn = matchedContact?.nickname ?? entryNickname, !nn.isEmpty {
                    existing = merged.values.first(where: { $0.nickname == nn })
                    if let oldId = existing?.id, oldId != canonicalPeerId {
                        merged.removeValue(forKey: oldId)
                    }
                }
                
                let lastSeenDate = dateFromEpoch(entry.lastSeen) ?? existing?.lastSeen ?? now

                merged[canonicalPeerId] = DashboardPeer(
                    id: canonicalPeerId,
                    peerId: canonicalPeerId,
                    publicKey: matchedContact?.publicKey ?? entryPublicKey ?? existing?.publicKey,
                    nickname: matchedContact?.nickname ?? entryNickname ?? existing?.nickname,
                    localNickname: matchedContact?.localNickname ?? existing?.localNickname,
                    libp2pPeerId: routePeerId,
                    blePeerId: existing?.blePeerId,
                    transport: transportFromMultiaddr(entry.multiaddr),
                    isOnline: isRecent(entry.lastSeen) || (existing?.isOnline == true),
                    isRelay: relay,
                    isFull: classifyPeerAsFull(
                        peerId: canonicalPeerId,
                        publicKey: matchedContact?.publicKey ?? entryPublicKey ?? existing?.publicKey,
                        nickname: matchedContact?.nickname ?? entryNickname ?? existing?.nickname,
                        localNickname: matchedContact?.localNickname ?? existing?.localNickname,
                        isRelay: relay
                    ),
                    lastSeen: lastSeenDate
                )
            }
        }

        // Final deduplication pass to merge any hanging libp2p nodes with identity nodes
        var finalMerged: [String: DashboardPeer] = [:]
        for (_, peer) in merged {
            var shouldKeep = true
            if let pk = peer.publicKey, !pk.isEmpty {
                if let identityPeer = merged.values.first(where: { $0.publicKey == pk && isIdentityId($0.id) }) {
                   if peer.id != identityPeer.id {
                       shouldKeep = false
                   }
                }
            } else if let nn = peer.nickname, !nn.isEmpty {
                 if let identityPeer = merged.values.first(where: { $0.nickname == nn && isIdentityId($0.id) }) {
                   if peer.id != identityPeer.id {
                       shouldKeep = false
                   }
                }
            }
            if shouldKeep && !peer.id.isEmpty {
                finalMerged[peer.id] = peer
            }
        }

        peersByKey = finalMerged
    }

    private func handlePeerEvent(_ event: MeshEventBus.PeerEvent) {
        switch event {
        case .discovered(let peerId):
            upsertPeer(
                canonicalPeerId: peerId,
                publicKey: nil,
                nickname: nil,
                localNickname: nil,
                libp2pPeerId: nil,
                blePeerId: nil,
                transport: .internet,
                isOnline: true
            )
        case .identityDiscovered(let peerId, let publicKey, let nickname, let libp2pPeerId, _, let blePeerId):
            upsertPeer(
                canonicalPeerId: peerId,
                publicKey: publicKey,
                nickname: nickname,
                localNickname: nil,
                libp2pPeerId: libp2pPeerId,
                blePeerId: blePeerId,
                transport: (blePeerId?.isEmpty == false) ? .ble : .internet,
                isOnline: true
            )
        case .connected(let peerId):
            markPeer(peerId: peerId, isOnline: true)
        case .disconnected(let peerId):
            markPeer(peerId: peerId, isOnline: false)
        case .connectionFailed(let peerId, _):
            markPeer(peerId: peerId, isOnline: false)
        }
    }

    private func upsertPeer(
        canonicalPeerId: String,
        publicKey: String?,
        nickname: String?,
        localNickname: String?,
        libp2pPeerId: String?,
        blePeerId: String?,
        transport: DashboardPeer.Transport,
        isOnline: Bool
    ) {
        let normalizedCanonical = canonicalPeerId.trimmingCharacters(in: .whitespacesAndNewlines)
        if normalizedCanonical.isEmpty { return }
        let normalizedLibp2p = libp2pPeerId?.trimmingCharacters(in: .whitespacesAndNewlines).nilIfEmpty
        let normalizedBlePeer = blePeerId?.trimmingCharacters(in: .whitespacesAndNewlines).nilIfEmpty
        let resolvedPublicKey = publicKey?.trimmingCharacters(in: .whitespacesAndNewlines).nilIfEmpty
        let resolvedNickname = nickname?.trimmingCharacters(in: .whitespacesAndNewlines).nilIfEmpty

        var merged = peersByKey

        var existing: DashboardPeer? =
            merged[normalizedCanonical] ??
            (normalizedLibp2p.flatMap { merged[$0] }) ??
            (normalizedBlePeer.flatMap { merged[$0] })

        if existing == nil {
            existing = merged.values.first(where: { peer in
                peer.peerId == normalizedCanonical ||
                    peer.libp2pPeerId == normalizedCanonical ||
                    peer.blePeerId == normalizedCanonical ||
                    (normalizedLibp2p != nil && peer.libp2pPeerId == normalizedLibp2p) ||
                    (normalizedBlePeer != nil && peer.blePeerId == normalizedBlePeer) ||
                    (resolvedPublicKey != nil && peer.publicKey == resolvedPublicKey) ||
                    (resolvedNickname != nil && peer.nickname == resolvedNickname)
            })
        }

        let relay = repository.isKnownRelay(normalizedCanonical)
            || (normalizedLibp2p.map(repository.isKnownRelay) ?? false)

        let resolvedLocNick = localNickname?.trimmingCharacters(in: .whitespacesAndNewlines).nilIfEmpty ?? existing?.localNickname

        let peer = DashboardPeer(
            id: existing?.id ?? normalizedCanonical, // Maintain identity primary key if possible
            peerId: existing?.peerId ?? normalizedCanonical,
            publicKey: resolvedPublicKey ?? existing?.publicKey,
            nickname: resolvedNickname ?? existing?.nickname,
            localNickname: resolvedLocNick,
            libp2pPeerId: normalizedLibp2p ?? existing?.libp2pPeerId,
            blePeerId: normalizedBlePeer ?? existing?.blePeerId,
            transport: transport,
            isOnline: isOnline,
            isRelay: relay,
            isFull: classifyPeerAsFull(
                peerId: existing?.peerId ?? normalizedCanonical,
                publicKey: resolvedPublicKey ?? existing?.publicKey,
                nickname: resolvedNickname ?? existing?.nickname,
                localNickname: resolvedLocNick,
                isRelay: relay
            ),
            lastSeen: Date()
        )

        if let oldId = existing?.id, oldId != peer.id {
            merged.removeValue(forKey: oldId)
        }
        if let oldLibp2p = normalizedLibp2p, oldLibp2p != peer.id {
            merged.removeValue(forKey: oldLibp2p)
        }
        if let oldBlePeer = normalizedBlePeer, oldBlePeer != peer.id {
            merged.removeValue(forKey: oldBlePeer)
        }
        
        merged[peer.id] = peer
        peersByKey = merged

        if localNickname == nil {
            // Trigger a re-sync with contacts to ensure canonical identity merges happen correctly
            refreshPeersFromRepository()
        }
    }

    private func markPeer(peerId: String, isOnline: Bool) {
        let normalized = peerId.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalized.isEmpty else { return }
        var merged = peersByKey

        if let existing = merged[normalized] {
            var updated = existing
            updated.isOnline = isOnline
            updated.lastSeen = Date()
            merged[normalized] = updated
            peersByKey = merged
            return
        }

        if let key = merged.first(where: { $0.value.libp2pPeerId == normalized || $0.value.blePeerId == normalized })?.key,
           let existing = merged[key] {
            var updated = existing
            updated.isOnline = isOnline
            updated.lastSeen = Date()
            merged[key] = updated
            peersByKey = merged
            return
        }

        upsertPeer(
            canonicalPeerId: normalized,
            publicKey: nil,
            nickname: nil,
            localNickname: nil,
            libp2pPeerId: nil,
            blePeerId: nil,
            transport: .unknown,
            isOnline: isOnline
        )
    }

    private func classifyPeerAsFull(
        peerId: String,
        publicKey: String?,
        nickname: String?,
        localNickname: String?,
        isRelay: Bool
    ) -> Bool {
        if isRelay { return false }
        if let key = publicKey?.trimmingCharacters(in: .whitespacesAndNewlines), !key.isEmpty { return true }
        let hasNickname = !(nickname?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ?? true)
        let hasLocalNickname = !(localNickname?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ?? true)
        if hasNickname || hasLocalNickname { return true }
        return isIdentityId(peerId)
    }

    private func parseRoutingLibp2pPeerId(from notes: String?) -> String? {
        guard let notes, !notes.isEmpty else { return nil }
        let segments = notes.split(whereSeparator: { $0 == ";" || $0 == "\n" })
        for segment in segments {
            let trimmed = segment.trimmingCharacters(in: .whitespacesAndNewlines)
            guard trimmed.hasPrefix("libp2p_peer_id:") else { continue }
            let value = trimmed.replacingOccurrences(of: "libp2p_peer_id:", with: "")
                .trimmingCharacters(in: .whitespacesAndNewlines)
            if !value.isEmpty {
                return value
            }
        }
        return nil
    }

    private func dateFromEpoch(_ epoch: UInt64?) -> Date? {
        guard let epoch else { return nil }
        return Date(timeIntervalSince1970: TimeInterval(epoch))
    }

    private func isRecent(_ epoch: UInt64?) -> Bool {
        guard let epoch else { return false }
        let now = UInt64(Date().timeIntervalSince1970)
        return epoch <= now && (now - epoch) < 300
    }

    private func isIdentityId(_ value: String) -> Bool {
        guard value.count == 64 else { return false }
        return value.unicodeScalars.allSatisfy { scalar in
            CharacterSet(charactersIn: "0123456789abcdefABCDEF").contains(scalar)
        }
    }

    private func transportFromMultiaddr(_ multiaddr: String) -> DashboardPeer.Transport {
        let trimmed = multiaddr.trimmingCharacters(in: .whitespacesAndNewlines)
        if trimmed.contains("/ble/") {
            return .ble
        }
        if trimmed.contains("/ip4/") || trimmed.contains("/ip6/") || trimmed.contains("/p2p-circuit/") {
            return .internet
        }
        return .unknown
    }
}

struct ServiceStatusCard: View {
    let stats: ServiceStats?

    var body: some View {
        VStack(alignment: .leading, spacing: Theme.spacingMedium) {
            HStack {
                Image(systemName: "network")
                    .font(.title2)
                Text("Service Status")
                    .font(Theme.titleLarge)
                Spacer()
                Circle()
                    .fill(stats != nil ? Color.green : Color.gray)
                    .frame(width: 12, height: 12)
            }

            if let stats = stats {
                Divider()

                StatRow(label: "Peers Discovered", value: "\(stats.peersDiscovered)")
                StatRow(label: "Messages Relayed", value: "\(stats.messagesRelayed)")
                StatRow(label: "Bytes Transferred", value: formatBytes(stats.bytesTransferred))
                StatRow(label: "Uptime", value: formatUptime(stats.uptimeSecs))
            } else {
                Text("Service not running")
                    .font(Theme.bodyMedium)
                    .foregroundStyle(Theme.onSurfaceVariant)
            }
        }
        .padding(Theme.spacingMedium)
        .themedCard()
    }

    private func formatBytes(_ bytes: UInt64) -> String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .binary
        return formatter.string(fromByteCount: Int64(bytes))
    }

    private func formatUptime(_ seconds: UInt64) -> String {
        let hours = seconds / 3600
        let minutes = (seconds % 3600) / 60
        return "\(hours)h \(minutes)m"
    }
}

struct DiscoveredNodesSection: View {
    let peers: [DashboardPeer]
    let fullPeers: Int
    let headlessPeers: Int

    var body: some View {
        VStack(alignment: .leading, spacing: Theme.spacingMedium) {
            Text("Nodes (\(fullPeers) Full / \(headlessPeers) Headless)")
                .font(Theme.titleLarge)

            if peers.isEmpty {
                Text("No nodes discovered yet. Check Relay status in Transport section.")
                    .font(Theme.bodyMedium)
                    .foregroundStyle(Theme.onSurfaceVariant)
            } else {
                ForEach(peers) { peer in
                    DashboardPeerRow(peer: peer)
                    if peer.id != peers.last?.id {
                        Divider()
                    }
                }
            }
        }
        .padding(Theme.spacingMedium)
        .themedCard()
    }
}

struct DashboardPeerRow: View {
    let peer: DashboardPeer

    var body: some View {
        HStack(spacing: Theme.spacingMedium) {
            Circle()
                .fill(peer.isOnline ? Theme.primaryContainer : Theme.surfaceVariant)
                .frame(width: 36, height: 36)
                .overlay {
                    Image(systemName: iconName)
                        .foregroundStyle(peer.isOnline ? Theme.onPrimaryContainer : Theme.onSurfaceVariant)
                }

            VStack(alignment: .leading, spacing: 2) {
                Text(peer.displayName)
                    .font(Theme.titleMedium)
                Text("ID: \(peer.peerId.prefix(12))... • \(peer.transport.rawValue) • \(peer.roleLabel)")
                    .font(.system(.caption, design: .monospaced))
                    .foregroundStyle(Theme.onSurfaceVariant)
            }

            Spacer()

            if peer.isOnline {
                Circle()
                    .fill(Color.green)
                    .frame(width: 8, height: 8)
            }
        }
    }

    private var iconName: String {
        if peer.isRelay { return "arrow.triangle.2.circlepath" }
        return peer.isFull ? "person.fill" : "person.2.fill"
    }
}

struct StatRow: View {
    let label: String
    let value: String

    var body: some View {
        HStack {
            Text(label)
                .font(Theme.bodyMedium)
            Spacer()
            Text(value)
                .font(Theme.titleMedium)
                .foregroundStyle(Theme.onPrimaryContainer)
        }
    }
}

struct TransportStatusSection: View {
    @Environment(MeshRepository.self) private var repository

    var body: some View {
        VStack(alignment: .leading, spacing: Theme.spacingMedium) {
            Text("Transports")
                .font(Theme.titleLarge)

            TransportStatusRow(type: .multipeer, isActive: true)
            TransportStatusRow(type: .ble, isActive: true)
            TransportStatusRow(type: .internet, isActive: repository.networkStatus.available)
        }
        .padding(Theme.spacingMedium)
        .themedCard()
    }
}

struct TransportStatusRow: View {
    let type: TransportType
    let isActive: Bool

    var body: some View {
        HStack(spacing: Theme.spacingMedium) {
            Image(systemName: type.icon)
                .font(.title3)
                .foregroundStyle(isActive ? Theme.onSuccessContainer : Theme.onSurfaceVariant)
                .frame(width: 30)

            Text(type.rawValue)
                .font(Theme.bodyMedium)

            Spacer()

            Circle()
                .fill(isActive ? Theme.onSuccessContainer : Color.gray)
                .frame(width: 8, height: 8)
        }
    }
}

struct RelayStatsSection: View {
    let stats: ServiceStats

    var body: some View {
        VStack(alignment: .leading, spacing: Theme.spacingMedium) {
            HStack {
                Image(systemName: "arrow.triangle.2.circlepath")
                    .font(.title2)
                Text("Relay Stats")
                    .font(Theme.titleLarge)
            }

            Divider()

            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Messages Relayed")
                        .font(Theme.bodySmall)
                        .foregroundStyle(Theme.onSurfaceVariant)
                    Text("\(stats.messagesRelayed)")
                        .font(Theme.headlineMedium)
                }

                Spacer()

                VStack(alignment: .trailing, spacing: 4) {
                    Text("Bytes Transferred")
                        .font(Theme.bodySmall)
                        .foregroundStyle(Theme.onSurfaceVariant)
                    Text(formatBytes(stats.bytesTransferred))
                        .font(Theme.headlineMedium)
                }
            }
        }
        .padding(Theme.spacingMedium)
        .themedCard()
    }

    private func formatBytes(_ bytes: UInt64) -> String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .binary
        return formatter.string(fromByteCount: Int64(bytes))
    }
}

private extension String {
    var nilIfEmpty: String? {
        isEmpty ? nil : self
    }
}
