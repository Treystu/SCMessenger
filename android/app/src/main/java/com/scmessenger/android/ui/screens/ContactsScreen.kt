package com.scmessenger.android.ui.screens

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.CameraAlt
import androidx.compose.material.icons.filled.ContentPaste
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.PersonAdd
import androidx.compose.material.icons.filled.Sensors
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.google.android.gms.common.api.CommonStatusCodes
import com.google.mlkit.common.MlKitException
import com.google.mlkit.vision.barcode.common.Barcode
import com.google.mlkit.vision.codescanner.GmsBarcodeScannerOptions
import com.google.mlkit.vision.codescanner.GmsBarcodeScanning
import com.scmessenger.android.ui.viewmodels.ContactsViewModel
import com.scmessenger.android.ui.viewmodels.NearbyPeer
import com.scmessenger.android.utils.ContactImportParseResult
import com.scmessenger.android.utils.parseContactImportPayload
import com.scmessenger.android.utils.toEpochMillis
import java.text.SimpleDateFormat
import java.util.*

/**
 * Contacts screen with list, search, and add/remove functionality.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ContactsScreen(
    viewModel: ContactsViewModel = hiltViewModel(),
    onNavigateToChat: (String) -> Unit
) {
    val contacts by viewModel.filteredContacts.collectAsState()
    val nearbyPeers by viewModel.nearbyPeers.collectAsState()
    val isLoading by viewModel.isLoading.collectAsState()
    val error by viewModel.error.collectAsState()
    val searchQuery by viewModel.searchQuery.collectAsState()

    var showAddDialog by remember { mutableStateOf(false) }
    var nearbyPrefilledPeer by remember { mutableStateOf<NearbyPeer?>(null) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Contacts (${contacts.size})") }
            )
        },
        floatingActionButton = {
            FloatingActionButton(onClick = { showAddDialog = true }) {
                Icon(Icons.Default.Add, contentDescription = "Add Contact")
            }
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
        ) {
            // Search bar
            OutlinedTextField(
                value = searchQuery,
                onValueChange = { viewModel.setSearchQuery(it) },
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                placeholder = { Text("Search contacts...") },
                singleLine = true
            )

            // Error snackbar
            error?.let { errorMsg ->
                Snackbar(
                    modifier = Modifier.padding(16.dp),
                    action = {
                        TextButton(onClick = { viewModel.clearError() }) {
                            Text("Dismiss")
                        }
                    }
                ) {
                    Text(errorMsg)
                }
            }

            // Loading indicator
            if (isLoading) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    CircularProgressIndicator()
                }
            } else if (contacts.isEmpty() && nearbyPeers.isEmpty()) {
                // Empty state
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Icon(
                            imageVector = Icons.Default.Person,
                            contentDescription = null,
                            modifier = Modifier.size(64.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(modifier = Modifier.height(16.dp))
                        Text(
                            text = if (searchQuery.isBlank()) {
                                "No contacts yet"
                            } else {
                                "No contacts found"
                            },
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        if (searchQuery.isBlank()) {
                            Spacer(modifier = Modifier.height(8.dp))
                            Text(
                                text = "Add contacts to start messaging",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }
            } else {
                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(horizontal = 16.dp, vertical = 8.dp)
                ) {
                    // Nearby peers section — discovered but not yet saved
                    if (nearbyPeers.isNotEmpty()) {
                        item {
                            Row(
                                verticalAlignment = Alignment.CenterVertically,
                                modifier = Modifier.padding(bottom = 4.dp, top = 4.dp)
                            ) {
                                Icon(
                                    imageVector = Icons.Default.Sensors,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.primary,
                                    modifier = Modifier.size(16.dp)
                                )
                                Spacer(modifier = Modifier.width(4.dp))
                                Text(
                                    text = "Nearby (${nearbyPeers.size})",
                                    style = MaterialTheme.typography.labelMedium,
                                    color = MaterialTheme.colorScheme.primary
                                )
                            }
                        }
                        items(nearbyPeers, key = { "nearby_${it.peerId}" }) { peer ->
                            NearbyPeerItem(
                                peer = peer,
                                onAdd = {
                                    nearbyPrefilledPeer = peer
                                    showAddDialog = true
                                },
                                onConnect = {
                                    val publicKey = peer.publicKey?.trim()
                                    if (publicKey.isNullOrEmpty()) {
                                        nearbyPrefilledPeer = peer
                                        showAddDialog = true
                                    } else {
                                        val bleRoute = peer.blePeerId?.takeIf { it.isNotBlank() }
                                        val notes = bleRoute?.let { "ble_peer_id:$it" }
                                        viewModel.addContact(
                                            peerId = peer.peerId,
                                            publicKey = publicKey,
                                            nickname = peer.nickname,
                                            libp2pPeerId = peer.libp2pPeerId,
                                            listeners = peer.listeners,
                                            notes = notes
                                        )
                                        onNavigateToChat(peer.peerId)
                                    }
                                }
                            )
                            Spacer(modifier = Modifier.height(8.dp))
                        }
                        if (contacts.isNotEmpty()) {
                            item {
                                Divider(modifier = Modifier.padding(vertical = 4.dp))
                                Text(
                                    text = "Contacts (${contacts.size})",
                                    style = MaterialTheme.typography.labelMedium,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                    modifier = Modifier.padding(bottom = 4.dp)
                                )
                            }
                        }
                    }
                    // Saved contacts
                    items(contacts, key = { it.peerId }) { contact ->
                        ContactItem(
                            contact = contact,
                            onClick = { onNavigateToChat(contact.peerId) },
                            onDelete = { viewModel.removeContact(contact.peerId) }
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                    }
                }
            }
        }
    }

    // Add contact dialog
    if (showAddDialog) {
        val nearbyBlePeerId = nearbyPrefilledPeer?.blePeerId ?: ""
        val nearbyLibp2p = nearbyPrefilledPeer?.libp2pPeerId ?: ""
        val nearbyListeners = nearbyPrefilledPeer?.listeners ?: emptyList()

        AddContactDialog(
            prefilledPeerId = nearbyPrefilledPeer?.peerId ?: "",
            prefilledPublicKey = nearbyPrefilledPeer?.publicKey ?: "",
            prefilledNickname = nearbyPrefilledPeer?.nickname ?: "",
            onDismiss = {
                showAddDialog = false
                nearbyPrefilledPeer = null
            },
            onAdd = { peerId, publicKey, nickname, importedLibp2p, importedListeners ->
                val effectiveLibp2p = importedLibp2p ?: nearbyLibp2p.takeIf { it.isNotBlank() }
                val effectiveListeners = if (importedListeners.isNotEmpty()) importedListeners else nearbyListeners
                val notes = nearbyBlePeerId.takeIf { it.isNotBlank() }?.let { "ble_peer_id:$it" }
                viewModel.addContact(peerId, publicKey, nickname, effectiveLibp2p, effectiveListeners, notes)
                showAddDialog = false
                nearbyPrefilledPeer = null
            },
            onAddAndChat = { peerId, publicKey, nickname, importedLibp2p, importedListeners ->
                val id = peerId.trim()
                if (id.isNotBlank() && publicKey.isNotBlank()) {
                    val effectiveLibp2p = importedLibp2p ?: nearbyLibp2p.takeIf { it.isNotBlank() }
                    val effectiveListeners = if (importedListeners.isNotEmpty()) importedListeners else nearbyListeners
                    val notes = nearbyBlePeerId.takeIf { it.isNotBlank() }?.let { "ble_peer_id:$it" }
                    viewModel.addContact(id, publicKey.trim(), nickname?.trim(), effectiveLibp2p, effectiveListeners, notes)
                    showAddDialog = false
                    nearbyPrefilledPeer = null
                    onNavigateToChat(id)
                }
            }
        )
    }
}

@Composable
fun ContactItem(
    contact: uniffi.api.Contact,
    onClick: () -> Unit,
    onDelete: () -> Unit
) {
    var showDeleteDialog by remember { mutableStateOf(false) }

    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(
                modifier = Modifier.weight(1f)
            ) {
                Text(
                    text = contact.nickname ?: contact.peerId.take(16) + "...",
                    style = MaterialTheme.typography.titleMedium
                )
                Text(
                    text = "ID: ${contact.peerId.take(16)}...",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                contact.lastSeen?.let { lastSeen ->
                    Text(
                        text = "Last seen: ${formatTimestamp(lastSeen)}",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }

            IconButton(onClick = { showDeleteDialog = true }) {
                Icon(
                    imageVector = Icons.Default.Delete,
                    contentDescription = "Delete Contact",
                    tint = MaterialTheme.colorScheme.error
                )
            }
        }
    }

    // Confirm delete dialog
    if (showDeleteDialog) {
        AlertDialog(
            onDismissRequest = { showDeleteDialog = false },
            title = { Text("Delete Contact?") },
            text = {
                Text("Are you sure you want to delete ${contact.nickname ?: "this contact"}?")
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        onDelete()
                        showDeleteDialog = false
                    }
                ) {
                    Text("Delete", color = MaterialTheme.colorScheme.error)
                }
            },
            dismissButton = {
                TextButton(onClick = { showDeleteDialog = false }) {
                    Text("Cancel")
                }
            }
        )
    }
}

@Composable
fun NearbyPeerItem(
    peer: NearbyPeer,
    onAdd: () -> Unit,
    onConnect: () -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.secondaryContainer.copy(alpha = 0.4f)
        )
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.weight(1f)
            ) {
                Icon(
                    imageVector = Icons.Default.Sensors,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.size(36.dp)
                )
                Spacer(modifier = Modifier.width(12.dp))
                Column {
                    Text(
                        text = peer.displayName,
                        style = MaterialTheme.typography.titleSmall
                    )
                    if (peer.hasFullIdentity) {
                        Text(
                            text = "● Identity verified",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.primary
                        )
                    } else {
                        Text(
                            text = peer.peerId.take(20) + "…",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace
                        )
                    }
                }
            }
            FilledTonalButton(onClick = if (peer.hasFullIdentity) onConnect else onAdd) {
                Icon(
                    imageVector = if (peer.hasFullIdentity) Icons.Default.Sensors else Icons.Default.PersonAdd,
                    contentDescription = null,
                    modifier = Modifier.size(16.dp)
                )
                Spacer(modifier = Modifier.width(4.dp))
                Text(if (peer.hasFullIdentity) "Connect" else "Add")
            }
        }
    }
}

@Composable
fun AddContactDialog(
    prefilledPeerId: String = "",
    prefilledPublicKey: String = "",
    prefilledNickname: String = "",
    onDismiss: () -> Unit,
    onAdd: (String, String, String?, String?, List<String>) -> Unit,
    onAddAndChat: (String, String, String?, String?, List<String>) -> Unit
) {
    var peerId by remember(prefilledPeerId) { mutableStateOf(prefilledPeerId) }
    var publicKey by remember(prefilledPublicKey) { mutableStateOf(prefilledPublicKey) }
    var nickname by remember(prefilledNickname) { mutableStateOf(prefilledNickname) }
    var libp2pPeerId by remember { mutableStateOf<String?>(null) }
    var listeners by remember { mutableStateOf<List<String>>(emptyList()) }
    var parseError by remember { mutableStateOf<String?>(null) }

    val clipboardManager = androidx.compose.ui.platform.LocalClipboardManager.current
    val context = LocalContext.current

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Add Contact") },
        text = {
            Column {
                OutlinedButton(
                    onClick = {
                        val text = clipboardManager.getText()?.text?.toString().orEmpty()
                        when (val parsed = parseContactImportPayload(text)) {
                            is ContactImportParseResult.Valid -> {
                                peerId = parsed.payload.peerId
                                publicKey = parsed.payload.publicKey
                                nickname = parsed.payload.nickname ?: nickname
                                libp2pPeerId = parsed.payload.libp2pPeerId
                                listeners = parsed.payload.listeners
                                parseError = null
                            }
                            is ContactImportParseResult.Invalid -> {
                                parseError = parsed.reason
                            }
                        }
                    },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Icon(Icons.Default.ContentPaste, contentDescription = null, modifier = Modifier.size(16.dp))
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("Paste Identity Export")
                }

                Spacer(modifier = Modifier.height(8.dp))
                OutlinedButton(
                    onClick = {
                        val options = GmsBarcodeScannerOptions.Builder()
                            .setBarcodeFormats(Barcode.FORMAT_QR_CODE)
                            .build()
                        val scanner = GmsBarcodeScanning.getClient(context, options)
                        scanner.startScan()
                            .addOnSuccessListener { barcode ->
                                val raw = barcode.rawValue.orEmpty()
                                when (val parsed = parseContactImportPayload(raw)) {
                                    is ContactImportParseResult.Valid -> {
                                        peerId = parsed.payload.peerId
                                        publicKey = parsed.payload.publicKey
                                        nickname = parsed.payload.nickname ?: nickname
                                        libp2pPeerId = parsed.payload.libp2pPeerId
                                        listeners = parsed.payload.listeners
                                        parseError = null
                                    }
                                    is ContactImportParseResult.Invalid -> {
                                        parseError = parsed.reason
                                    }
                                }
                            }
                            .addOnFailureListener { e ->
                                if (e is MlKitException && e.errorCode == CommonStatusCodes.CANCELED) {
                                    return@addOnFailureListener
                                }
                                parseError = "Unable to scan QR code."
                            }
                    },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Icon(Icons.Default.CameraAlt, contentDescription = null, modifier = Modifier.size(16.dp))
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("Scan Contact QR")
                }

                Spacer(modifier = Modifier.height(16.dp))
                Divider()
                Spacer(modifier = Modifier.height(16.dp))

                parseError?.let {
                    Text(
                        text = it,
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                }

                OutlinedTextField(
                    value = peerId,
                    onValueChange = { peerId = it },
                    label = { Text("Peer ID") },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true
                )
                Spacer(modifier = Modifier.height(8.dp))
                OutlinedTextField(
                    value = publicKey,
                    onValueChange = { publicKey = it },
                    label = { Text("Public Key") },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true
                )
                Spacer(modifier = Modifier.height(8.dp))
                OutlinedTextField(
                    value = nickname,
                    onValueChange = { nickname = it },
                    label = { Text("Nickname (Optional)") },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true
                )
            }
        },
        confirmButton = {
            Row {
                TextButton(
                    onClick = {
                        if (peerId.isNotBlank() && publicKey.isNotBlank()) {
                            onAdd(
                                peerId.trim(),
                                publicKey.trim(),
                                nickname.trim().ifBlank { null },
                                libp2pPeerId,
                                listeners
                            )
                        }
                    },
                    enabled = peerId.isNotBlank() && publicKey.isNotBlank()
                ) {
                    Text("Add")
                }
                Spacer(modifier = Modifier.width(8.dp))
                TextButton(
                    onClick = {
                        if (peerId.isNotBlank() && publicKey.isNotBlank()) {
                            onAddAndChat(
                                peerId.trim(),
                                publicKey.trim(),
                                nickname.ifBlank { null },
                                libp2pPeerId,
                                listeners
                            )
                        }
                    },
                    enabled = peerId.isNotBlank() && publicKey.isNotBlank()
                ) {
                    Text("Chat")
                }
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("Cancel")
            }
        }
    )
}

private fun formatTimestamp(timestamp: ULong): String {
    val timestampMillis = timestamp.toEpochMillis()
    val date = Date(timestampMillis)
    val now = System.currentTimeMillis()
    val diff = now - timestampMillis

    return when {
        diff < 60_000 -> "Just now"
        diff < 3600_000 -> "${diff / 60_000}m ago"
        diff < 86400_000 -> "${diff / 3600_000}h ago"
        else -> SimpleDateFormat("MMM d, yyyy", Locale.getDefault()).format(date)
    }
}
