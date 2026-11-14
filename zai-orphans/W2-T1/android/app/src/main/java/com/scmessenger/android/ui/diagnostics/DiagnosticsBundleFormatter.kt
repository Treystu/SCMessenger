package com.scmessenger.android.ui.diagnostics

import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

data class DiagnosticsBundleInput(
    val generatedAtEpochMs: Long,
    val appVersion: String,
    val serviceState: String,
    val connectionPathState: String,
    val natStatus: String,
    val discoveredPeers: Int,
    val pendingOutbox: Int,
    val missingPermissions: List<String>,
    val coreDiagnosticsJson: String,
    val recentLogs: String
)

object DiagnosticsBundleFormatter {
    private val timestampFormat = SimpleDateFormat("yyyy-MM-dd HH:mm:ss Z", Locale.US)

    fun format(input: DiagnosticsBundleInput): String {
        val generatedAt = timestampFormat.format(Date(input.generatedAtEpochMs))
        val missingPermissions = if (input.missingPermissions.isEmpty()) {
            "none"
        } else {
            input.missingPermissions.joinToString(", ")
        }

        return """
SCMessenger Diagnostics Bundle (v0.2.0 alpha)
Generated: $generatedAt
App version: ${input.appVersion}

== Runtime Summary ==
service_state=${input.serviceState}
connection_path_state=${input.connectionPathState}
nat_status=${input.natStatus}
discovered_peers=${input.discoveredPeers}
pending_outbox=${input.pendingOutbox}

== Delivery State Guide ==
pending: queued locally, first route attempt in progress.
stored: queued for retry, recipient currently unreachable.
forwarding: active retry is running now.
delivered: recipient delivery receipt confirmed.

== Reliability Notes For Testers ==
1) First send may stay pending/stored while peers discover each other.
2) Stored/forwarding are expected on unstable links; app keeps retrying.
3) Report any message that never reaches delivered after network recovery.

== Permissions Rationale ==
Missing runtime permissions: $missingPermissions
Bluetooth/Location/Nearby WiFi are required for direct peer discovery and transport selection.
Notifications are optional for delivery alerts but do not affect encryption semantics.

== Core Diagnostics JSON ==
${input.coreDiagnosticsJson.ifBlank { "{}" }}

== Recent Application Logs ==
${input.recentLogs.ifBlank { "(no logs)" }}
""".trimIndent()
    }
}
