package com.scmessenger.android.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Block
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Send
import androidx.compose.material.icons.outlined.ChatBubbleOutline
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.utils.toEpochMillis
import com.scmessenger.android.ui.viewmodels.ConversationsViewModel
import com.scmessenger.android.ui.viewmodels.ContactsViewModel
import kotlinx.coroutines.launch
import timber.log.Timber
import java.text.SimpleDateFormat
import java.util.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ChatScreen(
    conversationId: String, // Treat as PeerID
    onNavigateBack: () -> Unit,
    viewModel: ConversationsViewModel = hiltViewModel()
) {
    val messages by viewModel.messages.collectAsState()
    val error by viewModel.error.collectAsState()
    val isLoading by viewModel.isLoading.collectAsState()
    val blockedPeers by viewModel.blockedPeers.collectAsState()
    val isBlocked = remember(blockedPeers, conversationId) {
        blockedPeers.any { it.peerId == conversationId }
    }
    val chatMessages = remember(messages, conversationId) {
        // MSG-ORDER-001: Sort strictly by sender-assigned timestamp to ensure consistent ordering across platforms
        messages.filter { it.peerId == conversationId }.sortedBy { it.senderTimestamp }
    }

    var inputText by remember { mutableStateOf("") }
    val listState = rememberLazyListState()
    val coroutineScope = rememberCoroutineScope()

    // AND-SEND-BTN-001: Use remember with derivedStateOf to avoid calling getContactForPeer
    // and isPeerAvailable on every recomposition. The underlying canonicalContactId() now
    // uses an in-memory cache (identityIdCache), so the first call resolves via FFI and
    // subsequent calls hit the cache. Combined with remember keys, this prevents the
    // UI-thread-blocking FFI calls that froze the send button.
    val normalizedPeerId = com.scmessenger.android.utils.PeerIdValidator.normalize(conversationId)

    val contact = remember(normalizedPeerId) {
        viewModel.getContactForPeer(normalizedPeerId)
    }
    val isPeerAvailable = remember(normalizedPeerId) {
        viewModel.isPeerAvailable(normalizedPeerId)
    }

    val localNickname = contact?.localNickname?.trim().orEmpty()
    val federatedNickname = contact?.nickname?.trim().orEmpty()
    val displayName = when {
        localNickname.isNotEmpty() -> localNickname
        federatedNickname.isNotEmpty() -> federatedNickname
        else -> conversationId.take(12) + "..."
    }

    Timber.d("CHAT_SCREEN: conversationId=$conversationId, normalizedPeerId=$normalizedPeerId, displayName=$displayName, localNick=$localNickname, fedNick=$federatedNickname, contactFound=${contact != null}, isBlocked=$isBlocked")
    var showAddContactDialog by remember { mutableStateOf(false) }
    var showBlockConfirmation by remember { mutableStateOf(false) }

    LaunchedEffect(conversationId) {
        viewModel.loadMessages(limit = 200u)
    }

    LaunchedEffect(chatMessages.size) {
        if (chatMessages.isNotEmpty()) {
            listState.animateScrollToItem(chatMessages.size - 1)
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(displayName)
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "Back")
                    }
                },
                actions = {
                    // Block/Unblock button
                    IconButton(
                        onClick = {
                            if (isBlocked) {
                                viewModel.unblockPeer(conversationId)
                            } else {
                                showBlockConfirmation = true
                            }
                        }
                    ) {
                        Icon(
                            imageVector = Icons.Default.Block,
                            contentDescription = if (isBlocked) "Unblock" else "Block",
                            tint = if (isBlocked) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurface
                        )
                    }
                }
            )
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
        ) {
            error?.let { errorMsg ->
                Snackbar(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 8.dp),
                    action = {
                        TextButton(onClick = { viewModel.clearError() }) {
                            Text("Dismiss")
                        }
                    }
                ) {
                    Text(errorMsg)
                }
            }

            // Show banner if peer is not in contacts
            if (contact == null && isPeerAvailable) {
                Card(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 8.dp),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.secondaryContainer
                    )
                ) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(12.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = "Not in contacts",
                                style = MaterialTheme.typography.titleSmall,
                                color = MaterialTheme.colorScheme.onSecondaryContainer
                            )
                            Text(
                                text = "Add to send messages",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSecondaryContainer
                            )
                        }
                        Button(onClick = { showAddContactDialog = true }) {
                            Text("Add Contact")
                        }
                    }
                }
            }

            // Messages List
            when {
                chatMessages.isEmpty() && isLoading -> {
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxWidth(),
                        contentAlignment = Alignment.Center
                    ) {
                        CircularProgressIndicator()
                    }
                }
                chatMessages.isEmpty() && !isLoading -> {
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxWidth(),
                        contentAlignment = Alignment.Center
                    ) {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            Icon(
                                imageVector = Icons.Outlined.ChatBubbleOutline,
                                contentDescription = null,
                                modifier = Modifier.size(64.dp),
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                            Text(
                                "No messages yet",
                                style = MaterialTheme.typography.titleMedium
                            )
                            Text(
                                "Send a message to start the conversation",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }
                else -> {
                    LazyColumn(
                        state = listState,
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxWidth()
                            .padding(horizontal = 16.dp),
                        contentPadding = PaddingValues(vertical = 16.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        items(chatMessages) { message ->
                            MessageBubble(
                                message = message,
                                isMe = message.direction == uniffi.api.MessageDirection.SENT
                            )
                        }
                    }
                }
            }

            // Input Area
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .imePadding()
                    .padding(16.dp),
                contentAlignment = Alignment.Center
            ) {
                if (isBlocked) {
                    Card(
                        modifier = Modifier.fillMaxWidth(),
                        colors = CardDefaults.cardColors(
                            containerColor = MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.5f)
                        )
                    ) {
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(12.dp),
                            horizontalArrangement = Arrangement.Center,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Icon(Icons.Default.Block, contentDescription = "Blocked", tint = MaterialTheme.colorScheme.onErrorContainer)
                            Spacer(modifier = Modifier.width(8.dp))
                            Text(
                                "Peer blocked. Unblock to send messages.",
                                color = MaterialTheme.colorScheme.onErrorContainer,
                                style = MaterialTheme.typography.bodyMedium
                            )
                        }
                    }
                } else {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        OutlinedTextField(
                            value = inputText,
                            onValueChange = { inputText = it },
                            modifier = Modifier.weight(1f),
                            placeholder = { Text("Type a message...") },
                            shape = RoundedCornerShape(24.dp),
                            maxLines = 4,
                            enabled = !isBlocked
                        )

                        Spacer(modifier = Modifier.width(8.dp))

                        // AND-SEND-BTN-001: Use FloatingActionButton for reliable click handling
                        FloatingActionButton(
                            onClick = {
                                if (isBlocked) {
                                    Timber.w("SEND_BUTTON_CLICKED: Peer is blocked, ignoring click")
                                    return@FloatingActionButton
                                }

                                // Defensive logging to catch UI thread issues
                                Timber.d("SEND_BUTTON_CLICKED: Button handler invoked")
                                Timber.d("SEND_BUTTON_CLICKED: inputText.length=${inputText.length}, isNotBlank=${inputText.isNotBlank()}")

                                val messageToSend = inputText.trim()
                                if (messageToSend.isEmpty()) {
                                    Timber.w("SEND: Attempted to send empty message")
                                    return@FloatingActionButton
                                }

                                Timber.d("SEND: Processing message send, contentLength=${messageToSend.length}")

                                // Clear input immediately for instant feedback
                                val originalInput = inputText
                                inputText = ""
                                Timber.d("SEND: Input cleared immediately for instant feedback")

                                coroutineScope.launch {
                                    try {
                                        Timber.d("SEND: Launching sendMessage coroutine")
                                        val success = viewModel.sendMessage(conversationId, messageToSend)
                                        Timber.d("SEND: Message sent, success=$success")
                                        if (success) {
                                            // Scroll to bottom to show new message
                                            if (chatMessages.isNotEmpty()) {
                                                listState.animateScrollToItem(chatMessages.size)
                                            }
                                        }
                                    } catch (e: Exception) {
                                        Timber.e(e, "SEND: Failed to send message")
                                        // Restore input if send failed
                                        inputText = originalInput
                                    }
                                }
                            },
                            modifier = Modifier.size(48.dp),
                            containerColor = if (inputText.isNotBlank() && !isBlocked) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceVariant,
                            contentColor = if (inputText.isNotBlank() && !isBlocked) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurfaceVariant
                        ) {
                            Icon(
                                imageVector = Icons.Default.Send,
                                contentDescription = "Send message",
                                modifier = Modifier.size(24.dp)
                            )
                        }
                    }
                }
            }
        }
    }

    // Quick add contact dialog
    if (showAddContactDialog) {
        val peerInfo = viewModel.getPeerInfo(conversationId)
        if (peerInfo != null) {
            val (publicKey, suggestedNickname) = peerInfo
            var nickname by remember { mutableStateOf(suggestedNickname) }
            val contactsViewModel: ContactsViewModel = hiltViewModel()

            AlertDialog(
                onDismissRequest = { showAddContactDialog = false },
                title = { Text("Add Contact") },
                text = {
                    Column {
                        Text("Add this peer to your contacts to send messages.")
                        Spacer(modifier = Modifier.height(8.dp))
                        OutlinedTextField(
                            value = nickname,
                            onValueChange = { nickname = it },
                            label = { Text("Nickname") },
                            singleLine = true,
                            modifier = Modifier.fillMaxWidth()
                        )
                    }
                },
                confirmButton = {
                    TextButton(
                        onClick = {
                            coroutineScope.launch {
                                try {
                                    contactsViewModel.addContact(
                                        peerId = conversationId,
                                        publicKey = publicKey,
                                        nickname = nickname
                                    )
                                    showAddContactDialog = false
                                    Timber.i("Contact added successfully via ContactsViewModel")
                                } catch (e: Exception) {
                                    Timber.e(e, "Failed to add contact")
                                }
                            }
                        }
                    ) {
                        Text("Add")
                    }
                },
                dismissButton = {
                    TextButton(onClick = { showAddContactDialog = false }) {
                        Text("Cancel")
                    }
                }
            )
        } else {
            // Peer not available - close dialog
            showAddContactDialog = false
        }
    }

    // Block confirmation dialog
    if (showBlockConfirmation) {
        AlertDialog(
            onDismissRequest = { showBlockConfirmation = false },
            title = { Text("Block Peer?") },
            text = { Text("You will no longer receive notifications from this peer. Existing messages will be kept.") },
            confirmButton = {
                TextButton(
                    onClick = {
                        viewModel.blockPeer(conversationId, "Blocked from chat")
                        showBlockConfirmation = false
                    },
                    colors = ButtonDefaults.textButtonColors(contentColor = MaterialTheme.colorScheme.error)
                ) {
                    Text("Block")
                }
            },
            dismissButton = {
                TextButton(onClick = { showBlockConfirmation = false }) {
                    Text("Cancel")
                }
            }
        )
    }
}


@Composable
fun MessageBubble(
    message: uniffi.api.MessageRecord,
    isMe: Boolean
) {
    val bubbleColor = if (isMe) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceVariant
    val textColor = if (isMe) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurfaceVariant
    val alignment = if (isMe) Alignment.End else Alignment.Start
    val shape = if (isMe) RoundedCornerShape(16.dp, 16.dp, 4.dp, 16.dp) else RoundedCornerShape(16.dp, 16.dp, 16.dp, 4.dp)

    Column(
        modifier = Modifier.fillMaxWidth(),
        horizontalAlignment = alignment
    ) {
        Box(
            modifier = Modifier
                .clip(shape)
                .background(bubbleColor)
                .padding(horizontal = 16.dp, vertical = 10.dp)
        ) {
            Text(
                text = message.content,
                color = textColor,
                style = MaterialTheme.typography.bodyLarge
            )
        }
        // Zero-Status Architecture: show only sender-assigned timestamp, no delivery state.
        Text(
            text = formatTimestamp(message.senderTimestamp),
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.outline,
            modifier = Modifier.padding(top = 4.dp, start = 4.dp, end = 4.dp)
        )
    }
}

private fun formatTimestamp(timestamp: ULong): String {
    val date = Date(timestamp.toEpochMillis())
    val sdf = SimpleDateFormat("HH:mm", Locale.getDefault())
    return sdf.format(date)
}
