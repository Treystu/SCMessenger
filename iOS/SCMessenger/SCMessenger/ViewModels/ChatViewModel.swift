//
//  ChatViewModel.swift
//  SCMessenger
//
//  ViewModel for chat/messaging
//

import Foundation
import Combine

@Observable
final class ChatViewModel {
    private weak var repository: MeshRepository?
    private var cancellables = Set<AnyCancellable>()
    
    let conversation: Conversation
    var messages: [MessageRecord] = []
    var messageText = ""
    var isSending = false
    var error: String?
    
    init(conversation: Conversation, repository: MeshRepository) {
        self.conversation = conversation
        self.repository = repository
        loadMessages()
        subscribeToNewMessages()
    }
    
    func loadMessages() {
        do {
            messages = try repository?.getConversation(peerId: conversation.peerId) ?? []
        } catch {
            self.error = error.localizedDescription
        }
    }
    
    func sendMessage() async {
        guard !messageText.isEmpty else { return }
        
        let content = messageText
        messageText = ""
        isSending = true
        
        do {
            try await repository?.sendMessage(peerId: conversation.peerId, content: content)
            loadMessages()
            error = nil
        } catch {
            self.error = error.localizedDescription
            messageText = content // Restore text on error
        }
        
        isSending = false
    }
    
    private func subscribeToNewMessages() {
        repository?.incomingMessages
            .filter { [weak self] message in
                message.peerId == self?.conversation.peerId
            }
            .sink { [weak self] _ in
                self?.loadMessages()
            }
            .store(in: &cancellables)
    }
}
