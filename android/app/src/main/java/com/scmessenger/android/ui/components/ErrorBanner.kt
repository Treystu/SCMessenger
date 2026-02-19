package com.scmessenger.android.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Error
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp
import com.scmessenger.android.ui.theme.StatusError
import com.scmessenger.android.ui.theme.StatusWarning

/**
 * Dismissible banner for displaying errors, warnings, and info messages.
 *
 * Features:
 * - Different severity levels (error, warning, info)
 * - Dismissible with close button
 * - Optional retry action
 * - Maps IronCoreError to user-friendly messages
 * - Animated appearance/disappearance
 */
@Composable
fun ErrorBanner(
    message: String,
    severity: BannerSeverity = BannerSeverity.ERROR,
    visible: Boolean = true,
    onDismiss: () -> Unit,
    onRetry: (() -> Unit)? = null,
    modifier: Modifier = Modifier
) {
    AnimatedVisibility(
        visible = visible,
        enter = fadeIn() + expandVertically(),
        exit = fadeOut() + shrinkVertically(),
        modifier = modifier
    ) {
        Surface(
            color = severity.backgroundColor,
            contentColor = severity.contentColor,
            modifier = Modifier.fillMaxWidth()
        ) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    imageVector = severity.icon,
                    contentDescription = null,
                    modifier = Modifier.size(24.dp)
                )

                Spacer(modifier = Modifier.width(12.dp))

                Text(
                    text = message,
                    style = MaterialTheme.typography.bodyMedium,
                    modifier = Modifier.weight(1f)
                )

                if (onRetry != null) {
                    TextButton(onClick = onRetry) {
                        Text("Retry")
                    }
                }

                IconButton(onClick = onDismiss) {
                    Icon(
                        imageVector = Icons.Default.Close,
                        contentDescription = "Dismiss"
                    )
                }
            }
        }
    }
}

/**
 * Banner severity levels.
 */
enum class BannerSeverity(
    val backgroundColor: Color,
    val contentColor: Color,
    val icon: ImageVector
) {
    ERROR(
        backgroundColor = Color(0xFFFFEBEE),
        contentColor = StatusError,
        icon = Icons.Default.Error
    ),
    WARNING(
        backgroundColor = Color(0xFFFFF3E0),
        contentColor = StatusWarning,
        icon = Icons.Default.Warning
    ),
    INFO(
        backgroundColor = Color(0xFFE3F2FD),
        contentColor = Color(0xFF1976D2),
        icon = Icons.Default.Info
    )
}

/**
 * Helper to map error codes/messages to user-friendly descriptions.
 */
fun mapErrorToMessage(error: String): String {
    return when {
        error.contains("IronCoreError", ignoreCase = true) -> "Failed to connect to mesh network"
        error.contains("permission", ignoreCase = true) -> "Permission required to continue"
        error.contains("bluetooth", ignoreCase = true) -> "Bluetooth error - check settings"
        error.contains("wifi", ignoreCase = true) -> "WiFi error - check connection"
        error.contains("timeout", ignoreCase = true) -> "Connection timeout - try again"
        error.contains("not found", ignoreCase = true) -> "Peer not found"
        error.contains("encrypt", ignoreCase = true) -> "Failed to encrypt message"
        error.contains("decrypt", ignoreCase = true) -> "Failed to decrypt message"
        else -> error
    }
}

/**
 * Convenience composable for showing error state.
 */
@Composable
fun ErrorState(
    error: String,
    onDismiss: () -> Unit,
    onRetry: (() -> Unit)? = null,
    modifier: Modifier = Modifier
) {
    ErrorBanner(
        message = mapErrorToMessage(error),
        severity = BannerSeverity.ERROR,
        onDismiss = onDismiss,
        onRetry = onRetry,
        modifier = modifier
    )
}

/**
 * Convenience composable for showing warning state.
 */
@Composable
fun WarningBanner(
    message: String,
    visible: Boolean = true,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier
) {
    ErrorBanner(
        message = message,
        severity = BannerSeverity.WARNING,
        visible = visible,
        onDismiss = onDismiss,
        modifier = modifier
    )
}

/**
 * Convenience composable for showing info state.
 */
@Composable
fun InfoBanner(
    message: String,
    visible: Boolean = true,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier
) {
    ErrorBanner(
        message = message,
        severity = BannerSeverity.INFO,
        visible = visible,
        onDismiss = onDismiss,
        modifier = modifier
    )
}
