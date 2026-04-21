package com.scmessenger.android.ui

import android.os.Bundle
import android.os.Handler
import android.os.Looper
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.ui.Modifier
import androidx.core.view.WindowCompat
import com.scmessenger.android.service.AnrWatchdog
import com.scmessenger.android.ui.theme.SCMessengerTheme
import dagger.hilt.android.AndroidEntryPoint
import timber.log.Timber
import android.Manifest
import android.content.pm.PackageManager
import android.os.Build
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.ContextCompat
import com.scmessenger.android.data.MeshRepository
import javax.inject.Inject
import java.util.concurrent.atomic.AtomicBoolean
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * Main activity for SCMessenger.
 *
 * This is the UI entry point, hosting the Compose navigation graph.
 */
@AndroidEntryPoint
class MainActivity : ComponentActivity() {
    @Inject
    lateinit var meshRepository: MeshRepository

    private val permissionRequestInProgress = AtomicBoolean(false)
    private val permissionRequestDebounceMs = 500L
    private val handler = Handler(Looper.getMainLooper())

    // ANR watchdog for UI thread monitoring
    private lateinit var anrWatchdog: AnrWatchdog

    private val requestPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { permissions ->
        permissions.entries.forEach {
            Timber.d("Permission ${it.key} granted: ${it.value}")
        }
        val denied = permissions.filterValues { granted -> !granted }.keys
        if (denied.isNotEmpty()) {
            Timber.w("Permissions denied: $denied")
        }

        // Reset flag after handling
        schedulePermissionReset()

        if (meshRepository.hasRequiredRuntimePermissions()) {
            meshRepository.onRuntimePermissionsGranted()
        }
    }

    private fun schedulePermissionReset() {
        handler.postDelayed({
            permissionRequestInProgress.set(false)
            Timber.d("Permission request state reset")
        }, permissionRequestDebounceMs)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Enable edge-to-edge and proper IME handling
        WindowCompat.setDecorFitsSystemWindows(window, false)

        Timber.d("MainActivity created")

        // Start ANR watchdog monitoring for UI thread responsiveness
        startAnrMonitoring()

        checkPermissions()

        setContent {
            SCMessengerTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    MeshApp()
                }
            }
        }

        // Defer heavy UI work to background using lifecycleScope (structured concurrency)
        lifecycleScope.launch {
            withContext(Dispatchers.IO) {
                initializeUiComponents()
            }
        }
    }

    /**
     * Start ANR watchdog monitoring for UI thread responsiveness.
     */
    private fun startAnrMonitoring() {
        try {
            anrWatchdog = AnrWatchdog(this)
            anrWatchdog.start()
            Timber.i("ANR watchdog started for UI thread monitoring")
        } catch (e: Exception) {
            Timber.w(e, "Failed to start ANR watchdog")
        }
    }

    /**
     * Initialize heavy UI components on a background thread.
     * Called from lifecycleScope on Dispatchers.IO to avoid blocking onCreate.
     */
    private fun initializeUiComponents() {
        try {
            // Initialize repository (heavy storage/FFI operations)
            meshRepository.initializeRepository()

            // Pre-warm compose state caches
            Timber.d("UI components initialization completed")
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize UI components")
        }
    }

    private fun checkPermissions() {
        // Prevent concurrent permission requests
        if (!permissionRequestInProgress.compareAndSet(false, true)) {
            Timber.d("Permission request already in progress, skipping")
            return
        }

        val permissions = mutableListOf(
            Manifest.permission.ACCESS_FINE_LOCATION,
            Manifest.permission.ACCESS_COARSE_LOCATION
        )

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            permissions.add(Manifest.permission.BLUETOOTH_SCAN)
            permissions.add(Manifest.permission.BLUETOOTH_ADVERTISE)
            permissions.add(Manifest.permission.BLUETOOTH_CONNECT)
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            permissions.add(Manifest.permission.POST_NOTIFICATIONS)
            permissions.add(Manifest.permission.NEARBY_WIFI_DEVICES)
        }

        val toRequest = permissions.filter {
            ContextCompat.checkSelfPermission(this, it) != PackageManager.PERMISSION_GRANTED
        }

        if (toRequest.isNotEmpty()) {
            Timber.i("Requesting permissions: $toRequest")
            requestPermissionLauncher.launch(toRequest.toTypedArray())
        } else {
            // All permissions already granted, reset immediately
            permissionRequestInProgress.set(false)
            Timber.d("All permissions already granted")
        }
    }

    override fun onResume() {
        super.onResume()
        Timber.d("MainActivity resumed")
        checkPermissions()
        if (meshRepository.hasRequiredRuntimePermissions()) {
            meshRepository.onRuntimePermissionsGranted()
        }
    }

    override fun onPause() {
        super.onPause()
        Timber.d("MainActivity paused")
    }

    override fun onDestroy() {
        super.onDestroy()
        // Stop ANR watchdog when activity is destroyed
        try {
            anrWatchdog.stop()
            Timber.d("ANR watchdog stopped")
        } catch (e: Exception) {
            Timber.w(e, "Failed to stop ANR watchdog")
        }
    }
}
