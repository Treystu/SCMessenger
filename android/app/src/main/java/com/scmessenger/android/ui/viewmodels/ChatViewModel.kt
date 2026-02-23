package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.service.MeshEventBus
import com.scmessenger.android.service.MessageEvent
import com.scmessenger.android.utils.toEpochMillis
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import timber.log.Timber
import javax.inject.Inject

/**
 * ViewModel for a single chat conversation.
 *
 * Handles message sending, receiving, delivery status tracking,
 * and real-time updates for a specific peer conversation.
 */
@HiltViewModel
class ChatViewModel @Inject constructor(
    private val meshRepository: MeshRepository
) : ViewModel() {

    // Current peer ID
    private val _peerId = MutableStateFlow<String?>(null)
    val peerId: StateFlow<String?> = _peerId.asStateFlow()

    // Messages for this conversation
    private val _messages = MutableStateFlow<List<uniffi.api.MessageRecord>>(emptyList())
    val messages: StateFlow<List<uniffi.api.MessageRecord>> = _messages.asStateFlow()

    // Message being composed
    private val _inputText = MutableStateFlow("")
    val inputText: StateFlow<String> = _inputText.asStateFlow()

    // Sending state
    private val _isSending = MutableStateFlow(false)
    val isSending: StateFlow<Boolean> = _isSending.asStateFlow()

    // Error state
    private val _error = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = _error.asStateFlow()

    // Contact info
    private val _contact = MutableStateFlow<uniffi.api.Contact?>(null)
    val contact: StateFlow<uniffi.api.Contact?> = _contact.asStateFlow()

    // Typing indicator (placeholder for future)
    private val _isTyping = MutableStateFlow(false)
    val isTyping: StateFlow<Boolean> = _isTyping.asStateFlow()

    // Online status
    private val _isOnline = MutableStateFlow(false)
    val isOnline: StateFlow<Boolean> = _isOnline.asStateFlow()

    init {
        observeMessageEvents()
        observeIncomingMessages()
    }

    /**
     * Set the peer for this chat conversation.
     */
    fun setPeer(peerId: String) {
        _peerId.value = peerId
        loadMessages()
        loadContact()
    }

    /**
     * Load messages for the current peer.
     */
    private fun loadMessages() {
        viewModelScope.launch {
            try {
                val currentPeer = _peerId.value ?: return@launch
                val messageList = meshRepository.getConversation(currentPeer, limit = 200u)
                _messages.value = messageList.sortedBy { it.timestamp }

                Timber.d("Loaded ${messageList.size} messages for $currentPeer")
            } catch (e: Exception) {
                _error.value = "Failed to load messages: ${e.message}"
                Timber.e(e, "Failed to load messages")
            }
        }
    }

    /**
     * Load contact information.
     */
    private fun loadContact() {
        viewModelScope.launch {
            try {
                val currentPeer = _peerId.value ?: return@launch
                val contactInfo = meshRepository.getContact(currentPeer)
                _contact.value = contactInfo

                if (contactInfo == null) {
                    Timber.w("No contact found for peer: $currentPeer")
                }
            } catch (e: Exception) {
                Timber.e(e, "Failed to load contact")
            }
        }
    }

    /**
     * Send a message to the current peer.
     */
    fun sendMessage() {
        val currentPeer = _peerId.value
        val content = _inputText.value.trim()

        if (currentPeer == null) {
            _error.value = "No peer selected"
            return
        }

        if (content.isEmpty()) {
            return
        }

        viewModelScope.launch {
            try {
                _isSending.value = true
                _error.value = null

                meshRepository.sendMessage(currentPeer, content)

                // Clear input on success
                _inputText.value = ""

                // Reload messages to show the sent message
                loadMessages()

                Timber.i("Message sent to $currentPeer")
            } catch (e: Exception) {
                _error.value = "Failed to send message: ${e.message}"
                Timber.e(e, "Failed to send message")
            } finally {
                _isSending.value = false
            }
        }
    }

    /**
     * Send a message with specific content (for testing/quick sends).
     */
    fun sendMessage(content: String) {
        _inputText.value = content
        sendMessage()
    }

    /**
     * Update input text.
     */
    fun updateInputText(text: String) {
        _inputText.value = text
    }

    /**
     * Clear input text.
     */
    fun clearInput() {
        _inputText.value = ""
    }

    /**
     * Clear error state.
     */
    fun clearError() {
        _error.value = null
    }

    /**
     * Observe message events from MeshEventBus.
     */
    private fun observeMessageEvents() {
        viewModelScope.launch {
            MeshEventBus.messageEvents.collect { event ->
                when (event) {
                    is MessageEvent.Received -> {
                        // Reload if message is for current peer
                        if (event.messageRecord.peerId == _peerId.value) {
                            loadMessages()
                        }
                    }
                    is MessageEvent.Delivered -> {
                        // Update delivery status
                        updateMessageStatus(event.messageId, delivered = true)
                    }
                    is MessageEvent.Failed -> {
                        Timber.w("Message failed: ${event.messageId} - ${event.error}")
                    }
                    else -> {
                        // Handle other events if needed
                    }
                }
            }
        }
    }

    /**
     * Observe incoming messages from repository.
     */
    private fun observeIncomingMessages() {
        viewModelScope.launch {
            meshRepository.incomingMessages.collect { message ->
                if (message.peerId == _peerId.value) {
                    loadMessages()
                }
            }
        }
    }

    /**
     * Update message delivery status in the UI.
     */
    private fun updateMessageStatus(messageId: String, delivered: Boolean) {
        val updatedMessages = _messages.value.map { msg ->
            if (msg.id == messageId) {
                uniffi.api.MessageRecord(
                    id = msg.id,
                    peerId = msg.peerId,
                    direction = msg.direction,
                    content = msg.content,
                    timestamp = msg.timestamp,
                    delivered = delivered
                )
            } else {
                msg
            }
        }
        _messages.value = updatedMessages
    }

    /**
     * Get formatted timestamp for a message.
     */
    fun formatTimestamp(timestamp: ULong): String {
        val millis = timestamp.toEpochMillis()
        val date = java.util.Date(millis)
        val now = java.util.Date()

        val sdf = if (isSameDay(date, now)) {
            java.text.SimpleDateFormat("HH:mm", java.util.Locale.getDefault())
        } else {
            java.text.SimpleDateFormat("MMM d, HH:mm", java.util.Locale.getDefault())
        }

        return sdf.format(date)
    }

    private fun isSameDay(date1: java.util.Date, date2: java.util.Date): Boolean {
        val cal1 = java.util.Calendar.getInstance().apply { time = date1 }
        val cal2 = java.util.Calendar.getInstance().apply { time = date2 }

        return cal1.get(java.util.Calendar.YEAR) == cal2.get(java.util.Calendar.YEAR) &&
               cal1.get(java.util.Calendar.DAY_OF_YEAR) == cal2.get(java.util.Calendar.DAY_OF_YEAR)
    }
}
