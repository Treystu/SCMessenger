package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import timber.log.Timber
import javax.inject.Inject

/**
 * ViewModel for the contacts screen.
 *
 * Manages contact list, search, and CRUD operations.
 */
@HiltViewModel
class ContactsViewModel @Inject constructor(
    private val meshRepository: MeshRepository
) : ViewModel() {

    // All contacts
    private val _contacts = MutableStateFlow<List<uniffi.api.Contact>>(emptyList())
    val contacts: StateFlow<List<uniffi.api.Contact>> = _contacts.asStateFlow()

    // Search query
    private val _searchQuery = MutableStateFlow("")
    val searchQuery: StateFlow<String> = _searchQuery.asStateFlow()

    // Filtered contacts based on search
    val filteredContacts = combine(contacts, searchQuery) { contacts, query ->
        if (query.isBlank()) {
            contacts
        } else {
            meshRepository.searchContacts(query)
        }
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

    init {
        loadContacts()
    }

    /**
     * Load all contacts from repository.
     */
    fun loadContacts() {
        viewModelScope.launch {
            try {
                _isLoading.value = true
                _error.value = null

                val contactList = meshRepository.listContacts()
                _contacts.value = contactList

                Timber.d("Loaded ${contactList.size} contacts")
            } catch (e: Exception) {
                _error.value = "Failed to load contacts: ${e.message}"
                Timber.e(e, "Failed to load contacts")
            } finally {
                _isLoading.value = false
            }
        }
    }

    /**
     * Add a new contact.
     */
    fun addContact(
        peerId: String,
        publicKey: String,
        nickname: String? = null,
        notes: String? = null
    ) {
        viewModelScope.launch {
            try {
                val contact = uniffi.api.Contact(
                    peerId = peerId,
                    nickname = nickname,
                    publicKey = publicKey,
                    addedAt = System.currentTimeMillis().toULong(),
                    lastSeen = null,
                    notes = notes
                )

                meshRepository.addContact(contact)
                loadContacts()

                Timber.i("Contact added: $peerId")
            } catch (e: Exception) {
                _error.value = "Failed to add contact: ${e.message}"
                Timber.e(e, "Failed to add contact")
            }
        }
    }

    /**
     * Remove a contact.
     */
    fun removeContact(peerId: String) {
        viewModelScope.launch {
            try {
                meshRepository.removeContact(peerId)
                loadContacts()

                Timber.i("Contact removed: $peerId")
            } catch (e: Exception) {
                _error.value = "Failed to remove contact: ${e.message}"
                Timber.e(e, "Failed to remove contact")
            }
        }
    }

    /**
     * Update contact nickname.
     */
    fun setNickname(peerId: String, nickname: String?) {
        viewModelScope.launch {
            try {
                meshRepository.setContactNickname(peerId, nickname)
                loadContacts()

                Timber.d("Nickname updated for $peerId")
            } catch (e: Exception) {
                _error.value = "Failed to update nickname: ${e.message}"
                Timber.e(e, "Failed to update nickname")
            }
        }
    }

    /**
     * Update search query.
     */
    fun setSearchQuery(query: String) {
        _searchQuery.value = query
    }

    /**
     * Clear search query.
     */
    fun clearSearch() {
        _searchQuery.value = ""
    }

    /**
     * Clear error state.
     */
    fun clearError() {
        _error.value = null
    }

    /**
     * Get contact count.
     */


    /**
     * Import contact from JSON export string.
     */
    fun importContact(json: String) {
        viewModelScope.launch {
            try {
                // Basic regex parsing to avoid dependencies
                val idPattern = "\"identity_id\":\\s*\"(.*?)\"".toRegex()
                val keyPattern = "\"public_key\":\\s*\"(.*?)\"".toRegex()
                val nickPattern = "\"nickname\":\\s*\"(.*?)\"".toRegex()
                val listenersPattern = "\"listeners\":\\s*\\[(.*?)\\]".toRegex()

                val peerId = idPattern.find(json)?.groupValues?.get(1)
                val publicKey = keyPattern.find(json)?.groupValues?.get(1)
                val nickname = nickPattern.find(json)?.groupValues?.get(1)

                if (!peerId.isNullOrBlank() && !publicKey.isNullOrBlank()) {
                    addContact(peerId, publicKey, nickname)

                    // Parse Listeners
                    val listenersMatch = listenersPattern.find(json)?.groupValues?.get(1)
                    if (listenersMatch != null) {
                         val addresses = listenersMatch.split(",").map {
                             it.trim().trim('"').replace(" (Potential)", "")
                         }.filter { it.isNotBlank() }

                         if (addresses.isNotEmpty()) {
                             meshRepository.connectToPeer(peerId, addresses)
                             Timber.i("Connecting to imported peer: $peerId with addresses: $addresses")
                         }
                    }
                } else {
                     _error.value = "Invalid identity format: Missing ID or Key"
                }
            } catch (e: Exception) {
                _error.value = "Failed to import: ${e.message}"
            }
        }
    }
}
