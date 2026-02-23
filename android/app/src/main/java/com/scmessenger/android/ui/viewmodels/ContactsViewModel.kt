package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.service.MeshEventBus
import com.scmessenger.android.service.PeerEvent
import com.scmessenger.android.utils.ContactImportParseResult
import com.scmessenger.android.utils.parseContactImportPayload
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import timber.log.Timber
import javax.inject.Inject

/** A peer discovered on the mesh but not yet saved as a contact. */
data class NearbyPeer(
    val peerId: String,
    val publicKey: String? = null,
    val nickname: String? = null,
    val libp2pPeerId: String? = null,
    val listeners: List<String> = emptyList(),
    val isOnline: Boolean = true
) {
    val displayName: String get() = nickname?.takeIf { it.isNotBlank() } ?: peerId.take(16)
    val hasFullIdentity: Boolean get() = publicKey != null
}

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

    // Peers discovered on the mesh but not yet saved as contacts.
    private val _nearbyPeers = MutableStateFlow<List<NearbyPeer>>(emptyList())
    val nearbyPeers: StateFlow<List<NearbyPeer>> = _nearbyPeers.asStateFlow()

    init {
        loadContacts()
        observeNearbyPeers()
    }

    /**
     * Subscribe to MeshEventBus peer events.
     * Adds newly discovered peers to nearbyPeers if they aren't already contacts.
     * Removes peers from nearbyPeers when they disconnect.
     */
    private fun observeNearbyPeers() {
        viewModelScope.launch {
            MeshEventBus.peerEvents.collect { event ->
                when (event) {
                    is PeerEvent.IdentityDiscovered -> {
                        val alreadyContact = _contacts.value.any { it.peerId == event.peerId }
                        if (!alreadyContact) {
                            val current = _nearbyPeers.value.toMutableList()
                            if (event.blePeerId != null && event.blePeerId != event.peerId) {
                                current.removeAll { it.peerId == event.blePeerId }
                            }
                            val idx = current.indexOfFirst { it.peerId == event.peerId }
                            val updated = NearbyPeer(
                                peerId = event.peerId,
                                publicKey = event.publicKey,
                                nickname = event.nickname,
                                libp2pPeerId = event.libp2pPeerId,
                                listeners = event.listeners,
                                isOnline = true
                            )
                            if (idx >= 0) current[idx] = updated else current.add(updated)
                            _nearbyPeers.value = current
                            Timber.d("Nearby identity discovered: ${event.peerId.take(16)}")
                        }
                    }
                    is PeerEvent.Discovered -> {
                        val alreadyContact = _contacts.value.any { it.peerId == event.peerId }
                        val alreadyNearby = _nearbyPeers.value.any { it.peerId == event.peerId }
                        if (!alreadyContact && !alreadyNearby) {
                            _nearbyPeers.value = _nearbyPeers.value + NearbyPeer(event.peerId)
                            Timber.d("Nearby peer (no identity yet): ${event.peerId.take(16)}")
                        }
                    }
                    is PeerEvent.Disconnected -> {
                        _nearbyPeers.value = _nearbyPeers.value.filter { it.peerId != event.peerId }
                    }
                    else -> Unit
                }
            }
        }
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
                // Drop any nearby entry that is now a saved contact
                val contactIds = contactList.map { it.peerId }.toSet()
                _nearbyPeers.value = _nearbyPeers.value.filter { it.peerId !in contactIds }

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
        libp2pPeerId: String? = null,
        listeners: List<String> = emptyList(),
        notes: String? = null
    ) {
        viewModelScope.launch {
            try {
                val trimmedKey = publicKey.trim()

                // Validate public key format before storing
                if (trimmedKey.isEmpty()) {
                    _error.value = "Public key cannot be empty"
                    return@launch
                }
                if (trimmedKey.length != 64) {
                    _error.value = "Public key must be exactly 64 hex characters (got ${trimmedKey.length})"
                    return@launch
                }
                if (!trimmedKey.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' }) {
                    _error.value = "Public key contains invalid characters (must be hex: 0-9, a-f)"
                    return@launch
                }

                val generatedNotes = if (!libp2pPeerId.isNullOrEmpty()) {
                    "libp2p_peer_id:$libp2pPeerId;listeners:${listeners.joinToString(",")}"
                } else null

                val finalNotes = if (generatedNotes != null && notes != null) {
                    "$notes\n$generatedNotes"
                } else {
                    generatedNotes ?: notes
                }

                val contact = uniffi.api.Contact(
                    peerId = peerId.trim(),
                    nickname = nickname,
                    publicKey = trimmedKey,
                    addedAt = System.currentTimeMillis().toULong(),
                    lastSeen = null,
                    notes = finalNotes
                )

                meshRepository.addContact(contact)

                if (listeners.isNotEmpty()) {
                    val peerIdForDial = libp2pPeerId ?: peerId.trim()
                    meshRepository.connectToPeer(peerIdForDial, listeners)
                    Timber.i("Dialing nearby local peer after adding contact: $peerIdForDial")
                }

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
                when (val parsed = parseContactImportPayload(json)) {
                    is ContactImportParseResult.Invalid -> {
                        _error.value = parsed.reason
                    }
                    is ContactImportParseResult.Valid -> {
                        val payload = parsed.payload
                        addContact(
                            payload.peerId,
                            payload.publicKey,
                            payload.nickname,
                            libp2pPeerId = payload.libp2pPeerId,
                            listeners = payload.listeners
                        )

                        if (payload.listeners.isNotEmpty()) {
                            val peerIdForDial = payload.libp2pPeerId ?: payload.peerId
                            meshRepository.connectToPeer(peerIdForDial, payload.listeners)
                            Timber.i("Connecting to imported peer (libp2p: ${payload.libp2pPeerId != null}): $peerIdForDial with addresses: ${payload.listeners}")
                        }
                    }
                }
            } catch (e: Exception) {
                _error.value = "Failed to import: ${e.message}"
            }
        }
    }
}
