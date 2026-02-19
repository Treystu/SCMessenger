package com.scmessenger.android.ui.components

import androidx.compose.animation.core.*
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.scmessenger.android.service.TransportType
import com.scmessenger.android.ui.theme.*

/**
 * Status indicator dot for peer online/offline status and transport type.
 *
 * Colors:
 * - Green: Online (any transport)
 * - Grey: Offline
 * - Blue: BLE transport
 * - Green: WiFi (Aware/Direct)
 * - Purple: Internet transport
 *
 * Includes animated pulse for active connections.
 */
@Composable
fun StatusIndicator(
    isOnline: Boolean,
    transport: TransportType? = null,
    size: Dp = 12.dp,
    animated: Boolean = true,
    modifier: Modifier = Modifier
) {
    val color = when {
        !isOnline -> StatusOffline
        transport != null -> when (transport) {
            TransportType.BLE -> TransportBLE
            TransportType.WIFI_AWARE -> TransportWiFiAware
            TransportType.WIFI_DIRECT -> TransportWiFiDirect
            TransportType.INTERNET -> TransportInternet
        }
        else -> StatusOnline
    }

    Box(modifier = modifier) {
        if (animated && isOnline) {
            PulsingDot(color = color, size = size)
        } else {
            StaticDot(color = color, size = size)
        }
    }
}

/**
 * Static status dot (no animation).
 */
@Composable
private fun StaticDot(
    color: Color,
    size: Dp,
    modifier: Modifier = Modifier
) {
    Box(
        modifier = modifier
            .size(size)
            .clip(CircleShape)
            .background(color)
    )
}

/**
 * Pulsing status dot with animation.
 */
@Composable
private fun PulsingDot(
    color: Color,
    size: Dp,
    modifier: Modifier = Modifier
) {
    val infiniteTransition = rememberInfiniteTransition(label = "pulse")

    val scale by infiniteTransition.animateFloat(
        initialValue = 1f,
        targetValue = 1.3f,
        animationSpec = infiniteRepeatable(
            animation = tween(1000, easing = LinearEasing),
            repeatMode = RepeatMode.Reverse
        ),
        label = "scale"
    )

    val alpha by infiniteTransition.animateFloat(
        initialValue = 1f,
        targetValue = 0.5f,
        animationSpec = infiniteRepeatable(
            animation = tween(1000, easing = LinearEasing),
            repeatMode = RepeatMode.Reverse
        ),
        label = "alpha"
    )

    Box(
        modifier = modifier
            .size(size)
            .scale(scale)
            .clip(CircleShape)
            .background(color.copy(alpha = alpha))
    )
}

/**
 * Connection quality indicator.
 * Shows signal strength-style bars.
 */
@Composable
fun ConnectionQualityIndicator(
    quality: com.scmessenger.android.service.ConnectionQuality,
    modifier: Modifier = Modifier
) {
    val (bars, color) = when (quality) {
        com.scmessenger.android.service.ConnectionQuality.EXCELLENT -> 4 to QualityExcellent
        com.scmessenger.android.service.ConnectionQuality.GOOD -> 3 to QualityGood
        com.scmessenger.android.service.ConnectionQuality.FAIR -> 2 to QualityFair
        com.scmessenger.android.service.ConnectionQuality.POOR -> 1 to QualityPoor
        com.scmessenger.android.service.ConnectionQuality.UNKNOWN -> 0 to StatusOffline
    }

    // Simple representation: just show a colored dot for now
    // Could be enhanced with actual bar visualization
    StatusIndicator(
        isOnline = bars > 0,
        transport = null,
        size = 8.dp,
        animated = false,
        modifier = modifier
    )
}
