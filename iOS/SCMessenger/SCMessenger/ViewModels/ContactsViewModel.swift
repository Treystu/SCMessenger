//
//  ContactsViewModel.swift
//  SCMessenger
//
//  ViewModel for contacts management
//

import Combine
import Foundation

struct NearbyPeer: Identifiable, Equatable {
    let id: String   // == peerId
    let peerId: String
    let publicKey: String?
    let nickname: String?
    let blePeerId: String?
    let libp2pPeerId: String?
    let listeners: [String]
    let isOnline: Bool

    init(peerId: String, publicKey: String? = nil, nickname: String? = nil,
         blePeerId: String? = nil, libp2pPeerId: String? = nil, listeners: [String] = [],
         isOnline: Bool = true) {
        self.id = peerId
        self.peerId = peerId
        self.publicKey = publicKey
        self.nickname = nickname
        self.blePeerId = blePeerId
        self.libp2pPeerId = libp2pPeerId
        self.listeners = listeners
        self.isOnline = isOnline
    }

    var displayName: String { nickname?.isEmpty == false ? nickname! : String(peerId.prefix(16)) }
    var hasFullIdentity: Bool { publicKey != nil }

    static func == (lhs: NearbyPeer, rhs: NearbyPeer) -> Bool { lhs.peerId == rhs.peerId }
}

@Observable
final class ContactsViewModel {
    private weak var repository: MeshRepository?
    private var cancellables = Set<AnyCancellable>()
    private let nearbyDisconnectGraceSec: TimeInterval = 30
    private var pendingNearbyRemoval: [String: DispatchWorkItem] = [:]

    var contacts: [Contact] = []
    var searchText = ""
    var isLoading = false
    var error: String?

    /// Peers discovered on the mesh but not yet in the contacts list.
    var nearbyPeers: [NearbyPeer] = []

    var filteredContacts: [Contact] {
        if searchText.isEmpty {
            return contacts
        }
        return contacts.filter { contact in
            (contact.localNickname?.localizedCaseInsensitiveContains(searchText) ?? false) ||
            (contact.nickname?.localizedCaseInsensitiveContains(searchText) ?? false) ||
            contact.peerId.localizedCaseInsensitiveContains(searchText)
        }
    }

    init(repository: MeshRepository) {
        self.repository = repository
        subscribeToNearbyPeers()
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) { [weak self] in
            guard let self else { return }
            Task { @MainActor in
                self.repository?.replayDiscoveredPeerEvents()
            }
        }
    }

    private func normalizeNickname(_ nickname: String?) -> String? {
        let normalized = nickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        return normalized.isEmpty ? nil : normalized
    }

    private func isSyntheticFallbackNickname(_ nickname: String?) -> Bool {
        guard let normalized = normalizeNickname(nickname)?.lowercased() else { return false }
        return normalized.hasPrefix("peer-")
    }

    private func selectAuthoritativeNickname(incoming: String?, existing: String?) -> String? {
        let incomingNormalized = normalizeNickname(incoming)
        let existingNormalized = normalizeNickname(existing)

        let incomingSynthetic = isSyntheticFallbackNickname(incomingNormalized)
        let existingSynthetic = isSyntheticFallbackNickname(existingNormalized)
        if incomingNormalized == nil && existingSynthetic { return nil }
        if incomingNormalized == nil { return existingNormalized }
        if incomingSynthetic && existingNormalized == nil { return nil }
        if incomingSynthetic && existingSynthetic { return nil }
        if incomingSynthetic { return existingNormalized }
        if existingSynthetic { return incomingNormalized }
        return incomingNormalized
    }

    private func isLibp2pPeerId(_ value: String?) -> Bool {
        guard let normalized = value?.trimmingCharacters(in: .whitespacesAndNewlines),
              !normalized.isEmpty else { return false }
        return normalized.hasPrefix("12D3Koo") || normalized.hasPrefix("Qm")
    }

    private func isIdentityId(_ value: String?) -> Bool {
        guard let normalized = value?.trimmingCharacters(in: .whitespacesAndNewlines),
              normalized.count == 64 else { return false }
        return normalized.unicodeScalars.allSatisfy { scalar in
            CharacterSet(charactersIn: "0123456789abcdefABCDEF").contains(scalar)
        }
    }

    private func isBlePeerId(_ value: String?) -> Bool {
        guard let normalized = value?.trimmingCharacters(in: .whitespacesAndNewlines),
              !normalized.isEmpty else { return false }
        return UUID(uuidString: normalized) != nil
    }

    private func normalizedNonEmpty(_ value: String?) -> String? {
        guard let normalized = value?.trimmingCharacters(in: .whitespacesAndNewlines),
              !normalized.isEmpty else { return nil }
        return normalized
    }

    private func selectStablePeerId(incoming: String, existing: String?) -> String {
        let incomingId = incoming.trimmingCharacters(in: .whitespacesAndNewlines)
        let existingId = existing?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        if existingId.isEmpty || existingId == incomingId { return incomingId }

        let incomingIsLibp2p = isLibp2pPeerId(incomingId)
        let existingIsLibp2p = isLibp2pPeerId(existingId)
        let incomingIsIdentity = isIdentityId(incomingId)
        let existingIsIdentity = isIdentityId(existingId)
        let incomingIsBle = isBlePeerId(incomingId)
        let existingIsBle = isBlePeerId(existingId)

        if existingIsIdentity && incomingIsLibp2p { return existingId }
        if incomingIsIdentity && existingIsLibp2p { return incomingId }
        if existingIsBle && !incomingIsBle { return incomingId }
        if !existingIsBle && incomingIsBle { return existingId }
        return incomingId
    }

    private func isSameNearbyIdentity(
        _ peer: NearbyPeer,
        peerId: String,
        publicKey: String,
        libp2pPeerId: String?,
        blePeerId: String?
    ) -> Bool {
        let incomingPeerId = peerId.trimmingCharacters(in: .whitespacesAndNewlines)
        let incomingLibp2p = libp2pPeerId?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let incomingBle = blePeerId?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let peerLibp2p = peer.libp2pPeerId?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let peerBle = peer.blePeerId?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""

        let sameById =
            peer.peerId == incomingPeerId ||
            (!incomingLibp2p.isEmpty &&
                (peer.peerId == incomingLibp2p || peerLibp2p == incomingLibp2p)) ||
            (!incomingBle.isEmpty &&
                (peer.peerId == incomingBle || peerBle == incomingBle))

        let sameByKey =
            (peer.publicKey?.isEmpty == false) &&
            (peer.publicKey?.caseInsensitiveCompare(publicKey) == .orderedSame)

        return sameById || sameByKey
    }

    func loadContacts() {
        isLoading = true
        do {
            let rawContacts = try repository?.getContacts() ?? []
            contacts = deduplicateByPublicKey(rawContacts)
            error = nil
        } catch {
            self.error = error.localizedDescription
        }
        isLoading = false
        refreshNearbyFilter()
    }

    func addContact(_ contact: Contact) throws {
        try repository?.addContact(contact)
        loadContacts()
    }

    func removeContact(peerId: String) throws {
        try repository?.removeContact(peerId: peerId)
        loadContacts()
    }

    func setLocalNickname(peerId: String, nickname: String?) throws {
        try repository?.setLocalNickname(peerId: peerId, nickname: nickname)
        loadContacts()
    }

    func deleteContacts(at offsets: IndexSet) {
        for index in offsets {
            let contact = filteredContacts[index]
            try? removeContact(peerId: contact.peerId)
        }
    }

    // MARK: - Nearby Peers

    private func subscribeToNearbyPeers() {
        MeshEventBus.shared.peerEvents
            .receive(on: DispatchQueue.main)
            .sink { [weak self] event in
                guard let self else { return }
                switch event {
                case .identityDiscovered(let peerId, let publicKey, let nickname, let libp2pPeerId, let listeners, let blePeerId):
                    self.handleIdentityDiscovered(peerId: peerId, publicKey: publicKey,
                                                   nickname: nickname, libp2pPeerId: libp2pPeerId,
                                                   listeners: listeners, blePeerId: blePeerId)
                case .discovered(let peerId):
                    self.handleDiscovered(peerId: peerId)
                case .disconnected(let peerId):
                    self.handleDisconnected(peerId: peerId)
                default:
                    break
                }
            }
            .store(in: &cancellables)
    }

    private func isBootstrapRelayPeer(_ peerId: String) -> Bool {
        guard let repo = repository else { return false }
        return repo.isBootstrapRelayPeer(peerId)
    }

    private func handleIdentityDiscovered(peerId: String, publicKey: String, nickname: String?,
                                           libp2pPeerId: String?, listeners: [String], blePeerId: String?) {
        // Never surface bootstrap relay/headless nodes in the Contacts nearby list.
        let checkIds = [peerId, libp2pPeerId].compactMap { $0?.trimmingCharacters(in: .whitespacesAndNewlines) }.filter { !$0.isEmpty }
        if checkIds.contains(where: { isBootstrapRelayPeer($0) }) { return }

        cancelPendingNearbyRemoval(peerId: peerId)
        cancelPendingNearbyRemoval(peerId: libp2pPeerId)
        cancelPendingNearbyRemoval(peerId: blePeerId)

        let alreadySaved = contacts.contains {
            $0.peerId == peerId || $0.publicKey.caseInsensitiveCompare(publicKey) == .orderedSame
        }
        if alreadySaved {
            // Federated nickname/route hints can update in repository upsert;
            // refresh saved contacts so local UI reflects latest values.
            loadContacts()
            return
        }

        let matches = nearbyPeers.filter {
            isSameNearbyIdentity(
                $0,
                peerId: peerId,
                publicKey: publicKey,
                libp2pPeerId: libp2pPeerId,
                blePeerId: blePeerId
            )
        }
        let existing = matches.max { lhs, rhs in
            let lhsScore = (normalizeNickname(lhs.nickname) != nil ? 2 : 0) + (!isLibp2pPeerId(lhs.peerId) ? 1 : 0)
            let rhsScore = (normalizeNickname(rhs.nickname) != nil ? 2 : 0) + (!isLibp2pPeerId(rhs.peerId) ? 1 : 0)
            return lhsScore < rhsScore
        }
        if !matches.isEmpty {
            nearbyPeers.removeAll { candidate in
                isSameNearbyIdentity(
                    candidate,
                    peerId: peerId,
                    publicKey: publicKey,
                    libp2pPeerId: libp2pPeerId,
                    blePeerId: blePeerId
                )
            }
        }
        cancelPendingNearbyRemoval(peerId: existing?.peerId)

        let resolvedPeerId = selectStablePeerId(incoming: peerId, existing: existing?.peerId)
        let resolvedLibp2pPeerId =
            normalizedNonEmpty(libp2pPeerId) ??
            normalizedNonEmpty(existing?.libp2pPeerId) ??
            (isLibp2pPeerId(peerId) ? peerId : nil)
        let resolvedBlePeerId =
            normalizedNonEmpty(blePeerId) ??
            normalizedNonEmpty(existing?.blePeerId)

        let peer = NearbyPeer(peerId: resolvedPeerId, publicKey: publicKey,
                              nickname: selectAuthoritativeNickname(incoming: nickname, existing: existing?.nickname),
                              blePeerId: resolvedBlePeerId, libp2pPeerId: resolvedLibp2pPeerId,
                              listeners: listeners.isEmpty ? (existing?.listeners ?? []) : listeners,
                              isOnline: true)
        nearbyPeers.append(peer)
    }

    private func handleDiscovered(peerId: String) {
        cancelPendingNearbyRemoval(peerId: peerId)
        // Don't surface bootstrap relay nodes in the nearby list.
        guard !isBootstrapRelayPeer(peerId) else { return }
        let alreadySaved = contacts.contains { $0.peerId == peerId }
        guard !alreadySaved else { return }

        if let idx = nearbyPeers.firstIndex(where: { $0.peerId == peerId || $0.libp2pPeerId == peerId }) {
            let existing = nearbyPeers[idx]
            nearbyPeers[idx] = NearbyPeer(
                peerId: existing.peerId,
                publicKey: existing.publicKey,
                nickname: existing.nickname,
                blePeerId: existing.blePeerId,
                libp2pPeerId: existing.libp2pPeerId,
                listeners: existing.listeners,
                isOnline: true
            )
        } else {
            nearbyPeers.append(NearbyPeer(peerId: peerId, isOnline: true))
        }
    }

    private func handleDisconnected(peerId: String) {
        var changed = false
        for idx in nearbyPeers.indices {
            let peer = nearbyPeers[idx]
            if peer.peerId == peerId || peer.libp2pPeerId == peerId {
                if peer.isOnline {
                    nearbyPeers[idx] = NearbyPeer(
                        peerId: peer.peerId,
                        publicKey: peer.publicKey,
                        nickname: peer.nickname,
                        blePeerId: peer.blePeerId,
                        libp2pPeerId: peer.libp2pPeerId,
                        listeners: peer.listeners,
                        isOnline: false
                    )
                    changed = true
                }
            }
        }
        if changed {
            scheduleNearbyRemoval(peerId: peerId)
        }
    }

    private func cancelPendingNearbyRemoval(peerId: String?) {
        guard let peerId = peerId?.trimmingCharacters(in: .whitespacesAndNewlines),
              !peerId.isEmpty else { return }
        pendingNearbyRemoval.removeValue(forKey: peerId)?.cancel()
    }

    private func scheduleNearbyRemoval(peerId: String) {
        cancelPendingNearbyRemoval(peerId: peerId)
        let work = DispatchWorkItem { [weak self] in
            guard let self else { return }
            self.nearbyPeers.removeAll {
                ($0.peerId == peerId || $0.libp2pPeerId == peerId) && !$0.isOnline
            }
            self.pendingNearbyRemoval.removeValue(forKey: peerId)
        }
        pendingNearbyRemoval[peerId] = work
        DispatchQueue.main.asyncAfter(deadline: .now() + nearbyDisconnectGraceSec, execute: work)
    }

    /// Called after contacts change to drop any nearby entry that became a contact.
    private func refreshNearbyFilter() {
        let contactIds = Set(contacts.map { $0.peerId })
        nearbyPeers.removeAll { contactIds.contains($0.peerId) }
    }

    private func deduplicateByPublicKey(_ input: [Contact]) -> [Contact] {
        var byKey: [String: Contact] = [:]
        var passthrough: [Contact] = []

        for contact in input {
            let normalizedKey = contact.publicKey.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
            guard normalizedKey.count == 64 else {
                passthrough.append(contact)
                continue
            }

            if let current = byKey[normalizedKey] {
                byKey[normalizedKey] = preferredContact(current, contact)
            } else {
                byKey[normalizedKey] = contact
            }
        }

        return Array(byKey.values) + passthrough
    }

    private func preferredContact(_ a: Contact, _ b: Contact) -> Contact {
        let aLocal = a.localNickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let bLocal = b.localNickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let aName = !aLocal.isEmpty ? aLocal : (a.nickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? "")
        let bName = !bLocal.isEmpty ? bLocal : (b.nickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? "")

        if !aName.isEmpty && bName.isEmpty { return a }
        if !bName.isEmpty && aName.isEmpty { return b }
        if a.peerId.hasPrefix("12D3Koo") && !b.peerId.hasPrefix("12D3Koo") { return a }
        if b.peerId.hasPrefix("12D3Koo") && !a.peerId.hasPrefix("12D3Koo") { return b }
        return a
    }
}
