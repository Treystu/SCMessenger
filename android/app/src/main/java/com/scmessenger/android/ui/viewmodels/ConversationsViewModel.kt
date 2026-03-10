package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.service.MeshEventBus
import com.scmessenger.android.service.MessageEvent
import com.scmessenger.android.ui.chat.DeliveryStateMapper
import com.scmessenger.android.ui.chat.DeliveryStatePresentation
import com.scmessenger.android.ui.chat.PendingDeliverySnapshot
import com.scmessenger.android.utils.PeerIdValidator
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import timber.log.Timber
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
            // MSG-ORDER-001: Sort strictly by sender-assigned timestamp to ensure consistent ordering across platforms
            .mapValues { (_, msgs) -> msgs.sortedByDescending { it.senderTimestamp } }
            .toList()
            .sortedByDescending { (_, msgs) -> msgs.firstOrNull()?.senderTimestamp ?: 0u }
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

        // Listen for message updates (sent or received) to refresh the list
        viewModelScope.launch {
            meshRepository.messageUpdates.collect {
                loadMessages()
            }
        }

        // Receipt/transport events can change delivery state without a new message
        // body; refresh to keep conversation badges and previews accurate.
        viewModelScope.launch {
            MeshEventBus.messageEvents.collect { event ->
                when (event) {
                    is MessageEvent.Delivered,
                    is MessageEvent.Failed -> loadMessages()
                    else -> Unit
                }
            }
        }
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
    suspend fun sendMessage(peerId: String, content: String): Boolean {
        return withContext(Dispatchers.IO) {
            try {
                val normalizedPeerId = PeerIdValidator.normalize(peerId)
                Timber.d("Sending message to peer: $normalizedPeerId")
                
                // Call repository to handle encryption and transmission
                meshRepository.sendMessage(normalizedPeerId, content)

                // Reload messages to show the sent message
                loadMessages()

                Timber.i("Message sent to $normalizedPeerId")
                true
            } catch (e: Exception) {
                _error.value = "Failed to send message: ${e.message}"
                Timber.e(e, "Failed to send message")
                false
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
     * Check if a peer can be messaged (exists in contacts or discovered peers).
     */
    fun isPeerAvailable(peerId: String): Boolean {
        val contact = getContactForPeer(peerId)
        if (contact != null) return true
        
        // Check discovered peers
        val discoveredPeers = meshRepository.discoveredPeers.value
        return discoveredPeers[peerId]?.publicKey != null
    }

    /**
     * Get peer info for adding to contacts quickly.
     */
    fun getPeerInfo(peerId: String): Pair<String, String>? {
        // Check discovered peers for public key
        val discovered = meshRepository.discoveredPeers.value[peerId]
        return discovered?.publicKey?.let { pubKey ->
            val nickname = discovered.nickname ?: peerId.take(8)
            pubKey to nickname
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
     * Get contact info for a peer (for displaying nickname).
     */
    fun getContactForPeer(peerId: String): uniffi.api.Contact? {
        return try {
            meshRepository.getContact(peerId)
        } catch (e: Exception) {
            null
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

    fun resolveDeliveryState(
        message: uniffi.api.MessageRecord,
        nowEpochSec: Long = System.currentTimeMillis() / 1000
    ): DeliveryStatePresentation {
        val pendingPair = meshRepository.getPendingDeliverySnapshot(message.id)
        val pending = pendingPair?.let { PendingDeliverySnapshot(it.first, it.second) }
        return DeliveryStateMapper.resolve(
            delivered = message.delivered,
            pending = pending,
            nowEpochSec = nowEpochSec
        )
    }
}
