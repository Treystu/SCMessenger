package com.scmessenger.android.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

/**
 * A persistent banner displayed when device storage is critically low.
 */
@Composable
fun StorageWarningBanner(availableMB: Long) {
    Surface(
        color = MaterialTheme.colorScheme.errorContainer,
        contentColor = MaterialTheme.colorScheme.onErrorContainer,
        tonalElevation = 4.dp
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = Icons.Default.Warning,
                contentDescription = "Warning",
                modifier = Modifier.size(24.dp)
            )
            Spacer(modifier = Modifier.width(12.dp))
            Column {
                Text(
                    text = "Critical Storage Warning",
                    style = MaterialTheme.typography.labelLarge
                )
                Text(
                    text = "Only ${availableMB}MB available. Non-essential cache has been cleared, but critical message and contact storage may be affected soon. Please free up space at the OS level.",
                    style = MaterialTheme.typography.bodySmall
                )
            }
        }
    }
}
