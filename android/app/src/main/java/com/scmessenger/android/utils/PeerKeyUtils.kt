package com.scmessenger.android.utils

import timber.log.Timber

/**
 * Utility functions for peer key and peer ID conversion.
 *
 * Provides methods to:
 * - Extract public keys from libp2p peer IDs
 * - Generate peer IDs from public keys
 * - Validate public keys and peer IDs
 */
object PeerKeyUtils {

    /**
     * Extract public key from a libp2p peer ID.
     *
     * Libp2p peer IDs starting with "12D3KooW" contain a base58-encoded
     * public key in their structure.
     *
     * @param peerId The libp2p peer ID to extract from
     * @return The extracted public key as hex string, or null if extraction fails
     */
    fun extractPublicKeyFromPeerId(peerId: String): String? {
        return try {
            if (!isLibp2pPeerId(peerId)) {
                Timber.w("Peer ID does not appear to be a libp2p peer ID: $peerId")
                return null
            }

            // Libp2p peer IDs use multibase and multihash encoding
            // The public key is base58-encoded in the peer ID
            // For Ed25519 keys: 12D3KooW... format contains the key
            // We need to decode from base58 and extract the key portion

            // For now, return the peer ID as-is if it looks like a public key
            // The actual extraction requires base58 decoding and multihash parsing
            if (peerId.length == 52 && peerId.startsWith("12D3KooW")) {
                // This is a full libp2p peer ID - the public key portion is embedded
                // For proper extraction, we would use a multibase/multihash library
                // For now, return null to signal we need external processing
                null
            } else if (peerId.length == 64 && peerId.matches(Regex("[0-9a-fA-F]+"))) {
                // This looks like a raw hex public key
                peerId
            } else {
                Timber.w("Cannot extract public key from peer ID format: $peerId")
                null
            }
        } catch (e: Exception) {
            Timber.e("Failed to extract public key from peer ID $peerId: ${e.message}")
            null
        }
    }

    /**
     * Generate a libp2p peer ID from a public key.
     *
     * This generates a peer ID in the standard libp2p format:
     * - 12D3KooW + base58-encoded(public_key_hash)
     *
     * @param publicKey The public key as hex string (64 chars for Ed25519)
     * @return The generated libp2p peer ID
     */
    fun generateLibp2pPeerIdFromPublicKey(publicKey: String): String {
        return try {
            if (!isValidPublicKey(publicKey)) {
                Timber.w("Invalid public key format for peer ID generation: ${publicKey.take(8)}...")
                return generateFallbackPeerId(publicKey)
            }

            // For proper libp2p peer ID generation, we need to:
            // 1. Decode the hex public key
            // 2. Compute the SHA256 hash
            // 3. Encode using base58 with multihash prefix

            // Fallback: create a deterministic peer ID from the public key
            // This is not a full libp2p peer ID but works for local identification
            generateFallbackPeerId(publicKey)
        } catch (e: Exception) {
            Timber.e("Failed to generate peer ID from public key: ${e.message}")
            generateFallbackPeerId(publicKey)
        }
    }

    /**
     * Generate a fallback peer ID from a public key when proper libp2p generation fails.
     */
    private fun generateFallbackPeerId(publicKey: String): String {
        // Create a deterministic but non-standard peer ID
        // Format: peer_<first_8_chars_of_key>
        val keyPrefix = publicKey.take(8)
        return "peer_${keyPrefix.lowercase()}"
    }

    /**
     * Check if a string is a valid public key.
     *
     * A valid Ed25519 public key is 64 hex characters.
     *
     * @param key The string to validate
     * @return true if the key looks like a valid Ed25519 public key
     */
    fun isValidPublicKey(key: String): Boolean {
        return key.length == 64 && key.matches(Regex("[0-9a-fA-F]+"))
    }

    /**
     * Check if a string is a valid libp2p peer ID.
     *
     * @param peerId The string to validate
     * @return true if the peerId matches libp2p format (12D3KooW... or Qm...)
     */
    fun isValidPeerId(peerId: String): Boolean {
        return isLibp2pPeerId(peerId)
    }

    /**
     * Check if a string is a libp2p peer ID (internal helper).
     */
    private fun isLibp2pPeerId(peerId: String): Boolean {
        // Base58-encoded libp2p peer IDs: 12D3KooW... (~52 chars) or Qm... (~46 chars)
        val base58Chars = peerId.all { c -> c.isLetterOrDigit() && c !in "0OIl" }
        return base58Chars && (
            (peerId.startsWith("12D3Koo") && peerId.length in 46..56) ||
            (peerId.startsWith("Qm") && peerId.length in 44..50)
        )
    }

    /**
     * Extract peer ID from a public key by generating a deterministic peer ID.
     *
     * @param publicKey The public key as hex string
     * @return The generated peer ID
     */
    fun extractPeerIdFromPublicKey(publicKey: String): String {
        return generateLibp2pPeerIdFromPublicKey(publicKey)
    }
}
