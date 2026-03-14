# Android Critical Fixes - Implementation Plan
**Date**: 2026-03-14  
**Status**: Ready for implementation after comprehensive audit  
**Blocking**: v0.2.1 release

## Audit Complete ✅

**Duration**: 8 hours 42 minutes (noon-20:42 HST on 2026-03-13)  
**Method**: Real-time logcat monitoring + historical analysis  
**Criteria Met**: ✅ 5 minutes with no new log types (last novel type at 20:34:15, audit ended 20:42)  
**Documentation**: ✅ All issues documented in canonical docs per AGENTS.md rules

## Critical Issues Identified

### Issue #1: Send Message Failure (BLOCKER)
- **What**: Cannot send messages to contacts
- **Why**: Passing peer_id hash instead of Ed25519 public_key to `prepareMessageWithId()`
- **Impact**: Complete messaging failure
- **Doc**: `ANDROID_ID_UNIFICATION_BUG_2026-03-14.md` Issue #1
- **Canonical Doc**: `REMAINING_WORK_TRACKING.md` WS13.6+ Issue #1

### Issue #2: Contact Recognition Failure (HIGH)
- **What**: Contacts show as "not found" in chat despite being saved
- **Why**: ID truncation mismatch between ViewModels (16-char vs 64-char)
- **Impact**: User sees hashes instead of names; broken UX
- **Doc**: `ANDROID_ID_UNIFICATION_BUG_2026-03-14.md` Issue #2
- **Canonical Doc**: `REMAINING_WORK_TRACKING.md` WS13.6+ Issue #2

### Issue #3: Duplicate Auto-Create (MEDIUM)
- **What**: Same contact created 3x in 0.5 seconds
- **Why**: Multiple transport discoveries without deduplication
- **Impact**: Database write amplification
- **Doc**: `ANDROID_ID_UNIFICATION_BUG_2026-03-14.md` Issue #3
- **Canonical Doc**: `REMAINING_WORK_TRACKING.md` WS13.6+ Issue #3

## Implementation Phases

### Phase 1: ID Standardization (4-6 hours) - CRITICAL
**Goal**: Establish single source of truth for ID types

**Files to modify**:
- `android/app/src/main/java/com/scmessenger/android/utils/PeerIdentifier.kt` (NEW)
- `android/app/src/main/java/com/scmessenger/android/utils/PeerIdResolver.kt` (NEW)

**Changes**:
1. Create `PeerIdentifier` data class:
   ```kotlin
   data class PeerIdentifier(
       val canonicalId: String,        // peer_id for lookups
       val publicKey: String,           // for encryption  
       val libp2pId: String? = null    // for transport routing
   )
   ```

2. Create `PeerIdResolver` to handle all ID conversions

### Phase 2: Contact Lookup Fix (1-2 hours) - HIGH
**Goal**: Fix contact recognition in chat screen

**Files to modify**:
- `android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ContactsViewModel.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ConversationsViewModel.kt`

**Changes**:
1. Remove all `.take(16)` truncations
2. Use full 64-char peer_id for all lookups
3. Use `PeerIdResolver.resolve()` to normalize IDs before lookups

### Phase 3: Send Message Fix (1-2 hours) - CRITICAL  
**Goal**: Ensure correct public key passed to encryption

**Files to modify**:
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (sendMessage function)

**Changes**:
1. After contact lookup, ALWAYS use `contact.publicKey` for encryption
2. NEVER use `contact.peerId` for encryption
3. Add strict validation:
   ```kotlin
   if (publicKey.length != 64) {
       throw IllegalStateException(
           "Invalid public key: peerId=${contact.peerId}, " +
           "publicKey=${contact.publicKey}, expected 64 chars got ${publicKey.length}"
       )
   }
   ```

### Phase 4: Deduplication Guard (1 hour) - MEDIUM
**Goal**: Prevent redundant contact upserts

**Files to modify**:
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (upsertFederatedContact)

**Changes**:
1. Add `recentContactUpserts = ConcurrentHashMap<String, Long>()`
2. Skip upserts within 5-second window unless data changed

### Phase 5: Database Migration (2-3 hours if needed)
**Goal**: Fix any corrupted public_key fields in existing contacts

**Files to modify**:
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (add migration function)

**Changes**:
1. Add `migrateCorruptedPublicKeys()` function
2. Validate each contact's public_key is valid Ed25519 point
3. Attempt recovery from discovered peers or delete contact

## Testing Plan

### Test 1: Fresh Install ✅
**Actions**:
1. Uninstall app: `adb uninstall com.scmessenger.android`
2. Install fixed build
3. Create identity
4. Discover iOS peer "Christy"
5. Attempt to send message

**Expected**:
- ✅ Contact saved with correct `publicKey` (not peer_id)
- ✅ Message sends successfully
- ✅ Chat screen shows "Christy" (not hash)
- ✅ `contactFound=true` in logs

### Test 2: Existing Install Migration
**Actions**:
1. Deploy fixed build to current device (with corrupted contact)
2. Run migration on startup
3. Check logs for migration status
4. Attempt to send to "Christy"

**Expected**:
- ✅ Migration detects corrupted public_key
- ✅ Recovery successful OR contact deleted
- ✅ Send succeeds after recovery

### Test 3: Multiple Transport Discovery
**Actions**:
1. Connect to peer via WiFi, BLE, and relay simultaneously
2. Monitor contact upsert logs

**Expected**:
- ✅ Only one upsert OR quick succession is harmless (debounced)
- ✅ All transport hints captured

### Test 4: ID Consistency
**Actions**:
1. Add contact via nearby discovery
2. Navigate to chat screen
3. Check contact name display
4. Check nearby peers list

**Expected**:
- ✅ Contact name shows (not hash)
- ✅ `contactFound=true`
- ✅ Peer does NOT appear in nearby list

## Build & Deploy Commands

### Build Android
```bash
cd android
./gradlew clean app:assembleDebug
```

### Deploy to Device
```bash
ANDROID_SERIAL=26261JEGR01896 UNINSTALL_FIRST=1 ./android/install-clean.sh
```

### Launch App
```bash
adb -s 26261JEGR01896 shell am start -n com.scmessenger.android/.ui.MainActivity
```

### Monitor Logs
```bash
adb -s 26261JEGR01896 logcat -v threadtime -s SCMessenger:* AndroidRuntime:E | tee android_fix_verification.log
```

## Documentation Updates Required ✅

Per AGENTS.md mandatory closeout checklist:

- ✅ `REMAINING_WORK_TRACKING.md` - Updated with critical issues
- ✅ `ANDROID_ID_UNIFICATION_BUG_2026-03-14.md` - Created with RCA
- ✅ `ANDROID_AUDIT_2026-03-14.md` - Created with audit summary
- ⏭️ `docs/CURRENT_STATE.md` - TODO after fixes implemented
- ⏭️ `docs/V0.2.1_RESIDUAL_RISK_REGISTER.md` - TODO after fixes implemented
- ⏭️ `./scripts/docs_sync_check.sh` - TODO before finalizing

## Estimated Timeline

- **Implementation**: 11-17 hours (phases 1-5)
- **Testing**: 2-3 hours
- **Documentation sync**: 1 hour  
- **Total**: 14-21 hours

## Risk Assessment

**BLOCKER for v0.2.1 Release**

**High Risk Areas**:
- All messaging functionality (can't send)
- Contact management (can't recognize saved contacts)
- User experience (broken, confusing)

**Dependencies**:
- May need iOS audit to ensure not also affected
- Core Rust validation is correct; issue is Android-side only

## Next Steps

1. ⏭️ **User approval to proceed with fixes**
2. ⏭️ Implement Phase 1 (ID Standardization)
3. ⏭️ Implement Phase 2 (Contact Lookup)
4. ⏭️ Implement Phase 3 (Send Message Fix)
5. ⏭️ Build and deploy to test device
6. ⏭️ Run Test 1 (Fresh Install)
7. ⏭️ Run Test 2 (Existing Install)
8. ⏭️ Implement remaining phases if tests pass
9. ⏭️ Final verification and documentation sync
10. ⏭️ Push to repository

## Files Generated During Audit

All in `tmp/audit_20260313_2034/`:
- `ISSUES_FOUND.md` - Initial triage
- `ROOT_CAUSE_ANALYSIS.md` - Detailed RCA (copied to repo root)
- `AUDIT_SUMMARY.md` - Full audit summary (copied to repo root)
- `android_full_since_noon.log` - Full logcat (9228 lines)
- `android_app_pid_full.log` - App-specific (510 lines)
- `android_mesh_diagnostics.log` - Mesh diagnostics
- `contact_audit.log` - Contact logs (24 lines)
- `send_error_context.log` - Send failure details
- `identity_modal.log` - IME/keyboard logs (63 lines)

All temporary files are in `/tmp` (repo-local, gitignored) per AGENTS.md compliance.
