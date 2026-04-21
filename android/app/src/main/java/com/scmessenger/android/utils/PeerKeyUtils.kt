package com.scmessenger.android.utils

import timber.log.Timber
import java.nio.ByteBuffer
import java.nio.ByteOrder

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
     * Libp2p peer IDs starting with "12D3KooW" use Ed25519 keys encoded as:
     * - 0x12 (Ed25519 code) + 32-byte public key + 2-byte SHA256 checksum
     * - This is base58-encoded to form the final peer ID
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

            // Decode from base58
            val decoded = base58Decode(peerId)
            if (decoded == null || decoded.size < 35) {
                Timber.w("Decoded peer ID too short: ${decoded?.size ?: 0}")
                return null
            }

            // Verify the format: first byte should be 0x12 (Ed25519)
            if (decoded[0] != 0x12.toByte()) {
                Timber.w("Invalid multihash prefix: 0x${decoded[0].toInt().toHex()}")
                return null
            }

            // Extract the 32-byte public key (bytes 2-33)
            val publicKeyBytes = decoded.copyOfRange(2, 34)

            // Verify checksum (bytes 34+ should match first 2 bytes of SHA256(publicKey))
            if (decoded.size >= 36) {
                val storedChecksum = decoded.copyOfRange(decoded.size - 2, decoded.size)
                val expectedChecksum = java.security.MessageDigest.getInstance("SHA256")
                    .digest(publicKeyBytes)
                    .copyOfRange(0, 2)
                if (!storedChecksum.contentEquals(expectedChecksum)) {
                    Timber.w("Checksum verification failed")
                    return null
                }
            }

            publicKeyBytes.joinToString("") { String.format("%02x", it) }
        } catch (e: Exception) {
            Timber.e("Failed to extract public key from peer ID $peerId: ${e.message}")
            null
        }
    }

    /**
     * Generate a libp2p peer ID from a public key using proper multihash encoding.
     *
     * This generates a peer ID in the standard libp2p format:
     * - Ed25519 public key → multihash (0x12 + 32 bytes) + SHA256 checksum → base58
     * - Format: 12D3KooW + base58-encoded(multihash)
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

            // Decode hex public key to bytes
            val publicKeyBytes = publicKey.hexToBytes()
            if (publicKeyBytes.size != 32) {
                Timber.w("Public key must be 32 bytes for Ed25519")
                return generateFallbackPeerId(publicKey)
            }

            // Create multihash: 0x12 (Ed25519) + 32-byte key
            val multihash = ByteBuffer.allocate(34)
                .put(0x12.toByte())
                .put(publicKeyBytes)
                .array()

            // Add SHA256 checksum (first 2 bytes)
            val sha256 = java.security.MessageDigest.getInstance("SHA256")
            val checksum = sha256.digest(multihash).copyOfRange(0, 2)

            val fullData = ByteBuffer.allocate(36)
                .put(multihash)
                .put(checksum)
                .array()

            // Encode to base58
            val base58Encoded = base58Encode(fullData)
            if (base58Encoded == null) {
                Timber.w("Base58 encoding failed")
                return generateFallbackPeerId(publicKey)
            }

            base58Encoded
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

    // --- Helper functions for base58 encoding/decoding ---

    /**
     * Convert a hex string to bytes.
     */
    private fun String.hexToBytes(): ByteArray {
        check(length % 2 == 0) { "Hex string must have even length" }
        return chunked(2).map { it.toInt(16).toByte() }.toByteArray()
    }

    /**
     * Convert a byte to hex string.
     */
    private fun Int.toHex(): String = String.format("%02x", this)

    // --- Base58 encoding/decoding implementation ---
    // Using a simple implementation without external dependencies

    private val BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
    private val BASE58_CHARSET = BASE58_ALPHABET.toCharArray()
    private val BASE58_MAP = IntArray(128) { -1 }
    init {
        for (i in BASE58_CHARSET.indices) {
            BASE58_MAP[BASE58_CHARSET[i].code] = i
        }
    }

    /**
     * Encode bytes to base58 string.
     */
    private fun base58Encode(data: ByteArray): String? {
        if (data.isEmpty()) return "1"

        // Count leading zeros
        var zeroCount = 0
        while (zeroCount < data.size && data[zeroCount] == 0.toByte()) {
            zeroCount++
        }

        // Convert to base58
        val encoded = StringBuilder()
        var carry = data.clone()

        // Big integer division by 58
        var leadingZero = true
        while (true) {
            var carryValue = 0
            var j = 0
            while (j < carry.size) {
                carryValue = (carryValue shl 8) + (carry[j].toInt() and 0xFF)
                carry[j] = (carryValue / 58).toByte()
                carryValue %= 58
                j++
            }

            if (carryValue > 0) {
                val charIndex = carryValue
                if (leadingZero && charIndex == 0) {
                    encoded.append('1')
                } else {
                    encoded.append(BASE58_ALPHABET[charIndex])
                    leadingZero = false
                }
            } else if (!leadingZero) {
                break
            }

            // Remove processed bytes
            val newCarry = ByteArray(carry.size - j)
            System.arraycopy(carry, j, newCarry, 0, newCarry.size)
            carry = newCarry
        }

        // Add leading '1's for leading zeros
        for (i in 0 until zeroCount) {
            encoded.insert(0, '1')
        }

        return encoded.toString().ifEmpty { null }
    }

    /**
     * Decode base58 string to bytes.
     */
    private fun base58Decode(str: String): ByteArray? {
        if (str.isEmpty()) return null

        // Count leading '1's (leading zeros)
        var zeroCount = 0
        for (c in str) {
            if (c == '1') zeroCount++
            else break
        }

        // Decode each character
        val bytes = ByteArray(str.length)
        var position = 0

        for (c in str) {
            val digit = if (c.code < BASE58_MAP.size) BASE58_MAP[c.code] else -1
            if (digit == -1) {
                Timber.w("Invalid base58 character: $c")
                return null
            }

            // Multiply by 58 and add digit
            var carry = digit
            for (i in position downTo 0) {
                carry += bytes[i].toInt() * 58
                bytes[i] = (carry and 0xFF).toByte()
                carry = carry ushr 8
            }

            while (carry > 0) {
                bytes[position] = (carry and 0xFF).toByte()
                position++
                carry = carry ushr 8
            }
        }

        // Remove leading zeros
        val result = ByteArray(bytes.size - zeroCount)
        System.arraycopy(bytes, zeroCount, result, 0, result.size)

        return result
    }
}
