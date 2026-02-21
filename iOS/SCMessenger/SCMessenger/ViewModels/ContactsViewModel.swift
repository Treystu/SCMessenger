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
    let libp2pPeerId: String?
    let listeners: [String]

    init(peerId: String, publicKey: String? = nil, nickname: String? = nil,
         libp2pPeerId: String? = nil, listeners: [String] = []) {
        self.id = peerId
        self.peerId = peerId
        self.publicKey = publicKey
        self.nickname = nickname
        self.libp2pPeerId = libp2pPeerId
        self.listeners = listeners
    }

    var displayName: String { nickname?.isEmpty == false ? nickname! : String(peerId.prefix(16)) }
    var hasFullIdentity: Bool { publicKey != nil }

    static func == (lhs: NearbyPeer, rhs: NearbyPeer) -> Bool { lhs.peerId == rhs.peerId }
}

@Observable
final class ContactsViewModel {
    private weak var repository: MeshRepository?
    private var cancellables = Set<AnyCancellable>()

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
            contacts = try repository?.getContacts() ?? []
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
                    self.nearbyPeers.removeAll { $0.peerId == peerId }
                default:
                    break
                }
            }
            .store(in: &cancellables)
    }

    private func handleIdentityDiscovered(peerId: String, publicKey: String, nickname: String?,
                                           libp2pPeerId: String?, listeners: [String], blePeerId: String?) {
        let alreadySaved = contacts.contains { $0.peerId == peerId }
        guard !alreadySaved else { return }

        if let bleId = blePeerId, bleId != peerId {
            nearbyPeers.removeAll { $0.peerId == bleId }
        }

        let peer = NearbyPeer(peerId: peerId, publicKey: publicKey, nickname: nickname,
                              libp2pPeerId: libp2pPeerId, listeners: listeners)
        if let idx = nearbyPeers.firstIndex(where: { $0.peerId == peerId }) {
            nearbyPeers[idx] = peer
        } else {
            nearbyPeers.append(peer)
        }
    }

    private func handleDiscovered(peerId: String) {
        let alreadySaved = contacts.contains { $0.peerId == peerId }
        guard !alreadySaved && !nearbyPeers.contains(where: { $0.peerId == peerId }) else { return }
        nearbyPeers.append(NearbyPeer(peerId: peerId))
    }

    /// Called after contacts change to drop any nearby entry that became a contact.
    private func refreshNearbyFilter() {
        let contactIds = Set(contacts.map { $0.peerId })
        nearbyPeers.removeAll { contactIds.contains($0.peerId) }
    }
}
