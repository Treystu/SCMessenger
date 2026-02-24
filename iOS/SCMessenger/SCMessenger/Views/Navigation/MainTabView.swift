//
//  MainTabView.swift
//  SCMessenger
//
//  Main tab navigation
//

import SwiftUI

struct MainTabView: View {
    @Environment(MeshRepository.self) private var repository
    @AppStorage("hasCompletedOnboarding") private var hasCompletedOnboarding = false

    // Identity fail-safe alert state
    @State private var showIdentityAlert = false
    @State private var identityRecoveryError: String?

    var body: some View {
        TabView {
            NavigationStack {
                ConversationListView()
            }
            .tabItem {
                Label("Messages", systemImage: "message")
            }

            NavigationStack {
                ContactsListView()
            }
            .tabItem {
                Label("Contacts", systemImage: "person.2")
            }

            NavigationStack {
                MeshDashboardView()
            }
            .tabItem {
                Label("Mesh", systemImage: "network")
            }

            NavigationStack {
                SettingsView()
            }
            .tabItem {
                Label("Settings", systemImage: "gear")
            }
        }
        .alert("Identity Missing", isPresented: $showIdentityAlert) {
            Button("Re-create Identity") {
                do {
                    try repository.createIdentity()
                } catch {
                    identityRecoveryError = error.localizedDescription
                }
            }
            Button("Return to Setup", role: .destructive) {
                hasCompletedOnboarding = false
            }
            Button("Cancel", role: .cancel) { }
        } message: {
            Text("Your cryptographic identity could not be loaded. You can re-create it now (this will generate a new identity) or return to the setup screen.")
        }
        .alert("Recovery Failed", isPresented: Binding(
            get: { identityRecoveryError != nil },
            set: { if !$0 { identityRecoveryError = nil } }
        )) {
            Button("Return to Setup", role: .destructive) {
                hasCompletedOnboarding = false
            }
            Button("Cancel", role: .cancel) {
                identityRecoveryError = nil
            }
        } message: {
            if let err = identityRecoveryError {
                Text("Could not re-create identity: \(err)\n\nReturn to setup to start fresh.")
            }
        }
        .onAppear {
            let nickname = repository.getIdentityInfo()?.nickname?
                .trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
            if nickname.isEmpty {
                hasCompletedOnboarding = false
                return
            }

            repository.start()

            // Check if identity is truly available after service start
            if !repository.isIdentityInitialized() {
                showIdentityAlert = true
            }
        }
    }
}

struct ConversationListView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var conversations: [Conversation] = []
    
    var body: some View {
        List(conversations) { conversation in
            NavigationLink(value: conversation) {
                ConversationRow(conversation: conversation)
            }
        }
        .navigationTitle("Messages")
        .navigationDestination(for: Conversation.self) { conversation in
            ChatView(conversation: conversation)
        }
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button(action: {}) {
                    Image(systemName: "square.and.pencil")
                }
            }
        }
        .task {
            loadConversations()
        }
        .onReceive(repository.messageUpdates) { _ in
            loadConversations()
        }
        .onReceive(MeshEventBus.shared.peerEvents) { event in
            switch event {
            case .identityDiscovered:
                // Refresh display names when federated identity metadata changes.
                loadConversations()
            default:
                break
            }
        }
    }
    
    private func loadConversations() {
        // Load conversations from repository
        do {
            let contacts = try repository.getContacts()
            let deduped = deduplicateContactsByPublicKey(contacts)
            conversations = deduped.map { contact in
                let local = contact.localNickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
                let displayName = !local.isEmpty
                    ? local
                    : (contact.nickname ?? String(contact.peerId.prefix(8)) + "...")
                return Conversation(peerId: contact.peerId, peerNickname: displayName)
            }
        } catch {
            // Handle error
        }
    }

    private func deduplicateContactsByPublicKey(_ input: [Contact]) -> [Contact] {
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

struct ConversationRow: View {
    let conversation: Conversation
    
    var body: some View {
        HStack(spacing: Theme.spacingMedium) {
            Circle()
                .fill(Theme.primaryContainer)
                .frame(width: 50, height: 50)
                .overlay {
                    Text(conversation.peerNickname.prefix(1).uppercased())
                        .font(Theme.titleMedium)
                        .foregroundStyle(Theme.onPrimaryContainer)
                }
            
            VStack(alignment: .leading, spacing: 4) {
                Text(conversation.peerNickname)
                    .font(Theme.titleMedium)
                
                if let lastMessage = conversation.lastMessage {
                    Text(lastMessage)
                        .font(Theme.bodySmall)
                        .foregroundStyle(Theme.onSurfaceVariant)
                        .lineLimit(1)
                }
            }
            
            Spacer()
            
            if conversation.unreadCount > 0 {
                Text("\(conversation.unreadCount)")
                    .font(Theme.labelSmall)
                    .foregroundStyle(.white)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Theme.onPrimaryContainer)
                    .clipShape(Capsule())
            }
        }
        .padding(.vertical, 4)
    }
}

struct ChatView: View {
    @Environment(MeshRepository.self) private var repository
    let conversation: Conversation
    @State private var viewModel: ChatViewModel?
    
    var body: some View {
        VStack(spacing: 0) {
            ScrollView {
                LazyVStack {
                    ForEach(viewModel?.messages ?? [], id: \.id) { message in
                        MessageBubble(message: message)
                    }
                }
            }
            
            MessageInputBar(
                text: Binding(
                    get: { viewModel?.messageText ?? "" },
                    set: { viewModel?.messageText = $0 }
                ),
                isSending: viewModel?.isSending ?? false,
                onSend: {
                    Task {
                        await viewModel?.sendMessage()
                    }
                }
            )

            if let error = viewModel?.error {
                Text(error)
                    .foregroundStyle(.white)
                    .padding()
                    .background(.red)
                    .cornerRadius(8)
                    .padding()
                    .onTapGesture {
                        viewModel?.error = nil
                    }
            }
        }
        .navigationTitle(conversation.peerNickname)
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            if viewModel == nil {
                viewModel = ChatViewModel(conversation: conversation, repository: repository)
            }
        }
    }
}

struct MessageBubble: View {
    let message: MessageRecord
    
    private var isSent: Bool {
        message.direction == .sent
    }
    
    var body: some View {
        HStack {
            if isSent { Spacer() }
            
            Text(message.content)
                .font(Theme.bodyMedium)
                .padding(Theme.spacingMedium)
                .background(isSent ? Theme.primaryContainer : Theme.surfaceVariant)
                .foregroundStyle(isSent ? Theme.onPrimaryContainer : Theme.onSurface)
                .cornerRadius(Theme.cornerRadiusMedium)
            
            if !isSent { Spacer() }
        }
        .padding(.horizontal, Theme.spacingMedium)
        .padding(.vertical, Theme.spacingSmall)
    }
}

struct MessageInputBar: View {
    @Binding var text: String
    let isSending: Bool
    let onSend: () -> Void
    
    var body: some View {
        HStack(spacing: Theme.spacingSmall) {
            TextField("Message", text: $text, axis: .vertical)
                .textFieldStyle(.roundedBorder)
                .lineLimit(1...4)
            
            Button(action: onSend) {
                if isSending {
                    ProgressView()
                        .frame(width: 24, height: 24)
                } else {
                    Image(systemName: "arrow.up.circle.fill")
                        .font(.title2)
                        .foregroundStyle(text.isEmpty ? .gray : Theme.onPrimaryContainer)
                }
            }
            .disabled(text.isEmpty || isSending)
        }
        .padding(Theme.spacingMedium)
        .background(Theme.surface)
    }
}
