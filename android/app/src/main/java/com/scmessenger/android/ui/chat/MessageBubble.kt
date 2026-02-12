package com.scmessenger.android.ui.chat

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.scmessenger.android.ui.theme.*
import java.text.SimpleDateFormat
import java.util.*

/**
 * Message bubble component for chat UI.
 * 
 * Displays sent/received messages with delivery status indicators,
 * timestamps, and appropriate styling based on message direction.
 */
@Composable
fun MessageBubble(
    message: uniffi.api.MessageRecord,
    modifier: Modifier = Modifier
) {
    val isSent = message.direction == uniffi.api.MessageDirection.SENT
    
    Row(
        modifier = modifier.fillMaxWidth(),
        horizontalArrangement = if (isSent) Arrangement.End else Arrangement.Start
    ) {
        if (!isSent) {
            Spacer(modifier = Modifier.width(48.dp))
        }
        
        Column(
            horizontalAlignment = if (isSent) Alignment.End else Alignment.Start,
            modifier = Modifier.widthIn(max = 280.dp)
        ) {
            // Message bubble
            Surface(
                shape = RoundedCornerShape(
                    topStart = 16.dp,
                    topEnd = 16.dp,
                    bottomStart = if (isSent) 16.dp else 4.dp,
                    bottomEnd = if (isSent) 4.dp else 16.dp
                ),
                color = if (isSent) MessageSentBubble else MessageReceivedBubble,
                shadowElevation = 1.dp
            ) {
                Column(
                    modifier = Modifier.padding(12.dp)
                ) {
                    // Message content
                    Text(
                        text = message.content,
                        color = if (isSent) MessageSentText else MessageReceivedText,
                        fontSize = 15.sp,
                        lineHeight = 20.sp
                    )
                }
            }
            
            // Timestamp and status
            Row(
                modifier = Modifier.padding(top = 4.dp, start = 8.dp, end = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(4.dp)
            ) {
                Text(
                    text = formatTimestamp(message.timestamp),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Normal
                )
                
                // Delivery status for sent messages
                if (isSent) {
                    DeliveryStatusIndicator(delivered = message.delivered)
                }
            }
        }
        
        if (isSent) {
            Spacer(modifier = Modifier.width(48.dp))
        }
    }
}

/**
 * Delivery status indicator icon.
 */
@Composable
private fun DeliveryStatusIndicator(
    delivered: Boolean,
    modifier: Modifier = Modifier
) {
    if (delivered) {
        Icon(
            imageVector = Icons.Default.Check,
            contentDescription = "Delivered",
            modifier = modifier.size(14.dp),
            tint = StatusSuccess.copy(alpha = 0.7f)
        )
    } else {
        Icon(
            imageVector = Icons.Default.Clear,
            contentDescription = "Pending",
            modifier = modifier.size(14.dp),
            tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f)
        )
    }
}

/**
 * Format timestamp for display.
 */
private fun formatTimestamp(timestamp: ULong): String {
    val millis = timestamp.toLong() * 1000
    val date = Date(millis)
    val now = Date()
    
    val sdf = if (isSameDay(date, now)) {
        SimpleDateFormat("HH:mm", Locale.getDefault())
    } else {
        SimpleDateFormat("MMM d, HH:mm", Locale.getDefault())
    }
    
    return sdf.format(date)
}

/**
 * Check if two dates are on the same day.
 */
private fun isSameDay(date1: Date, date2: Date): Boolean {
    val cal1 = Calendar.getInstance().apply { time = date1 }
    val cal2 = Calendar.getInstance().apply { time = date2 }
    
    return cal1.get(Calendar.YEAR) == cal2.get(Calendar.YEAR) &&
           cal1.get(Calendar.DAY_OF_YEAR) == cal2.get(Calendar.DAY_OF_YEAR)
}
