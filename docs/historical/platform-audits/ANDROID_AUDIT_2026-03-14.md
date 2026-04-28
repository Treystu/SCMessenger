# Android Real-Time Audit Summary - 2026-03-14

## Audit Methodology
- **Start**: 2026-03-13 12:00 HST (noon local time)
- **End**: 2026-03-13 20:42 HST (current)
- **Duration**: 8 hours 42 minutes of log coverage
- **Method**: Continuous logcat monitoring + historical log analysis
- **Device**: Android Pixel 6a (Serial: 26261JEGR01896, PID: 32459)

## Log Sources Examined
✅ Android logcat (full device logs since noon)  
✅ Android app-specific logs (filtered by PID)  
✅ Mesh diagnostics log (`/data/user/0/com.scmessenger.android/files/mesh_diagnostics.log`)  
✅ Real-time monitoring (background logcat capture)  
⚠️ Database direct query (failed - requires root or run-as)  
⚠️ iOS logs (no iOS device connected during audit)  

## Critical Issues Found

### 1. **SEND MESSAGE FAILURE** (BLOCKER)
- **Status**: Reproducible, root cause identified
- **Impact**: User cannot send messages to saved contacts
- **Root Cause**: ID type confusion - passing peer_id hash instead of Ed25519 public_key
- **Evidence**: `IronCoreException$InvalidInput` at `prepareMessageWithId()`
- **Fix Required**: Yes - Phase 3 of fix plan
- **Doc**: `ROOT_CAUSE_ANALYSIS.md` Issue #1

### 2. **CONTACT RECOGNITION FAILURE** (HIGH)
- **Status**: Reproducible, root cause identified
- **Impact**: Contacts show as "not found" in chat despite being saved
- **Root Cause**: ID truncation/normalization mismatch across ViewModels
- **Evidence**: `contactFound=false` when contact exists
- **Fix Required**: Yes - Phase 2 of fix plan
- **Doc**: `ROOT_CAUSE_ANALYSIS.md` Issue #2

### 3. **DUPLICATE AUTO-CREATE EVENTS** (MEDIUM)
- **Status**: Observed, understood
- **Impact**: Database write amplification, potential race conditions
- **Root Cause**: Multiple transport discoveries without deduplication guard
- **Evidence**: 3x contact upsert in 0.5 seconds
- **Fix Required**: Yes - Phase 4 of fix plan (debounce)
- **Doc**: `ROOT_CAUSE_ANALYSIS.md` Issue #3

### 4. **NEARBY PEER FILTERING** (LOW - needs verification)
- **Status**: Uncertain - logs show correct behavior but user reports issue
- **Impact**: Saved contacts may still appear in "nearby" list
- **Evidence**: Conflicting (logs show filter working)
- **Fix Required**: Maybe - needs user confirmation
- **Doc**: `ROOT_CAUSE_ANALYSIS.md` Issue #4

## Log Types Observed (No New Types in Last 30 Minutes)

| Log Type | Count | Status |
|----------|-------|--------|
| Mesh Stats Updates | ~200 | Routine (every 5s) |
| BLE Scan Windows | ~50 | Routine (30s on/off cycle) |
| Transport Identity Resolution | 5 | Normal discovery |
| Contact Auto-Create | 3 | Duplicate (Issue #3) |
| Send Message Error | 1 | Critical (Issue #1) |
| Chat Screen Lookup | 5 | Normal with Issue #2 |
| Peer Connected/Disconnected | 10 | Normal mesh activity |
| Storage Maintenance | 1 | Routine |
| Bluetooth GATT Events | 15 | Normal BLE activity |

**5-Minute No-New-Types Criteria**: ✅ MET (last novel log type was at 20:34:15, current time 20:42)

## No Freezing/Crashing Observed
- App remained responsive throughout audit period
- No ANR (Application Not Responding) events
- No uncaught exceptions
- No system crash dumps
- Uptime: 1896 seconds (~31.6 minutes) as of last stats

## Identity Modal/Keyboard Issues
**User reported**: Android identity modal keyboard flapping issue

**Audit findings**: 
- ✅ IME (keyboard) events logged normally
- ✅ WindowInsets changes look normal
- ❌ NO flapping/rapid open-close cycle observed in logs
- **Conclusion**: Issue may have been fixed in a previous session, OR it only manifests during first-run onboarding which wasn't triggered during this audit window

**Recommendation**: Test fresh install onboarding flow separately

## Stale Contact Data (User Question)
**User asked**: "Why would Android be using stale data on fresh install?"

**Answer from audit**:
- Contact "Christy" was discovered via relay at 20:33:59
- libp2pPeerId: `12D3KooWMDrHwP6CiVdHWSwD2RNJNM9VDBTeX8vKTqn9YZxAX198`
- Listeners included relay-circuit paths to external relays
- **This is NOT stale data** - it's live relay-relayed discovery
- The bug is in how the discovered data is being stored/retrieved, not the freshness

## Next Actions

1. ✅ Audit complete (5-min no-new-types criteria met)
2. ⏭️ Update canonical documentation per AGENTS.md rules
3. ⏭️ Implement fixes according to ROOT_CAUSE_ANALYSIS.md
4. ⏭️ Build and deploy to device
5. ⏭️ Verify all issues resolved

## Files Generated

- `ISSUES_FOUND.md` - Initial triage
- `ROOT_CAUSE_ANALYSIS.md` - Detailed RCA + fix plan
- `AUDIT_SUMMARY.md` - This file
- `android_full_since_noon.log` - Full logcat (9228 lines)
- `android_app_pid_full.log` - App-specific (510 lines)
- `android_mesh_diagnostics.log` - Mesh diag file
- `contact_audit.log` - Contact-related logs (24 lines)
- `send_error_context.log` - Send failure details
- `identity_modal.log` - IME/keyboard logs (63 lines)

## Compliance with Repo Rules

Per `AGENTS.md` and `.github/copilot-instructions.md`:

✅ All temporary files in `/tmp` (repo-local, gitignored)  
✅ No files created outside repo  
✅ Documentation sync pending (next step)  
✅ Build verification pending (after fixes implemented)  
