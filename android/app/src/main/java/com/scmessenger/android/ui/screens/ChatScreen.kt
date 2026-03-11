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
import com.scmessenger.android.ui.chat.DeliveryStatePresentation
import com.scmessenger.android.ui.chat.DeliveryStateSurface
import com.scmessenger.android.utils.toEpochMillis
import com.scmessenger.android.ui.viewmodels.ConversationsViewModel
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
    val chatMessages = remember(messages, conversationId) {
        // MSG-ORDER-001: Sort strictly by sender-assigned timestamp to ensure consistent ordering across platforms
        messages.filter { it.peerId == conversationId }.sortedBy { it.senderTimestamp }
    }
    val nowEpochSec = System.currentTimeMillis() / 1000

    var inputText by remember { mutableStateOf("") }
    val listState = rememberLazyListState()
    val coroutineScope = rememberCoroutineScope()
    
    val contact = viewModel.getContactForPeer(conversationId)
    val localNickname = contact?.localNickname?.trim().orEmpty()
    val federatedNickname = contact?.nickname?.trim().orEmpty()
    val displayName = when {
        localNickname.isNotEmpty() -> localNickname
        federatedNickname.isNotEmpty() -> federatedNickname
        else -> conversationId.take(12) + "..."
    }
    
    Timber.d("CHAT_SCREEN: conversationId=$conversationId, displayName=$displayName, localNick=$localNickname, fedNick=$federatedNickname")
    val isPeerAvailable = viewModel.isPeerAvailable(conversationId)
    var showAddContactDialog by remember { mutableStateOf(false) }

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
                    val isBlocked = viewModel.isBlocked(conversationId)
                    IconButton(
                        onClick = {
                            if (isBlocked) {
                                viewModel.unblockPeer(conversationId)
                                Timber.i("Unblocked peer: $conversationId")
                            } else {
                                viewModel.blockPeer(conversationId, "Blocked from chat")
                                Timber.i("Blocked peer: $conversationId")
                            }
                        }
                    ) {
                        Icon(
                            imageVector = if (isBlocked) Icons.Default.CheckCircle else Icons.Default.Block,
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
            StateLegendCard(modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp))
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
                    val deliveryState = remember(message.id, message.delivered, nowEpochSec) {
                        viewModel.resolveDeliveryState(message, nowEpochSec)
                    }
                    MessageBubble(
                        message = message,
                        isMe = message.direction == uniffi.api.MessageDirection.SENT,
                        deliveryState = deliveryState
                    )
                }
            }

            // Input Area
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .imePadding()  // Add IME (keyboard) padding
                    .padding(16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                OutlinedTextField(
                    value = inputText,
                    onValueChange = { inputText = it },
                    modifier = Modifier.weight(1f),
                    placeholder = { Text("Type a message...") },
                    shape = RoundedCornerShape(24.dp),
                    maxLines = 4
                )

                Spacer(modifier = Modifier.width(8.dp))

                IconButton(
                    onClick = {
                        val messageToSend = inputText.trim()
                        if (messageToSend.isNotEmpty()) {
                            Timber.d("SEND: Clearing input immediately for instant feedback")
                            inputText = ""
                            coroutineScope.launch {
                                val success = viewModel.sendMessage(conversationId, messageToSend)
                                Timber.d("SEND: Message sent, success=$success")
                                listState.animateScrollToItem(chatMessages.size)
                            }
                        }
                    },
                    modifier = Modifier
                        .background(MaterialTheme.colorScheme.primary, CircleShape)
                        .size(48.dp)
                ) {
                    Icon(
                        imageVector = Icons.Default.Send,
                        contentDescription = "Send",
                        tint = MaterialTheme.colorScheme.onPrimary
                    )
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
                                    // Add via ContactsViewModel - need to get it
                                    // For now, add directly via repository
                                    // TODO: Better integration with ContactsViewModel
                                    showAddContactDialog = false
                                    // This will be handled by the user via Contacts screen
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
}

@Composable
fun MessageBubble(
    message: uniffi.api.MessageRecord,
    isMe: Boolean,
    deliveryState: DeliveryStatePresentation
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
        Text(
            text = formatTimestamp(message.timestamp),
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.outline,
            modifier = Modifier.padding(top = 4.dp, start = 4.dp, end = 4.dp)
        )
        if (isMe) {
            Text(
                text = deliveryState.label,
                style = MaterialTheme.typography.labelSmall,
                color = when (deliveryState.state) {
                    DeliveryStateSurface.DELIVERED -> MaterialTheme.colorScheme.primary
                    DeliveryStateSurface.FORWARDING -> MaterialTheme.colorScheme.tertiary
                    else -> MaterialTheme.colorScheme.onSurfaceVariant
                },
                fontWeight = FontWeight.SemiBold,
                modifier = Modifier.padding(top = 2.dp, start = 4.dp, end = 4.dp)
            )
        }
    }
}

@Composable
private fun StateLegendCard(modifier: Modifier = Modifier) {
    Card(modifier = modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(12.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            Text(
                text = "Delivery states",
                style = MaterialTheme.typography.labelLarge,
                fontWeight = FontWeight.SemiBold
            )
            Text(
                text = "pending: first attempt in progress • stored: queued for retry • forwarding: retry active • delivered: receipt confirmed",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

private fun formatTimestamp(timestamp: ULong): String {
    val date = Date(timestamp.toEpochMillis())
    val sdf = SimpleDateFormat("HH:mm", Locale.getDefault())
    return sdf.format(date)
}
