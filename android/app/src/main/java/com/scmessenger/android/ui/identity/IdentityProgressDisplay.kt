package com.scmessenger.android.ui.identity

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.scmessenger.android.ui.viewmodels.IdentityProgressStage

/**
 * 6-stage proof-of-work progress display for identity creation.
 *
 * Shows:
 *  - Header row: "Step N of 6 — <stage label>" + percent complete
 *  - Linear progress bar
 *  - Detail line (what the current step is doing)
 *  - ETA hint ("About N seconds remaining")
 *  - Per-stage rows with checkmark / spinner / dim bullet
 *
 * Extracted from IdentityScreen.kt (was private there) so OnboardingScreen
 * can also show it. The fix for the "Generating Identity Keys… hangs without
 * ETA" bug — the user now sees real progress feedback during the 3-5 second
 * Ed25519 keygen + storage persist.
 *
 * v0.3.4 (P0_ANDROID_CRASHFIX): parameter is non-nullable. See the long comment
 * in IdentityViewModel._progressStage for the full rationale — the previous
 * `IdentityProgressStage?` allowed a null to reach `currentStage.id`, crashing
 * the activity. With IdentityViewModel._progressStage typed as non-nullable
 * StateFlow<IdentityProgressStage> and the call sites gated on `!is Idle`,
 * the compiler now enforces non-null at this call site.
 */
@Composable
fun IdentityProgressDisplay(
    currentStage: IdentityProgressStage,
    modifier: Modifier = Modifier
) {
    val stage = currentStage
    val allStages = IdentityProgressStage.ALL

    // Sum of etaMs for stages strictly before the current one, divided by total.
    // The bar fills as stages complete, regardless of how long each actually
    // takes on this device.
    val completedEtaMs = allStages
        .filter { it.id < stage.id }
        .sumOf { it.etaMs }
    val rawFraction = completedEtaMs.toFloat() /
        IdentityProgressStage.TOTAL_ETA_MS.toFloat()
    val fraction = rawFraction.coerceIn(0f, 1f)
    val percentComplete = (fraction * 100f).toInt().coerceIn(0, 99)

    // ETA: total minus the sum of completed etas. Floor at "a few seconds" so
    // the user never sees "About 0s remaining" while the spinner is still
    // running on the longest step.
    val remainingMs = (IdentityProgressStage.TOTAL_ETA_MS - completedEtaMs)
        .coerceAtLeast(500L)
    val remainingSec = (remainingMs + 999L) / 1000L // round up
    val etaText = if (remainingSec <= 1L) "Less than a second remaining"
                  else "About $remainingSec seconds remaining"

    Card(
        modifier = modifier,
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant
        )
    ) {
        Column(
            modifier = Modifier.fillMaxWidth().padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            // Header row: step counter + percent-complete
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = "Step ${stage.id} of ${IdentityProgressStage.TOTAL} — ${stage.label}",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold,
                    modifier = Modifier.weight(1f)
                )
                Text(
                    text = "$percentComplete%",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.primary
                )
            }
            // Smooth progress bar
            LinearProgressIndicator(
                progress = { fraction },
                modifier = Modifier
                    .fillMaxWidth()
                    .height(6.dp),
                color = MaterialTheme.colorScheme.primary,
                trackColor = MaterialTheme.colorScheme.surfaceVariant,
            )
            // Detail line: what the current step is doing
            Text(
                text = stage.detail,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            // ETA hint
            Text(
                text = etaText,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(modifier = Modifier.height(4.dp))
            // 6-row stage list
            allStages.forEach { s ->
                IdentityProgressRow(
                    stage = s,
                    isDone = s.id < stage.id,
                    isActive = s.id == stage.id
                )
            }
        }
    }
}

@Composable
private fun IdentityProgressRow(
    stage: IdentityProgressStage,
    isDone: Boolean,
    isActive: Boolean
) {
    val rowColor = when {
        isDone -> MaterialTheme.colorScheme.primary
        isActive -> MaterialTheme.colorScheme.onSurface
        else -> MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f)
    }
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        when {
            isDone -> Text("✓", color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Bold)
            isActive -> CircularProgressIndicator(modifier = Modifier.size(14.dp), strokeWidth = 2.dp)
            else -> Text("·", color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f), fontWeight = FontWeight.Bold)
        }
        Text(
            text = "${stage.id}. ${stage.label}",
            style = MaterialTheme.typography.bodyMedium,
            color = rowColor,
            fontWeight = if (isActive) FontWeight.SemiBold else FontWeight.Normal
        )
    }
}
