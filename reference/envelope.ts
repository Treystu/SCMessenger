/**
 * Envelope encryption utilities
 * Provides secure envelope sign/verify/encrypt/decrypt operations
 */

import { ed25519 } from '@noble/curves/ed25519.js';
import { x25519 } from '@noble/curves/ed25519.js';
import { sha256 } from '@noble/hashes/sha2.js';
import { xchacha20poly1305 } from '@noble/ciphers/chacha.js';
import { hkdf } from '@noble/hashes/hkdf.js';
import { randomBytes } from './primitives.js';

/**
 * Encrypted envelope structure
 */
export interface EncryptedEnvelope {
  /** Ephemeral public key for ECDH */
  ephemeralPublicKey: Uint8Array;
  /** Nonce for XChaCha20-Poly1305 */
  nonce: Uint8Array;
  /** Encrypted payload with auth tag */
  ciphertext: Uint8Array;
  /** Ed25519 signature of the envelope */
  signature: Uint8Array;
  /** Timestamp for replay protection */
  timestamp: number;
  /** Version for future compatibility */
  version: number;
}

/**
 * Signed envelope structure
 */
export interface SignedEnvelope {
  /** Original data */
  data: Uint8Array;
  /** Ed25519 signature */
  signature: Uint8Array;
  /** Sender's public key */
  senderPublicKey: Uint8Array;
  /** Timestamp */
  timestamp: number;
}

const CURRENT_VERSION = 1;
const HKDF_INFO = new TextEncoder().encode('SC-Envelope-v1');

/**
 * Encrypt data into a sealed envelope
 * Uses X25519 ECDH + XChaCha20-Poly1305 + Ed25519 signing
 * 
 * @param data - Data to encrypt
 * @param senderPrivateKey - Sender's Ed25519 private key for signing
 * @param recipientPublicKey - Recipient's X25519 public key
 * @returns Encrypted and signed envelope
 */
export function encryptEnvelope(
  data: Uint8Array,
  senderPrivateKey: Uint8Array,
  recipientPublicKey: Uint8Array
): EncryptedEnvelope {
  if (senderPrivateKey.length !== 32) {
    throw new Error('Sender private key must be 32 bytes');
  }
  if (recipientPublicKey.length !== 32) {
    throw new Error('Recipient public key must be 32 bytes');
  }

  // Generate ephemeral keypair for forward secrecy
  const ephemeralPrivateKey = randomBytes(32);
  const ephemeralPublicKey = x25519.getPublicKey(ephemeralPrivateKey);

  // Perform ECDH
  const sharedSecret = x25519.getSharedSecret(ephemeralPrivateKey, recipientPublicKey);

  // Derive encryption key using HKDF
  const encryptionKey = hkdf(
    sha256,
    sharedSecret,
    ephemeralPublicKey, // Use ephemeral public key as salt
    HKDF_INFO,
    32
  );

  // Generate nonce
  const nonce = randomBytes(24);

  // Encrypt with XChaCha20-Poly1305
  const cipher = xchacha20poly1305(encryptionKey, nonce);
  const ciphertext = cipher.encrypt(data);

  const timestamp = Date.now();

  // Create envelope data to sign (without signature)
  const envelopeData = new Uint8Array([
    ...ephemeralPublicKey,
    ...nonce,
    ...ciphertext,
    ...numberToBytes(timestamp),
    CURRENT_VERSION
  ]);

  // Sign the envelope
  const signature = ed25519.sign(envelopeData, senderPrivateKey);

  // Wipe sensitive data
  ephemeralPrivateKey.fill(0);
  sharedSecret.fill(0);
  encryptionKey.fill(0);

  return {
    ephemeralPublicKey,
    nonce,
    ciphertext,
    signature,
    timestamp,
    version: CURRENT_VERSION
  };
}

/**
 * Decrypt an envelope
 * 
 * @param envelope - Encrypted envelope
 * @param recipientPrivateKey - Recipient's X25519 private key
 * @param senderPublicKey - Sender's Ed25519 public key for verification
 * @returns Decrypted data
 */
export function decryptEnvelope(
  envelope: EncryptedEnvelope,
  recipientPrivateKey: Uint8Array,
  senderPublicKey: Uint8Array
): Uint8Array {
  if (recipientPrivateKey.length !== 32) {
    throw new Error('Recipient private key must be 32 bytes');
  }
  if (senderPublicKey.length !== 32) {
    throw new Error('Sender public key must be 32 bytes');
  }

  // Verify version
  if (envelope.version !== CURRENT_VERSION) {
    throw new Error(`Unsupported envelope version: ${envelope.version}`);
  }

  // Recreate envelope data for signature verification
  const envelopeData = new Uint8Array([
    ...envelope.ephemeralPublicKey,
    ...envelope.nonce,
    ...envelope.ciphertext,
    ...numberToBytes(envelope.timestamp),
    envelope.version
  ]);

  // Verify signature
  const isValid = ed25519.verify(envelope.signature, envelopeData, senderPublicKey);
  if (!isValid) {
    throw new Error('Envelope signature verification failed');
  }

  // Perform ECDH
  const sharedSecret = x25519.getSharedSecret(recipientPrivateKey, envelope.ephemeralPublicKey);

  // Derive decryption key using HKDF
  const decryptionKey = hkdf(
    sha256,
    sharedSecret,
    envelope.ephemeralPublicKey,
    HKDF_INFO,
    32
  );

  // Decrypt with XChaCha20-Poly1305
  try {
    const cipher = xchacha20poly1305(decryptionKey, envelope.nonce);
    const plaintext = cipher.decrypt(envelope.ciphertext);

    // Wipe sensitive data
    sharedSecret.fill(0);
    decryptionKey.fill(0);

    return plaintext;
  } catch {
    sharedSecret.fill(0);
    decryptionKey.fill(0);
    throw new Error('Envelope decryption failed: authentication tag mismatch');
  }
}

/**
 * Sign data and create a signed envelope
 * 
 * @param data - Data to sign
 * @param privateKey - Ed25519 private key
 * @returns Signed envelope
 */
export function signEnvelope(data: Uint8Array, privateKey: Uint8Array): SignedEnvelope {
  if (privateKey.length !== 32) {
    throw new Error('Private key must be 32 bytes');
  }

  const publicKey = ed25519.getPublicKey(privateKey);
  const timestamp = Date.now();

  // Create data to sign (data + timestamp)
  const signedData = new Uint8Array([
    ...data,
    ...numberToBytes(timestamp)
  ]);

  const signature = ed25519.sign(signedData, privateKey);

  return {
    data,
    signature,
    senderPublicKey: publicKey,
    timestamp
  };
}

/**
 * Verify a signed envelope
 * 
 * @param envelope - Signed envelope to verify
 * @param expectedPublicKey - Optional expected public key to verify against
 * @returns true if signature is valid
 */
export function verifyEnvelope(
  envelope: SignedEnvelope,
  expectedPublicKey?: Uint8Array
): boolean {
  // Verify public key if expected
  if (expectedPublicKey) {
    if (envelope.senderPublicKey.length !== expectedPublicKey.length) {
      return false;
    }
    for (let i = 0; i < expectedPublicKey.length; i++) {
      if (envelope.senderPublicKey[i] !== expectedPublicKey[i]) {
        return false;
      }
    }
  }

  // Recreate signed data
  const signedData = new Uint8Array([
    ...envelope.data,
    ...numberToBytes(envelope.timestamp)
  ]);

  try {
    return ed25519.verify(envelope.signature, signedData, envelope.senderPublicKey);
  } catch {
    return false;
  }
}

/**
 * Serialize an encrypted envelope to bytes
 */
export function serializeEncryptedEnvelope(envelope: EncryptedEnvelope): Uint8Array {
  const timestampBytes = numberToBytes(envelope.timestamp);
  const ciphertextLength = numberToBytes(envelope.ciphertext.length);

  return new Uint8Array([
    envelope.version,
    ...envelope.ephemeralPublicKey,    // 32 bytes
    ...envelope.nonce,                  // 24 bytes
    ...ciphertextLength,                // 8 bytes
    ...envelope.ciphertext,             // variable
    ...envelope.signature,              // 64 bytes
    ...timestampBytes                   // 8 bytes
  ]);
}

/**
 * Deserialize bytes to an encrypted envelope
 */
export function deserializeEncryptedEnvelope(bytes: Uint8Array): EncryptedEnvelope {
  if (bytes.length < 137) { // minimum: 1 + 32 + 24 + 8 + 0 + 64 + 8
    throw new Error('Invalid envelope: too short');
  }

  let offset = 0;

  const version = bytes[offset++];
  const ephemeralPublicKey = bytes.slice(offset, offset + 32);
  offset += 32;
  const nonce = bytes.slice(offset, offset + 24);
  offset += 24;
  const ciphertextLength = bytesToNumber(bytes.slice(offset, offset + 8));
  offset += 8;
  const ciphertext = bytes.slice(offset, offset + ciphertextLength);
  offset += ciphertextLength;
  const signature = bytes.slice(offset, offset + 64);
  offset += 64;
  const timestamp = bytesToNumber(bytes.slice(offset, offset + 8));

  return {
    version,
    ephemeralPublicKey,
    nonce,
    ciphertext,
    signature,
    timestamp
  };
}

/**
 * Serialize a signed envelope to bytes
 */
export function serializeSignedEnvelope(envelope: SignedEnvelope): Uint8Array {
  const timestampBytes = numberToBytes(envelope.timestamp);
  const dataLength = numberToBytes(envelope.data.length);

  return new Uint8Array([
    ...envelope.senderPublicKey,  // 32 bytes
    ...dataLength,                // 8 bytes
    ...envelope.data,             // variable
    ...envelope.signature,        // 64 bytes
    ...timestampBytes             // 8 bytes
  ]);
}

/**
 * Deserialize bytes to a signed envelope
 */
export function deserializeSignedEnvelope(bytes: Uint8Array): SignedEnvelope {
  if (bytes.length < 112) { // minimum: 32 + 8 + 0 + 64 + 8
    throw new Error('Invalid envelope: too short');
  }

  let offset = 0;

  const senderPublicKey = bytes.slice(offset, offset + 32);
  offset += 32;
  const dataLength = bytesToNumber(bytes.slice(offset, offset + 8));
  offset += 8;
  const data = bytes.slice(offset, offset + dataLength);
  offset += dataLength;
  const signature = bytes.slice(offset, offset + 64);
  offset += 64;
  const timestamp = bytesToNumber(bytes.slice(offset, offset + 8));

  return {
    senderPublicKey,
    data,
    signature,
    timestamp
  };
}

/**
 * Convert a number to 8 bytes (big-endian)
 */
function numberToBytes(num: number): Uint8Array {
  const bytes = new Uint8Array(8);
  let n = num;
  for (let i = 7; i >= 0; i--) {
    bytes[i] = n & 0xff;
    n = Math.floor(n / 256);
  }
  return bytes;
}

/**
 * Convert 8 bytes to a number (big-endian)
 */
function bytesToNumber(bytes: Uint8Array): number {
  let num = 0;
  for (let i = 0; i < 8; i++) {
    num = num * 256 + bytes[i];
  }
  return num;
}
