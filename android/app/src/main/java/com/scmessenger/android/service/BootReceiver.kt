package com.scmessenger.android.service

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import com.scmessenger.android.data.PreferencesRepository
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import timber.log.Timber
import javax.inject.Inject

/**
 * Broadcast receiver to restart the mesh service on device boot.
 *
 * Only starts the service if auto-start is enabled in preferences.
 */
@AndroidEntryPoint
class BootReceiver : BroadcastReceiver() {

    @Inject
    lateinit var preferencesRepository: PreferencesRepository

    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == Intent.ACTION_BOOT_COMPLETED ||
            intent.action == "android.intent.action.QUICKBOOT_POWERON") {

            Timber.i("Boot completed, checking auto-start preference")

            // Check if auto-start is enabled
            val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
            scope.launch {
                try {
                    val autoStart = preferencesRepository.serviceAutoStart.first()

                    if (autoStart) {
                        Timber.i("Auto-start enabled, starting mesh service")
                        startMeshService(context)
                    } else {
                        Timber.d("Auto-start disabled, not starting service")
                    }
                } finally {
                    scope.cancel()
                }
            }
        }
    }

    private fun startMeshService(context: Context) {
        val intent = Intent(context, MeshForegroundService::class.java).apply {
            action = MeshForegroundService.ACTION_START
        }

        try {
            context.startForegroundService(intent)
        } catch (e: Exception) {
            Timber.e(e, "Failed to start mesh service from BootReceiver (likely Android 12+ background restriction)")
        }
    }
}
