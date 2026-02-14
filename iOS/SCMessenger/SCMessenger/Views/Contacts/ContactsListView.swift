//
//  ContactsListView.swift
//  SCMessenger
//
//  Contacts list view
//

import SwiftUI

struct ContactsListView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var viewModel: ContactsViewModel?
    @State private var showingAddContact = false
    
    var body: some View {
        List {
            ForEach(viewModel?.filteredContacts ?? [], id: \.peerId) { contact in
                ContactRow(contact: contact)
            }
            .onDelete { offsets in
                viewModel?.deleteContacts(at: offsets)
            }
        }
        .searchable(text: Binding(
            get: { viewModel?.searchText ?? "" },
            set: { viewModel?.searchText = $0 }
        ))
        .navigationTitle("Contacts")
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button {
                    showingAddContact = true
                } label: {
                    Image(systemName: "plus")
                }
            }
        }
        .sheet(isPresented: $showingAddContact) {
            AddContactView()
        }
        .onAppear {
            if viewModel == nil {
                viewModel = ContactsViewModel(repository: repository)
                viewModel?.loadContacts()
            }
        }
    }
}

struct ContactRow: View {
    let contact: Contact
    
    var body: some View {
        HStack(spacing: Theme.spacingMedium) {
            Circle()
                .fill(Theme.primaryContainer)
                .frame(width: 44, height: 44)
                .overlay {
                    Text((contact.nickname ?? "?").prefix(1).uppercased())
                        .font(Theme.titleMedium)
                        .foregroundStyle(Theme.onPrimaryContainer)
                }
            
            VStack(alignment: .leading, spacing: 4) {
                Text(contact.nickname ?? "Unknown")
                    .font(Theme.titleMedium)
                
                Text(contact.peerId.prefix(8))
                    .font(.system(.caption, design: .monospaced))
                    .foregroundStyle(Theme.onSurfaceVariant)
            }
        }
        .padding(.vertical, 4)
    }
}

struct AddContactView: View {
    @Environment(\.dismiss) private var dismiss
    @Environment(MeshRepository.self) private var repository
    
    @State private var nickname = ""
    @State private var publicKey = ""
    @State private var error: String?
    
    var body: some View {
        NavigationStack {
            Form {
                Section("Contact Information") {
                    TextField("Nickname", text: $nickname)
                    TextField("Public Key", text: $publicKey)
                        .font(.system(.body, design: .monospaced))
                }
                
                if let error = error {
                    Section {
                        Text(error)
                            .foregroundStyle(.red)
                            .font(Theme.bodySmall)
                    }
                }
                
                Section {
                    Button("Add Contact") {
                        addContact()
                    }
                    .disabled(nickname.isEmpty || publicKey.isEmpty)
                }
            }
            .navigationTitle("Add Contact")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
            }
        }
    }
    
    private func addContact() {
        let contact = Contact(
            peerId: publicKey.prefix(16).description,
            nickname: nickname,
            publicKey: publicKey,
            addedAt: UInt64(Date().timeIntervalSince1970),
            lastSeen: nil,
            notes: nil
        )
        
        do {
            try repository.addContact(contact)
            dismiss()
        } catch {
            self.error = error.localizedDescription
        }
    }
}
