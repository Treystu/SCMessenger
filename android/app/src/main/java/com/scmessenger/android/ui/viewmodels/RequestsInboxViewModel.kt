package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.ui.screens.RequestsInboxScreen.RequestItem
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import timber.log.Timber
import javax.inject.Inject
import uniffi.api.MessageRequest

/**
 * ViewModel for the Requests Inbox screen.
 *
 * Manages pending DM Request items, displaying them to the user,
 * and handling Accept/Reject/Block actions.
 */
@HiltViewModel
class RequestsInboxViewModel @Inject constructor(
    private val meshRepository: MeshRepository
) : ViewModel() {

    // Pending message requests
    private val _requests = MutableStateFlow<List<RequestItem>>(emptyList())
    val requests: StateFlow<List<RequestItem>> = _requests.asStateFlow()

    // Loading state
    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()

    // Error state
    private val _error = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = _error.asStateFlow()

    init {
        loadRequests()
    }

    /**
     * Load pending message requests from repository.
     */
    fun loadRequests() {
        viewModelScope.launch {
            try {
                _isLoading.value = true
                _error.value = null

                // Get pending requests from repository
                val requests = meshRepository.getPendingMessageRequests()
                _requests.value = requests.map { req ->
                    RequestItem(
                        peerId = req.peerId,
                        nickname = req.nickname,
                        messagePreview = req.messagePreview,
                        messageTimestamp = req.messageTimestamp.toLong(),
                        messageCount = req.messageCount
                    )
                }

                Timber.d("Loaded ${_requests.value.size} pending message requests")
            } catch (e: Exception) {
                _error.value = "Failed to load requests: ${e.message}"
                Timber.e(e, "Failed to load message requests")
            } finally {
                _isLoading.value = false
            }
        }
    }

    /**
     * Accept a message request - adds sender as contact.
     */
    fun acceptRequest(peerId: String) {
        viewModelScope.launch {
            try {
                withContext(Dispatchers.IO) {
                    meshRepository.addContact(peerId, null, null, null)
                }
                Timber.i("Accepted message request from $peerId")
                loadRequests()
            } catch (e: Exception) {
                Timber.e(e, "Failed to accept request from $peerId")
            }
        }
    }

    /**
     * Reject a message request - blocks the sender and removes from requests.
     */
    fun rejectRequest(peerId: String) {
        viewModelScope.launch {
            try {
                withContext(Dispatchers.IO) {
                    meshRepository.blockPeer(peerId)
                }
                Timber.i("Rejected message request from $peerId")
                loadRequests()
            } catch (e: Exception) {
                Timber.e(e, "Failed to reject request from $peerId")
            }
        }
    }

    /**
     * Block and delete - blocks sender AND deletes all their messages.
     */
    fun blockAndDelete(peerId: String) {
        viewModelScope.launch {
            try {
                withContext(Dispatchers.IO) {
                    meshRepository.blockAndDeletePeer(peerId)
                }
                Timber.i("Blocked and deleted messages from $peerId")
                loadRequests()
            } catch (e: Exception) {
                Timber.e(e, "Failed to block and delete from $peerId")
            }
        }
    }

    /**
     * Clear error state.
     */
    fun clearError() {
        _error.value = null
    }
}

/**
 * Data class representing a message request item for the UI.
 */
data class RequestItem(
    val peerId: String,
    val nickname: String?,
    val messagePreview: String,
    val messageTimestamp: Long,
    val messageCount: UInt
)
