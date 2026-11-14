# ZAI BACKLOG — Sprint 1 and beyond

Sequenced from `docs/zai/plans/V1_0_0_EXECUTION_PLAN.md` (WS-A…WS-F),
`FFI_WIRING_BURNDOWN.md` (162 unwired), `docs/zai/plans/CODEBASE_UNIFICATION_PLAN.md`,
and `HANDOFF_AUDIT/WORLDWIDE_AUDIT_RESULTS.jsonl` (9,667 findings; 128 CRITICAL).
Status legend: [ ] open · [~] in progress · [x] done (this repo).

## Sprint 1 — wiring + unification (current)

- [x] **W2-M · Burndown pipeline repair** (DONE 2026-08-16, Worker branch
  `zai/w2m-burndown-pipeline`, 4 commits, Integrator re-verified: GATE_EXIT=0
  on graph build, md generation, rules_check; determinism = timestamp-only
  drift). **True baseline is 788 unwired, not 162** — the old scoreboard ran
  on a dead corpus (pre-strip + removed discovery overlay; its parser
  crashes on today's corpus). Ghosts: 7 files / 72 funcs dropped, printed.
  Full gate-5 run deferred behind the in-flight gradle build (R-Z5/R-Z6).
- [~] **W1 · AndroidManifest registrations** — register `MeshVpnService`,
  `BootReceiver`, `ShareReceiver` (tag-blocking per CEO_RULINGS_2026-08-16;
  smallest chunk, ideal first upstream cherry-pick later).
- [ ] **W2 · Wiring burndown** — 788 → 0 (honest baseline; was mis-scored
  162 on the dead corpus). The repaired scoreboard concentrates in CORE, not
  Kotlin: iron_core.rs (72), wasm/src/lib.rs (58), swarm.rs (36),
  mobile_bridge.rs (30), daemon_bridge.rs (25), privacy/padding.rs (24),
  crypto/encrypt.rs (21), abuse/reputation.rs (16)... Each entry is a triage:
  wire it / expose it over FFI / retire it as dead code — with the
  simple-name-resolution under-wiring caveat checked before retiring
  anything. Batches touching crypto/transport/routing/privacy trees carry
  Validator verdicts (PHIL-003/012). Regenerate burndown per batch;
  re-rank batches off the new distribution.
- [ ] **W3 · Mock-channel elimination** — replace mock channels with real
  libp2p/sled calls per burndown action plan.
- [ ] **U1 · Escalation single-authority** (WS-B B2) — core
  `EscalationEngine` becomes the only decision-maker; strip Kotlin
  `SmartTransportRouter` dead params + `MeshRepository` bootstrap-priority
  override.
- [ ] **U2 · WiFi-Aware send() no-op** (WS-B B3) — fix the hardcoded
  `false` return in `PeerConnection.send()`.
- [ ] **U3 · Dedup pass** — per CODEBASE_UNIFICATION_PLAN ranked list:
  `transport.rs` ×3, `storage.rs` ×3, `bootstrap.rs` ×3, `notification.rs` ×2,
  `server`/`reputation`/`observability`/`types` ×2.
- [ ] **U4 · backup.rs KDF** (WS-B B1) — memory-hard KDF for the non-fast
  path; document the fast-path rationale.

## Sprint 2 — security + audit

- [ ] Re-run dual-pass audit engine on this tree (post-strip corpus changes:
  AgentSwarmCline gone) → refreshed CRITICAL list.
- [ ] CRITICAL burn-down in class order: PRIVACY_CORRELATION (62) →
  METADATA_LEAK (54) → ADVERSARIAL_DOS (52) → SCALE_RESOURCE_BOUND (46) →
  ERROR_LEAK (44) → PANIC_RISK (39) → FFI_SAFETY (36).
- [ ] WS-A verification debt: backup round-trip test (T1), bincode inbox
  exposure (T6), record verify output for T2–T7, mobile T8–T17.
- [ ] Absorb upstream v0.4.0 tag: `git fetch upstream`, rebase zai/main onto
  the tag, reconcile.

## Sprint 3 — KMP desktop (WS-D)

- [ ] D1: desktop_bridge linuxX64 UniFFI build proven (CI, not local).
- [ ] D2: Compose Multiplatform architecture in `shared/` (replace 5-file
  skeleton).
- [ ] D3: packaging (.deb/AppImage).
- [ ] D4: interop QA (WSL2, BlueZ caveats documented).

## Sprint 4 — PQC tail + CI + close-out

- [ ] PQC-09 hybrid onion (re-enable `integration_relay_onion.rs.disabled`).
- [ ] PQC-08 legacy crypto retirement (last-but-one before ship).
- [ ] PQC-13 cross-version matrix incl. desktop peer class.
- [ ] iOS CI runners per `docs/zai/plans/GITHUB_V1_0_0_RUNNER_SETUP.md`.
- [ ] WS-F: CHANGELOG truthing, version → 1.0.0, device-matrix regression,
  final review against ZAI_CHARTER exit criteria.

## Parking lot (post-1.0.0 or upstream-merged first)

- 13 dependabot bumps / 7 CVEs (upstream S4-1 policy: defer-all).
- iOS parity lane (upstream S4-3), Android last mile (S4-4),
  josh-variant isolation (S4-11), consent-gate pilot (S4-10).
- External crypto audit engagement (S4-2).
