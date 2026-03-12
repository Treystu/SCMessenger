//
//  ChatViewModel.swift
//  SCMessenger
//
//  ViewModel for chat/messaging
//

import Foundation
import Combine

@MainActor
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
            let fetched = try repository?.getConversation(peerId: conversation.peerId) ?? []
            messages = fetched.sorted(by: { a, b in
                let t1 = a.senderTimestamp > 0 ? a.senderTimestamp : a.timestamp
                let t2 = b.senderTimestamp > 0 ? b.senderTimestamp : b.timestamp
                if t1 == t2 { return a.timestamp < b.timestamp }
                return t1 < t2
            })
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

    private var reloadDebounceTask: Task<Void, Never>?

    private func subscribeToNewMessages() {
        repository?.messageUpdates
            .filter { [weak self] message in
                message.peerId == self?.conversation.peerId
            }
            .sink { [weak self] _ in
                // Debounce: cancel any pending reload, schedule a new one 80ms out
                self?.reloadDebounceTask?.cancel()
                self?.reloadDebounceTask = Task { @MainActor [weak self] in
                    try? await Task.sleep(nanoseconds: 80_000_000)
                    guard !Task.isCancelled else { return }
                    self?.loadMessages()
                }
            }
            .store(in: &cancellables)
    }
}
