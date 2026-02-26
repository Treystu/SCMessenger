package com.scmessenger.android

import android.app.Application
import dagger.hilt.android.HiltAndroidApp

/**
 * SCMessenger Application class with Hilt dependency injection.
 *
 * This is the entry point for the Android application and initializes
 * Hilt's dependency graph.
 */
@HiltAndroidApp
class MeshApplication : Application() {

    override fun onCreate() {
        super.onCreate()

        // Initialize Timber logging
        if (BuildConfig.DEBUG) {
            timber.log.Timber.plant(timber.log.Timber.DebugTree())
        }
        
        // Always plant FileLoggingTree for debugging "out and about"
        timber.log.Timber.plant(com.scmessenger.android.utils.FileLoggingTree(this))

        timber.log.Timber.i("SCMessenger application started")

        // Application-level initialization
        // The actual mesh service will be started/stopped by user action
    }
}
