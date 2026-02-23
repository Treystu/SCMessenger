package com.scmessenger.android.utils

data class ImportedContactPayload(
    val peerId: String,
    val publicKey: String,
    val nickname: String?,
    val libp2pPeerId: String?,
    val listeners: List<String>
)

sealed class ContactImportParseResult {
    data class Valid(val payload: ImportedContactPayload) : ContactImportParseResult()
    data class Invalid(val reason: String) : ContactImportParseResult()
}

fun parseContactImportPayload(raw: String): ContactImportParseResult {
    if (raw.isBlank()) return ContactImportParseResult.Invalid("No identity data found.")

    val peerId = """"identity_id"\s*:\s*"([^"]+)"""".toRegex().find(raw)?.groupValues?.get(1)
        ?: """"identityId"\s*:\s*"([^"]+)"""".toRegex().find(raw)?.groupValues?.get(1)
        ?: """"peerId"\s*:\s*"([^"]+)"""".toRegex().find(raw)?.groupValues?.get(1)

    val publicKey = """"public_key"\s*:\s*"([^"]+)"""".toRegex().find(raw)?.groupValues?.get(1)
        ?: """"publicKeyHex"\s*:\s*"([^"]+)"""".toRegex().find(raw)?.groupValues?.get(1)
        ?: """"publicKey"\s*:\s*"([^"]+)"""".toRegex().find(raw)?.groupValues?.get(1)

    if (peerId.isNullOrBlank()) return ContactImportParseResult.Invalid("Missing identity ID in payload.")
    if (publicKey.isNullOrBlank()) return ContactImportParseResult.Invalid("Missing public key in payload.")

    val nickname = """"nickname"\s*:\s*"([^"]*)"""".toRegex()
        .find(raw)?.groupValues?.get(1)?.takeIf { it.isNotBlank() }?.trim()

    val libp2pPeerId = """"libp2p_peer_id"\s*:\s*"([^"]+)"""".toRegex().find(raw)?.groupValues?.get(1)
        ?: """"libp2pPeerId"\s*:\s*"([^"]+)"""".toRegex().find(raw)?.groupValues?.get(1)

    val listenersRaw = """"listeners"\s*:\s*\[(.*?)\]""".toRegex()
        .find(raw)?.groupValues?.get(1).orEmpty()
    val listeners = listenersRaw
        .split(",")
        .map { it.trim().trim('"').replace(" (Potential)", "") }
        .filter { it.isNotBlank() }

    return ContactImportParseResult.Valid(
        ImportedContactPayload(
            peerId = peerId.trim(),
            publicKey = publicKey.trim(),
            nickname = nickname,
            libp2pPeerId = libp2pPeerId?.trim()?.takeIf { it.isNotBlank() },
            listeners = listeners
        )
    )
}
