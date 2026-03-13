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

    func sendMessage() {
        let content = messageText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !content.isEmpty else { return }

        messageText = ""
        error = nil

        let now = UInt64(Date().timeIntervalSince1970)
        let optimisticMessage = MessageRecord(
            id: UUID().uuidString,
            direction: .sent,
            peerId: conversation.peerId,
            content: content,
            timestamp: now,
            senderTimestamp: now,
            delivered: false
        )
        messages.append(optimisticMessage)
        messages.sort { a, b in
            let t1 = a.senderTimestamp > 0 ? a.senderTimestamp : a.timestamp
            let t2 = b.senderTimestamp > 0 ? b.senderTimestamp : b.timestamp
            if t1 == t2 { return a.timestamp < b.timestamp }
            return t1 < t2
        }

        Task { @MainActor [weak self] in
            guard let self else { return }

            do {
                try await repository?.sendMessage(peerId: conversation.peerId, content: content)
                self.error = nil
            } catch let error as IronCoreError {
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
                self.loadMessages()
            } catch {
                self.error = error.localizedDescription
                self.loadMessages()
            }
        }
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
