package com.scmessenger.android.ui.contacts

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.ui.components.CopyableText
import com.scmessenger.android.ui.components.ErrorBanner
import com.scmessenger.android.ui.components.IdenticonFromPeerId
import com.scmessenger.android.ui.theme.StatusOnline
import com.scmessenger.android.ui.theme.StatusOffline
import com.scmessenger.android.ui.viewmodels.ContactsViewModel
import java.text.SimpleDateFormat
import java.util.*

/**
 * Contact Detail screen - Display contact info, metrics, and actions.
 * 
 * Shows detailed information about a contact including:
 * - Identity information (peer ID, public key)
 * - Connection metrics (last seen, message count)
 * - Actions (send message, edit, delete)
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ContactDetailScreen(
    contactId: String,
    onNavigateBack: () -> Unit,
    onNavigateToChat: (String) -> Unit = {},
    viewModel: ContactsViewModel = hiltViewModel()
) {
    val contacts by viewModel.contacts.collectAsState()
    val error by viewModel.error.collectAsState()
    
    val contact = remember(contacts, contactId) {
        contacts.find { it.peerId == contactId }
    }
    
    var showDeleteDialog by remember { mutableStateOf(false) }
    var showEditDialog by remember { mutableStateOf(false) }
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(contact?.nickname ?: "Contact Details") },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "Back")
                    }
                },
                actions = {
                    IconButton(onClick = { showEditDialog = true }) {
                        Icon(Icons.Default.Edit, contentDescription = "Edit")
                    }
                    IconButton(onClick = { showDeleteDialog = true }) {
                        Icon(Icons.Default.Delete, contentDescription = "Delete")
                    }
                }
            )
        }
    ) { paddingValues ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
        ) {
            if (contact == null) {
                // Contact not found
                Column(
                    modifier = Modifier
                        .align(Alignment.Center)
                        .padding(32.dp),
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    Icon(
                        imageVector = Icons.Default.Close,
                        contentDescription = null,
                        modifier = Modifier.size(64.dp),
                        tint = MaterialTheme.colorScheme.error
                    )
                    
                    Spacer(modifier = Modifier.height(16.dp))
                    
                    Text(
                        text = "Contact not found",
                        style = MaterialTheme.typography.titleLarge
                    )
                }
            } else {
                // Show contact details
                ContactDetailContent(
                    contact = contact,
                    error = error,
                    onClearError = { viewModel.clearError() },
                    onSendMessage = { onNavigateToChat(contact.peerId) }
                )
            }
        }
    }
    
    // Delete confirmation dialog
    if (showDeleteDialog) {
        AlertDialog(
            onDismissRequest = { showDeleteDialog = false },
            title = { Text("Delete Contact") },
            text = { Text("Are you sure you want to delete this contact?") },
            confirmButton = {
                TextButton(
                    onClick = {
                        viewModel.removeContact(contactId)
                        showDeleteDialog = false
                        onNavigateBack()
                    }
                ) {
                    Text("Delete")
                }
            },
            dismissButton = {
                TextButton(onClick = { showDeleteDialog = false }) {
                    Text("Cancel")
                }
            }
        )
    }
    
    // Edit nickname dialog
    if (showEditDialog && contact != null) {
        var newNickname by remember { mutableStateOf(contact.nickname ?: "") }
        
        AlertDialog(
            onDismissRequest = { showEditDialog = false },
            title = { Text("Edit Nickname") },
            text = {
                OutlinedTextField(
                    value = newNickname,
                    onValueChange = { newNickname = it },
                    label = { Text("Nickname") },
                    singleLine = true
                )
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        viewModel.setNickname(
                            contactId,
                            newNickname.takeIf { it.isNotBlank() }
                        )
                        showEditDialog = false
                    }
                ) {
                    Text("Save")
                }
            },
            dismissButton = {
                TextButton(onClick = { showEditDialog = false }) {
                    Text("Cancel")
                }
            }
        )
    }
}

@Composable
private fun ContactDetailContent(
    contact: uniffi.api.Contact,
    error: String?,
    onClearError: () -> Unit,
    onSendMessage: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        // Error banner
        error?.let {
            ErrorBanner(
                message = it,
                onDismiss = onClearError
            )
        }
        
        // Identity card
        Card {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(24.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                IdenticonFromPeerId(
                    peerId = contact.peerId,
                    size = 96.dp
                )
                
                Text(
                    text = contact.nickname ?: "Unknown",
                    style = MaterialTheme.typography.headlineMedium,
                    fontWeight = FontWeight.Bold
                )
                
                // Online status
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Icon(
                        imageVector = Icons.Default.CheckCircle,
                        contentDescription = null,
                        modifier = Modifier.size(16.dp),
                        tint = if (contact.lastSeen != null) StatusOnline else StatusOffline
                    )
                    Text(
                        text = if (contact.lastSeen != null) "Last seen recently" else "Never seen",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                
                // Send message button
                Button(
                    onClick = onSendMessage,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Icon(Icons.Default.Send, contentDescription = null)
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("Send Message")
                }
            }
        }
        
        // Peer ID
        Card {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    text = "Peer ID",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )
                
                Spacer(modifier = Modifier.height(8.dp))
                
                CopyableText(
                    text = contact.peerId,
                    monospace = true
                )
            }
        }
        
        // Public Key
        Card {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    text = "Public Key",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )
                
                Spacer(modifier = Modifier.height(8.dp))
                
                CopyableText(
                    text = contact.publicKey,
                    monospace = true
                )
            }
        }
        
        // Metadata
        Card {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    text = "Metadata",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )
                
                Spacer(modifier = Modifier.height(8.dp))
                
                MetadataRow(label = "Added", value = formatTimestamp(contact.addedAt))
                
                contact.lastSeen?.let {
                    MetadataRow(label = "Last Seen", value = formatTimestamp(it))
                }
                
                contact.notes?.let {
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = "Notes:",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = it,
                        style = MaterialTheme.typography.bodyMedium
                    )
                }
            }
        }
    }
}

@Composable
private fun MetadataRow(label: String, value: String) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodyMedium
        )
    }
}

private fun formatTimestamp(timestamp: ULong): String {
    val millis = timestamp.toLong()
    val date = Date(millis)
    val sdf = SimpleDateFormat("MMM d, yyyy HH:mm", Locale.getDefault())
    return sdf.format(date)
}
