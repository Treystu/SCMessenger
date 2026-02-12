package com.scmessenger.android.utils

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationChannelGroup
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.app.Person
import androidx.core.app.RemoteInput
import androidx.core.graphics.drawable.IconCompat
import com.scmessenger.android.R
import com.scmessenger.android.ui.components.generateIdenticonBitmap
import timber.log.Timber

/**
 * Notification helper for mesh messaging.
 * 
 * Features:
 * - 4 notification channels (Messages/high, Mesh Status/low, Peer Events/default, System/low)
 * - Grouped message notifications per contact
 * - Reply-from-notification with RemoteInput
 * - Notification actions (Mark Read, Reply, Mute)
 * - Identicon-based contact avatars
 * - Respects DND (Do Not Disturb) settings
 */
object NotificationHelper {
    
    // Channel IDs
    private const val CHANNEL_MESSAGES = "messages"
    private const val CHANNEL_MESH_STATUS = "mesh_status"
    private const val CHANNEL_PEER_EVENTS = "peer_events"
    private const val CHANNEL_SYSTEM = "system"
    
    // Channel Group
    private const val GROUP_MESH = "mesh_group"
    
    // Notification IDs
    const val NOTIFICATION_ID_FOREGROUND_SERVICE = 1001
    private const val NOTIFICATION_ID_MESSAGE_BASE = 2000
    private const val NOTIFICATION_ID_MESH_STATUS = 3000
    private const val NOTIFICATION_ID_PEER_EVENT = 4000
    
    // Actions
    const val ACTION_REPLY = "com.scmessenger.ACTION_REPLY"
    const val ACTION_MARK_READ = "com.scmessenger.ACTION_MARK_READ"
    const val ACTION_MUTE = "com.scmessenger.ACTION_MUTE"
    const val EXTRA_PEER_ID = "peer_id"
    const val EXTRA_MESSAGE_ID = "message_id"
    const val KEY_REPLY_TEXT = "key_reply_text"
    
    // Message grouping
    private val messageGroups = mutableMapOf<String, MutableList<NotificationMessage>>()
    
    /**
     * Initialize notification channels on app start.
     * Must be called before posting any notifications.
     */
    fun createNotificationChannels(context: Context) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val notificationManager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            
            // Create channel group
            val group = NotificationChannelGroup(GROUP_MESH, "Mesh Network")
            notificationManager.createNotificationChannelGroup(group)
            
            // 1. Messages Channel (HIGH priority)
            val messagesChannel = NotificationChannel(
                CHANNEL_MESSAGES,
                "Messages",
                NotificationManager.IMPORTANCE_HIGH
            ).apply {
                description = "New messages from contacts"
                group = GROUP_MESH
                enableLights(true)
                enableVibration(true)
                setShowBadge(true)
            }
            
            // 2. Mesh Status Channel (LOW priority)
            val meshStatusChannel = NotificationChannel(
                CHANNEL_MESH_STATUS,
                "Mesh Status",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Mesh network connection status"
                group = GROUP_MESH
                enableLights(false)
                enableVibration(false)
                setShowBadge(false)
            }
            
            // 3. Peer Events Channel (DEFAULT priority)
            val peerEventsChannel = NotificationChannel(
                CHANNEL_PEER_EVENTS,
                "Peer Events",
                NotificationManager.IMPORTANCE_DEFAULT
            ).apply {
                description = "Peer discovery and connection events"
                group = GROUP_MESH
                enableLights(false)
                enableVibration(false)
                setShowBadge(false)
            }
            
            // 4. System Channel (LOW priority)
            val systemChannel = NotificationChannel(
                CHANNEL_SYSTEM,
                "System",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "System notifications and updates"
                group = GROUP_MESH
                enableLights(false)
                enableVibration(false)
                setShowBadge(false)
            }
            
            notificationManager.createNotificationChannels(
                listOf(messagesChannel, meshStatusChannel, peerEventsChannel, systemChannel)
            )
            
            Timber.d("Notification channels created")
        }
    }
    
    /**
     * Build foreground service notification for MeshForegroundService.
     */
    fun buildForegroundServiceNotification(
        context: Context,
        peerCount: Int,
        relayCount: Int
    ): Notification {
        val intent = context.packageManager.getLaunchIntentForPackage(context.packageName)
        val pendingIntent = PendingIntent.getActivity(
            context,
            0,
            intent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )
        
        val contentText = "Connected: $peerCount peers • Relayed: $relayCount messages"
        
        return NotificationCompat.Builder(context, CHANNEL_MESH_STATUS)
            .setContentTitle("Mesh Network Active")
            .setContentText(contentText)
            .setSmallIcon(R.drawable.ic_notification)
            .setOngoing(true)
            .setContentIntent(pendingIntent)
            .setSilent(true)
            .build()
    }
    
    /**
     * Show a new message notification.
     * Groups messages by contact/peerId.
     */
    fun showMessageNotification(
        context: Context,
        peerId: String,
        messageId: String,
        content: String,
        nickname: String?,
        timestamp: Long
    ) {
        // Check DND
        if (isDndEnabled(context)) {
            Timber.d("DND enabled, skipping notification")
            return
        }
        
        // Add to group
        val message = NotificationMessage(messageId, content, timestamp)
        messageGroups.getOrPut(peerId) { mutableListOf() }.add(message)
        
        val messages = messageGroups[peerId] ?: return
        val displayName = nickname ?: peerId.take(8)
        
        // Generate identicon for avatar
        val identicon = try {
            generateIdenticonBitmap(peerId.toByteArray(), 128)
        } catch (e: Exception) {
            Timber.e(e, "Failed to generate identicon")
            null
        }
        
        // Create person for messaging style
        val person = Person.Builder()
            .setName(displayName)
            .apply { identicon?.let { setIcon(IconCompat.createWithBitmap(it)) } }
            .build()
        
        // Build messaging style notification
        val messagingStyle = NotificationCompat.MessagingStyle(person)
        messages.forEach { msg ->
            messagingStyle.addMessage(msg.content, msg.timestamp, person)
        }
        
        // Reply action with RemoteInput
        val replyIntent = createReplyIntent(context, peerId, messageId)
        val replyPendingIntent = PendingIntent.getBroadcast(
            context,
            peerId.hashCode(),
            replyIntent,
            PendingIntent.FLAG_MUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )
        
        val remoteInput = RemoteInput.Builder(KEY_REPLY_TEXT)
            .setLabel("Reply")
            .build()
        
        val replyAction = NotificationCompat.Action.Builder(
            R.drawable.ic_notification,
            "Reply",
            replyPendingIntent
        )
            .addRemoteInput(remoteInput)
            .setAllowGeneratedReplies(true)
            .build()
        
        // Mark Read action
        val markReadIntent = createMarkReadIntent(context, peerId, messageId)
        val markReadPendingIntent = PendingIntent.getBroadcast(
            context,
            peerId.hashCode() + 1,
            markReadIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )
        
        val markReadAction = NotificationCompat.Action.Builder(
            R.drawable.ic_notification,
            "Mark Read",
            markReadPendingIntent
        ).build()
        
        // Mute action
        val muteIntent = createMuteIntent(context, peerId)
        val mutePendingIntent = PendingIntent.getBroadcast(
            context,
            peerId.hashCode() + 2,
            muteIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )
        
        val muteAction = NotificationCompat.Action.Builder(
            R.drawable.ic_notification,
            "Mute",
            mutePendingIntent
        ).build()
        
        // Tap to open chat
        val chatIntent = context.packageManager.getLaunchIntentForPackage(context.packageName)?.apply {
            putExtra(EXTRA_PEER_ID, peerId)
        }
        val chatPendingIntent = PendingIntent.getActivity(
            context,
            peerId.hashCode(),
            chatIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )
        
        // Build notification
        val notification = NotificationCompat.Builder(context, CHANNEL_MESSAGES)
            .setStyle(messagingStyle)
            .setSmallIcon(R.drawable.ic_notification)
            .setGroup(peerId)
            .setAutoCancel(true)
            .setContentIntent(chatPendingIntent)
            .addAction(replyAction)
            .addAction(markReadAction)
            .addAction(muteAction)
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setCategory(NotificationCompat.CATEGORY_MESSAGE)
            .build()
        
        val notificationId = NOTIFICATION_ID_MESSAGE_BASE + peerId.hashCode()
        NotificationManagerCompat.from(context).notify(notificationId, notification)
        
        Timber.d("Message notification shown for $peerId")
    }
    
    /**
     * Clear message notifications for a specific peer.
     */
    fun clearMessageNotifications(context: Context, peerId: String) {
        messageGroups.remove(peerId)
        val notificationId = NOTIFICATION_ID_MESSAGE_BASE + peerId.hashCode()
        NotificationManagerCompat.from(context).cancel(notificationId)
        Timber.d("Cleared notifications for $peerId")
    }
    
    /**
     * Show peer discovery notification.
     */
    fun showPeerDiscoveredNotification(
        context: Context,
        peerId: String,
        transport: String
    ) {
        if (isDndEnabled(context)) return
        
        val notification = NotificationCompat.Builder(context, CHANNEL_PEER_EVENTS)
            .setContentTitle("Peer Discovered")
            .setContentText("$peerId via $transport")
            .setSmallIcon(R.drawable.ic_notification)
            .setAutoCancel(true)
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .build()
        
        NotificationManagerCompat.from(context).notify(
            NOTIFICATION_ID_PEER_EVENT + peerId.hashCode(),
            notification
        )
    }
    
    /**
     * Show mesh status notification (connection issues, etc).
     */
    fun showMeshStatusNotification(
        context: Context,
        title: String,
        message: String
    ) {
        val notification = NotificationCompat.Builder(context, CHANNEL_MESH_STATUS)
            .setContentTitle(title)
            .setContentText(message)
            .setSmallIcon(R.drawable.ic_notification)
            .setAutoCancel(true)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .build()
        
        NotificationManagerCompat.from(context).notify(NOTIFICATION_ID_MESH_STATUS, notification)
    }
    
    // Helper methods
    
    private fun createReplyIntent(context: Context, peerId: String, messageId: String): Intent {
        return Intent(ACTION_REPLY).apply {
            setPackage(context.packageName)
            putExtra(EXTRA_PEER_ID, peerId)
            putExtra(EXTRA_MESSAGE_ID, messageId)
        }
    }
    
    private fun createMarkReadIntent(context: Context, peerId: String, messageId: String): Intent {
        return Intent(ACTION_MARK_READ).apply {
            setPackage(context.packageName)
            putExtra(EXTRA_PEER_ID, peerId)
            putExtra(EXTRA_MESSAGE_ID, messageId)
        }
    }
    
    private fun createMuteIntent(context: Context, peerId: String): Intent {
        return Intent(ACTION_MUTE).apply {
            setPackage(context.packageName)
            putExtra(EXTRA_PEER_ID, peerId)
        }
    }
    
    private fun isDndEnabled(context: Context): Boolean {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            val notificationManager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            return notificationManager.currentInterruptionFilter != NotificationManager.INTERRUPTION_FILTER_ALL
        }
        return false
    }
    
    /**
     * Data class for grouping messages.
     */
    private data class NotificationMessage(
        val messageId: String,
        val content: String,
        val timestamp: Long
    )
}
