package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import timber.log.Timber
import uniffi.api.*
import javax.inject.Inject

/**
 * ViewModel for the conversations/chat list screen.
 * 
 * Manages message history and conversation threads.
 */
@HiltViewModel
class ConversationsViewModel @Inject constructor(
    private val meshRepository: MeshRepository
) : ViewModel() {
    
    // Recent messages
    private val _messages = MutableStateFlow<List<uniffi.api.MessageRecord>>(emptyList())
    val messages: StateFlow<List<uniffi.api.MessageRecord>> = _messages.asStateFlow()
    
    // Grouped conversations (by peer)
    val conversations = messages.map { messageList ->
        messageList
            .groupBy { it.peerId }
            .mapValues { (_, msgs) -> msgs.sortedByDescending { it.timestamp } }
            .toList()
            .sortedByDescending { (_, msgs) -> msgs.firstOrNull()?.timestamp ?: 0u }
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = emptyList()
    )
    
    // Loading state
    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()
    
    // Error state
    private val _error = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = _error.asStateFlow()
    
    // Stats
    private val _stats = MutableStateFlow<uniffi.api.HistoryStats?>(null)
    val stats: StateFlow<uniffi.api.HistoryStats?> = _stats.asStateFlow()
    
    init {
        loadMessages()
        loadStats()
    }
    
    /**
     * Load recent messages.
     */
    fun loadMessages(limit: UInt = 100u) {
        viewModelScope.launch {
            try {
                _isLoading.value = true
                _error.value = null
                
                val messageList = meshRepository.getRecentMessages(limit = limit)
                _messages.value = messageList
                
                Timber.d("Loaded ${messageList.size} messages")
            } catch (e: Exception) {
                _error.value = "Failed to load messages: ${e.message}"
                Timber.e(e, "Failed to load messages")
            } finally {
                _isLoading.value = false
            }
        }
    }
    
    /**
     * Load conversation with specific peer.
     */
    fun loadConversation(peerId: String, limit: UInt = 100u): Flow<List<uniffi.api.MessageRecord>> {
        return flow {
            try {
                val messages = meshRepository.getConversation(peerId, limit)
                emit(messages)
                Timber.d("Loaded ${messages.size} messages for $peerId")
            } catch (e: Exception) {
                Timber.e(e, "Failed to load conversation")
                emit(emptyList())
            }
        }
    }
    
    /**
     * Send a message to a peer.
     */
    /**
     * Send a message to a peer.
     */
    fun sendMessage(peerId: String, content: String) {
        viewModelScope.launch(Dispatchers.IO) {
            try {
                // Call repository to handle encryption and transmission
                meshRepository.sendMessage(peerId, content)
                
                // Reload messages to show the sent message
                loadMessages()
                
                Timber.i("Message sent to $peerId")
            } catch (e: Exception) {
                _error.value = "Failed to send message: ${e.message}"
                Timber.e(e, "Failed to send message")
            }
        }
    }
    
    /**
     * Mark a message as delivered.
     */
    fun markDelivered(messageId: String) {
        viewModelScope.launch {
            try {
                meshRepository.markMessageDelivered(messageId)
                loadMessages()
                
                Timber.d("Message marked as delivered: $messageId")
            } catch (e: Exception) {
                Timber.e(e, "Failed to mark message as delivered")
            }
        }
    }
    
    /**
     * Clear conversation with a peer.
     */
    fun clearConversation(peerId: String) {
        viewModelScope.launch {
            try {
                meshRepository.clearConversation(peerId)
                loadMessages()
                
                Timber.i("Conversation cleared: $peerId")
            } catch (e: Exception) {
                _error.value = "Failed to clear conversation: ${e.message}"
                Timber.e(e, "Failed to clear conversation")
            }
        }
    }
    
    /**
     * Clear all message history.
     */
    fun clearAllHistory() {
        viewModelScope.launch {
            try {
                meshRepository.clearHistory()
                loadMessages()
                loadStats()
                
                Timber.i("All history cleared")
            } catch (e: Exception) {
                _error.value = "Failed to clear history: ${e.message}"
                Timber.e(e, "Failed to clear history")
            }
        }
    }
    
    /**
     * Load message statistics.
     */
    private fun loadStats() {
        viewModelScope.launch {
            try {
                val historyStats = meshRepository.getHistoryStats()
                _stats.value = historyStats
                
                Timber.d("Loaded stats: $historyStats")
            } catch (e: Exception) {
                Timber.e(e, "Failed to load stats")
            }
        }
    }
    
    /**
     * Search messages.
     */
    fun searchMessages(query: String, limit: UInt = 50u): Flow<List<uniffi.api.MessageRecord>> {
        return flow {
            try {
                val results = meshRepository.searchMessages(query, limit)
                emit(results)
                Timber.d("Found ${results.size} messages matching '$query'")
            } catch (e: Exception) {
                Timber.e(e, "Failed to search messages")
                emit(emptyList())
            }
        }
    }
    
    /**
     * Clear error state.
     */
    fun clearError() {
        _error.value = null
    }
    
    /**
     * Get total message count.
     */
    fun getMessageCount(): UInt {
        return meshRepository.getMessageCount()
    }
}
