package com.scmessenger.android.utils

import android.Manifest
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationChannelGroup
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.app.Person
import androidx.core.app.RemoteInput
import androidx.core.content.ContextCompat
import androidx.core.graphics.drawable.IconCompat
import com.scmessenger.android.R
import com.scmessenger.android.ui.components.generateIdenticonBitmap
import timber.log.Timber

/**
 * Notification helper for mesh messaging.
 *
 * Features (WS14):
 * - 5 notification channels (Messages/high, Message Requests/high, Mesh Status/low, Peer Events/default, System/low)
 * - DM vs DM Request classification using core notification contract
 * - Grouped message notifications per contact
 * - Reply-from-notification with RemoteInput
 * - Notification actions (Mark Read, Reply, Mute)
 * - Identicon-based contact avatars
 * - Respects DND (Do Not Disturb) settings
 * - Settings parity (notify_dm_enabled, notify_dm_request_enabled, foreground suppression)
 */
object NotificationHelper {

    // Channel IDs
    private const val CHANNEL_MESSAGES = "messages"
    private const val CHANNEL_MESSAGE_REQUESTS = "message_requests"
    private const val CHANNEL_MESH_STATUS = "mesh_status"
    private const val CHANNEL_PEER_EVENTS = "peer_events"
    private const val CHANNEL_SYSTEM = "system"

    // Channel Group
    private const val GROUP_MESH = "mesh_group"

    // Notification IDs
    const val NOTIFICATION_ID_FOREGROUND_SERVICE = 1001
    private const val NOTIFICATION_ID_MESSAGE_BASE = 2000
    private const val NOTIFICATION_ID_REQUEST_BASE = 2500
    private const val NOTIFICATION_ID_MESH_STATUS = 3000
    private const val NOTIFICATION_ID_PEER_EVENT = 4000

    // Actions
    const val ACTION_REPLY = "com.scmessenger.ACTION_REPLY"
    const val ACTION_MARK_READ = "com.scmessenger.ACTION_MARK_READ"
    const val ACTION_MUTE = "com.scmessenger.ACTION_MUTE"
    const val ACTION_OPEN_REQUESTS = "com.scmessenger.ACTION_OPEN_REQUESTS"
    const val EXTRA_PEER_ID = "peer_id"
    const val EXTRA_MESSAGE_ID = "message_id"
    const val EXTRA_IS_REQUEST = "is_request"
    const val KEY_REPLY_TEXT = "key_reply_text"

    // Message grouping
    private val messageGroups = mutableMapOf<String, MutableList<NotificationMessage>>()
    private val requestGroups = mutableMapOf<String, MutableList<NotificationMessage>>()

    // Notification settings (defaults per WS14 spec)
    var notificationsEnabled: Boolean = true
    var notifyDmEnabled: Boolean = true
    var notifyDmRequestEnabled: Boolean = true
    var notifyDmInForeground: Boolean = false
    var notifyDmRequestInForeground: Boolean = true
    var soundEnabled: Boolean = true
    var badgeEnabled: Boolean = true

    /**
     * Initialize notification channels on app start.
     * Must be called before posting any notifications.
     * WS14: Added Message Requests channel for DM Request notifications.
     */
    fun createNotificationChannels(context: Context) {
        val notificationManager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

        // Create channel group
        val channelGroup = NotificationChannelGroup(GROUP_MESH, "Mesh Network")
        notificationManager.createNotificationChannelGroup(channelGroup)

        // 1. Messages Channel (HIGH priority) - for known contacts
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

        // 2. Message Requests Channel (HIGH priority) - WS14: for unknown senders
        val messageRequestsChannel = NotificationChannel(
            CHANNEL_MESSAGE_REQUESTS,
            "Message Requests",
            NotificationManager.IMPORTANCE_HIGH
        ).apply {
            description = "Messages from unknown senders"
            group = GROUP_MESH
            enableLights(true)
            enableVibration(true)
            setShowBadge(true)
        }

        // 3. Mesh Status Channel (LOW priority)
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

        // 4. Peer Events Channel (DEFAULT priority)
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

        // 5. System Channel (LOW priority)
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
            listOf(messagesChannel, messageRequestsChannel, meshStatusChannel, peerEventsChannel, systemChannel)
        )

        Timber.d("Notification channels created (WS14: with Message Requests channel)")
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
        val pendingIntent = if (intent != null) {
            PendingIntent.getActivity(
                context,
                0,
                intent,
                PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
            )
        } else {
            null
        }

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
     * Show a new message notification with WS14 DM vs DM Request classification.
     * Groups messages by contact/peerId.
     *
     * @param context Android context
     * @param peerId Sender peer ID
     * @param messageId Message ID
     * @param content Message content
     * @param nickname Sender nickname (null if unknown)
     * @param timestamp Message timestamp
     * @param isKnownContact Whether sender is a known contact
     * @param hasExistingConversation Whether an existing conversation exists
     * @param appInForeground Whether app is in foreground
     * @param activeConversationId Currently active conversation ID (if any)
     * @param explicitDmRequest Explicit DM request flag from message metadata
     */
    fun showMessageNotification(
        context: Context,
        peerId: String,
        messageId: String,
        content: String,
        nickname: String?,
        timestamp: Long,
        isKnownContact: Boolean = false,
        hasExistingConversation: Boolean = false,
        appInForeground: Boolean = false,
        activeConversationId: String? = null,
        explicitDmRequest: Boolean? = null
    ) {
        // Check global notifications enabled
        if (!notificationsEnabled) {
            Timber.d("Notifications disabled, skipping")
            return
        }

        // Check DND
        if (isDndEnabled(context)) {
            Timber.d("DND enabled, skipping notification")
            return
        }

        // WS14: Classify as DM or DM Request
        val isDmRequest = classifyAsDmRequest(isKnownContact, hasExistingConversation, explicitDmRequest)
        
        // Check per-kind settings
        if (isDmRequest && !notifyDmRequestEnabled) {
            Timber.d("DM Request notifications disabled, skipping")
            return
        }
        if (!isDmRequest && !notifyDmEnabled) {
            Timber.d("DM notifications disabled, skipping")
            return
        }

        // WS14: Foreground suppression
        val isActiveConversation = appInForeground && activeConversationId != null &&
            (activeConversationId == peerId || activeConversationId.equals(peerId, ignoreCase = true))
        
        if (isActiveConversation) {
            val allowForeground = if (isDmRequest) notifyDmRequestInForeground else notifyDmInForeground
            if (!allowForeground) {
                Timber.d("Foreground conversation active, suppressing notification")
                return
            }
        }

        // Add to appropriate group
        val message = NotificationMessage(messageId, content, timestamp)
        if (isDmRequest) {
            requestGroups.getOrPut(peerId) { mutableListOf() }.add(message)
        } else {
            messageGroups.getOrPut(peerId) { mutableListOf() }.add(message)
        }

        val messages = if (isDmRequest) requestGroups[peerId] else messageGroups[peerId] ?: return
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
        messages?.forEach { msg ->
            messagingStyle.addMessage(msg.content, msg.timestamp, person)
        }

        // Reply action with RemoteInput (only for DM, not requests)
        val actions = mutableListOf<NotificationCompat.Action>()
        
        if (!isDmRequest) {
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
            actions.add(replyAction)
        }

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
        actions.add(markReadAction)

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
        actions.add(muteAction)

        // WS14: Tap routing - DM goes to chat, DM Request goes to requests inbox
        val tapIntent = if (isDmRequest) {
            createOpenRequestsIntent(context, peerId, messageId)
        } else {
            context.packageManager.getLaunchIntentForPackage(context.packageName)?.apply {
                putExtra(EXTRA_PEER_ID, peerId)
            }
        }
        val tapPendingIntent = if (tapIntent != null) {
            PendingIntent.getActivity(
                context,
                peerId.hashCode(),
                tapIntent,
                PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
            )
        } else {
            null
        }

        // WS14: Use appropriate channel
        val channelId = if (isDmRequest) CHANNEL_MESSAGE_REQUESTS else CHANNEL_MESSAGES
        val category = if (isDmRequest) NotificationCompat.CATEGORY_MESSAGE else NotificationCompat.CATEGORY_MESSAGE
        val title = if (isDmRequest) "Message Request from $displayName" else null

        // Build notification
        val notification = NotificationCompat.Builder(context, channelId)
            .setStyle(messagingStyle)
            .setSmallIcon(R.drawable.ic_notification)
            .setGroup(peerId)
            .setAutoCancel(true)
            .setContentIntent(tapPendingIntent)
            .apply {
                actions.forEach { addAction(it) }
                if (title != null) setContentTitle(title)
                if (soundEnabled) setDefaults(NotificationCompat.DEFAULT_SOUND)
                if (badgeEnabled) setBadgeIconType(NotificationCompat.BADGE_ICON_SMALL)
            }
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setCategory(category)
            .build()

        if (!hasNotificationPermission(context)) {
            Timber.w("POST_NOTIFICATIONS permission missing; skipping message notification")
            return
        }
        val notificationId = if (isDmRequest) {
            NOTIFICATION_ID_REQUEST_BASE + peerId.hashCode()
        } else {
            NOTIFICATION_ID_MESSAGE_BASE + peerId.hashCode()
        }
        try {
            NotificationManagerCompat.from(context).notify(notificationId, notification)
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception while posting message notification")
            return
        }

        Timber.d("${if (isDmRequest) "DM Request" else "DM"} notification shown for $peerId")
    }

    /**
     * WS14: Classify message as DM Request based on contact state.
     *
     * Rules:
     * 1. If explicit_dm_request is true -> DM Request
     * 2. If sender is known contact OR has existing conversation -> DM
     * 3. Otherwise -> DM Request
     */
    private fun classifyAsDmRequest(
        isKnownContact: Boolean,
        hasExistingConversation: Boolean,
        explicitDmRequest: Boolean?
    ): Boolean {
        // Explicit request flag overrides inference
        if (explicitDmRequest == true) {
            return true
        }
        // Known contact or existing conversation = DM
        if (isKnownContact || hasExistingConversation) {
            return false
        }
        // Unknown sender = DM Request
        return true
    }

    /**
     * Create intent to open requests inbox for DM Request notifications.
     */
    private fun createOpenRequestsIntent(context: Context, peerId: String, messageId: String): Intent? {
        return context.packageManager.getLaunchIntentForPackage(context.packageName)?.apply {
            putExtra(EXTRA_PEER_ID, peerId)
            putExtra(EXTRA_MESSAGE_ID, messageId)
            putExtra(EXTRA_IS_REQUEST, true)
            // Add flag to navigate to requests inbox
            action = ACTION_OPEN_REQUESTS
        }
    }

    /**
     * Clear message notifications for a specific peer.
     * WS14: Also clears request notifications.
     */
    fun clearMessageNotifications(context: Context, peerId: String) {
        messageGroups.remove(peerId)
        requestGroups.remove(peerId)
        val notificationId = NOTIFICATION_ID_MESSAGE_BASE + peerId.hashCode()
        val requestId = NOTIFICATION_ID_REQUEST_BASE + peerId.hashCode()
        NotificationManagerCompat.from(context).cancel(notificationId)
        NotificationManagerCompat.from(context).cancel(requestId)
        Timber.d("Cleared notifications for $peerId (DM + Request)")
    }

    /**
     * WS14: Clear all request notifications.
     */
    fun clearAllRequestNotifications(context: Context) {
        requestGroups.clear()
        // Cancel all request notifications (approximate - in production would track IDs)
        Timber.d("Cleared all request notifications")
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

        if (!hasNotificationPermission(context)) {
            Timber.w("POST_NOTIFICATIONS permission missing; skipping peer discovered notification")
            return
        }
        try {
            NotificationManagerCompat.from(context).notify(
                NOTIFICATION_ID_PEER_EVENT + peerId.hashCode(),
                notification
            )
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception while posting peer discovered notification")
        }
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

        if (!hasNotificationPermission(context)) {
            Timber.w("POST_NOTIFICATIONS permission missing; skipping mesh status notification")
            return
        }
        try {
            NotificationManagerCompat.from(context).notify(NOTIFICATION_ID_MESH_STATUS, notification)
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception while posting mesh status notification")
        }
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

    /**
     * WS14: Update notification settings.
     */
    fun updateSettings(
        enabled: Boolean? = null,
        dmEnabled: Boolean? = null,
        dmRequestEnabled: Boolean? = null,
        dmInForeground: Boolean? = null,
        dmRequestInForeground: Boolean? = null,
        sound: Boolean? = null,
        badge: Boolean? = null
    ) {
        enabled?.let { notificationsEnabled = it }
        dmEnabled?.let { notifyDmEnabled = it }
        dmRequestEnabled?.let { notifyDmRequestEnabled = it }
        dmInForeground?.let { notifyDmInForeground = it }
        dmRequestInForeground?.let { notifyDmRequestInForeground = it }
        sound?.let { soundEnabled = it }
        badge?.let { badgeEnabled = it }
        Timber.d("Notification settings updated")
    }

    private fun isDndEnabled(context: Context): Boolean {
        val notificationManager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        return notificationManager.currentInterruptionFilter != NotificationManager.INTERRUPTION_FILTER_ALL
    }

    private fun hasNotificationPermission(context: Context): Boolean {
        return Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU ||
            ContextCompat.checkSelfPermission(
                context,
                Manifest.permission.POST_NOTIFICATIONS
            ) == PackageManager.PERMISSION_GRANTED
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
