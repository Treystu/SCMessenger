# T3.4 — Routing telemetry for field debugging (zero-infrastructure observability)

**Status:** completed
**Track:** 3 (Mycorrhizal Routing & Hardware-Aware Heuristics)
**Dependencies:** T3.3
**Blocks:** none

## Technical Context
- `observability.rs` (audit events), `transport/diagnostics.rs` (`NetworkDiagnosticsReport`), `DiagnosticsReporter.kt` export path
- Survival deployments can't attach debuggers — the device must self-report why a message took the path it took

## Implementation
1. Ring buffer (last 256) of `RoutingDecision`s (already serializable-shaped: decided_by layer, confidence, primary/alternatives) attached to `NetworkDiagnosticsReport`
2. Expose via existing diagnostics FFI
3. Include drift custody stats (count, oldest age, drop reasons) and per-transport health from `health.rs`
4. No new persistent storage — memory ring only (privacy: cleared on app kill)

## Edge Cases
- Report must redact recipient hints (4-byte hints are already privacy-preserving, but don't include message_ids alongside peer_ids in the same record — keep them unjoinable)
- Bound report size (<256 KiB) for export-via-QR/file use

## Verification
- [x] Unit test: 300 decisions -> ring holds last 256
- [x] Report JSON schema-validated and size-bounded
- [x] Redaction asserted (no message_id+peer_id co-occurrence)
