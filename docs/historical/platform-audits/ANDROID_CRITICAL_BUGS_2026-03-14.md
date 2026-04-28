# Comprehensive Audit Summary
**Date:** 2026-03-14 (06:00 UTC / 20:00 Local)
**Duration:** ~2 hours  
**Platform:** Android Pixel 6a, API 35
**Scope:** Real-time log monitoring from noon onwards + comprehensive debugging

---

## Executive Summary

Conducted comprehensive real-time audit of SCMessenger Android application, monitoring logs from 12:00 PM through current time. Identified **4 critical bugs** blocking core messaging functionality, with 2 requiring immediate fixes.

**Status:** ✅ Audit Complete | ⏳ Fixes Pending Implementation

---

## Critical Issues Identified

| # | Issue | Severity | Impact | Status |
|---|-------|----------|--------|--------|
| 1 | Public Key Truncation | 🔴 CRITICAL | All message sends fail | Root cause confirmed |
| 2 | Contact ID Mismatch | 🔴 CRITICAL | Duplicate contacts, broken UI | Root cause confirmed |
| 3 | Stale Nearby Peer | 🟡 HIGH | Confusing UX | Linked to #2 |
| 4 | Fresh Install Stale Data | 🟡 MEDIUM | Privacy concern | Investigation pending |

---

## Issue #1: Public Key Truncation (**BLOCKS MESSAGING**)

### Symptoms
- User adds contact successfully
- Sends message → **InvalidInput error**
- All subsequent sends fail

### Root Cause
When retrieving contact for message send, public key is **truncated to 8 characters**:
```
Expected: a974b6f989bde92863315c7a398631fb4da2a3f8b9d0b42a835544ed5af5a4f7 (64 chars)
Actual:   f77690ef (8 chars)
```

### Evidence
```
Log excerpt from android_app_pid.log:
20:05:26.537 - IdentityDiscovered: publicKey=a974b6f989bde...af5a4f7 (64 chars) ✓
20:05:32.397 - Contact added: df222906d561a0bd...
20:05:36.343 - SEND_MSG_START: publicKey='f77690ef' (8 chars!) ✗
20:05:36.371 - ERROR: InvalidInput from prepareMessageWithId()
```

### Fix Required
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:2415-2418`

Add validation + recovery:
- Check retrieved public key is exactly 64 hex chars
- If truncated, fallback to discovered peers cache
- Add detailed error logging to identify Rust/FFI corruption

---

## Issue #2: Contact ID Mismatch (**BREAKS DEDUPLICATION**)

### Symptoms
- Add contact from nearby
- Contact still shows in "Nearby to Add" list
- System treats as two different entities

### Root Cause
Three different ID schemes used inconsistently:

| Context | ID Format | Example |
|---------|-----------|---------|
| Identity/Discovery | SHA-256(publicKey) | `f77690efd3e6...` |
| Storage/Database | Hash(pubkey+identity) | `df222906d561...` |
| Transport | LibP2P Peer ID | `12D3KooWMDr...` |

Nearby filtering checks: "Is `df222906d561...` in nearby list?"  
Nearby list contains: `f77690efd3e6...`  
Result: **NO MATCH** → Shows as duplicate

### Fix Required
**Unify ID scheme:**  
1. Use identity ID as canonical key everywhere
2. Add `canonicalContactId()` normalization function
3. Update all `contactManager?.get()` calls
4. Fix nearby peer filtering to use public key matching

---

## Issue #3: Stale Nearby Peer After Add

**Root Cause:** Linked to Issue #2 - ID mismatch prevents proper filtering
**Fix:** Resolved by Issue #2 fix

---

## Issue #4: Fresh Install Has Stale History

### Symptoms
- Freshly installed app (19:34:02)
- First contact add (20:05:32)
- History shows 4 messages pre-existing

### Possible Causes
1. Android Auto Backup restoring data
2. SharedPreferences surviving reinstall
3. External storage not cleared

### Investigation Needed
- Check `android:allowBackup` in AndroidManifest.xml
- Verify backup exclusion rules
- Test with `adb shell pm clear` before reinstall

---

## Detailed Documentation

All supporting evidence and analysis available in:
```
tmp/audit_20260313_200506/
├── COMPREHENSIVE_AUDIT_REPORT.md    # High-level findings
├── ROOT_CAUSE_ANALYSIS.md           # Technical deep-dive
├── FIX_IMPLEMENTATION_PLAN.md       # Step-by-step fixes
├── android_app_pid.log              # Full app session (498 lines)
├── android_full.log                 # System logs from noon (10K+ lines)
├── send_error_timeline.log          # Timeline of send failure
└── extended_monitor.log             # Real-time monitoring (7.5K lines)
```

---

## Fix Implementation Status

### ✅ Completed
- [x] Comprehensive log audit
- [x] Root cause identification
- [x] Fix plan documentation
- [x] Test scenario definition

### ⏳ Pending
- [ ] Implement Fix #1 (Public key validation + recovery)
- [ ] Implement Fix #2 (Canonical ID normalization)
- [ ] Build and deploy to device
- [ ] Verify fixes with fresh install test
- [ ] Investigate backup/restore behavior

---

## Next Actions

**Immediate (Today):**
1. Implement Fix #1 and #2 in MeshRepository.kt
2. Add comprehensive debug logging
3. Build and test on physical device
4. Verify message send succeeds

**Follow-up (Tomorrow):**
5. Investigate Rust ContactManager for truncation bug
6. Check uniffi bindings integrity
7. Add integration tests
8. Fix backup/restore behavior

---

## Testing Verification Plan

```bash
# 1. Clean slate
adb shell pm clear com.scmessenger.android
adb uninstall com.scmessenger.android

# 2. Fresh install
adb install app-debug.apk

# 3. Monitor logs
adb logcat -c
adb logcat -v threadtime > test_run.log &

# 4. Test scenario
# - Open app, create identity
# - Add contact from nearby
# - Verify nearby list clears
# - Send message "test"
# - Verify: no InvalidInput error
# - Verify: message delivered

# 5. Analyze logs
grep "SEND_MSG" test_run.log
grep "publicKey=" test_run.log
grep "ERROR" test_run.log
```

---

## Documentation Compliance

Per repository governance rules (AGENTS.md):

✅ **Documentation Updates Required:**
- [ ] Update `docs/CURRENT_STATE.md` with audit findings
- [ ] Update `REMAINING_WORK_TRACKING.md` with fix tasks
- [ ] Update `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` with new risks
- [ ] Run `./scripts/docs_sync_check.sh` before commit

✅ **Build Verification Required:**
- [ ] `cd android && ./gradlew assembleDebug` after code changes
- [ ] Verify successful build before git commit
- [ ] Test on physical device before push

---

## Audit Methodology

1. **Log Collection:**
   - Android: `adb logcat` from noon onwards (PID-filtered)
   - Real-time: Continuous monitoring with background process
   - Duration: 2+ hours of active monitoring

2. **Analysis:**
   - Error pattern identification
   - Timeline reconstruction  
   - Cross-reference with code
   - Root cause confirmation via evidence chain

3. **Documentation:**
   - All findings in `/tmp/audit_*/` (repo-local)
   - Comprehensive markdown reports
   - Fix plans with code examples
   - Test verification procedures

---

## Contact

**Audit Performed By:** AI Assistant (GitHub Copilot CLI)  
**Reviewed By:** [Pending]  
**Approved By:** [Pending]

**Questions/Issues:** See `SUPPORT.md` for contact routing

