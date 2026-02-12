package com.scmessenger.android.service

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import com.scmessenger.android.R
import com.scmessenger.android.ui.MainActivity
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
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
 */
@AndroidEntryPoint
class MeshForegroundService : Service() {
    
    private val serviceScope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    
    @Inject
    lateinit var meshRepository: com.scmessenger.android.data.MeshRepository
    
    @Inject
    lateinit var platformBridge: AndroidPlatformBridge
    
    private var isRunning = false
    
    override fun onCreate() {
        super.onCreate()
        Timber.d("MeshForegroundService created")
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
        
        // Initialize platform bridge to monitor system state
        platformBridge.initialize()
        
        // Create mesh service configuration
        val config = uniffi.api.MeshServiceConfig(
            discoveryIntervalMs = 30000u,  // 30 seconds
            relayBudgetPerHour = 200u,
            batteryFloorPct = 20u
        )
        
        // Start mesh service via repository
        try {
            meshRepository.startMeshService(config)
            isRunning = true
            
            // Listen for incoming messages and show notifications
            serviceScope.launch {
                meshRepository.incomingMessages.collect { message ->
                    showMessageNotification(message)
                }
            }
            
            Timber.i("Mesh service started successfully")
        } catch (e: Exception) {
            Timber.e(e, "Failed to start mesh service")
            stopSelf()
        }
    }
    
    private fun stopMeshService() {
        if (!isRunning) {
            Timber.w("Mesh service not running")
            return
        }
        
        Timber.i("Stopping mesh service")
        
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
        
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle(getString(R.string.mesh_service_notification_title))
            .setContentText(getString(R.string.mesh_service_notification_text))
            .setSmallIcon(R.drawable.ic_notification)  // TODO: Create icon
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .setForegroundServiceBehavior(NotificationCompat.FOREGROUND_SERVICE_IMMEDIATE)
            .build()
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
