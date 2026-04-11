package com.scmessenger.android.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Block
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.ui.viewmodels.ConversationsViewModel
import java.text.SimpleDateFormat
import java.util.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun BlockedPeersScreen(
    onNavigateBack: () -> Unit,
    viewModel: ConversationsViewModel = hiltViewModel()
) {
    val blockedPeers by viewModel.blockedPeers.collectAsState()
    var showUnblockConfirm by remember { mutableStateOf<uniffi.api.BlockedIdentity?>(null) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Blocked Peers") },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "Back")
                    }
                }
            )
        }
    ) { paddingValues ->
        if (blockedPeers.isEmpty()) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(paddingValues),
                contentAlignment = Alignment.Center
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Icon(
                        Icons.Default.Block,
                        contentDescription = null,
                        modifier = Modifier.size(64.dp),
                        tint = MaterialTheme.colorScheme.outline
                    )
                    Spacer(modifier = Modifier.height(16.dp))
                    Text(
                        "No blocked peers",
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.outline
                    )
                }
            }
        } else {
            LazyColumn(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(paddingValues),
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                items(blockedPeers) { blocked ->
                    BlockedPeerItem(
                        blocked = blocked,
                        onUnblock = { showUnblockConfirm = blocked }
                    )
                }
            }
        }
    }

    if (showUnblockConfirm != null) {
        val peer = showUnblockConfirm!!
        AlertDialog(
            onDismissRequest = { showUnblockConfirm = null },
            title = { Text("Unblock Peer?") },
            text = { Text("Peer ID: ${peer.peerId}\n\nYou will receive messages and notifications from this peer again.") },
            confirmButton = {
                TextButton(
                    onClick = {
                        viewModel.unblockPeer(peer.peerId)
                        showUnblockConfirm = null
                    }
                ) {
                    Text("Unblock")
                }
            },
            dismissButton = {
                TextButton(onClick = { showUnblockConfirm = null }) {
                    Text("Cancel")
                }
            }
        )
    }
}

@Composable
fun BlockedPeerItem(
    blocked: uniffi.api.BlockedIdentity,
    onUnblock: () -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = blocked.peerId.take(24) + "...",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold
                )
                Text(
                    text = "Blocked on: ${formatDate(blocked.blockedAt)}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                if (!blocked.reason.isNullOrBlank()) {
                    Text(
                        text = "Reason: ${blocked.reason}",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
            IconButton(onClick = onUnblock) {
                Icon(
                    Icons.Default.Delete,
                    contentDescription = "Unblock",
                    tint = MaterialTheme.colorScheme.error
                )
            }
        }
    }
}

private fun formatDate(seconds: ULong): String {
    val date = Date(seconds.toLong() * 1000)
    val sdf = SimpleDateFormat("MMM d, yyyy HH:mm", Locale.getDefault())
    return sdf.format(date)
}
