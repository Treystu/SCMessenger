/**
 * X3DH (Extended Triple Diffie-Hellman) Key Exchange Protocol
 *
 * Implements the Signal Protocol's X3DH for establishing shared secrets
 * between two parties, even when the recipient is offline.
 *
 * Based on Signal Protocol specification:
 * https://signal.org/docs/specifications/x3dh/
 *
 * Key Types:
 * - Identity Key (IK): Long-term Ed25519 key pair, converted to X25519 for DH
 * - Signed Prekey (SPK): Medium-term X25519 key pair, rotated periodically
 * - One-Time Prekeys (OPK): Single-use X25519 keys for enhanced forward secrecy
 *
 * Security Properties:
 * - Forward secrecy: Compromise of long-term keys doesn't expose past sessions
 * - Deniability: Cannot prove to third party that a message was sent
 * - Offline initiation: Can start session even when recipient is offline
 */

import { x25519 } from "@noble/curves/ed25519.js";
import { sha256 } from "@noble/hashes/sha2.js";
import { hkdf } from "@noble/hashes/hkdf.js";
import {
  randomBytes,
  signMessage,
  verifySignature,
  convertEd25519PublicKeyToX25519,
  convertEd25519PrivateKeyToX25519,
  secureWipe,
} from "./primitives.js";

/**
 * Ed25519 identity key pair
 */
export interface IdentityKeyPair {
  /** Ed25519 public key (32 bytes) */
  publicKey: Uint8Array;
  /** Ed25519 private key/seed (32 bytes) */
  privateKey: Uint8Array;
}

/**
 * X25519 key pair for DH operations
 */
export interface X25519KeyPair {
  /** X25519 public key (32 bytes) */
  publicKey: Uint8Array;
  /** X25519 private key (32 bytes) */
  privateKey: Uint8Array;
}

/**
 * Signed prekey with signature
 */
export interface SignedPrekey {
  /** X25519 key pair */
  keyPair: X25519KeyPair;
  /** Ed25519 signature over the public key */
  signature: Uint8Array;
  /** Timestamp when this prekey was generated */
  timestamp: number;
  /** Unique prekey ID */
  id: number;
}

/**
 * One-time prekey
 */
export interface OneTimePrekey {
  /** X25519 key pair */
  keyPair: X25519KeyPair;
  /** Unique prekey ID */
  id: number;
  /** Whether this prekey has been used */
  used: boolean;
}

/**
 * Complete X3DH key bundle for a node
 */
export interface X3DHKeyBundle {
  /** Ed25519 identity key pair */
  identityKey: IdentityKeyPair;
  /** Current signed prekey */
  signedPrekey: SignedPrekey;
  /** Pool of one-time prekeys */
  oneTimePrekeys: OneTimePrekey[];
  /** Next one-time prekey ID */
  nextOTPId: number;
  /** Next signed prekey ID */
  nextSPKId: number;
}

/**
 * Public prekey bundle (shared with others via DHT)
 */
export interface PrekeyBundle {
  /** Peer ID (16-char hex from Ed25519 public key) */
  peerId: string;
  /** Ed25519 identity public key */
  identityKey: Uint8Array;
  /** Signed prekey public key */
  signedPrekeyPublic: Uint8Array;
  /** Signed prekey ID */
  signedPrekeyId: number;
  /** Signature over signed prekey */
  signedPrekeySignature: Uint8Array;
  /** One-time prekey public keys (may be empty) */
  oneTimePrekeyPublics: Array<{
    id: number;
    publicKey: Uint8Array;
  }>;
  /** Timestamp when bundle was created */
  timestamp: number;
}

/**
 * Result of X3DH key exchange initiation
 */
export interface X3DHInitResult {
  /** Shared secret (32 bytes) */
  sharedSecret: Uint8Array;
  /** Ephemeral public key to send to recipient */
  ephemeralPublic: Uint8Array;
  /** Used one-time prekey ID (if any) */
  usedOneTimePrekeyId?: number;
  /** Used signed prekey ID */
  usedSignedPrekeyId: number;
  /** Associated data for AEAD (IKa || IKb) */
  associatedData: Uint8Array;
}

/**
 * X3DH initial message (sent with first message)
 */
export interface X3DHInitialMessage {
  /** Sender's identity public key */
  identityKey: Uint8Array;
  /** Sender's ephemeral public key */
  ephemeralKey: Uint8Array;
  /** Used signed prekey ID */
  signedPrekeyId: number;
  /** Used one-time prekey ID (if any) */
  oneTimePrekeyId?: number;
}

/**
 * Configuration for X3DH
 */
export interface X3DHConfig {
  /** Number of one-time prekeys to generate initially */
  initialOneTimePrekeys: number;
  /** Minimum one-time prekeys before replenishment */
  minOneTimePrekeys: number;
  /** Signed prekey rotation interval in milliseconds */
  signedPrekeyRotationInterval: number;
  /** Info string for HKDF */
  hkdfInfo: string;
}

/**
 * Default X3DH configuration
 */
export const DEFAULT_X3DH_CONFIG: X3DHConfig = {
  initialOneTimePrekeys: 100,
  minOneTimePrekeys: 20,
  signedPrekeyRotationInterval: 7 * 24 * 60 * 60 * 1000, // 7 days
  hkdfInfo: "SC_X3DH_v1",
};

/**
 * X3DH Key Manager
 *
 * Manages X3DH key generation, bundle creation, and key exchange operations.
 */
export class X3DHKeyManager {
  private config: X3DHConfig;
  private keyBundle: X3DHKeyBundle | null = null;
  private otpLocks = new Set<number>();

  constructor(config: Partial<X3DHConfig> = {}) {
    this.config = { ...DEFAULT_X3DH_CONFIG, ...config };
  }

  /**
   * Generate a new X3DH key bundle
   */
  generateKeyBundle(identityKey: IdentityKeyPair): X3DHKeyBundle {
    // Generate signed prekey
    const signedPrekey = this.generateSignedPrekey(identityKey, 1);

    // Generate one-time prekeys
    const oneTimePrekeys: OneTimePrekey[] = [];
    for (let i = 0; i < this.config.initialOneTimePrekeys; i++) {
      oneTimePrekeys.push(this.generateOneTimePrekey(i + 1));
    }

    this.keyBundle = {
      identityKey,
      signedPrekey,
      oneTimePrekeys,
      nextOTPId: this.config.initialOneTimePrekeys + 1,
      nextSPKId: 2,
    };

    return this.keyBundle;
  }

  /**
   * Generate a signed prekey
   */
  private generateSignedPrekey(
    identityKey: IdentityKeyPair,
    id: number,
  ): SignedPrekey {
    const privateKey = randomBytes(32);
    const publicKey = x25519.getPublicKey(privateKey);

    // Sign the public key with identity key
    const signature = signMessage(publicKey, identityKey.privateKey);

    return {
      keyPair: { publicKey, privateKey },
      signature,
      timestamp: Date.now(),
      id,
    };
  }

  /**
   * Generate a one-time prekey
   */
  private generateOneTimePrekey(id: number): OneTimePrekey {
    const privateKey = randomBytes(32);
    const publicKey = x25519.getPublicKey(privateKey);

    return {
      keyPair: { publicKey, privateKey },
      id,
      used: false,
    };
  }

  /**
   * Get the public prekey bundle for sharing via DHT
   */
  getPublicBundle(peerId: string): PrekeyBundle | null {
    if (!this.keyBundle) return null;

    // Get unused one-time prekeys
    const availableOTPs = this.keyBundle.oneTimePrekeys
      .filter((otp) => !otp.used)
      .slice(0, 20) // Limit to 20 for the bundle
      .map((otp) => ({
        id: otp.id,
        publicKey: new Uint8Array(otp.keyPair.publicKey),
      }));

    return {
      peerId,
      identityKey: new Uint8Array(this.keyBundle.identityKey.publicKey),
      signedPrekeyPublic: new Uint8Array(
        this.keyBundle.signedPrekey.keyPair.publicKey,
      ),
      signedPrekeyId: this.keyBundle.signedPrekey.id,
      signedPrekeySignature: new Uint8Array(
        this.keyBundle.signedPrekey.signature,
      ),
      oneTimePrekeyPublics: availableOTPs,
      timestamp: Date.now(),
    };
  }

  /**
   * Replenish one-time prekeys if needed
   */
  replenishOneTimePrekeys(): number {
    if (!this.keyBundle) return 0;

    const unusedCount = this.keyBundle.oneTimePrekeys.filter(
      (otp) => !otp.used,
    ).length;

    if (unusedCount >= this.config.minOneTimePrekeys) return 0;

    const needed = this.config.initialOneTimePrekeys - unusedCount;
    let generated = 0;

    for (let i = 0; i < needed; i++) {
      this.keyBundle.oneTimePrekeys.push(
        this.generateOneTimePrekey(this.keyBundle.nextOTPId++),
      );
      generated++;
    }

    return generated;
  }

  /**
   * Check if signed prekey needs rotation
   */
  needsSignedPrekeyRotation(): boolean {
    if (!this.keyBundle) return false;
    const age = Date.now() - this.keyBundle.signedPrekey.timestamp;
    return age > this.config.signedPrekeyRotationInterval;
  }

  /**
   * Rotate signed prekey
   */
  rotateSignedPrekey(): SignedPrekey | null {
    if (!this.keyBundle) return null;

    // Wipe old private key
    secureWipe(this.keyBundle.signedPrekey.keyPair.privateKey);

    // Generate new signed prekey
    this.keyBundle.signedPrekey = this.generateSignedPrekey(
      this.keyBundle.identityKey,
      this.keyBundle.nextSPKId++,
    );

    return this.keyBundle.signedPrekey;
  }

  /**
   * Mark a one-time prekey as used
   */
  markOneTimePrekeyUsed(id: number): boolean {
    if (!this.keyBundle || this.otpLocks.has(id)) return false;
    this.otpLocks.add(id);

    try {
      const otp = this.keyBundle.oneTimePrekeys.find((o) => o.id === id);
      if (otp && !otp.used) {
        otp.used = true;
        // Wipe private key after use
        secureWipe(otp.keyPair.privateKey);
        return true;
      }
      return false;
    } finally {
      this.otpLocks.delete(id);
    }
  }

  /**
   * Get one-time prekey private key by ID
   */
  getOneTimePrekeyPrivate(id: number): Uint8Array | null {
    if (!this.keyBundle) return null;

    const otp = this.keyBundle.oneTimePrekeys.find((o) => o.id === id);
    if (otp && !otp.used) {
      return otp.keyPair.privateKey;
    }
    return null;
  }

  /**
   * Get signed prekey private key by ID
   */
  getSignedPrekeyPrivate(id: number): Uint8Array | null {
    if (!this.keyBundle) return null;

    if (this.keyBundle.signedPrekey.id === id) {
      return this.keyBundle.signedPrekey.keyPair.privateKey;
    }
    return null;
  }

  /**
   * Get the current key bundle
   */
  getKeyBundle(): X3DHKeyBundle | null {
    return this.keyBundle;
  }

  /**
   * Import a key bundle (from storage)
   */
  importKeyBundle(bundle: X3DHKeyBundle): void {
    this.keyBundle = bundle;
  }

  /**
   * Export key bundle for storage (WARNING: contains private keys)
   */
  exportKeyBundle(): X3DHKeyBundle | null {
    if (!this.keyBundle) return null;

    // Deep copy
    return {
      identityKey: {
        publicKey: new Uint8Array(this.keyBundle.identityKey.publicKey),
        privateKey: new Uint8Array(this.keyBundle.identityKey.privateKey),
      },
      signedPrekey: {
        keyPair: {
          publicKey: new Uint8Array(
            this.keyBundle.signedPrekey.keyPair.publicKey,
          ),
          privateKey: new Uint8Array(
            this.keyBundle.signedPrekey.keyPair.privateKey,
          ),
        },
        signature: new Uint8Array(this.keyBundle.signedPrekey.signature),
        timestamp: this.keyBundle.signedPrekey.timestamp,
        id: this.keyBundle.signedPrekey.id,
      },
      oneTimePrekeys: this.keyBundle.oneTimePrekeys.map((otp) => ({
        keyPair: {
          publicKey: new Uint8Array(otp.keyPair.publicKey),
          privateKey: new Uint8Array(otp.keyPair.privateKey),
        },
        id: otp.id,
        used: otp.used,
      })),
      nextOTPId: this.keyBundle.nextOTPId,
      nextSPKId: this.keyBundle.nextSPKId,
    };
  }

  /**
   * Destroy key bundle (secure wipe)
   */
  destroy(): void {
    if (!this.keyBundle) return;

    secureWipe(this.keyBundle.identityKey.privateKey);
    secureWipe(this.keyBundle.signedPrekey.keyPair.privateKey);
    for (const otp of this.keyBundle.oneTimePrekeys) {
      if (!otp.used) {
        secureWipe(otp.keyPair.privateKey);
      }
    }
    this.keyBundle = null;
  }
}

/**
 * Verify a prekey bundle's signature
 */
export function verifyPrekeyBundle(
  bundle: PrekeyBundle,
  maxAge: number = 24 * 60 * 60 * 1000, // Default 24 hours
): boolean {
  try {
    // Validate timestamp freshness
    const age = Date.now() - bundle.timestamp;
    if (age < 0 || age > maxAge) {
      return false;
    }

    return verifySignature(
      bundle.signedPrekeyPublic,
      bundle.signedPrekeySignature,
      bundle.identityKey,
    );
  } catch {
    return false;
  }
}

/**
 * Initiate X3DH key exchange (sender/Alice side)
 *
 * Computes shared secret using:
 * - DH1 = DH(IKa, SPKb) - Identity to Signed Prekey
 * - DH2 = DH(EKa, IKb) - Ephemeral to Identity
 * - DH3 = DH(EKa, SPKb) - Ephemeral to Signed Prekey
 * - DH4 = DH(EKa, OPKb) - Ephemeral to One-Time Prekey (if available)
 *
 * SK = KDF(DH1 || DH2 || DH3 || DH4)
 */
export function initiateX3DH(
  myIdentityKey: IdentityKeyPair,
  theirBundle: PrekeyBundle,
  config: Partial<X3DHConfig> = {},
): X3DHInitResult {
  const fullConfig = { ...DEFAULT_X3DH_CONFIG, ...config };

  // Verify the bundle's signature first
  if (!verifyPrekeyBundle(theirBundle)) {
    throw new Error("Invalid prekey bundle signature");
  }

  // Generate ephemeral key pair
  const ephemeralPrivate = randomBytes(32);
  const ephemeralPublic = x25519.getPublicKey(ephemeralPrivate);

  // Convert Ed25519 identity keys to X25519
  const myX25519Private = convertEd25519PrivateKeyToX25519(
    myIdentityKey.privateKey,
  );
  const theirX25519Public = convertEd25519PublicKeyToX25519(
    theirBundle.identityKey,
  );

  // DH1 = DH(IKa_private, SPKb_public)
  const dh1 = x25519.getSharedSecret(
    myX25519Private,
    theirBundle.signedPrekeyPublic,
  );

  // DH2 = DH(EKa_private, IKb_public)
  const dh2 = x25519.getSharedSecret(ephemeralPrivate, theirX25519Public);

  // DH3 = DH(EKa_private, SPKb_public)
  const dh3 = x25519.getSharedSecret(
    ephemeralPrivate,
    theirBundle.signedPrekeyPublic,
  );

  // DH4 = DH(EKa_private, OPKb_public) if available
  let dh4: Uint8Array | null = null;
  let usedOneTimePrekeyId: number | undefined;

  if (theirBundle.oneTimePrekeyPublics.length > 0) {
    // Select a random one-time prekey from the bundle
    const randomIndex = Math.floor(
      Math.random() * theirBundle.oneTimePrekeyPublics.length,
    );
    const otp = theirBundle.oneTimePrekeyPublics[randomIndex];
    dh4 = x25519.getSharedSecret(ephemeralPrivate, otp.publicKey);
    usedOneTimePrekeyId = otp.id;
  }

  // Concatenate DH outputs
  const dhConcat = dh4
    ? new Uint8Array([...dh1, ...dh2, ...dh3, ...dh4])
    : new Uint8Array([...dh1, ...dh2, ...dh3]);

  // Derive shared secret using HKDF
  const info = new TextEncoder().encode(fullConfig.hkdfInfo);
  const sharedSecret = hkdf(sha256, dhConcat, new Uint8Array(32), info, 32);

  // Associated data = IKa || IKb
  const associatedData = new Uint8Array([
    ...myIdentityKey.publicKey,
    ...theirBundle.identityKey,
  ]);

  // Wipe intermediate secrets
  secureWipe(dh1);
  secureWipe(dh2);
  secureWipe(dh3);
  if (dh4) secureWipe(dh4);
  secureWipe(dhConcat);
  secureWipe(ephemeralPrivate);
  secureWipe(myX25519Private);

  return {
    sharedSecret,
    ephemeralPublic,
    usedOneTimePrekeyId,
    usedSignedPrekeyId: theirBundle.signedPrekeyId,
    associatedData,
  };
}

/**
 * Complete X3DH key exchange (receiver/Bob side)
 *
 * Computes the same shared secret using the initial message.
 */
export function completeX3DH(
  myIdentityKey: IdentityKeyPair,
  signedPrekeyPrivate: Uint8Array,
  oneTimePrekeyPrivate: Uint8Array | null,
  initialMessage: X3DHInitialMessage,
  config: Partial<X3DHConfig> = {},
): { sharedSecret: Uint8Array; associatedData: Uint8Array } {
  const fullConfig = { ...DEFAULT_X3DH_CONFIG, ...config };

  // Convert Ed25519 identity keys to X25519
  const myX25519Private = convertEd25519PrivateKeyToX25519(
    myIdentityKey.privateKey,
  );
  const theirX25519Public = convertEd25519PublicKeyToX25519(
    initialMessage.identityKey,
  );

  // DH1 = DH(SPKb_private, IKa_public)
  const dh1 = x25519.getSharedSecret(signedPrekeyPrivate, theirX25519Public);

  // DH2 = DH(IKb_private, EKa_public)
  const dh2 = x25519.getSharedSecret(
    myX25519Private,
    initialMessage.ephemeralKey,
  );

  // DH3 = DH(SPKb_private, EKa_public)
  const dh3 = x25519.getSharedSecret(
    signedPrekeyPrivate,
    initialMessage.ephemeralKey,
  );

  // DH4 = DH(OPKb_private, EKa_public) if one-time prekey was used
  let dh4: Uint8Array | null = null;
  if (oneTimePrekeyPrivate) {
    dh4 = x25519.getSharedSecret(
      oneTimePrekeyPrivate,
      initialMessage.ephemeralKey,
    );
  }

  // Concatenate DH outputs
  const dhConcat = dh4
    ? new Uint8Array([...dh1, ...dh2, ...dh3, ...dh4])
    : new Uint8Array([...dh1, ...dh2, ...dh3]);

  // Derive shared secret using HKDF
  const info = new TextEncoder().encode(fullConfig.hkdfInfo);
  const sharedSecret = hkdf(sha256, dhConcat, new Uint8Array(32), info, 32);

  // Associated data = IKa || IKb
  const associatedData = new Uint8Array([
    ...initialMessage.identityKey,
    ...myIdentityKey.publicKey,
  ]);

  // Wipe intermediate secrets
  secureWipe(dh1);
  secureWipe(dh2);
  secureWipe(dh3);
  if (dh4) secureWipe(dh4);
  secureWipe(dhConcat);
  secureWipe(myX25519Private);

  return { sharedSecret, associatedData };
}

/**
 * Create X3DH initial message to send with first Double Ratchet message
 */
export function createX3DHInitialMessage(
  myIdentityKey: IdentityKeyPair,
  ephemeralPublic: Uint8Array,
  usedSignedPrekeyId: number,
  usedOneTimePrekeyId?: number,
): X3DHInitialMessage {
  return {
    identityKey: new Uint8Array(myIdentityKey.publicKey),
    ephemeralKey: new Uint8Array(ephemeralPublic),
    signedPrekeyId: usedSignedPrekeyId,
    oneTimePrekeyId: usedOneTimePrekeyId,
  };
}

/**
 * Serialize a prekey bundle for DHT storage
 */
export function serializePrekeyBundle(bundle: PrekeyBundle): Uint8Array {
  const json = JSON.stringify({
    peerId: bundle.peerId,
    identityKey: Array.from(bundle.identityKey),
    signedPrekeyPublic: Array.from(bundle.signedPrekeyPublic),
    signedPrekeyId: bundle.signedPrekeyId,
    signedPrekeySignature: Array.from(bundle.signedPrekeySignature),
    oneTimePrekeyPublics: bundle.oneTimePrekeyPublics.map((otp) => ({
      id: otp.id,
      publicKey: Array.from(otp.publicKey),
    })),
    timestamp: bundle.timestamp,
  });
  return new TextEncoder().encode(json);
}

/**
 * Deserialize a prekey bundle from DHT storage
 */
export function deserializePrekeyBundle(data: Uint8Array): PrekeyBundle {
  const json = new TextDecoder().decode(data);
  const obj = JSON.parse(json);

  return {
    peerId: obj.peerId,
    identityKey: new Uint8Array(obj.identityKey),
    signedPrekeyPublic: new Uint8Array(obj.signedPrekeyPublic),
    signedPrekeyId: obj.signedPrekeyId,
    signedPrekeySignature: new Uint8Array(obj.signedPrekeySignature),
    oneTimePrekeyPublics: obj.oneTimePrekeyPublics.map(
      (otp: { id: number; publicKey: number[] }) => ({
        id: otp.id,
        publicKey: new Uint8Array(otp.publicKey),
      }),
    ),
    timestamp: obj.timestamp,
  };
}

/**
 * Serialize X3DH initial message
 */
export function serializeX3DHInitialMessage(
  msg: X3DHInitialMessage,
): Uint8Array {
  const json = JSON.stringify({
    identityKey: Array.from(msg.identityKey),
    ephemeralKey: Array.from(msg.ephemeralKey),
    signedPrekeyId: msg.signedPrekeyId,
    oneTimePrekeyId: msg.oneTimePrekeyId,
  });
  return new TextEncoder().encode(json);
}

/**
 * Deserialize X3DH initial message
 */
export function deserializeX3DHInitialMessage(
  data: Uint8Array,
): X3DHInitialMessage {
  const json = new TextDecoder().decode(data);
  const obj = JSON.parse(json);

  return {
    identityKey: new Uint8Array(obj.identityKey),
    ephemeralKey: new Uint8Array(obj.ephemeralKey),
    signedPrekeyId: obj.signedPrekeyId,
    oneTimePrekeyId: obj.oneTimePrekeyId,
  };
}
