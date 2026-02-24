package com.scmessenger.android.ui

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.ui.Modifier
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

/**
 * Main activity for SCMessenger.
 *
 * This is the entry point for the UI, hosting the Compose navigation graph.
 */
@AndroidEntryPoint
class MainActivity : ComponentActivity() {
    @Inject
    lateinit var meshRepository: MeshRepository

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
        if (meshRepository.hasRequiredRuntimePermissions()) {
            meshRepository.onRuntimePermissionsGranted()
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        Timber.d("MainActivity created")
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
    }

    private fun checkPermissions() {
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
}
