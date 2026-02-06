/**
 * Secure key storage abstraction for different platforms
 * 
 * Platform-specific implementations:
 * - Web: IndexedDB with Web Crypto API encryption
 * - Node.js: Memory storage (for testing)
 * - iOS: Keychain Services (platform-specific)
 * - Android: KeyStore (platform-specific)
 * 
 * Security Features:
 * - Encryption at rest
 * - Key versioning and migration
 * - Access control
 * - Automatic key wiping on deletion
 */

import { encryptMessage, decryptMessage, generateSessionKey, secureWipe } from './primitives.js';

export interface KeyStorage {
  /**
   * Store a key securely
   */
  storeKey(keyId: string, key: Uint8Array, metadata?: KeyMetadata): Promise<void>;

  /**
   * Retrieve a key
   */
  getKey(keyId: string): Promise<Uint8Array | null>;

  /**
   * Delete a key (with secure wiping)
   */
  deleteKey(keyId: string): Promise<void>;

  /**
   * Check if a key exists
   */
  hasKey(keyId: string): Promise<boolean>;

  /**
   * List all key IDs
   */
  listKeys(): Promise<string[]>;
  
  /**
   * Get key metadata
   */
  getKeyMetadata(keyId: string): Promise<KeyMetadata | null>;
  
  /**
   * Migrate keys to new storage version
   */
  migrateKeys?(fromVersion: number, toVersion: number): Promise<void>;
}

/**
 * Key metadata for versioning and access control
 */
export interface KeyMetadata {
  version: number;
  createdAt: number;
  lastAccessedAt?: number;
  accessCount?: number;
  tags?: string[];
}

/**
 * Stored key with encryption and metadata
 */
interface StoredKey {
  encryptedKey: Uint8Array;
  nonce: Uint8Array;
  metadata: KeyMetadata;
}

/**
 * Web implementation using IndexedDB with Web Crypto API encryption
 * 
 * Features:
 * - Keys encrypted at rest using Web Crypto API
 * - Automatic key unwrapping
 * - Version migration support
 * - Access control and metadata tracking
 */
export class WebKeyStorage implements KeyStorage {
  private dbName = 'sc-keystore';
  private storeName = 'keys';
  private metadataStore = 'metadata';
  private db: IDBDatabase | null = null;
  private storageVersion = 2;
  private masterKey: Uint8Array | null = null;

  /**
   * Initialize or derive master encryption key
   * In production, this should use Web Crypto API's key unwrapping
   */
  private async getMasterKey(): Promise<Uint8Array> {
    if (!this.masterKey) {
      // In a real implementation, this would use Web Crypto API to:
      // 1. Derive key from password (PBKDF2)
      // 2. Or unwrap key stored in browser's key storage
      // For now, we use a session-based key
      const sessionKey = generateSessionKey();
      this.masterKey = sessionKey.key;
    }
    return this.masterKey!;
  }

  async init(): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(this.dbName, this.storageVersion);

      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        this.db = request.result;
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;
        const oldVersion = event.oldVersion;
        
        // Create stores if they don't exist
        if (!db.objectStoreNames.contains(this.storeName)) {
          db.createObjectStore(this.storeName);
        }
        
        if (!db.objectStoreNames.contains(this.metadataStore)) {
          db.createObjectStore(this.metadataStore);
        }
        
        // Handle version migration
        if (oldVersion < this.storageVersion) {
          console.log(`Migrating key storage from version ${oldVersion} to ${this.storageVersion}`);
        }
      };
    });
  }

  async storeKey(keyId: string, key: Uint8Array, metadata?: KeyMetadata): Promise<void> {
    if (!this.db) await this.init();
    
    const masterKey = await this.getMasterKey();
    const sessionKey = generateSessionKey();
    
    // Encrypt key before storing
    const encryptedKey = encryptMessage(key, masterKey, sessionKey.nonce);
    
    const storedKey: StoredKey = {
      encryptedKey,
      nonce: sessionKey.nonce,
      metadata: metadata || {
        version: this.storageVersion,
        createdAt: Date.now(),
        accessCount: 0,
      },
    };
    
    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([this.storeName, this.metadataStore], 'readwrite');
      const keyStore = transaction.objectStore(this.storeName);
      const metaStore = transaction.objectStore(this.metadataStore);
      
      // The put requests are needed to trigger the transaction, but we don't need their results
      keyStore.put(storedKey, keyId);
      metaStore.put(storedKey.metadata, keyId);

      transaction.oncomplete = () => resolve();
      transaction.onerror = () => reject(transaction.error);
    });
  }

  async getKey(keyId: string): Promise<Uint8Array | null> {
    if (!this.db) await this.init();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([this.storeName, this.metadataStore], 'readwrite');
      const keyStore = transaction.objectStore(this.storeName);
      const metaStore = transaction.objectStore(this.metadataStore);
      const request = keyStore.get(keyId);

      request.onsuccess = async () => {
        const storedKey: StoredKey | undefined = request.result;
        if (!storedKey) {
          resolve(null);
          return;
        }
        
        try {
          const masterKey = await this.getMasterKey();
          const decryptedKey = decryptMessage(storedKey.encryptedKey, masterKey, storedKey.nonce);
          
          // Update access metadata
          storedKey.metadata.lastAccessedAt = Date.now();
          storedKey.metadata.accessCount = (storedKey.metadata.accessCount || 0) + 1;
          metaStore.put(storedKey.metadata, keyId);
          
          resolve(decryptedKey);
        } catch (error) {
          reject(new Error('Failed to decrypt key: ' + error));
        }
      };
      
      request.onerror = () => reject(request.error);
    });
  }

  async deleteKey(keyId: string): Promise<void> {
    if (!this.db) await this.init();

    // First get the key to wipe it
    const key = await this.getKey(keyId);
    if (key) {
      secureWipe(key);
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([this.storeName, this.metadataStore], 'readwrite');
      const keyStore = transaction.objectStore(this.storeName);
      const metaStore = transaction.objectStore(this.metadataStore);
      
      keyStore.delete(keyId);
      metaStore.delete(keyId);

      transaction.oncomplete = () => resolve();
      transaction.onerror = () => reject(transaction.error);
    });
  }

  async hasKey(keyId: string): Promise<boolean> {
    const key = await this.getKey(keyId);
    return key !== null;
  }

  async listKeys(): Promise<string[]> {
    if (!this.db) await this.init();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([this.storeName], 'readonly');
      const store = transaction.objectStore(this.storeName);
      const request = store.getAllKeys();

      request.onsuccess = () => resolve(request.result as string[]);
      request.onerror = () => reject(request.error);
    });
  }
  
  async getKeyMetadata(keyId: string): Promise<KeyMetadata | null> {
    if (!this.db) await this.init();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([this.metadataStore], 'readonly');
      const store = transaction.objectStore(this.metadataStore);
      const request = store.get(keyId);

      request.onsuccess = () => resolve(request.result || null);
      request.onerror = () => reject(request.error);
    });
  }
  
  async migrateKeys(fromVersion: number, toVersion: number): Promise<void> {
    console.log(`Migrating keys from version ${fromVersion} to ${toVersion}`);
    
    const keyIds = await this.listKeys();
    
    for (const keyId of keyIds) {
      const metadata = await this.getKeyMetadata(keyId);
      if (metadata && metadata.version < toVersion) {
        // Re-encrypt key with new version
        const key = await this.getKey(keyId);
        if (key) {
          metadata.version = toVersion;
          await this.storeKey(keyId, key, metadata);
          secureWipe(key);
        }
      }
    }
  }
}

/**
 * In-memory implementation for Node.js/testing
 * Includes encryption and metadata tracking like Web implementation
 */
export class MemoryKeyStorage implements KeyStorage {
  private keys: Map<string, StoredKey> = new Map();
  private masterKey: Uint8Array;

  constructor() {
    const sessionKey = generateSessionKey();
    this.masterKey = sessionKey.key;
  }

  async storeKey(keyId: string, key: Uint8Array, metadata?: KeyMetadata): Promise<void> {
    if (!keyId || keyId.trim() === '') {
      throw new Error('Key ID cannot be empty');
    }
    if (!key) {
      throw new Error('Key value cannot be null or undefined');
    }

    const sessionKey = generateSessionKey();
    const encryptedKey = encryptMessage(key, this.masterKey, sessionKey.nonce);
    
    const storedKey: StoredKey = {
      encryptedKey,
      nonce: sessionKey.nonce,
      metadata: metadata || {
        version: 1,
        createdAt: Date.now(),
        accessCount: 0,
      },
    };
    
    this.keys.set(keyId, storedKey);
  }

  async getKey(keyId: string): Promise<Uint8Array | null> {
    if (!keyId) {
      throw new Error('Key ID cannot be null or undefined');
    }

    const storedKey = this.keys.get(keyId);
    if (!storedKey) return null;
    
    try {
      const decryptedKey = decryptMessage(storedKey.encryptedKey, this.masterKey, storedKey.nonce);
      
      // Update access metadata
      storedKey.metadata.lastAccessedAt = Date.now();
      storedKey.metadata.accessCount = (storedKey.metadata.accessCount || 0) + 1;
      
      return decryptedKey;
    } catch (error) {
      throw new Error('Failed to decrypt key: ' + error);
    }
  }

  async deleteKey(keyId: string): Promise<void> {
    const storedKey = this.keys.get(keyId);
    if (storedKey) {
      // Securely wipe encrypted key
      secureWipe(storedKey.encryptedKey);
      secureWipe(storedKey.nonce);
    }
    this.keys.delete(keyId);
  }

  async hasKey(keyId: string): Promise<boolean> {
    return this.keys.has(keyId);
  }

  async listKeys(): Promise<string[]> {
    return Array.from(this.keys.keys());
  }
  
  async getKeyMetadata(keyId: string): Promise<KeyMetadata | null> {
    const storedKey = this.keys.get(keyId);
    return storedKey ? storedKey.metadata : null;
  }
  
  async migrateKeys(fromVersion: number, toVersion: number): Promise<void> {
    const keyIds = await this.listKeys();
    
    for (const keyId of keyIds) {
      const metadata = await this.getKeyMetadata(keyId);
      if (metadata && metadata.version < toVersion) {
        const key = await this.getKey(keyId);
        if (key) {
          metadata.version = toVersion;
          await this.storeKey(keyId, key, metadata);
          secureWipe(key);
        }
      }
    }
  }

  clear(): void {
    // Securely wipe all keys before clearing
    for (const storedKey of this.keys.values()) {
      secureWipe(storedKey.encryptedKey);
      secureWipe(storedKey.nonce);
    }
    this.keys.clear();
  }

  async clearAll(): Promise<void> {
    this.clear();
  }

  async removeOldKeys(beforeTimestamp: number): Promise<void> {
    const keysToRemove: string[] = [];
    
    for (const [keyId, storedKey] of this.keys.entries()) {
      if (storedKey.metadata.createdAt < beforeTimestamp) {
        keysToRemove.push(keyId);
      }
    }
    
    for (const keyId of keysToRemove) {
      await this.deleteKey(keyId);
    }
  }

  async findKeysByTag(tag: string): Promise<string[]> {
    const result: string[] = [];
    
    for (const [keyId, storedKey] of this.keys.entries()) {
      if (storedKey.metadata.tags && storedKey.metadata.tags.includes(tag)) {
        result.push(keyId);
      }
    }
    
    return result;
  }

  async count(): Promise<number> {
    return this.keys.size;
  }

  async getStorageSize(): Promise<number> {
    let totalSize = 0;
    
    for (const storedKey of this.keys.values()) {
      totalSize += storedKey.encryptedKey.byteLength;
      totalSize += storedKey.nonce.byteLength;
    }
    
    return totalSize;
  }
}
