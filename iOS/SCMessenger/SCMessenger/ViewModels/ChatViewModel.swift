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
            // loadMessages() triggered automatically via subscription
            error = nil
        } catch let error as IronCoreError {
            // Extract detailed message from IronCoreError
            switch error {
            case .CryptoError(let message):
                self.error = "Crypto Error: \(message)"
            case .NetworkError(let message):
                self.error = "Network Error: \(message)"
            case .StorageError(let message):
                self.error = "Storage Error: \(message)"
            case .NotInitialized(let message):
                self.error = "Not Initialized: \(message)"
            case .InvalidInput(_):
                self.error = "Could not encrypt message — this contact may have an invalid public key. Try re-adding them using their identity export."
            case .Internal(let message):
                self.error = "Internal Error: \(message)"
            case .AlreadyRunning(let message):
                self.error = "Already Running: \(message)"
            @unknown default:
                self.error = "Unknown IronCore Error"
            }
            messageText = content // Restore text on error
        } catch {
            self.error = error.localizedDescription
            messageText = content // Restore text on error
        }
        
        isSending = false
    }
    
    private func subscribeToNewMessages() {
        repository?.messageUpdates
            .filter { [weak self] message in
                message.peerId == self?.conversation.peerId
            }
            .sink { [weak self] _ in
                // Reload messages on any update (sent or received)
                self?.loadMessages()
            }
            .store(in: &cancellables)
    }
}
