/**
 * Double Ratchet Protocol for Perfect Forward Secrecy
 * 
 * Implements Signal Protocol's Double Ratchet algorithm to provide:
 * - Perfect Forward Secrecy: Compromising current keys doesn't expose past messages
 * - Future Secrecy: Compromising current keys doesn't expose future messages
 * - Out-of-order message handling
 * - Message key caching for missed messages
 * 
 * Based on Signal Protocol specification:
 * https://signal.org/docs/specifications/doubleratchet/
 * 
 * Security Properties:
 * - Each message encrypted with unique key (never reused)
 * - Keys deleted immediately after use
 * - Compromise of session state limits exposure to window of messages
 * - Ratchet advances automatically with each message exchange
 * 
 * Use Cases:
 * - Store-and-forward messaging (offline messages)
 * - Asynchronous communication
 * - Group messaging with per-peer ratchets
 */

import { hkdf } from '@noble/hashes/hkdf.js';
import { sha256 } from '@noble/hashes/sha2.js';
import { x25519 } from '@noble/curves/ed25519.js';
import { xchacha20poly1305 } from '@noble/ciphers/chacha.js';
import { randomBytes } from './primitives.js';

/**
 * Ratchet state for a conversation
 */
export interface RatchetState {
  /** Root key (KDF chain) */
  rootKey: Uint8Array;

  /** Sending chain key */
  sendingChainKey: Uint8Array;

  /** Receiving chain key */
  receivingChainKey: Uint8Array;

  /** Our current DH key pair */
  dhKeyPair: {
    publicKey: Uint8Array;
    privateKey: Uint8Array;
  };

  /** Peer's current DH public key */
  peerDHPublicKey: Uint8Array;

  /** Number of messages sent in current chain */
  sendingChainLength: number;

  /** Number of messages received in current chain */
  receivingChainLength: number;

  /** Previous sending chain length (for skipped messages) */
  previousSendingChainLength: number;

  /** Skipped message keys (for out-of-order delivery) */
  skippedMessageKeys: Map<string, Uint8Array>;
}

/**
 * Message header for Double Ratchet
 */
export interface RatchetHeader {
  /** Sender's current DH public key */
  dhPublicKey: Uint8Array;

  /** Message number in current chain */
  messageNumber: number;

  /** Previous chain length */
  previousChainLength: number;
}

/**
 * Encrypted message with ratchet header
 */
export interface RatchetMessage {
  /** Ratchet header (unencrypted) */
  header: RatchetHeader;

  /** Encrypted payload */
  ciphertext: Uint8Array;
}

/**
 * Configuration for Double Ratchet
 */
export interface RatchetConfig {
  /** Maximum number of skipped message keys to store */
  maxSkippedMessageKeys: number;

  /** Whether to enable automatic key deletion */
  autoDeleteKeys: boolean;

  /** Maximum message number gap (prevents memory exhaustion) */
  maxMessageGap: number;
}

const DEFAULT_RATCHET_CONFIG: RatchetConfig = {
  maxSkippedMessageKeys: 2000,
  autoDeleteKeys: true,
  maxMessageGap: 1000
};

/**
 * Double Ratchet implementation for Perfect Forward Secrecy
 */
export class DoubleRatchet {
  private state: RatchetState;
  private config: RatchetConfig;

  /**
   * Initialize ratchet for sender (Alice)
   * 
   * @param sharedSecret - Initial shared secret (from X25519 key exchange)
   * @param peerDHPublicKey - Peer's initial DH public key
   * @param config - Ratchet configuration
   */
  static initializeAsSender(
    sharedSecret: Uint8Array,
    peerDHPublicKey: Uint8Array,
    config: Partial<RatchetConfig> = {}
  ): DoubleRatchet {
    // Generate our initial DH key pair
    const dhPrivateKey = randomBytes(32);
    const dhPublicKey = x25519.getPublicKey(dhPrivateKey);

    // Derive root key and sending chain key from initial DH
    const dhOutput = x25519.getSharedSecret(dhPrivateKey, peerDHPublicKey);
    const kdfOutput = hkdf(sha256, dhOutput, sharedSecret, new Uint8Array(0), 64);
    const rootKey = kdfOutput.slice(0, 32);
    const sendingChainKey = kdfOutput.slice(32, 64);

    const state: RatchetState = {
      rootKey,
      sendingChainKey,
      receivingChainKey: new Uint8Array(32), // Will be set on first receive
      dhKeyPair: {
        publicKey: dhPublicKey,
        privateKey: dhPrivateKey
      },
      peerDHPublicKey,
      sendingChainLength: 0,
      receivingChainLength: 0,
      previousSendingChainLength: 0,
      skippedMessageKeys: new Map()
    };

    return new DoubleRatchet(state, config);
  }

  /**
   * Initialize ratchet for receiver (Bob)
   * 
   * @param sharedSecret - Initial shared secret (from X25519 key exchange)
   * @param dhKeyPair - Our initial DH key pair
   * @param config - Ratchet configuration
   */
  static initializeAsReceiver(
    sharedSecret: Uint8Array,
    dhKeyPair: { publicKey: Uint8Array; privateKey: Uint8Array },
    config: Partial<RatchetConfig> = {}
  ): DoubleRatchet {
    const state: RatchetState = {
      rootKey: sharedSecret,
      sendingChainKey: new Uint8Array(32), // Will be set on first DH ratchet
      receivingChainKey: new Uint8Array(32), // Will be set on first receive
      dhKeyPair,
      peerDHPublicKey: new Uint8Array(32), // Will be set on first receive
      sendingChainLength: 0,
      receivingChainLength: 0,
      previousSendingChainLength: 0,
      skippedMessageKeys: new Map()
    };

    return new DoubleRatchet(state, config);
  }

  private constructor(state: RatchetState, config: Partial<RatchetConfig> = {}) {
    this.state = state;
    this.config = { ...DEFAULT_RATCHET_CONFIG, ...config };
  }

  /**
   * Encrypt a message
   * 
   * @param plaintext - Message to encrypt
   * @returns Encrypted message with ratchet header
   */
  encrypt(plaintext: Uint8Array): RatchetMessage {
    // Derive message key from sending chain
    const messageKey = this.deriveMessageKey(this.state.sendingChainKey);

    // Encrypt message
    const nonce = randomBytes(24);
    const cipher = xchacha20poly1305(messageKey, nonce);
    const ciphertext = new Uint8Array([...nonce, ...cipher.encrypt(plaintext)]);

    // Create header
    const header: RatchetHeader = {
      dhPublicKey: this.state.dhKeyPair.publicKey,
      messageNumber: this.state.sendingChainLength,
      previousChainLength: this.state.previousSendingChainLength
    };

    // Advance sending chain
    this.state.sendingChainKey = this.kdfChain(this.state.sendingChainKey);
    this.state.sendingChainLength++;

    // Delete message key (forward secrecy)
    if (this.config.autoDeleteKeys) {
      messageKey.fill(0);
    }

    return { header, ciphertext };
  }

  /**
   * Decrypt a message
   * 
   * @param message - Encrypted message with header
   * @returns Decrypted plaintext
   * @throws Error if decryption fails
   */
  decrypt(message: RatchetMessage): Uint8Array {
    const { header, ciphertext } = message;

    // Check if we need to perform DH ratchet
    const needsDHRatchet = !this.arraysEqual(
      header.dhPublicKey,
      this.state.peerDHPublicKey
    );

    if (needsDHRatchet) {
      this.performDHRatchet(header);
    }

    // Try to get message key for this message number
    const messageKey = this.getMessageKey(header);

    // Decrypt
    const nonce = ciphertext.slice(0, 24);
    const encrypted = ciphertext.slice(24);

    try {
      const cipher = xchacha20poly1305(messageKey, nonce);
      const plaintext = cipher.decrypt(encrypted);

      // Delete message key (forward secrecy)
      if (this.config.autoDeleteKeys) {
        messageKey.fill(0);
      }

      return plaintext;
    } catch (error) {
      throw new Error('Decryption failed: invalid message or corrupted data');
    }
  }

  /**
   * Perform DH ratchet step
   */
  private performDHRatchet(header: RatchetHeader): void {
    // Store previous chain length
    this.state.previousSendingChainLength = this.state.sendingChainLength;

    // Update peer's DH public key
    this.state.peerDHPublicKey = header.dhPublicKey;

    // Derive new receiving chain
    const dhOutput = x25519.getSharedSecret(
      this.state.dhKeyPair.privateKey,
      this.state.peerDHPublicKey
    );
    const kdfOutput = hkdf(sha256, dhOutput, this.state.rootKey, new Uint8Array(0), 64);
    this.state.rootKey = kdfOutput.slice(0, 32);
    this.state.receivingChainKey = kdfOutput.slice(32, 64);
    this.state.receivingChainLength = 0;

    // Generate new DH key pair
    const newDhPrivateKey = randomBytes(32);
    const newDhPublicKey = x25519.getPublicKey(newDhPrivateKey);

    // Derive new sending chain
    const dhOutput2 = x25519.getSharedSecret(newDhPrivateKey, this.state.peerDHPublicKey);
    const kdfOutput2 = hkdf(sha256, dhOutput2, this.state.rootKey, new Uint8Array(0), 64);
    this.state.rootKey = kdfOutput2.slice(0, 32);
    this.state.sendingChainKey = kdfOutput2.slice(32, 64);
    this.state.sendingChainLength = 0;

    // Update our DH key pair
    this.state.dhKeyPair = {
      publicKey: newDhPublicKey,
      privateKey: newDhPrivateKey
    };
  }

  /**
   * Get message key for a specific message number
   * Handles skipped messages (out-of-order delivery)
   */
  private getMessageKey(header: RatchetHeader): Uint8Array {
    const { messageNumber } = header;

    // Check for skipped message key
    const skippedKey = this.state.skippedMessageKeys.get(
      this.makeSkippedKeyId(header.dhPublicKey, messageNumber)
    );

    if (skippedKey) {
      this.state.skippedMessageKeys.delete(
        this.makeSkippedKeyId(header.dhPublicKey, messageNumber)
      );
      return skippedKey;
    }

    // Check message number gap
    const gap = messageNumber - this.state.receivingChainLength;
    if (gap > this.config.maxMessageGap) {
      throw new Error('Message number gap too large: possible attack or corruption');
    }

    // Skip messages until we reach the desired message number
    while (this.state.receivingChainLength < messageNumber) {
      // Store skipped message key
      const skippedMessageKey = this.deriveMessageKey(this.state.receivingChainKey);
      const keyId = this.makeSkippedKeyId(
        header.dhPublicKey,
        this.state.receivingChainLength
      );

      // Enforce maximum skipped keys
      if (this.state.skippedMessageKeys.size >= this.config.maxSkippedMessageKeys) {
        // Remove oldest skipped key
        const firstKey = this.state.skippedMessageKeys.keys().next().value;
        if (firstKey !== undefined) {
          this.state.skippedMessageKeys.delete(firstKey);
        }
      }

      this.state.skippedMessageKeys.set(keyId, skippedMessageKey);

      // Advance receiving chain
      this.state.receivingChainKey = this.kdfChain(this.state.receivingChainKey);
      this.state.receivingChainLength++;
    }

    // Derive message key for current message
    const messageKey = this.deriveMessageKey(this.state.receivingChainKey);

    // Advance receiving chain
    this.state.receivingChainKey = this.kdfChain(this.state.receivingChainKey);
    this.state.receivingChainLength++;

    return messageKey;
  }

  /**
   * Derive message key from chain key
   */
  private deriveMessageKey(chainKey: Uint8Array): Uint8Array {
    const messageKey = hkdf(sha256, chainKey, new Uint8Array(0), new TextEncoder().encode('MessageKey'), 32);
    return messageKey;
  }

  /**
   * Advance chain key using KDF
   */
  private kdfChain(chainKey: Uint8Array): Uint8Array {
    return hkdf(sha256, chainKey, new Uint8Array(0), new TextEncoder().encode('ChainKey'), 32);
  }

  /**
   * Create unique ID for skipped message key
   */
  private makeSkippedKeyId(dhPublicKey: Uint8Array, messageNumber: number): string {
    return `${Buffer.from(dhPublicKey).toString('hex')}-${messageNumber}`;
  }

  /**
   * Compare two Uint8Arrays for equality
   */
  private arraysEqual(a: Uint8Array, b: Uint8Array): boolean {
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
      if (a[i] !== b[i]) return false;
    }
    return true;
  }

  /**
   * Export ratchet state for persistence
   * 
   * WARNING: Exported state contains sensitive key material.
   * Must be encrypted before storage.
   */
  exportState(): RatchetState {
    return {
      rootKey: new Uint8Array(this.state.rootKey),
      sendingChainKey: new Uint8Array(this.state.sendingChainKey),
      receivingChainKey: new Uint8Array(this.state.receivingChainKey),
      dhKeyPair: {
        publicKey: new Uint8Array(this.state.dhKeyPair.publicKey),
        privateKey: new Uint8Array(this.state.dhKeyPair.privateKey)
      },
      peerDHPublicKey: new Uint8Array(this.state.peerDHPublicKey),
      sendingChainLength: this.state.sendingChainLength,
      receivingChainLength: this.state.receivingChainLength,
      previousSendingChainLength: this.state.previousSendingChainLength,
      skippedMessageKeys: new Map(this.state.skippedMessageKeys)
    };
  }

  /**
   * Import ratchet state from persistence
   */
  static importState(state: RatchetState, config: Partial<RatchetConfig> = {}): DoubleRatchet {
    return new DoubleRatchet(state, config);
  }

  /**
   * Get statistics about ratchet state
   */
  getStats() {
    return {
      sendingChainLength: this.state.sendingChainLength,
      receivingChainLength: this.state.receivingChainLength,
      skippedMessageKeys: this.state.skippedMessageKeys.size,
      previousSendingChainLength: this.state.previousSendingChainLength
    };
  }

  /**
   * Clean up old skipped message keys (prevent memory growth)
   * 
   * @param maxAge - Maximum age in milliseconds (not implemented yet - needs timestamp tracking)
   */
  cleanupSkippedKeys(): number {
    const before = this.state.skippedMessageKeys.size;

    // For now, just enforce the maximum
    while (this.state.skippedMessageKeys.size > this.config.maxSkippedMessageKeys) {
      const firstKey = this.state.skippedMessageKeys.keys().next().value;
      if (firstKey !== undefined) {
        this.state.skippedMessageKeys.delete(firstKey);
      } else {
        break; // Safety check to prevent infinite loop
      }
    }

    return before - this.state.skippedMessageKeys.size;
  }
}
