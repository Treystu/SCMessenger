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
        if (!isBootAction(intent.action)) {
            return
        }

        Timber.i("Boot completed, checking auto-start preference")

        // Check if auto-start is enabled
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
        scope.launch {
            try {
                val autoStart = preferencesRepository.serviceAutoStart.first()

                if (shouldAutoStart(intent.action, autoStart)) {
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

    companion object {
        internal const val ACTION_QUICKBOOT_POWERON = "android.intent.action.QUICKBOOT_POWERON"

        /** True for the boot-completed broadcasts this receiver is registered for. */
        internal fun isBootAction(action: String?): Boolean {
            return action == Intent.ACTION_BOOT_COMPLETED || action == ACTION_QUICKBOOT_POWERON
        }

        /**
         * Decides whether a boot-completed broadcast should start the mesh
         * foreground service: only when it's actually a boot action (a stray
         * call with some other action must never auto-start) and the user
         * has opted into auto-start.
         */
        internal fun shouldAutoStart(action: String?, autoStartEnabled: Boolean): Boolean {
            return isBootAction(action) && autoStartEnabled
        }
    }
}
