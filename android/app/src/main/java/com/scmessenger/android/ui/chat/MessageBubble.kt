package com.scmessenger.android.ui.chat

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.scmessenger.android.ui.theme.*
import com.scmessenger.android.utils.toEpochMillis
import java.text.SimpleDateFormat
import java.util.*

/**
 * Message bubble component for chat UI.
 *
 * Zero-Status Architecture: displays only message content (text)
 * and the sender-assigned message timestamp (`senderTimestamp`). No delivery status indicators.
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

            // senderTimestamp: the time the message was saved to local storage for sending
            Text(
                text = formatTimestamp(message.senderTimestamp),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                fontSize = 11.sp,
                fontWeight = FontWeight.Normal,
                modifier = Modifier.padding(top = 4.dp, start = 8.dp, end = 8.dp)
            )
        }

        if (isSent) {
            Spacer(modifier = Modifier.width(48.dp))
        }
    }
}

/**
 * Format timestamp for display.
 */
private fun formatTimestamp(timestamp: ULong): String {
    val millis = timestamp.toEpochMillis()
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
