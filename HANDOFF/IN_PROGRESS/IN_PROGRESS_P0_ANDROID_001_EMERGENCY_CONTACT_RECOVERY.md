# P0_ANDROID_001: Emergency Contact Recovery

**Priority:** P0 CRITICAL
**Platform:** Android
**Status:** Open
**Routing Tags:** [REQUIRES: TECH_DEBT] [REQUIRES: FINALIZATION]

## Objective
Emergency recovery of corrupted contacts database on Android to restore messaging functionality. Contacts are completely missing (0 contacts loaded) despite message history existing, preventing message decryption and making the app unusable.

## Root Cause
From ANDROID_PIXEL_6A_AUDIT_2026-04-17:
- `contacts.db` directory exists with data (`contactsOnDisk=true`)
- Migration already completed (`v2_contacts_db_migration` flag set)  
- ContactManager initialization succeeds but returns 0 contacts
- `resolveTransportIdentity()` returns `null` for discovered peers → no auto-contact creation
- **CRITICAL:** 8 messages exist but contacts database appears corrupted

## Implementation Plan

### 1. Emergency Contact Reconstruction
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
```kotlin
private fun emergencyContactRecovery() {
    // Reconstruct contacts from message history
    val messages = historyManager.getAllMessages()
    val peerIds = messages.map { it.peerId }.distinct()
    
    peerIds.forEach { peerId ->
        if (contactManager.getContactByPeerId(peerId) == null) {
            // Extract public key from peer ID and create contact
            val publicKey = extractPublicKeyFromPeerId(peerId)
            if (publicKey != null) {
                upsertFederatedContact(publicKey, peerId, "Recovered Contact")
            }
        }
    }
}
```

### 2. Database Integrity Verification
**File:** `core/src/store/contact_manager.rs`
```rust
pub fn verify_integrity(&self) -> Result<(), ContactError> {
    // Check sled database integrity
    let contact_count = self.get_contact_count()?;
    let db_size = self.db.size_on_disk()?;
    
    if contact_count == 0 && db_size > 1024 {
        // Database has data but returns 0 contacts → corruption detected
        return Err(ContactError::CorruptionDetected);
    }
    Ok(())
}
```

### 3. Auto-Contact Creation Fix
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
```kotlin
override fun resolveTransportIdentity(normalizedKey: String): TransportIdentity? {
    val canonicalContact = contactManager.getContactByPublicKey(normalizedKey)
    
    if (canonicalContact == null) {
        // EMERGENCY FIX: Create contact for non-relay peers instead of returning null
        if (!isBootstrapRelayPeerFromKey(normalizedKey)) {
            Timber.w("Creating emergency contact for unknown peer: ${normalizedKey.take(8)}...")
            return createEmergencyContact(normalizedKey)
        }
        Timber.d("No existing contact for transport key ${normalizedKey.take(8)}..., treating as transient relay")
        return null
    }
    
    return TransportIdentity(
        publicKey = canonicalContact.publicKey,
        peerId = canonicalContact.peerId,
        nickname = canonicalContact.nickname
    )
}
```

### 4. Corruption Detection and Repair
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
```kotlin
private fun detectAndRepairCorruption() {
    val contactCount = contactManager.getContactCount()
    val messageCount = historyManager.getMessageCount()
    
    if (contactCount == 0 && messageCount > 0) {
        Timber.e("CRITICAL: Data corruption detected - $messageCount messages but 0 contacts")
        emergencyContactRecovery()
        
        // Backup corrupted database
        backupCorruptedDatabase()
        
        // Reinitialize contact manager
        reinitializeContactManager()
    }
}
```

## Files to Modify
1. `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (4 locations)
2. `core/src/store/contact_manager.rs` (integrity verification)
3. `core/src/error.rs` (new ContactError::CorruptionDetected)
4. `android/app/src/main/java/com/scmessenger/android/utils/FileUtils.kt` (backup utilities)

## Test Plan
1. **Simulate Corruption**: Create test scenario with corrupted contacts.db
2. **Verify Recovery**: Test emergency contact reconstruction from message history
3. **Integrity Check**: Verify corruption detection triggers recovery
4. **Auto-Creation**: Test contact auto-creation for discovered peers
5. **Persistence**: Verify contacts persist across app restarts

## Success Criteria
- ✅ Contacts recovered from message history
- ✅ Database integrity verification working
- ✅ Auto-contact creation for non-relay peers
- ✅ Corruption detection and repair mechanism
- ✅ Contacts persist across app restarts

## Priority: URGENT
This issue blocks ALL messaging functionality. Without contacts, messages cannot be decrypted and the app is completely unusable.

**Estimated LOC:** ~150-200 LOC across 4 files
**Time Estimate:** 2-3 hours implementation + 1 hour testing