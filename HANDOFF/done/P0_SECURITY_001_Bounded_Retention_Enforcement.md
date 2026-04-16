# P0_SECURITY_001: Bounded Retention Enforcement

**Priority:** P0 (Critical Security)
**Platform:** Core/Rust, Android
**Status:** DONE
**Completed:** 2026-04-15

## Fix Applied

### Root Cause
The sled database grew without bound. The only retention mechanism was a commented-out `enforceRetention(10000u)` call in MeshRepository.kt and a reactive `perform_maintenance()` that only pruned when disk space dropped below 20%. The mobile bridge `HistoryManager` was never pruned at all.

### Changes Made

1. **`core/src/store/storage.rs` - Added `RetentionConfig` struct**: 
   - `max_messages: u32` (default 50,000) — count-based retention cap
   - `max_age_days: u32` (default 90) — time-based retention in days
   - `with_retention()` constructor for custom configs
   - `perform_maintenance()` now enforces retention in priority order:
     1. Time-based: prune messages older than `max_age_days`
     2. Count-based: enforce `max_messages` cap
     3. Emergency: if disk < 20% free, prune 10% + logs

2. **`core/src/store/mod.rs`**: Exported `RetentionConfig` and `StorageManager`

3. **`core/src/lib.rs` - `perform_maintenance()`**: Added pruning of the mobile bridge `HistoryManager` alongside the generic `StorageManager`. Previously only the generic history store was pruned — the mobile bridge sled DB was never pruned.

4. **`core/src/mobile_bridge.rs` - `clear_conversation()`**: Fixed case-sensitivity bug (`record.peer_id == peer_id` → `record.peer_id.eq_ignore_ascii_case(&peer_id)`) to match the generic `HistoryManager` behavior.

5. **`MeshRepository.kt` - `initializeManagers()`**: Added startup retention enforcement:
   - Prunes messages older than 90 days via `historyManager?.pruneBefore()`
   - Enforces 50k message cap via `historyManager?.enforceRetention()`
   - The periodic maintenance loop (every 15 min) also enforces these via `ironCore?.performMaintenance()`

## Build Verification
- Rust `cargo check`: PASSED
- Android Kotlin: UniFFI binding conflicts are pre-existing (not caused by our changes)