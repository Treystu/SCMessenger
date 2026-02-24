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
    val blePeerId: String? = null,
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
    private val nearbyDisconnectGraceMs = 30_000L
    private val pendingNearbyRemovalJobs = mutableMapOf<String, kotlinx.coroutines.Job>()

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
            val needle = query.trim()
            contacts.filter { contact ->
                contact.peerId.contains(needle, ignoreCase = true) ||
                    contact.publicKey.contains(needle, ignoreCase = true) ||
                    (contact.nickname?.contains(needle, ignoreCase = true) == true) ||
                    (contact.localNickname?.contains(needle, ignoreCase = true) == true) ||
                    (contact.notes?.contains(needle, ignoreCase = true) == true)
            }
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
        viewModelScope.launch {
            // Contacts can open after discovery already happened; replay cached identities
            // so Nearby stays aligned with Dashboard/Settings discovery state.
            kotlinx.coroutines.delay(100)
            meshRepository.replayDiscoveredPeerEvents()
        }
    }

    private fun normalizeNickname(value: String?): String? {
        return value?.trim()?.takeIf { it.isNotEmpty() }
    }

    private fun isSyntheticFallbackNickname(value: String?): Boolean {
        val normalized = normalizeNickname(value)?.lowercase() ?: return false
        return normalized.startsWith("peer-")
    }

    private fun selectAuthoritativeNickname(incoming: String?, existing: String?): String? {
        val incomingNormalized = normalizeNickname(incoming)
        val existingNormalized = normalizeNickname(existing)

        val incomingSynthetic = isSyntheticFallbackNickname(incomingNormalized)
        val existingSynthetic = isSyntheticFallbackNickname(existingNormalized)

        return when {
            incomingNormalized == null && existingSynthetic -> null
            incomingNormalized == null -> existingNormalized
            incomingSynthetic && existingNormalized == null -> null
            incomingSynthetic && existingSynthetic -> null
            incomingSynthetic -> existingNormalized
            existingSynthetic -> incomingNormalized
            else -> incomingNormalized
        }
    }

    private fun isLibp2pPeerId(value: String?): Boolean {
        val normalized = value?.trim().orEmpty()
        return normalized.startsWith("12D3Koo") || normalized.startsWith("Qm")
    }

    private fun isIdentityId(value: String?): Boolean {
        val normalized = value?.trim().orEmpty()
        return normalized.length == 64 && normalized.all {
            (it in '0'..'9') || (it in 'a'..'f') || (it in 'A'..'F')
        }
    }

    private fun isBlePeerId(value: String?): Boolean {
        val normalized = value?.trim().orEmpty()
        if (normalized.isEmpty()) return false
        return runCatching { java.util.UUID.fromString(normalized) }.isSuccess
    }

    private fun selectStablePeerId(incomingPeerId: String, existingPeerId: String?): String {
        val incoming = incomingPeerId.trim()
        val existing = existingPeerId?.trim().orEmpty()
        if (existing.isEmpty() || existing == incoming) return incoming

        val incomingIsLibp2p = isLibp2pPeerId(incoming)
        val existingIsLibp2p = isLibp2pPeerId(existing)
        val incomingIsIdentity = isIdentityId(incoming)
        val existingIsIdentity = isIdentityId(existing)
        val incomingIsBle = isBlePeerId(incoming)
        val existingIsBle = isBlePeerId(existing)

        return when {
            existingIsIdentity && incomingIsLibp2p -> existing
            incomingIsIdentity && existingIsLibp2p -> incoming
            existingIsBle && !incomingIsBle -> incoming
            !existingIsBle && incomingIsBle -> existing
            else -> incoming
        }
    }

    private fun isSameNearbyIdentity(peer: NearbyPeer, event: PeerEvent.IdentityDiscovered): Boolean {
        val incomingPeerId = event.peerId.trim()
        val incomingLibp2p = event.libp2pPeerId?.trim().orEmpty()
        val incomingBle = event.blePeerId?.trim().orEmpty()
        val peerLibp2p = peer.libp2pPeerId?.trim().orEmpty()
        val peerBle = peer.blePeerId?.trim().orEmpty()

        val sameById = peer.peerId == incomingPeerId ||
            (incomingLibp2p.isNotEmpty() && (
                peer.peerId == incomingLibp2p ||
                    peerLibp2p == incomingLibp2p
                )) ||
            (incomingBle.isNotEmpty() && (
                peer.peerId == incomingBle ||
                    peerBle == incomingBle
                ))
        val sameByPublicKey = !peer.publicKey.isNullOrBlank() &&
            peer.publicKey.equals(event.publicKey, ignoreCase = true)
        return sameById || sameByPublicKey
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
                        cancelPendingNearbyRemoval(event.peerId)
                        cancelPendingNearbyRemoval(event.libp2pPeerId)
                        cancelPendingNearbyRemoval(event.blePeerId)

                        val alreadyContact = _contacts.value.any { contact ->
                            contact.peerId == event.peerId ||
                                contact.publicKey.equals(event.publicKey, ignoreCase = true)
                        }
                        if (alreadyContact) {
                            // Federated nickname/route hints can update in repository upsert;
                            // refresh saved contacts so local UI reflects latest values.
                            loadContacts()
                            return@collect
                        }
                        if (!alreadyContact) {
                            val current = _nearbyPeers.value.toMutableList()
                            val matches = current.filter { peer -> isSameNearbyIdentity(peer, event) }
                            val existing = matches.maxByOrNull { peer ->
                                val hasNickname = if (normalizeNickname(peer.nickname) != null) 2 else 0
                                val hasStableId = if (!isLibp2pPeerId(peer.peerId)) 1 else 0
                                hasNickname + hasStableId
                            }
                            if (matches.isNotEmpty()) {
                                current.removeAll(matches.toSet())
                            }
                            cancelPendingNearbyRemoval(existing?.peerId)

                            val resolvedPeerId = selectStablePeerId(event.peerId, existing?.peerId)
                            val resolvedLibp2pPeerId = event.libp2pPeerId?.trim()?.takeIf { it.isNotEmpty() }
                                ?: existing?.libp2pPeerId?.trim()?.takeIf { it.isNotEmpty() }
                                ?: event.peerId.takeIf { isLibp2pPeerId(it) }
                            val resolvedBlePeerId = event.blePeerId?.trim()?.takeIf { it.isNotEmpty() }
                                ?: existing?.blePeerId?.trim()?.takeIf { it.isNotEmpty() }
                            val updated = NearbyPeer(
                                peerId = resolvedPeerId,
                                publicKey = event.publicKey,
                                nickname = selectAuthoritativeNickname(event.nickname, existing?.nickname),
                                blePeerId = resolvedBlePeerId,
                                libp2pPeerId = resolvedLibp2pPeerId,
                                listeners = if (event.listeners.isNotEmpty()) event.listeners else (existing?.listeners ?: emptyList()),
                                isOnline = true
                            )
                            current.add(updated)
                            _nearbyPeers.value = current
                            Timber.d("Nearby identity discovered: ${resolvedPeerId.take(16)}")
                        }
                    }
                    is PeerEvent.Discovered -> {
                        val alreadyContact = _contacts.value.any { it.peerId == event.peerId }
                        cancelPendingNearbyRemoval(event.peerId)
                        val current = _nearbyPeers.value.toMutableList()
                        val existingIdx = current.indexOfFirst { it.peerId == event.peerId || it.libp2pPeerId == event.peerId }
                        if (existingIdx >= 0) {
                            current[existingIdx] = current[existingIdx].copy(isOnline = true)
                            _nearbyPeers.value = current
                        } else if (!alreadyContact) {
                            _nearbyPeers.value = current + NearbyPeer(event.peerId, isOnline = true)
                            Timber.d("Nearby peer (no identity yet): ${event.peerId.take(16)}")
                        }
                    }
                    is PeerEvent.Disconnected -> {
                        val current = _nearbyPeers.value.toMutableList()
                        var changed = false
                        current.indices.forEach { idx ->
                            val peer = current[idx]
                            if (peer.peerId == event.peerId || peer.libp2pPeerId == event.peerId) {
                                if (peer.isOnline) {
                                    current[idx] = peer.copy(isOnline = false)
                                    changed = true
                                }
                            }
                        }
                        if (changed) {
                            _nearbyPeers.value = current
                            scheduleNearbyRemoval(event.peerId)
                        }
                    }
                    else -> Unit
                }
            }
        }
    }

    private fun cancelPendingNearbyRemoval(peerId: String?) {
        val id = peerId?.trim().orEmpty()
        if (id.isEmpty()) return
        pendingNearbyRemovalJobs.remove(id)?.cancel()
    }

    private fun scheduleNearbyRemoval(peerId: String) {
        cancelPendingNearbyRemoval(peerId)
        pendingNearbyRemovalJobs[peerId] = viewModelScope.launch {
            kotlinx.coroutines.delay(nearbyDisconnectGraceMs)
            _nearbyPeers.value = _nearbyPeers.value.filterNot {
                it.peerId == peerId || it.libp2pPeerId == peerId
            }
            pendingNearbyRemovalJobs.remove(peerId)
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

                val finalNotes = listOfNotNull(
                    notes?.trim()?.takeIf { it.isNotEmpty() },
                    generatedNotes?.trim()?.takeIf { it.isNotEmpty() }
                ).joinToString(";").takeIf { it.isNotEmpty() }

                val contact = uniffi.api.Contact(
                    peerId = peerId.trim(),
                    nickname = nickname,
                    localNickname = null,
                    publicKey = trimmedKey,
                    addedAt = (System.currentTimeMillis() / 1000).toULong(),
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
    fun setLocalNickname(peerId: String, nickname: String?) {
        viewModelScope.launch {
            try {
                meshRepository.setLocalNickname(peerId, nickname)
                loadContacts()

                Timber.d("Local nickname updated for $peerId")
            } catch (e: Exception) {
                _error.value = "Failed to update local nickname: ${e.message}"
                Timber.e(e, "Failed to update local nickname")
            }
        }
    }

    fun setNickname(peerId: String, nickname: String?) {
        // Backward-compatible alias for existing callers.
        setLocalNickname(peerId, nickname)
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

    override fun onCleared() {
        pendingNearbyRemovalJobs.values.forEach { it.cancel() }
        pendingNearbyRemovalJobs.clear()
        super.onCleared()
    }
}
