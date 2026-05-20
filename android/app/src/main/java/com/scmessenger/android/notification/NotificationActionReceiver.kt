package com.scmessenger.android.notification

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build
import com.scmessenger.android.utils.NotificationHelper
import com.scmessenger.android.utils.PeerIdValidator
import com.scmessenger.android.data.MeshRepository
import dagger.hilt.EntryPoint
import dagger.hilt.InstallIn
import dagger.hilt.android.EntryPointAccessors
import dagger.hilt.components.SingletonComponent
import timber.log.Timber
import kotlinx.coroutines.*
import kotlinx.coroutines.Dispatchers

/**
 * Broadcast receiver for handling notification actions.
 *
 * Handles:
 * - ACTION_REPLY: Reply to a message from notification
 * - ACTION_MARK_READ: Mark conversation as read
 * - ACTION_MUTE: Mute notifications for a peer
 * - ACTION_OPEN_REQUESTS: Navigate to requests inbox
 */
class NotificationActionReceiver : BroadcastReceiver() {

    private lateinit var meshRepository: MeshRepository

    @EntryPoint
    @InstallIn(SingletonComponent::class)
    interface NotificationActionReceiverEntryPoint {
        fun meshRepository(): MeshRepository
    }

    private val serviceScope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    override fun onReceive(context: Context, intent: Intent) {
        Timber.d("NotificationActionReceiver received action: ${intent.action}")

        // Hilt EntryPoint injection for BroadcastReceiver (not supported by @AndroidEntryPoint)
        meshRepository = EntryPointAccessors.fromApplication(
            context.applicationContext,
            NotificationActionReceiverEntryPoint::class.java
        ).meshRepository()

        when (intent.action) {
            NotificationHelper.ACTION_REPLY -> handleReply(context, intent)
            NotificationHelper.ACTION_MARK_READ -> handleMarkRead(context, intent)
            NotificationHelper.ACTION_MUTE -> handleMute(context, intent)
            NotificationHelper.ACTION_OPEN_REQUESTS -> handleOpenRequests(context, intent)
            else -> {
                Timber.w("Unknown notification action: ${intent.action}")
            }
        }
    }

    /**
     * Handle reply action - send a message in response to a notification.
     */
    private fun handleReply(context: Context, intent: Intent) {
        val peerId = intent.getStringExtra(NotificationHelper.EXTRA_PEER_ID)
        val messageId = intent.getStringExtra(NotificationHelper.EXTRA_MESSAGE_ID)
        val replyText = intent.getStringExtra(NotificationHelper.KEY_REPLY_TEXT)

        if (peerId.isNullOrBlank()) {
            Timber.e("Reply action missing peerId")
            return
        }

        if (replyText.isNullOrBlank()) {
            Timber.w("Reply action missing reply text")
            return
        }

        Timber.i("Replying to $peerId with message: '$replyText' (original message: $messageId)")

        serviceScope.launch {
            try {
                val normalizedPeerId = withContext(Dispatchers.IO) {
                    PeerIdValidator.normalize(peerId)
                }
                withContext(Dispatchers.IO) {
                    meshRepository.sendMessage(normalizedPeerId, replyText)
                }
                Timber.i("Reply sent successfully to $normalizedPeerId")

                // Clear notification for this peer after reply
                withContext(Dispatchers.Main) {
                    NotificationHelper.clearMessageNotifications(context, peerId)
                }
            } catch (e: Exception) {
                Timber.e(e, "Failed to send reply to $peerId")
            }
        }
    }

    /**
     * Handle mark read action - mark conversation as read.
     */
    private fun handleMarkRead(context: Context, intent: Intent) {
        val peerId = intent.getStringExtra(NotificationHelper.EXTRA_PEER_ID)
        val messageId = intent.getStringExtra(NotificationHelper.EXTRA_MESSAGE_ID)

        if (peerId.isNullOrBlank()) {
            Timber.e("Mark read action missing peerId")
            return
        }

        Timber.i("Marking conversation as read for $peerId (message: $messageId)")

        serviceScope.launch {
            try {
                // Mark messages as delivered
                messageId?.let { msgId ->
                    withContext(Dispatchers.IO) {
                        meshRepository.markMessageDelivered(msgId)
                    }
                }
                // Clear notification for this peer
                withContext(Dispatchers.Main) {
                    NotificationHelper.clearMessageNotifications(context, peerId)
                }
                Timber.i("Conversation marked as read for $peerId")
            } catch (e: Exception) {
                Timber.e(e, "Failed to mark conversation as read for $peerId")
            }
        }
    }

    /**
     * Handle mute action - mute notifications for a peer.
     */
    private fun handleMute(context: Context, intent: Intent) {
        val peerId = intent.getStringExtra(NotificationHelper.EXTRA_PEER_ID)

        if (peerId.isNullOrBlank()) {
            Timber.e("Mute action missing peerId")
            return
        }

        Timber.i("Muting notifications for $peerId")

        serviceScope.launch {
            try {
                withContext(Dispatchers.IO) {
                    meshRepository.blockPeer(peerId)
                }
                // Clear notifications for this peer
                withContext(Dispatchers.Main) {
                    NotificationHelper.clearMessageNotifications(context, peerId)
                }
                Timber.i("Notifications muted for $peerId")
            } catch (e: Exception) {
                Timber.e(e, "Failed to mute notifications for $peerId")
            }
        }
    }

    /**
     * Handle open requests action - navigate to the requests inbox screen.
     */
    private fun handleOpenRequests(context: Context, intent: Intent) {
        val peerId = intent.getStringExtra(NotificationHelper.EXTRA_PEER_ID)

        Timber.d("Opening requests inbox for peer: $peerId")

        // Launch main activity with request navigation
        val launchIntent = context.packageManager.getLaunchIntentForPackage(context.packageName)?.apply {
            action = NotificationHelper.ACTION_OPEN_REQUESTS
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP
            putExtra(NotificationHelper.EXTRA_PEER_ID, peerId ?: "")
            putExtra(NotificationHelper.EXTRA_IS_REQUEST, true)
        }

        if (launchIntent != null) {
            try {
                context.startActivity(launchIntent)
                Timber.i("Started MainActivity to open requests inbox")
            } catch (e: Exception) {
                Timber.e(e, "Failed to start MainActivity for requests inbox")
            }
        } else {
            Timber.e("No launch intent available for requests inbox")
        }
    }
}
