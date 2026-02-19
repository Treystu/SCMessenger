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
    @State private var pendingChatConversation: Conversation?
    @State private var navigateToPendingChat = false

    var body: some View {
        List {
            ForEach(viewModel?.filteredContacts ?? [], id: \.peerId) { contact in
                NavigationLink(value: Conversation(peerId: contact.peerId, peerNickname: contact.nickname ?? "Unknown")) {
                    ContactRow(contact: contact)
                }
            }
            .onDelete { offsets in
                viewModel?.deleteContacts(at: offsets)
            }
        }
        .navigationDestination(for: Conversation.self) { conversation in
            ChatView(conversation: conversation)
        }
        .navigationDestination(isPresented: $navigateToPendingChat) {
            if let conversation = pendingChatConversation {
                ChatView(conversation: conversation)
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
        .sheet(isPresented: $showingAddContact, onDismiss: {
            viewModel?.loadContacts()
            if pendingChatConversation != nil {
                navigateToPendingChat = true
            }
        }) {
            AddContactView(pendingChatConversation: $pendingChatConversation)
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
    @Binding var pendingChatConversation: Conversation?

    @State private var nickname = ""
    @State private var publicKey = ""
    @State private var peerId = ""
    @State private var listeners: [String] = []
    @State private var libp2pPeerId: String = ""
    @State private var error: String?

    var body: some View {
        NavigationStack {
            Form {
                Section("Contact Information") {
                    Button(action: pasteIdentity) {
                        Label("Paste Identity Export", systemImage: "doc.on.clipboard")
                    }

                    TextField("Nickname", text: $nickname)
                    TextField("Public Key", text: $publicKey)
                        .font(.system(.body, design: .monospaced))
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()

                    if !peerId.isEmpty {
                        Text("ID: \(peerId)")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
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
                        addContact(andChat: false)
                    }
                    .disabled(nickname.isEmpty || publicKey.isEmpty)

                    Button("Add & Chat") {
                        addContact(andChat: true)
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

    private func pasteIdentity() {
        guard let string = UIPasteboard.general.string else { return }

        // Simple JSON parsing
        guard let data = string.data(using: .utf8),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            error = "Invalid format"
            return
        }

        if let key = json["public_key"] as? String { publicKey = key.trimmingCharacters(in: .whitespacesAndNewlines) }
        if let nick = json["nickname"] as? String { nickname = nick.trimmingCharacters(in: .whitespacesAndNewlines) }
        if let pid = json["identity_id"] as? String { peerId = pid.trimmingCharacters(in: .whitespacesAndNewlines) }
        if let lpid = json["libp2p_peer_id"] as? String, !lpid.isEmpty { libp2pPeerId = lpid.trimmingCharacters(in: .whitespacesAndNewlines) }
        if let list = json["listeners"] as? [String] {
            listeners = list.map { $0.replacingOccurrences(of: " (Potential)", with: "") }
        }

        error = nil
    }

    private func addContact(andChat: Bool) {
        let finalPublicKey = publicKey.trimmingCharacters(in: .whitespacesAndNewlines)
        var finalPeerId = peerId.trimmingCharacters(in: .whitespacesAndNewlines)
        if finalPeerId.isEmpty { finalPeerId = String(finalPublicKey.prefix(16)) }

        // Validate public key format before storing
        if finalPublicKey.isEmpty {
            self.error = "Public key cannot be empty"
            return
        }

        // Must be exactly 64 hex characters (32 bytes)
        if finalPublicKey.count != 64 {
            self.error = "Public key must be exactly 64 hex characters (got \(finalPublicKey.count))"
            return
        }

        // Must be valid hex
        let hexCharacterSet = CharacterSet(charactersIn: "0123456789abcdefABCDEF")
        if !finalPublicKey.unicodeScalars.allSatisfy({ hexCharacterSet.contains($0) }) {
            self.error = "Public key contains invalid characters (must be hex: 0-9, a-f)"
            return
        }

        // Store libp2p PeerId in notes for use in connectToPeer/sendMessage
        let notesValue: String? = libp2pPeerId.isEmpty ? nil : libp2pPeerId
        let contact = Contact(
            peerId: finalPeerId,
            nickname: nickname,
            publicKey: finalPublicKey,
            addedAt: UInt64(Date().timeIntervalSince1970),
            lastSeen: nil,
            notes: notesValue
        )

        do {
            try repository.addContact(contact)

            // Initiate connection if listeners provided.
            // Use libp2p PeerId for the /p2p/ suffix if we have it — enables proper peer verification.
            if !listeners.isEmpty {
                let peerIdForDial = libp2pPeerId.isEmpty ? finalPeerId : libp2pPeerId
                repository.connectToPeer(peerIdForDial, addresses: listeners)
            }

            if andChat {
                pendingChatConversation = Conversation(peerId: finalPeerId, peerNickname: nickname)
            } else {
                pendingChatConversation = nil
            }

            dismiss()
        } catch {
            self.error = error.localizedDescription
        }
    }
}
