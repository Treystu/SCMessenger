# TASK: U7 — Schema Drift Audit (CLI/Android/iOS Persistence Formats)

Status: READY FOR QWEN DELEGATION
Owner: Qwen (investigation + audit)
Scope: Unification U7 (farm-relevant for long-lived deployments)

## Objective

Verify that message history, ledger, and persisted message formats are NOT drifting between CLI (Rust sled), Android (Kotlin persistence layer), and iOS (Swift persistence layer).

## Current State

- **CLI:** `core/src/store/` — sled-backed, `MessageRecord` schema at `core/src/message.rs`
- **Android:** Kotlin persistence in `android/app/src/main/kotlin/.../db/` — SQLite or Room
- **iOS:** Swift Core Data model in `iOS/SCMessenger/Models/`
- **Drift risk:** Different serialization formats, nullable field handling, timestamp precision

## Investigation Tasks

1. **CLI message schema (Rust sled):**
   - File: `core/src/message.rs` (MessageRecord struct)
   - Serialize format: bincode or JSON?
   - Fields: id, direction, peer_id, content, timestamp, sender_timestamp, delivered, status
   - Output: list field names + types + serialization method

2. **Android message schema (Kotlin):**
   - File: `android/app/src/main/kotlin/.../db/MessageEntity.kt`
   - Persistence: SQLite (Room) or other?
   - Fields: must match CLI (id, direction, peer_id, content, timestamps, delivery_status)
   - Output: schema definition + nullable/required field rules

3. **iOS message schema (Swift):**
   - File: `iOS/SCMessenger/Models/MessageModel.swift` or Core Data model
   - Persistence: Core Data, UserDefaults, or JSON?
   - Fields: same 8 fields as CLI/Android
   - Output: schema + type mappings

4. **Drift analysis:**
   - Compare field names, types, nullable rules across all three
   - Timestamp precision: milliseconds vs seconds vs nanoseconds?
   - Status enum: do all three platforms use same StatusEnum values?
   - Serialization: if using different formats (bincode vs JSON vs SQLite), can they round-trip?

5. **Versioning assessment:**
   - Do any platforms have schema versioning/migration logic?
   - If drift detected: recommend explicit versioning strategy or unification path

## Acceptance Criteria

- [DONE] Audit report: 3-table comparison (CLI/Android/iOS fields, types, nullable)
- [DONE] Drift assessment: "No drift" or "Drift detected at: <field>" with severity
- [DONE] Timestamp precision unified: all platforms use same resolution
- [DONE] Status enum: all platforms use identical enum values
- [DONE] If drift: recommend fix (unification or explicit versioning)
- [DONE] Commit: `audit: U7 schema drift check (CLI/Android/iOS message persistence)`

## Output Format

Create file: `HANDOFF/review/U7_SCHEMA_DRIFT_AUDIT_REPORT.md`

```markdown
# U7 Schema Drift Audit Report

**Date:** 2026-07-17
**Scope:** CLI (sled), Android (Kotlin), iOS (Swift) persistence formats

## Message Schema Comparison

| Field | CLI (Rust) | Android (Kotlin) | iOS (Swift) | Match? |
|-------|-----------|------------------|------------|--------|
| id | String | String | String | [OK] |
| ... |  |  |  |  |

## Findings

1. **Drift detected:** [list specific differences]
2. **Timestamp precision:** [resolution across platforms]
3. **Status enum:** [values across platforms]

## Recommendations

[Unification path or versioning strategy]
```

## Blocking/Blocked

**Blocked by:** None
**Blocks:** F0 delivery-truth (low priority, informational)

## Time Estimate

45-60 minutes (file inspection + comparison + report)
