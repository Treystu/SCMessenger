# P0_SECURITY_005: Audit Logging System

**Priority:** P0 (Critical Security)
**Platform:** Core/Rust
**Status:** DONE
**Completed:** 2026-04-15

## Findings

The tamper-evident audit log (`AuditLog` with `AuditEvent` chain hashing) already existed in `core/src/observability.rs`. However, it was **in-memory only** — events were never persisted and would be lost on restart. There was no retention/pruning mechanism.

## Changes Made

1. **`core/src/observability.rs`** — Added persistence and retention to `AuditLog`:
   - `persist(&self, backend)`: Serializes the full log to JSON and stores it under key `audit_log_v1` in the storage backend
   - `load(backend)`: Deserializes the persisted log, or starts fresh if none exists
   - `prune_before(&mut self, before_timestamp)`: Time-based pruning that preserves chain integrity by recording the last pruned event's hash in `pruned_head_hash`
   - Added `pruned_head_hash: Option<String>` field to `AuditLog` struct (with `#[serde(default)]` for backward compatibility)
   - Updated `validate_chain()` to accept `pruned_head_hash` as the valid head when events have been pruned
   - Added `PersistenceError` variant to `AuditLogError`

2. **`core/src/store/history.rs`** — Added `backend()` method to `HistoryManager` to expose the storage backend for audit log persistence

3. **`core/src/lib.rs`** — IronCore initialization:
   - Loads the audit log from storage on startup via `AuditLogType::load()`
   - Persists the audit log during `perform_maintenance()` (every 15 minutes)
   - Prunes audit events older than 365 days during maintenance

4. **`core/src/store/mod.rs`** — Exported `RetentionConfig` and `StorageManager`

5. **`core/src/store/storage.rs`** — Added `RetentionConfig` with configurable `max_messages` and `max_age_days`, and updated `perform_maintenance()` to enforce retention proactively (not just reactively when disk space is low)

## Build Verification
- Rust `cargo check`: PASSED