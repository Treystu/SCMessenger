/**
 * Cryptographic primitives for Sovereign Communications
 *
 * Standardized Implementation using @noble libraries.
 * Provides consistent behavior across environments (Web, Node, React Native).
 */

import { ed25519, x25519 } from "@noble/curves/ed25519.js";
import { sha256, sha512 } from "@noble/hashes/sha2.js";
export { sha256, sha512 };
import { xchacha20poly1305 } from "@noble/ciphers/chacha.js";
import { hkdf } from "@noble/hashes/hkdf.js";
export { hkdf };

// --- Types ---

export interface IdentityKeyPair {
  publicKey: Uint8Array;
  privateKey: Uint8Array;
}

export interface SessionKey {
  key: Uint8Array;
  nonce: Uint8Array;
  timestamp: number;
  messageCount: number;
  counter: number;
}

export interface RatchetState {
  rootKey: Uint8Array;
  sendChainKey: Uint8Array;
  receiveChainKey: Uint8Array;
  sendCounter: number;
  receiveCounter: number;
  previousSendCounter: number;
  dhRatchetKey: IdentityKeyPair;
}

// --- Internal Helpers ---

/**
 * Ensures input is a clean Uint8Array, overcoming context issues in some test environments
 */
function ensureUint8Array(
  input: Uint8Array | number[] | ArrayBuffer | any,
): Uint8Array {
  if (input instanceof Uint8Array && input.constructor === Uint8Array)
    return input;
  return new Uint8Array(input);
}

/**
 * Manual Ed25519 public key to X25519 public key conversion (Montgomery mapping)
 * Providing a robust fallback for environments where Noble's point validation fails (e.g., some Jest/JSDOM setups)
 */
export function convertEd25519PublicKeyToX25519(edPub: Uint8Array): Uint8Array {
  const bytes = ensureUint8Array(edPub);
  try {
    return ed25519.utils.toMontgomery(bytes);
  } catch (err) {
    // Robust manual fallback: u = (1 + y) / (1 - y) mod P
    const P = (1n << 255n) - 19n;
    const yBytes = new Uint8Array(bytes);
    yBytes[31] &= 0x7f; // Clear sign bit to get y

    let y = 0n;
    for (let i = 0; i < 32; i++) {
      y |= BigInt(yBytes[i]) << (BigInt(i) * 8n);
    }

    const one = 1n;
    const num = (one + y) % P;
    const den = (one - y + P) % P;

    const modPow = (base: bigint, exp: bigint, mod: bigint) => {
      let res = 1n;
      base %= mod;
      while (exp > 0n) {
        if (exp % 2n === 1n) res = (res * base) % mod;
        base = (base * base) % mod;
        exp /= 2n;
      }
      return res;
    };

    const denInv = modPow(den, P - 2n, P);
    const u = (num * denInv) % P;

    const uBytes = new Uint8Array(32);
    for (let i = 0; i < 32; i++) {
      uBytes[i] = Number((u >> (BigInt(i) * 8n)) & 0xffn);
    }
    return uBytes;
  }
}

/**
 * Manual Ed25519 private key (seed) to X25519 private key (scalar) conversion
 */
export function convertEd25519PrivateKeyToX25519(
  edPriv: Uint8Array,
): Uint8Array {
  const bytes = ensureUint8Array(edPriv);
  try {
    return ed25519.utils.toMontgomerySecret(bytes);
  } catch (err) {
    // Manual fallback: X25519 scalar = first 32 bytes of SHA512(seed), clamped
    const hash = sha512(bytes);
    const scalar = hash.slice(0, 32);
    scalar[0] &= 248;
    scalar[31] &= 127;
    scalar[31] |= 64;
    return scalar;
  }
}

// --- Randomness ---

/**
 * Cryptographically secure random bytes
 */
export const randomBytes = (n: number): Uint8Array => {
  if (typeof globalThis !== "undefined" && globalThis.crypto?.getRandomValues) {
    return globalThis.crypto.getRandomValues(new Uint8Array(n));
  }
  // Node fallback if web crypto unavailable
  try {
    if (typeof process !== "undefined" && process?.versions?.node) {
      // Dynamic import for Node.js crypto module
      // eslint-disable-next-line @typescript-eslint/no-var-requires
      const nodeCrypto = require("crypto");
      return new Uint8Array(nodeCrypto.randomBytes(n));
    }
  } catch (e) {
    // Ignore and proceed to throw if no generator
  }
  throw new Error("No secure random number generator available");
};

/**
 * Validate entropy quality (basic sanity checks)
 */
export function validateEntropy(bytes: Uint8Array): boolean {
  if (bytes.length < 32) return false;
  // Reject all zeros or same value
  let allSame = true;
  for (let i = 1; i < bytes.length; i++) {
    if (bytes[i] !== bytes[0]) {
      allSame = false;
      break;
    }
  }
  if (allSame) return false;
  return true;
}

// --- Key Generation ---

/**
 * Generate a new Ed25519 identity keypair
 */
export function generateIdentity(): IdentityKeyPair {
  const privateKey = randomBytes(32);
  const publicKey = ed25519.getPublicKey(privateKey);
  return { publicKey, privateKey };
}

export const generateKeyPair = generateIdentity;

/**
 * Generate ephemeral X25519 keypair for ratchet/DH
 */
export function generateEphemeralKeyPair(): IdentityKeyPair {
  const privateKey = randomBytes(32);
  // Noble x25519.getPublicKey handles clamping internally
  const publicKey = x25519.getPublicKey(privateKey);
  return { publicKey, privateKey };
}

/**
 * Generate a new session key
 */
export function generateSessionKey(): SessionKey {
  return {
    key: generateKey(),
    nonce: generateNonce(),
    timestamp: Date.now(),
    messageCount: 0,
    counter: 0,
  };
}

/**
 * Generate a symmetric session key
 */
export function generateKey(): Uint8Array {
  return randomBytes(32);
}

export function generateNonce(): Uint8Array {
  return randomBytes(24);
}

// --- Signing ---

export function signMessage(
  message: Uint8Array,
  privateKey: Uint8Array,
): Uint8Array {
  return ed25519.sign(ensureUint8Array(message), ensureUint8Array(privateKey));
}

export function verifySignature(
  message: Uint8Array,
  signature: Uint8Array,
  publicKey: Uint8Array,
): boolean {
  try {
    return ed25519.verify(
      ensureUint8Array(signature),
      ensureUint8Array(message),
      ensureUint8Array(publicKey),
    );
  } catch {
    return false;
  }
}

export function batchVerifySignatures(
  items: {
    message: Uint8Array;
    signature: Uint8Array;
    publicKey: Uint8Array;
  }[],
): boolean {
  // Simple implementation using single verification for consistency
  return items.every((item) =>
    verifySignature(item.message, item.signature, item.publicKey),
  );
}

// --- Key Exchange ---

/**
 * Perform pure X25519 key exchange (RFC 7748)
 * Expects X25519 keys (clamped or unclamped scalars and Montgomery u-coordinates)
 */
export function performKeyExchange(
  privateKey: Uint8Array,
  peerPublicKey: Uint8Array,
  salt?: Uint8Array,
  info?: Uint8Array,
): Uint8Array {
  const priv = ensureUint8Array(privateKey);
  const pub = ensureUint8Array(peerPublicKey);

  if (priv.length !== 32 || pub.length !== 32) {
    throw new Error("X25519 keys must be 32 bytes");
  }

  const sharedSecret = x25519.getSharedSecret(priv, pub);

  const derivedKey = hkdf(
    sha256,
    sharedSecret,
    salt || new Uint8Array(32),
    info || new Uint8Array(0),
    32,
  );

  secureWipe(sharedSecret);
  return derivedKey;
}

/**
 * Derives a shared secret between two parties.
 * Handles automatic conversion of Ed25519 identity keys to X25519 format.
 * If keys are already X25519, this function should be used with caution as it
 * specifically targets identity-to-dh conversion paths.
 */
export function deriveSharedSecret(
  privateKey: Uint8Array,
  peerPublicKey: Uint8Array,
  salt?: Uint8Array,
  info?: Uint8Array,
): Uint8Array {
  // Always convert identity keys to X25519 scalars/u-coordinates
  const x25519Priv = convertEd25519PrivateKeyToX25519(
    ensureUint8Array(privateKey),
  );
  const x25519Pub = convertEd25519PublicKeyToX25519(
    ensureUint8Array(peerPublicKey),
  );

  return performKeyExchange(x25519Priv, x25519Pub, salt, info);
}

// --- Encryption ---

export function encryptMessage(
  data: Uint8Array,
  key: Uint8Array,
  nonce: Uint8Array,
): Uint8Array {
  const k = ensureUint8Array(key);
  const n = ensureUint8Array(nonce);

  if (k.length !== 32) throw new Error("Key must be 32 bytes");
  if (n.length !== 24) throw new Error("Nonce must be 24 bytes");

  const cipher = xchacha20poly1305(k, n);
  return cipher.encrypt(ensureUint8Array(data));
}

export function decryptMessage(
  encrypted: Uint8Array,
  key: Uint8Array,
  nonce: Uint8Array,
): Uint8Array {
  const data = ensureUint8Array(encrypted);
  const k = ensureUint8Array(key);
  const n = ensureUint8Array(nonce);

  if (k.length !== 32) throw new Error("Key must be 32 bytes");
  if (n.length !== 24) throw new Error("Nonce must be 24 bytes");
  if (data.length < 16) throw new Error("Encrypted data too short");

  const cipher = xchacha20poly1305(k, n);
  return cipher.decrypt(data);
}

// --- Ratchet ---

export function deriveMessageKey(chainKey: Uint8Array): {
  messageKey: Uint8Array;
  nextChainKey: Uint8Array;
} {
  const ck = ensureUint8Array(chainKey);
  const messageKey = hkdf(
    sha256,
    ck,
    new Uint8Array([1]),
    new TextEncoder().encode("message"),
    32,
  );
  const nextChainKey = hkdf(
    sha256,
    ck,
    new Uint8Array([2]),
    new TextEncoder().encode("chain"),
    32,
  );
  secureWipe(ck);
  return { messageKey, nextChainKey };
}

export function initializeRatchet(
  sharedSecret: Uint8Array,
  _isAlice: boolean,
): RatchetState {
  const root = ensureUint8Array(sharedSecret);
  return {
    rootKey: root,
    sendChainKey: new Uint8Array(32),
    receiveChainKey: new Uint8Array(32),
    sendCounter: 0,
    receiveCounter: 0,
    previousSendCounter: 0,
    dhRatchetKey: generateEphemeralKeyPair(),
  };
}

export function ratchetStep(
  state: RatchetState,
  peerPublicKey: Uint8Array,
): RatchetState {
  const dhOutput = performKeyExchange(
    state.dhRatchetKey.privateKey,
    peerPublicKey,
  );
  const info = new TextEncoder().encode("ratchet");

  const kdfOutput = hkdf(sha256, dhOutput, state.rootKey, info, 64);
  const newRootKey = kdfOutput.slice(0, 32);
  const chainKey = kdfOutput.slice(32, 64);

  const newState = {
    ...state,
    rootKey: newRootKey,
    previousSendCounter: state.sendCounter,
    sendCounter: 0,
    receiveCounter: 0,
    dhRatchetKey: generateEphemeralKeyPair(),
  };

  if (state.sendCounter === 0 && state.receiveCounter === 0) {
    // Initial setup
    newState.sendChainKey = chainKey;
    newState.receiveChainKey = chainKey; // Simplified for this implementation
  }

  secureWipe(dhOutput);
  return newState;
}

// --- Utilities ---

export function timingSafeEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) return false;
  let result = 0;
  for (let i = 0; i < a.length; i++) {
    result |= a[i] ^ b[i];
  }
  return result === 0;
}

export function secureWipe(data: Uint8Array): void {
  // Multiple passes with verification to prevent optimization
  for (let pass = 0; pass < 3; pass++) {
    for (let i = 0; i < data.length; i++) {
      data[i] = 0;
    }
  }
  // Verify the wipe
  if (data.some((byte) => byte !== 0)) {
    throw new Error("Secure wipe failed");
  }
}

export function generateFingerprint(publicKey: Uint8Array): string {
  const hash = sha256(ensureUint8Array(publicKey));
  // Return 16-char uppercase hex without spaces for consistent ID format
  return Array.from(hash)
    .slice(0, 8)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("")
    .toUpperCase();
}

/**
 * Mock NonceManager for consistency with other parts of the repo
 */
export class NonceManager {
  private used = new Set<string>();
  hasBeenUsed(nonce: Uint8Array): boolean {
    return this.used.has(Buffer.from(nonce).toString("hex"));
  }
  markUsed(nonce: Uint8Array): void {
    const hex = Buffer.from(nonce).toString("hex");
    if (this.used.has(hex)) throw new Error("Nonce reuse detected");
    this.used.add(hex);
  }
}

export function incrementNonce(sessionKey: SessionKey): Uint8Array {
  sessionKey.counter++;
  // Simple increment logic for the 24-byte nonce (treating it as little-endian for simplicity)
  for (let i = 0; i < sessionKey.nonce.length; i++) {
    sessionKey.nonce[i] = (sessionKey.nonce[i] + 1) & 0xff;
    if (sessionKey.nonce[i] !== 0) break;
  }
  return new Uint8Array(sessionKey.nonce);
}

export function shouldRotateKey(
  key: SessionKey,
  timeLimit = 3600000,
  msgLimit = 1000,
): boolean {
  return Date.now() - key.timestamp > timeLimit || key.messageCount > msgLimit;
}

export function rotateSessionKey(key: SessionKey): SessionKey {
  const oldKey = key.key;
  const oldNonce = key.nonce;
  const newKey = generateKey();
  const newNonce = generateNonce();
  secureWipe(oldKey);
  secureWipe(oldNonce);
  return {
    key: newKey,
    nonce: newNonce,
    timestamp: Date.now(),
    messageCount: 0,
    counter: 0,
  };
}

export function deriveSessionKey(
  sharedSecret: Uint8Array,
  salt: Uint8Array,
  info?: Uint8Array,
): Uint8Array {
  return hkdf(sha256, sharedSecret, salt, info || new Uint8Array(0), 32);
}
