package com.scmessenger.android.utils

object PeerIdValidator {
    private val IDENTITY_ID_REGEX = Regex("^[a-fA-F0-9]{64}$")

    fun validate(id: String): Boolean =
        normalize(id).let { normalized ->
            normalized.matches(IDENTITY_ID_REGEX) || isLibp2pPeerId(normalized)
        }

    fun normalize(id: String): String {
        val trimmed = id.trim()
        // 64 hex chars are case-insensitive public keys - normalize to lower
        if (trimmed.length == 64 && trimmed.matches(IDENTITY_ID_REGEX)) {
            return trimmed.lowercase()
        }
        // Base58 libp2p IDs (starting with 12D3Koo or Qm) are case-sensitive - preserve case
        return trimmed
    }

    fun isLibp2pPeerId(id: String): Boolean =
        id.startsWith("12D3Koo") || id.startsWith("Qm")

    fun isIdentityId(id: String): Boolean =
        id.matches(IDENTITY_ID_REGEX)

    fun isSame(id1: String, id2: String): Boolean =
        normalize(id1) == normalize(id2)
}
