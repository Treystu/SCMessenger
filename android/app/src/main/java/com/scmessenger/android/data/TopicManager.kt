package com.scmessenger.android.data

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import timber.log.Timber
import uniffi.api.*

/**
 * TopicManager maps gossipsub topics to Android UI.
 * 
 * Features:
 * - Subscribe to default topics on start
 * - Topic list from SwarmHandle.get_topics() + LedgerManager.all_known_topics()
 * - Auto-subscribe to peer topics
 * - Topic-based message filtering
 * 
 * Default Topics:
 * - /scmessenger/global/v1 (global mesh chat)
 * - /scmessenger/discovery/v1 (peer announcements)
 * - /scmessenger/relay/v1 (message relaying)
 */
class TopicManager(
    private val meshRepository: MeshRepository
) {
    
    // Default topics
    companion object {
        const val TOPIC_GLOBAL = "/scmessenger/global/v1"
        const val TOPIC_DISCOVERY = "/scmessenger/discovery/v1"
        const val TOPIC_RELAY = "/scmessenger/relay/v1"
        
        val DEFAULT_TOPICS = listOf(
            TOPIC_GLOBAL,
            TOPIC_DISCOVERY,
            TOPIC_RELAY
        )
    }
    
    // Subscribed topics
    private val _subscribedTopics = MutableStateFlow<Set<String>>(emptySet())
    val subscribedTopics: StateFlow<Set<String>> = _subscribedTopics.asStateFlow()
    
    // All known topics (from SwarmHandle + Ledger)
    private val _knownTopics = MutableStateFlow<Set<String>>(emptySet())
    val knownTopics: StateFlow<Set<String>> = _knownTopics.asStateFlow()
    
    /**
     * Initialize and subscribe to default topics.
     */
    fun initialize() {
        Timber.d("Initializing TopicManager")
        
        // Subscribe to default topics
        DEFAULT_TOPICS.forEach { topic ->
            subscribe(topic)
        }
        
        // Load known topics from ledger
        refreshKnownTopics()
    }
    
    /**
     * Subscribe to a topic via SwarmHandle.
     */
    fun subscribe(topic: String) {
        try {
            // TODO: Call SwarmHandle.subscribe(topic) via SwarmBridge
            // For now, just track locally
            val current = _subscribedTopics.value.toMutableSet()
            current.add(topic)
            _subscribedTopics.value = current
            
            Timber.i("Subscribed to topic: $topic")
        } catch (e: Exception) {
            Timber.e(e, "Failed to subscribe to topic: $topic")
        }
    }
    
    /**
     * Unsubscribe from a topic.
     */
    fun unsubscribe(topic: String) {
        try {
            // TODO: Call SwarmHandle.unsubscribe(topic) via SwarmBridge
            val current = _subscribedTopics.value.toMutableSet()
            current.remove(topic)
            _subscribedTopics.value = current
            
            Timber.i("Unsubscribed from topic: $topic")
        } catch (e: Exception) {
            Timber.e(e, "Failed to unsubscribe from topic: $topic")
        }
    }
    
    /**
     * Refresh known topics from SwarmHandle and LedgerManager.
     */
    fun refreshKnownTopics() {
        try {
            val topics = mutableSetOf<String>()
            
            // Add subscribed topics
            topics.addAll(_subscribedTopics.value)
            
            // TODO: Add topics from SwarmHandle.get_topics() via SwarmBridge
            // TODO: Add topics from LedgerManager.all_known_topics()
            
            _knownTopics.value = topics
            
            Timber.d("Known topics refreshed: ${topics.size} topics")
        } catch (e: Exception) {
            Timber.e(e, "Failed to refresh known topics")
        }
    }
    
    /**
     * Auto-subscribe to peer topics when discovering new peers.
     */
    fun autoSubscribeToPeerTopics(peerId: String) {
        // Generate peer topic pattern: /scmessenger/peer/{peerId}/v1
        val peerTopic = "/scmessenger/peer/$peerId/v1"
        
        if (!_subscribedTopics.value.contains(peerTopic)) {
            subscribe(peerTopic)
            Timber.d("Auto-subscribed to peer topic: $peerTopic")
        }
    }
    
    /**
     * Filter messages by topic.
     */
    fun filterMessagesByTopic(messages: List<uniffi.api.MessageRecord>, topic: String): List<uniffi.api.MessageRecord> {
        // TODO: Add topic field to MessageRecord
        // For now, return all messages
        return messages
    }
    
    /**
     * Check if subscribed to a topic.
     */
    fun isSubscribed(topic: String): Boolean {
        return _subscribedTopics.value.contains(topic)
    }
    
    /**
     * Get all subscribed topics as a list.
     */
    fun getSubscribedTopicsList(): List<String> {
        return _subscribedTopics.value.toList()
    }
    
    /**
     * Get all known topics as a list.
     */
    fun getKnownTopicsList(): List<String> {
        return _knownTopics.value.toList()
    }
}
