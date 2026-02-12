package com.scmessenger.android.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

// TODO: Phase 7 - Implement Onboarding UI
// Requirements:
// - Welcome screen
// - Create/Import Identity
// - Permissions request (BLE, Location, etc.)
// - Setup Completion
@OptIn(com.google.accompanist.permissions.ExperimentalPermissionsApi::class)
@Composable
fun OnboardingScreen(
    onOnboardingComplete: () -> Unit
) {
    val permissionsToRequest = remember {
        val list = mutableListOf(
            android.Manifest.permission.ACCESS_FINE_LOCATION
            // Add Bluetooth Scan/Connect/Advertise if API >= 31
        ).apply {
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.S) {
                add(android.Manifest.permission.BLUETOOTH_SCAN)
                add(android.Manifest.permission.BLUETOOTH_ADVERTISE)
                add(android.Manifest.permission.BLUETOOTH_CONNECT)
            }
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU) {
                add(android.Manifest.permission.NEARBY_WIFI_DEVICES)
                add(android.Manifest.permission.POST_NOTIFICATIONS)
            }
        }
        list.toList()
    }
    
    val permissionsState = com.google.accompanist.permissions.rememberMultiplePermissionsState(
        permissions = permissionsToRequest
    )
    
    var isCreating by remember { mutableStateOf(false) }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp),
        contentAlignment = Alignment.Center
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center,
            modifier = Modifier.fillMaxWidth()
        ) {
            Icon(
                imageVector = androidx.compose.material.icons.Icons.Filled.Lock,
                contentDescription = null,
                modifier = Modifier.size(80.dp),
                tint = MaterialTheme.colorScheme.primary
            )
            
            Spacer(modifier = Modifier.height(32.dp))
            
            Text(
                text = "Welcome to SCMessenger",
                style = MaterialTheme.typography.headlineMedium,
                textAlign = androidx.compose.ui.text.style.TextAlign.Center
            )
            
            Spacer(modifier = Modifier.height(16.dp))
            
            Text(
                text = "Secure, private communication without central servers. Your identity is generated locally and never leaves your device.",
                style = MaterialTheme.typography.bodyLarge,
                textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            
            Spacer(modifier = Modifier.height(48.dp))
            
            if (isCreating) {
                CircularProgressIndicator()
                Spacer(modifier = Modifier.height(16.dp))
                Text("Generating Identity keys...")
            } else {
                if (permissionsState.allPermissionsGranted) {
                    Button(
                        onClick = {
                            isCreating = true
                            onOnboardingComplete()
                        },
                        modifier = Modifier.fillMaxWidth().height(56.dp)
                    ) {
                        Text("Create New Identity")
                    }
                } else {
                    Button(
                        onClick = {
                            permissionsState.launchMultiplePermissionRequest()
                        },
                        modifier = Modifier.fillMaxWidth().height(56.dp)
                    ) {
                        Text("Grant Permissions")
                    }
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = "Bluetooth and Location permissions are required for mesh networking.",
                        style = MaterialTheme.typography.bodySmall,
                        textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                        color = MaterialTheme.colorScheme.error
                    )
                }
                
                Spacer(modifier = Modifier.height(16.dp))
                
                OutlinedButton(
                    onClick = { /* TODO: Import logic */ },
                    modifier = Modifier.fillMaxWidth().height(56.dp),
                    enabled = false // Import not yet supported in Core
                ) {
                    Text("Import Existing Identity")
                }
            }
        }
    }
}
