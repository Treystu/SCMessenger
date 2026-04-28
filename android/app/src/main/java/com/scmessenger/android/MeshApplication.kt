package com.scmessenger.android

import android.app.Application
import dagger.hilt.android.HiltAndroidApp
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import timber.log.Timber

/**
 * SCMessenger Application class with Hilt dependency injection.
 *
 * This is the entry point for the Android application and initializes
 * Hilt's dependency graph.
 */
@HiltAndroidApp
class MeshApplication : Application() {

    private val applicationScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    override fun onCreate() {
        super.onCreate()

        // Initialize Timber logging first for proper logging
        Timber.plant(Timber.DebugTree())
        Timber.plant(com.scmessenger.android.utils.FileLoggingTree(this))

        // Storage health and maintenance - run on background thread to avoid blocking
        // startup. These operations can be slow on large storage or busy devices.
        applicationScope.launch {
            try {
                com.scmessenger.android.utils.StorageManager.performStartupMaintenance(this@MeshApplication)
                Timber.i("Startup storage maintenance completed")
            } catch (e: Exception) {
                Timber.w(e, "Startup maintenance failed")
            }
        }

        // Initialize notification channels before any notification can be posted
        // (including the mesh foreground service notification)
        com.scmessenger.android.utils.NotificationHelper.createNotificationChannels(this)
        Timber.i("Notification channels created")

        Timber.i("SCMessenger application started")

        // Application-level initialization
        // The actual mesh service will be started/stopped by user action
    }

    override fun onTerminate() {
        super.onTerminate()
        applicationScope.cancel()
    }
}
