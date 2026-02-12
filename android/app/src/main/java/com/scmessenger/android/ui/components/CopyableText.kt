package com.scmessenger.android.ui.components

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.widget.Toast
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ExpandLess
import androidx.compose.material.icons.filled.ExpandMore
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

/**
 * Text component that can be copied to clipboard on long-press.
 * 
 * Features:
 * - Long-press to copy
 * - Toast confirmation
 * - Optional truncation with expand/collapse
 * - Monospace font option for keys/hashes
 */
@OptIn(ExperimentalFoundationApi::class)
@Composable
fun CopyableText(
    text: String,
    label: String = "Text",
    maxLines: Int = Int.MAX_VALUE,
    expandable: Boolean = false,
    monospace: Boolean = false,
    modifier: Modifier = Modifier
) {
    val context = LocalContext.current
    var isExpanded by remember { mutableStateOf(false) }
    
    val displayLines = if (expandable && !isExpanded) 1 else maxLines
    
    Column(modifier = modifier) {
        Text(
            text = text,
            fontFamily = if (monospace) FontFamily.Monospace else FontFamily.Default,
            maxLines = displayLines,
            overflow = TextOverflow.Ellipsis,
            style = MaterialTheme.typography.bodyMedium,
            modifier = Modifier
                .fillMaxWidth()
                .combinedClickable(
                    onClick = {
                        if (expandable) {
                            isExpanded = !isExpanded
                        }
                    },
                    onLongClick = {
                        copyToClipboard(context, label, text)
                    }
                )
        )
        
        if (expandable && text.length > 50) {
            TextButton(
                onClick = { isExpanded = !isExpanded },
                modifier = Modifier.align(Alignment.End)
            ) {
                Text(if (isExpanded) "Show less" else "Show more")
                Icon(
                    imageVector = if (isExpanded) Icons.Default.ExpandLess else Icons.Default.ExpandMore,
                    contentDescription = if (isExpanded) "Collapse" else "Expand"
                )
            }
        }
    }
}

/**
 * Copyable text with a label, formatted as a key-value pair.
 */
@OptIn(ExperimentalFoundationApi::class)
@Composable
fun LabeledCopyableText(
    label: String,
    text: String,
    monospace: Boolean = false,
    modifier: Modifier = Modifier
) {
    val context = LocalContext.current
    
    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(vertical = 8.dp)
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        
        Spacer(modifier = Modifier.height(4.dp))
        
        Text(
            text = text,
            fontFamily = if (monospace) FontFamily.Monospace else FontFamily.Default,
            style = MaterialTheme.typography.bodyMedium,
            modifier = Modifier
                .fillMaxWidth()
                .combinedClickable(
                    onClick = { },
                    onLongClick = {
                        copyToClipboard(context, label, text)
                    }
                )
        )
    }
}

/**
 * Truncated text with copy button.
 */
@Composable
fun TruncatedCopyableText(
    text: String,
    label: String = "Text",
    maxLength: Int = 16,
    showCopyIcon: Boolean = true,
    modifier: Modifier = Modifier
) {
    val context = LocalContext.current
    val displayText = if (text.length > maxLength) {
        text.take(maxLength) + "..."
    } else {
        text
    }
    
    Row(
        modifier = modifier,
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(
            text = displayText,
            style = MaterialTheme.typography.bodyMedium,
            fontFamily = FontFamily.Monospace,
            modifier = Modifier.weight(1f)
        )
        
        if (showCopyIcon) {
            IconButton(
                onClick = {
                    copyToClipboard(context, label, text)
                }
            ) {
                Icon(
                    imageVector = androidx.compose.material.icons.Icons.Default.ContentCopy,
                    contentDescription = "Copy $label"
                )
            }
        }
    }
}

/**
 * Helper function to copy text to clipboard and show confirmation.
 */
private fun copyToClipboard(context: Context, label: String, text: String) {
    val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    val clip = ClipData.newPlainText(label, text)
    clipboard.setPrimaryClip(clip)
    
    Toast.makeText(context, "$label copied to clipboard", Toast.LENGTH_SHORT).show()
}
