//
//  MainTabView.swift
//  SCMessenger
//
//  Main tab navigation
//

import SwiftUI

struct MainTabView: View {
    @Environment(MeshRepository.self) private var repository
    
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
    }
    
    private func loadConversations() {
        // Load conversations from repository
        do {
            let contacts = try repository.getContacts()
            conversations = contacts.map { contact in
                Conversation(peerId: contact.peerId, peerNickname: contact.nickname)
            }
        } catch {
            // Handle error
        }
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
