//
//  ContactsViewModel.swift
//  SCMessenger
//
//  ViewModel for contacts management
//

import Foundation

@Observable
final class ContactsViewModel {
    private weak var repository: MeshRepository?
    
    var contacts: [Contact] = []
    var searchText = ""
    var isLoading = false
    var error: String?
    
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
}
