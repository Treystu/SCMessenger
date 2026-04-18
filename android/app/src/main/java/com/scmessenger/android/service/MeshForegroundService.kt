package com.scmessenger.android.service

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import androidx.core.app.NotificationCompat
import androidx.core.app.ServiceCompat
import com.scmessenger.android.R
import com.scmessenger.android.ui.MainActivity
import com.scmessenger.android.utils.NotificationHelper
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import timber.log.Timber
import java.util.Collections
import java.util.concurrent.atomic.AtomicInteger
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

    @Volatile private var isRunning = false
    private val connectedPeers: MutableSet<String> = Collections.synchronizedSet(mutableSetOf())
    private val messagesRelayed = AtomicInteger(0)

    // WakeLock for BLE scan windows
    private var wakeLock: PowerManager.WakeLock? = null
    private lateinit var anrWatchdog: AnrWatchdog

    // Performance monitor for ANR tracking and health diagnostics
    private lateinit var performanceMonitor: PerformanceMonitor

    override fun onCreate() {
        super.onCreate()
        Timber.d("MeshForegroundService created")

        // Initialize WakeLock
        val powerManager = getSystemService(POWER_SERVICE) as PowerManager
        wakeLock = powerManager.newWakeLock(
            PowerManager.PARTIAL_WAKE_LOCK,
            "SCMessenger::MeshService"
        )

        // Initialize ANR watchdog
        anrWatchdog = AnrWatchdog(this)

        // Initialize performance monitor
        performanceMonitor = PerformanceMonitor(this)
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        serviceScope.launch {
            val repoRunning = withContext(Dispatchers.Default) {
                meshRepository.getServiceState() == uniffi.api.ServiceState.RUNNING
            }
            when (decideCommand(intent?.action, isRunning, repoRunning)) {
                StartDecision.Start -> startMeshService()
                StartDecision.Stop -> stopMeshService()
                StartDecision.Pause -> pauseMeshService()
                StartDecision.Resume -> resumeMeshService()
                StartDecision.NoOp -> Timber.w("Ignoring pause request while service is not running")
            }
        }

        return START_STICKY
    }

    private fun startMeshService() {
        serviceScope.launch {
            val repoRunning = withContext(Dispatchers.Default) {
                meshRepository.getServiceState() == uniffi.api.ServiceState.RUNNING
            }
            if (isRunning || repoRunning) {
                if (!tryStartForeground()) {
                    Timber.e("Foreground promotion denied while mesh repository is already running")
                    stopSelf()
                    return@launch
                }
                isRunning = true
                updateNotification()
                Timber.w("Mesh service already running; foreground promotion refreshed")
                return@launch
            }

            Timber.i("Starting mesh service")

            // Start foreground with notification. Android 14+ can reject this if
            // app state/permissions are not currently eligible.
            if (!tryStartForeground()) {
                Timber.e("Foreground start denied by OS; aborting mesh startup")
                stopSelf()
                return@launch
            }

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
                withContext(Dispatchers.Default) {
                    meshRepository.startMeshService(config)
                    meshRepository.setPlatformBridge(platformBridge)
                }

                val started = withContext(Dispatchers.Default) {
                    meshRepository.getServiceState() == uniffi.api.ServiceState.RUNNING
                }
                if (!started) {
                    throw IllegalStateException("Repository did not reach RUNNING state")
                }
                isRunning = true

                // Wire CoreDelegate callbacks to MeshEventBus
                wireCoreDelegate()

                // Listen for incoming messages and show notifications (WS14: with classification)
                launch {
                    meshRepository.incomingMessages.collect { message ->
                        showMessageNotificationWithClassification(message)
                    }
                }

                // Listen for peer events to update notification
                launch {
                    MeshEventBus.peerEvents.collect { event ->
                        when (event) {
                            is PeerEvent.Connected -> {
                                connectedPeers.add(event.peerId)
                                updateNotification()
                            }
                            is PeerEvent.Disconnected -> {
                                connectedPeers.remove(event.peerId)
                                updateNotification()
                            }
                            else -> {}
                        }
                    }
                }

                // Listen for status events to update relay stats
                launch {
                    MeshEventBus.statusEvents.collect { event ->
                        when (event) {
                            is StatusEvent.StatsUpdated -> {
                                messagesRelayed.set(event.stats.messagesRelayed.toInt())
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

                // Start ANR watchdog after service is running
                anrWatchdog.start()

                // Record service start for uptime tracking
                performanceMonitor.recordServiceStart()

                Timber.i("Mesh service started successfully - ANR watchdog active")
            } catch (e: Exception) {
                Timber.e(e, "Failed to start mesh service")
                isRunning = false
                releaseWakeLock()
                stopForeground(STOP_FOREGROUND_REMOVE)
                stopSelf()
            }
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
        serviceScope.launch {
            val repoRunning = withContext(Dispatchers.Default) {
                meshRepository.getServiceState() == uniffi.api.ServiceState.RUNNING
            }
            if (!isRunning && !repoRunning) {
                Timber.w("Mesh service stop requested while already stopped")
            }

            Timber.i("Stopping mesh service")

            // Release WakeLock
            releaseWakeLock()

            // Stop mesh service via repository
            withContext(Dispatchers.Default) {
                kotlin.runCatching { meshRepository.stopMeshService() }
                    .onFailure { Timber.e(it, "Error while stopping mesh repository") }
            }

            isRunning = false
            connectedPeers.clear()
            messagesRelayed.set(0)
            anrWatchdog.stop()

            // Record service stop for health metrics
            performanceMonitor.recordServiceStop()

            // Clean up
            withContext(Dispatchers.Default) {
                kotlin.runCatching { platformBridge.cleanup() }
                    .onFailure { Timber.w(it, "Platform bridge cleanup failed during stop") }
            }

            stopForeground(STOP_FOREGROUND_REMOVE)
            stopSelf()
        }
    }

    private fun pauseMeshService() {
        serviceScope.launch {
            withContext(Dispatchers.Default) {
                Timber.i("Pausing mesh service (reduced activity)")
                meshRepository.pauseMeshService()
            }
        }
    }

    private fun resumeMeshService() {
        serviceScope.launch {
            withContext(Dispatchers.Default) {
                Timber.i("Resuming mesh service (full activity)")
                meshRepository.resumeMeshService()
            }
        }
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

        val contentText = if (connectedPeers.isNotEmpty()) {
            "Connected to ${connectedPeers.size} peers • ${messagesRelayed.get()} relayed"
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

    private fun tryStartForeground(): Boolean {
        val notification = createNotification()
        return try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                ServiceCompat.startForeground(
                    this,
                    NOTIFICATION_ID,
                    notification,
                    ServiceInfo.FOREGROUND_SERVICE_TYPE_CONNECTED_DEVICE
                )
            } else {
                startForeground(NOTIFICATION_ID, notification)
            }
            true
        } catch (e: SecurityException) {
            Timber.e(e, "SecurityException while starting foreground service")
            false
        } catch (e: IllegalStateException) {
            val isForegroundServiceStartNotAllowed = Build.VERSION.SDK_INT >= Build.VERSION_CODES.S &&
                e.javaClass.name == "android.app.ForegroundServiceStartNotAllowedException"
            if (isForegroundServiceStartNotAllowed) {
                Timber.e(e, "ForegroundServiceStartNotAllowedException while starting foreground service")
                false
            } else {
                throw e
            }
        }
    }

    /**
     * WS14: Show notification with DM vs DM Request classification.
     * Uses NotificationHelper for proper channel routing and settings.
     *
     * This method runs on the repoScope (IO dispatcher) to avoid blocking the main thread.
     * All FFI calls are dispatched to the IO thread, and the notification is posted
     * on the main thread using the main handler.
     */
    private fun showMessageNotificationWithClassification(message: uniffi.api.MessageRecord) {
        serviceScope.launch {
            try {
                // Run FFI calls on IO dispatcher to avoid main thread blocking
                val contactData = withContext(Dispatchers.Default) {
                    try {
                        meshRepository.getContact(message.peerId)
                    } catch (e: Exception) {
                        Timber.w(e, "Failed to get contact for ${message.peerId}")
                        null
                    }
                }
                val isKnownContact = contactData != null

                // Check if conversation exists
                val hasExistingConversation = withContext(Dispatchers.Default) {
                    try {
                        meshRepository.hasConversationWith(message.peerId)
                    } catch (e: Exception) {
                        Timber.w(e, "Failed to check conversation for ${message.peerId}")
                        false
                    }
                }

                // Get UI state for foreground suppression (cheap check)
                val appInForeground = isAppInForeground()
                val activeConversationId = getActiveConversationId()

                // Get nickname from contact if available
                val nickname = contactData?.nickname ?: contactData?.localNickname

                // Post notification to main thread
                withContext(Dispatchers.Main) {
                    // Use NotificationHelper with WS14 classification
                    // Note: explicitDmRequest is not available in MessageRecord; classification will infer from contact state
                    NotificationHelper.showMessageNotification(
                        context = this@MeshForegroundService,
                        peerId = message.peerId,
                        messageId = message.id,
                        content = message.content,
                        nickname = nickname,
                        timestamp = message.timestamp.toLong(),
                        isKnownContact = isKnownContact,
                        hasExistingConversation = hasExistingConversation,
                        appInForeground = appInForeground,
                        activeConversationId = activeConversationId,
                        explicitDmRequest = null
                    )
                }
            } catch (e: Exception) {
                Timber.e(e, "Failed to process message notification for ${message.peerId}")
            }
        }
    }

    /**
     * Check if app is in foreground.
     *
     * Optimized for ANR prevention:
     * - Caches the activity manager reference
     * - Uses a simpler process check that's less likely to block
     */
    @Volatile private var activityManager: android.app.ActivityManager? = null

    private fun isAppInForeground(): Boolean {
        // Fast path: use cached activity manager
        val am = activityManager
            ?: (getSystemService(Context.ACTIVITY_SERVICE) as android.app.ActivityManager).also {
                activityManager = it
            }

        return try {
            // Use runningAppProcesses - it's cached by the system
            // Only check if we're the topmost foreground process
            val runningAppProcesses = am.runningAppProcesses
            runningAppProcesses?.any {
                it.processName == packageName &&
                it.importance == android.app.ActivityManager.RunningAppProcessInfo.IMPORTANCE_FOREGROUND
            } ?: false
        } catch (e: Exception) {
            Timber.w(e, "isAppInForeground check failed")
            false
        }
    }

    /**
     * Get currently active conversation ID.
     */
    private fun getActiveConversationId(): String? {
        // In production, would track this via activity lifecycle or event bus
        return null
    }

    private fun createNotificationChannel() {
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

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        super.onDestroy()
        Timber.d("MeshForegroundService destroyed")
        releaseWakeLock()
        serviceScope.launch {
            val repoRunning = withContext(Dispatchers.Default) {
                meshRepository.getServiceState() == uniffi.api.ServiceState.RUNNING
            }
            if (repoRunning) {
                withContext(Dispatchers.Default) {
                    kotlin.runCatching { meshRepository.stopMeshService() }
                        .onFailure { Timber.w(it, "Repository stop failed during service destroy") }
                }
            }
            serviceScope.cancel()
        }
    }

    companion object {
        private const val CHANNEL_ID = "mesh_service_channel"
        private const val NOTIFICATION_ID = 1001

        const val ACTION_START = "com.scmessenger.android.service.START"
        const val ACTION_STOP = "com.scmessenger.android.service.STOP"
        const val ACTION_PAUSE = "com.scmessenger.android.service.PAUSE"
        const val ACTION_RESUME = "com.scmessenger.android.service.RESUME"

        internal enum class StartDecision {
            Start,
            Stop,
            Pause,
            Resume,
            NoOp
        }

        internal fun decideCommand(
            action: String?,
            serviceRunning: Boolean,
            repositoryRunning: Boolean
        ): StartDecision {
            return when (action) {
                null -> StartDecision.Start
                ACTION_START -> StartDecision.Start
                ACTION_STOP -> StartDecision.Stop
                ACTION_PAUSE -> if (serviceRunning || repositoryRunning) {
                    StartDecision.Pause
                } else {
                    StartDecision.NoOp
                }
                ACTION_RESUME -> if (serviceRunning && repositoryRunning) {
                    StartDecision.Resume
                } else {
                    StartDecision.Start
                }
                else -> StartDecision.Start
            }
        }
    }
}
