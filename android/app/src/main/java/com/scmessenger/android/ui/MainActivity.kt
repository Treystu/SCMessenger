package com.scmessenger.android.ui

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.ui.Modifier
import com.scmessenger.android.ui.theme.SCMessengerTheme
import com.scmessenger.android.service.MeshForegroundService
import dagger.hilt.android.AndroidEntryPoint
import timber.log.Timber

/**
 * Main activity for SCMessenger.
 *
 * This is the entry point for the UI, hosting the Compose navigation graph.
 */
@AndroidEntryPoint
class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        Timber.d("MainActivity created")

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

    override fun onResume() {
        super.onResume()
        startService(Intent(this, MeshForegroundService::class.java).apply {
            action = MeshForegroundService.ACTION_RESUME
        })
        Timber.d("MainActivity resumed")
    }

    override fun onPause() {
        super.onPause()
        startService(Intent(this, MeshForegroundService::class.java).apply {
            action = MeshForegroundService.ACTION_PAUSE
        })
        Timber.d("MainActivity paused")
    }
}
