//
//  TopicManager.swift
//  SCMessenger
//
//  Gossipsub topic management
//

import Foundation
import os

/// Manages Gossipsub topic subscriptions and publishing
@Observable
final class TopicManager {
    private let logger = Logger(subsystem: "com.scmessenger", category: "Topics")
    private weak var meshRepository: MeshRepository?
    
    private(set) var subscribedTopics: Set<String> = []
    
    init(meshRepository: MeshRepository) {
        self.meshRepository = meshRepository
        self.subscribedTopics = Set(meshRepository.getTopics())
    }
    
    // MARK: - Topic Management
    
    func subscribe(to topic: String) throws {
        logger.info("Subscribing to topic: \(topic)")
        
        // Validate topic name
        guard !topic.isEmpty else {
            throw TopicError.invalidTopic("Topic name cannot be empty")
        }
        
        guard let meshRepository = meshRepository else {
            throw TopicError.publishFailed("Mesh repository unavailable")
        }
        try meshRepository.subscribeTopic(topic)
        subscribedTopics.insert(topic)
        
        logger.info("✓ Subscribed to topic: \(topic)")
    }
    
    func unsubscribe(from topic: String) throws {
        logger.info("Unsubscribing from topic: \(topic)")
        
        guard let meshRepository = meshRepository else {
            throw TopicError.publishFailed("Mesh repository unavailable")
        }
        try meshRepository.unsubscribeTopic(topic)
        subscribedTopics.remove(topic)
        
        logger.info("✓ Unsubscribed from topic: \(topic)")
    }
    
    func publish(to topic: String, data: Data) throws {
        logger.info("Publishing to topic: \(topic) (\(data.count) bytes)")
        
        guard subscribedTopics.contains(topic) else {
            throw TopicError.notSubscribed("Not subscribed to topic: \(topic)")
        }
        
        guard let meshRepository = meshRepository else {
            throw TopicError.publishFailed("Mesh repository unavailable")
        }
        try meshRepository.publishTopic(topic, data: data)
        
        logger.debug("✓ Published to topic: \(topic)")
    }
    
    func listTopics() -> [String] {
        Array(subscribedTopics).sorted()
    }
    
    func isSubscribed(to topic: String) -> Bool {
        subscribedTopics.contains(topic)
    }
}

// MARK: - Error Types

enum TopicError: LocalizedError {
    case invalidTopic(String)
    case notSubscribed(String)
    case publishFailed(String)
    
    var errorDescription: String? {
        switch self {
        case .invalidTopic(let msg): return msg
        case .notSubscribed(let msg): return msg
        case .publishFailed(let msg): return msg
        }
    }
}
