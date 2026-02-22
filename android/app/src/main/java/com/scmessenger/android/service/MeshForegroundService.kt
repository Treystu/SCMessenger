package com.scmessenger.android.service

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import androidx.core.app.NotificationCompat
import com.scmessenger.android.R
import com.scmessenger.android.ui.MainActivity
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import timber.log.Timber
import javax.inject.Inject

/**
 * Foreground service maintaining mesh network connectivity.
 *
 * This service:
 * - Keeps the mesh network alive while the app is backgrounded
 * - Manages BLE, WiFi Aware, and WiFi Direct transports
 * - Relays messages according to configured budget
 * - Adapts behavior based on battery/network state via AutoAdjustEngine
 * - Uses WakeLock for BLE scan windows
 * - Periodically computes AutoAdjust profile adjustments
 */
@AndroidEntryPoint
class MeshForegroundService : Service() {

    private val serviceScope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

    @Inject
    lateinit var meshRepository: com.scmessenger.android.data.MeshRepository

    @Inject
    lateinit var platformBridge: AndroidPlatformBridge

    private var isRunning = false
    private var peerCount = 0
    private var messagesRelayed = 0

    // WakeLock for BLE scan windows
    private var wakeLock: PowerManager.WakeLock? = null

    override fun onCreate() {
        super.onCreate()
        Timber.d("MeshForegroundService created")

        // Initialize WakeLock
        val powerManager = getSystemService(POWER_SERVICE) as PowerManager
        wakeLock = powerManager.newWakeLock(
            PowerManager.PARTIAL_WAKE_LOCK,
            "SCMessenger::MeshService"
        )
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_START -> startMeshService()
            ACTION_STOP -> stopMeshService()
            ACTION_PAUSE -> pauseMeshService()
            ACTION_RESUME -> resumeMeshService()
        }

        return START_STICKY
    }

    private fun startMeshService() {
        if (isRunning) {
            Timber.w("Mesh service already running")
            return
        }

        Timber.i("Starting mesh service")

        // Start foreground with notification
        startForeground(NOTIFICATION_ID, createNotification())

        // Acquire WakeLock for scan windows
        acquireWakeLock()

        // Initialize platform bridge to monitor system state
        platformBridge.initialize()

        // Create mesh service configuration
        val config = uniffi.api.MeshServiceConfig(
            discoveryIntervalMs = 30000u,  // 30 seconds
            batteryFloorPct = 20u
        )

        // Start mesh service via repository
        try {
            meshRepository.startMeshService(config)
            meshRepository.setPlatformBridge(platformBridge)
            isRunning = true

            // Wire CoreDelegate callbacks to MeshEventBus
            wireCoreDelegate()

            // Listen for incoming messages and show notifications
            serviceScope.launch {
                meshRepository.incomingMessages.collect { message ->
                    showMessageNotification(message)
                }
            }

            // Listen for peer events to update notification
            serviceScope.launch {
                MeshEventBus.peerEvents.collect { event ->
                    when (event) {
                        is PeerEvent.Connected -> {
                            peerCount++
                            updateNotification()
                        }
                        is PeerEvent.Disconnected -> {
                            peerCount = maxOf(0, peerCount - 1)
                            updateNotification()
                        }
                        else -> {}
                    }
                }
            }

            // Listen for status events to update relay stats
            serviceScope.launch {
                MeshEventBus.statusEvents.collect { event ->
                    when (event) {
                        is StatusEvent.StatsUpdated -> {
                            messagesRelayed = event.stats.messagesRelayed.toInt()
                            updateNotification()
                        }
                        else -> {}
                    }
                }
            }

            // Start periodic AutoAdjust profile computation
            startPeriodicAdjustments()

            // Start periodic WakeLock renewal
            startPeriodicWakeLockRenewal()

            Timber.i("Mesh service started successfully")
        } catch (e: Exception) {
            Timber.e(e, "Failed to start mesh service")
            releaseWakeLock()
            stopSelf()
        }
    }

    private fun wireCoreDelegate() {
        // Core delegate wiring is handled in MeshRepository
        // We listen to MeshEventBus which receives events from CoreDelegate
        Timber.d("CoreDelegate wired to MeshEventBus")
    }

    private fun startPeriodicAdjustments() {
        serviceScope.launch {
            while (isActive && isRunning) {
                try {
                    // Compute adjustments every 30 seconds
                    delay(30000L)

                    if (isRunning) {
                        // Trigger battery/network state update
                        // which will compute new adjustment profile
                        platformBridge.checkBatteryState()
                        platformBridge.checkNetworkState()

                        Timber.d("Periodic AutoAdjust profile computed")
                    }
                } catch (e: Exception) {
                    Timber.e(e, "Error in periodic adjustments")
                }
            }
        }
    }

    private fun acquireWakeLock() {
        try {
            val lock = wakeLock
            if (lock != null && lock.isHeld) {
                lock.release()
            }
            wakeLock?.acquire(10 * 60 * 1000L) // 10 minutes timeout
            Timber.d("WakeLock acquired for BLE scan windows")
        } catch (e: Exception) {
            Timber.e(e, "Failed to acquire WakeLock")
        }
    }

    private fun releaseWakeLock() {
        try {
            if (wakeLock?.isHeld == true) {
                wakeLock?.release()
                Timber.d("WakeLock released")
            }
        } catch (e: Exception) {
            Timber.e(e, "Failed to release WakeLock")
        }
    }

    private fun startPeriodicWakeLockRenewal() {
        serviceScope.launch {
            while (isActive && isRunning) {
                delay(9 * 60 * 1000L) // Re-acquire every 9 minutes
                if (isRunning) {
                    acquireWakeLock()
                }
            }
        }
    }

    private fun stopMeshService() {
        if (!isRunning) {
            Timber.w("Mesh service not running")
            return
        }

        Timber.i("Stopping mesh service")

        // Release WakeLock
        releaseWakeLock()

        // Stop mesh service via repository
        meshRepository.stopMeshService()

        isRunning = false

        // Clean up
        platformBridge.cleanup()

        stopForeground(STOP_FOREGROUND_REMOVE)
        stopSelf()
    }

    private fun pauseMeshService() {
        Timber.i("Pausing mesh service (reduced activity)")
        meshRepository.pauseMeshService()
    }

    private fun resumeMeshService() {
        Timber.i("Resuming mesh service (full activity)")
        meshRepository.resumeMeshService()
    }

    private fun createNotification(): Notification {
        createNotificationChannel()

        val notificationIntent = Intent(this, MainActivity::class.java)
        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            notificationIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        // Action: Pause Relay
        val pauseIntent = Intent(this, MeshForegroundService::class.java).apply {
            action = ACTION_PAUSE
        }
        val pausePendingIntent = PendingIntent.getService(
            this,
            1,
            pauseIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        // Action: Stop Service
        val stopIntent = Intent(this, MeshForegroundService::class.java).apply {
            action = ACTION_STOP
        }
        val stopPendingIntent = PendingIntent.getService(
            this,
            2,
            stopIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        val contentText = if (peerCount > 0) {
            "Connected to $peerCount peers • $messagesRelayed relayed"
        } else {
            getString(R.string.mesh_service_notification_text)
        }

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle(getString(R.string.mesh_service_notification_title))
            .setContentText(contentText)
            .setSmallIcon(R.drawable.ic_notification)
            .setContentIntent(pendingIntent)
            .addAction(0, "Pause", pausePendingIntent)
            .addAction(0, "Stop", stopPendingIntent)
            .setOngoing(true)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .setForegroundServiceBehavior(NotificationCompat.FOREGROUND_SERVICE_IMMEDIATE)
            .build()
    }

    private fun updateNotification() {
        val notification = createNotification()
        val notificationManager = getSystemService(NotificationManager::class.java)
        notificationManager.notify(NOTIFICATION_ID, notification)
    }

    private fun showMessageNotification(message: uniffi.api.MessageRecord) {
        val notificationManager = getSystemService(NotificationManager::class.java)

        // Intent that opens chat
        val chatIntent = Intent(this, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK
            // Ideally put extra "peerId" -> message.peerId
            putExtra("peerId", message.peerId)
        }
        val pendingIntent = PendingIntent.getActivity(
            this,
            message.peerId.hashCode(),
            chatIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        val notification = NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("Message from " + message.peerId.take(8))
            .setContentText(message.content)
            .setSmallIcon(R.drawable.ic_notification)
            .setContentIntent(pendingIntent)
            .setAutoCancel(true)
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .build()

        notificationManager.notify(message.id.hashCode(), notification)
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                getString(R.string.mesh_service_channel_name),
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = getString(R.string.mesh_service_channel_description)
                setShowBadge(false)
            }

            val notificationManager = getSystemService(NotificationManager::class.java)
            notificationManager.createNotificationChannel(channel)
        }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        super.onDestroy()
        Timber.d("MeshForegroundService destroyed")
        serviceScope.cancel()
    }

    companion object {
        private const val CHANNEL_ID = "mesh_service_channel"
        private const val NOTIFICATION_ID = 1001

        const val ACTION_START = "com.scmessenger.android.service.START"
        const val ACTION_STOP = "com.scmessenger.android.service.STOP"
        const val ACTION_PAUSE = "com.scmessenger.android.service.PAUSE"
        const val ACTION_RESUME = "com.scmessenger.android.service.RESUME"
    }
}
