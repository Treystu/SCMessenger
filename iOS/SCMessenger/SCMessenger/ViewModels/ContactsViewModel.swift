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
            (contact.nickname?.localizedCaseInsensitiveContains(searchText) ?? false) ||
            contact.peerId.localizedCaseInsensitiveContains(searchText)
        }
    }

    init(repository: MeshRepository) {
        self.repository = repository
        subscribeToNearbyPeers()
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

    private func handleIdentityDiscovered(peerId: String, publicKey: String, nickname: String?,
                                           libp2pPeerId: String?, listeners: [String], blePeerId: String?) {
        cancelPendingNearbyRemoval(peerId: peerId)
        cancelPendingNearbyRemoval(peerId: libp2pPeerId)
        cancelPendingNearbyRemoval(peerId: blePeerId)

        let alreadySaved = contacts.contains {
            $0.peerId == peerId || $0.publicKey.caseInsensitiveCompare(publicKey) == .orderedSame
        }
        guard !alreadySaved else { return }

        if let bleId = blePeerId, bleId != peerId {
            nearbyPeers.removeAll { $0.peerId == bleId }
        }
        nearbyPeers.removeAll {
            $0.peerId != peerId &&
            ($0.publicKey?.caseInsensitiveCompare(publicKey) ?? .orderedDescending) == .orderedSame
        }

        let existing = nearbyPeers.first { $0.peerId == peerId }
        let peer = NearbyPeer(peerId: peerId, publicKey: publicKey, nickname: nickname ?? existing?.nickname,
                              blePeerId: blePeerId, libp2pPeerId: libp2pPeerId,
                              listeners: listeners.isEmpty ? (existing?.listeners ?? []) : listeners,
                              isOnline: true)
        if let idx = nearbyPeers.firstIndex(where: { $0.peerId == peerId }) {
            nearbyPeers[idx] = peer
        } else {
            nearbyPeers.append(peer)
        }
    }

    private func handleDiscovered(peerId: String) {
        cancelPendingNearbyRemoval(peerId: peerId)
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
        let aName = a.nickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let bName = b.nickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""

        if !aName.isEmpty && bName.isEmpty { return a }
        if !bName.isEmpty && aName.isEmpty { return b }
        if a.peerId.hasPrefix("12D3Koo") && !b.peerId.hasPrefix("12D3Koo") { return a }
        if b.peerId.hasPrefix("12D3Koo") && !a.peerId.hasPrefix("12D3Koo") { return b }
        return a
    }
}
