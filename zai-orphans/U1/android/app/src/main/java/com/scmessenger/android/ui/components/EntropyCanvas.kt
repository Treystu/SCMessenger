package com.scmessenger.android.ui.components

import androidx.compose.animation.core.*
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.*
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp
import java.nio.ByteBuffer
import java.security.MessageDigest
import kotlin.math.cos
import kotlin.math.sin
import kotlin.random.Random

data class TouchPoint(val x: Float, val y: Float, val timestamp: Long)

class Particle(
    var position: Offset,
    val velocity: Offset,
    val color: Color,
    val size: Float,
    var alpha: Float,
    val lifeTime: Float,
    var age: Float = 0f
)

@Composable
fun EntropyCanvas(
    modifier: Modifier = Modifier,
    targetPointsCount: Int = 80,
    onEntropyComplete: (ByteArray) -> Unit
) {
    var touchPoints by remember { mutableStateOf(listOf<TouchPoint>()) }
    var currentProgress by remember { mutableStateOf(0f) }
    var isComplete by remember { mutableStateOf(false) }

    // Trailing points for visual trail
    var visualTrail by remember { mutableStateOf(listOf<Offset>()) }
    var particles by remember { mutableStateOf(listOf<Particle>()) }

    // Pulsing background animation
    val infiniteTransition = rememberInfiniteTransition(label = "BackgroundPulse")
    val pulseAlpha by infiniteTransition.animateFloat(
        initialValue = 0.15f,
        targetValue = 0.35f,
        animationSpec = infiniteRepeatable(
            animation = tween(2000, easing = LinearEasing),
            repeatMode = RepeatMode.Reverse
        ),
        label = "Pulse"
    )

    // Particle updater loop
    LaunchedEffect(visualTrail) {
        while (true) {
            if (visualTrail.isNotEmpty() && !isComplete) {
                val lastPoint = visualTrail.last()
                // Emit particles at the last touch point
                val newParticles = List(3) {
                    val angle = Random.nextFloat() * 2 * Math.PI
                    val speed = Random.nextFloat() * 4f + 1f
                    Particle(
                        position = lastPoint,
                        velocity = Offset(
                            (cos(angle) * speed).toFloat(),
                            (sin(angle) * speed).toFloat()
                        ),
                        color = Color(0xFF00E5FF).copy(alpha = 0.8f),
                        size = Random.nextFloat() * 8f + 4f,
                        alpha = 1.0f,
                        lifeTime = 1f
                    )
                }
                particles = (particles + newParticles)
            }
            // Update active particles
            particles = particles.filter { it.age < it.lifeTime }.onEach {
                it.position = it.position + it.velocity
                it.age += 0.05f
                it.alpha = (1f - (it.age / it.lifeTime)).coerceAtLeast(0f)
            }
            kotlinx.coroutines.delay(16)
        }
    }

    Box(
        modifier = modifier
            .fillMaxWidth()
            .height(280.dp)
            .clip(RoundedCornerShape(24.dp))
            .background(
                Brush.radialGradient(
                    colors = listOf(
                        Color(0xFF00E5FF).copy(alpha = pulseAlpha),
                        Color(0xFF121212)
                    )
                )
            )
            .border(
                width = 2.dp,
                brush = Brush.horizontalGradient(
                    colors = listOf(
                        Color(0xFF00E5FF).copy(alpha = if (isComplete) 1f else 0.5f),
                        Color(0xFF00E676).copy(alpha = if (isComplete) 1f else 0.5f)
                    )
                ),
                shape = RoundedCornerShape(24.dp)
            ),
        contentAlignment = Alignment.Center
    ) {
        Canvas(
            modifier = Modifier
                .fillMaxSize()
                .pointerInput(isComplete) {
                    if (isComplete) return@pointerInput
                    detectDragGestures(
                        onDragStart = { offset ->
                            visualTrail = listOf(offset)
                        },
                        onDrag = { change, _ ->
                            change.consume()
                            val newPoint = change.position
                            visualTrail = (visualTrail + newPoint).takeLast(25)

                            val touchPoint = TouchPoint(
                                x = newPoint.x,
                                y = newPoint.y,
                                timestamp = System.nanoTime()
                            )
                            touchPoints = touchPoints + touchPoint

                            val progression = (touchPoints.size.toFloat() / targetPointsCount).coerceIn(0f, 1f)
                            currentProgress = progression

                            if (touchPoints.size >= targetPointsCount && !isComplete) {
                                isComplete = true
                                val salt = generateSaltFromTouchPoints(touchPoints)
                                onEntropyComplete(salt)
                            }
                        },
                        onDragEnd = {
                            visualTrail = emptyList()
                        }
                    )
                }
        ) {
            // Draw visual trail
            if (visualTrail.size > 1) {
                val path = Path().apply {
                    moveTo(visualTrail.first().x, visualTrail.first().y)
                    for (i in 1 until visualTrail.size) {
                        lineTo(visualTrail[i].x, visualTrail[i].y)
                    }
                }
                drawPath(
                    path = path,
                    brush = Brush.horizontalGradient(
                        colors = listOf(Color(0xFF00E5FF), Color(0xFF00E676))
                    ),
                    style = Stroke(width = 6.dp.toPx(), cap = StrokeCap.Round, join = StrokeJoin.Round)
                )
            }

            // Draw active particles
            particles.forEach { particle ->
                drawCircle(
                    color = particle.color.copy(alpha = particle.alpha),
                    radius = particle.size,
                    center = particle.position
                )
            }

            // Draw progress ring in the center
            val ringRadius = 60.dp.toPx()
            val centerOffset = Offset(size.width / 2, size.height / 2)

            // Background ring
            drawCircle(
                color = Color.White.copy(alpha = 0.1f),
                radius = ringRadius,
                center = centerOffset,
                style = Stroke(width = 8.dp.toPx())
            )

            // Animated progress ring
            drawArc(
                brush = Brush.sweepGradient(
                    colors = listOf(Color(0xFF00E5FF), Color(0xFF00E676), Color(0xFF00E5FF))
                ),
                startAngle = -90f,
                sweepAngle = currentProgress * 360f,
                useCenter = false,
                topLeft = Offset(centerOffset.x - ringRadius, centerOffset.y - ringRadius),
                size = androidx.compose.ui.geometry.Size(ringRadius * 2, ringRadius * 2),
                style = Stroke(width = 8.dp.toPx(), cap = StrokeCap.Round)
            )
        }

        // Overlay text instructions
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier.padding(16.dp)
        ) {
            if (isComplete) {
                Text(
                    text = "Entropy Generated!",
                    style = MaterialTheme.typography.titleMedium,
                    color = Color(0xFF00E676)
                )
                Text(
                    text = "Ready to proceed",
                    style = MaterialTheme.typography.bodySmall,
                    color = Color.White.copy(alpha = 0.7f)
                )
            } else {
                Text(
                    text = "Entropy Gathering Box",
                    style = MaterialTheme.typography.titleMedium,
                    color = Color.White
                )
                Text(
                    text = "Move your finger around inside the box",
                    style = MaterialTheme.typography.bodyMedium,
                    color = Color.White.copy(alpha = 0.7f)
                )
                Spacer(modifier = Modifier.height(8.dp))
                Text(
                    text = "${(currentProgress * 100).toInt()}% Complete",
                    style = MaterialTheme.typography.titleLarge,
                    color = Color(0xFF00E5FF)
                )
            }
        }
    }
}

private fun generateSaltFromTouchPoints(points: List<TouchPoint>): ByteArray {
    val byteBuffer = ByteBuffer.allocate(points.size * 16)
    points.forEach { point ->
        byteBuffer.putFloat(point.x)
        byteBuffer.putFloat(point.y)
        byteBuffer.putLong(point.timestamp)
    }
    val md = MessageDigest.getInstance("SHA-256")
    val hash = md.digest(byteBuffer.array())
    return hash.copyOf(16)
}
