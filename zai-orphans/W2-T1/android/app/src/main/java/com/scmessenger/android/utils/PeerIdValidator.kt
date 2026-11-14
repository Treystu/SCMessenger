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

    fun isLibp2pPeerId(id: String): Boolean {
        // Base58-encoded libp2p peer IDs: 12D3KooW... (~52 chars) or Qm... (~46 chars)
        // Validate prefix + reasonable length + base58 charset (no 0, O, I, l)
        val base58Chars = id.all { c -> c.isLetterOrDigit() && c !in "0OIl" }
        return base58Chars && (
            (id.startsWith("12D3Koo") && id.length in 46..56) ||
            (id.startsWith("Qm") && id.length in 44..50)
        )
    }

    fun isIdentityId(id: String): Boolean =
        id.matches(IDENTITY_ID_REGEX)

    fun isSame(id1: String, id2: String): Boolean =
        normalize(id1) == normalize(id2)
}
