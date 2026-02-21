//
//  ContactsViewModel.swift
//  SCMessenger
//
//  ViewModel for contacts management
//

import Combine
import Foundation

@Observable
final class ContactsViewModel {
    private weak var repository: MeshRepository?
    private var cancellables = Set<AnyCancellable>()

    var contacts: [Contact] = []
    var searchText = ""
    var isLoading = false
    var error: String?

    /// Peer IDs discovered on the mesh but not yet in the contacts list.
    /// These come from the MeshEventBus peer-discovered events.
    var nearbyPeers: [String] = []

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
                case .discovered(let peerId):
                    self.handleDiscovered(peerId: peerId)
                case .disconnected(let peerId):
                    self.nearbyPeers.removeAll { $0 == peerId }
                default:
                    break
                }
            }
            .store(in: &cancellables)
    }

    private func handleDiscovered(peerId: String) {
        // Only show in "Nearby" if not already a saved contact
        let alreadySaved = contacts.contains { $0.peerId == peerId }
        if !alreadySaved && !nearbyPeers.contains(peerId) {
            nearbyPeers.append(peerId)
        }
    }

    /// Called after contacts change to drop any nearby entry that became a contact.
    private func refreshNearbyFilter() {
        let contactIds = Set(contacts.map { $0.peerId })
        nearbyPeers.removeAll { contactIds.contains($0) }
    }
}
