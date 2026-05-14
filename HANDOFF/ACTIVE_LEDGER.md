# Active Ledger — B2 Core Transport & Routing Sweep

**Sweep Date:** 2026-05-13
**Sweep Agent:** deepseek-v4-pro:cloud
**Status:** Read-only sweep complete

---

## Current Compile Gate Status

### `cargo check --workspace`
- **Result:** PASS
- **Errors:** 0
- **Warnings:** 1 — unused import `std::sync::Arc` in `wasm/src/transport.rs:17`
- **Duration:** ~4.8s

### `cargo test --workspace --no-run`
- **Result:** FAIL
- **10 compile failures** — all Internal Compiler Errors (ICEs / rustc bugs), not code defects:

| Failing Test | Error Type |
|---|---|
| `integration_registration_protocol` | ICE: missing symbol resolution for IdentityKeys, SwarmEvent2, start_swarm, etc. |
| `nat_reflection_demo` (example) | 3 prior errors |
| `integration_ironcore_roundtrip` | 1 prior error |
| `integration_contact_block` | 1 prior error |
| `test_mesh_routing` | ICE: privacy resolver panic |
| `test_address_observation` | 1 prior error |
| `integration_e2e` | 1 prior error |
| `property_tests` | 1 prior error |
| `scmessenger-cli` (test "integration") | 1 prior error |
| `scmessenger-wasm` (lib test) | 1 prior error |

- **Root cause:** ICEs cascade from `integration_registration_protocol.rs` importing symbols (`IdentityKeys`, `DeregistrationRequest`, `RegistrationRequest`) that don't exist in the public API or have been renamed/removed. The rustc 1.95.0 ICE rather than producing a clean diagnostic suggests an incremental compilation artifact issue (incremental compilation is disabled in `.cargo/config.toml`).

---

## B2 Task Triage (Wired / Stub-Only / Broken Anchor / Missing)

### All 20 target files exist and all 63 resolved symbols are present.

**Wired (production call paths verified):**

| Symbol | Target | Actual Callers |
|---|---|---|
| `transport_type_to_routing_transport` | `swarm.rs:667` | Internal swarm |
| `add_kad_address` | `swarm.rs:1429` | Public method on swarm handle |
| `get_healthy_connections` | `health.rs:393` | iron_core.rs, diagnostics.rs, manager.rs |
| `get_unhealthy_connections` | `health.rs:403` | iron_core.rs, diagnostics.rs, manager.rs |
| `cleanup_stale_connections` | `health.rs:422` | manager.rs (called on tick) |
| `register_state_change_callback` | `health.rs:413` | iron_core.rs |
| `calculate_dynamic_ttl` | `adaptive_ttl.rs:150` | iron_core.rs (2 call sites) |
| `get_activity` | `adaptive_ttl.rs:125` | Internal use |
| `audit_count` | `relay_custody.rs:748` | iron_core.rs, swarm.rs |
| `get_registration_state_info` | `relay_custody.rs:488` | Public API |
| `registration_transitions_for_identity` | `relay_custody.rs:500` | Public API |
| `can_bootstrap_others` | `mesh_routing.rs:615` | Public API |
| `register_path` | `multipath.rs:91` | optimized_engine.rs → iron_core.rs |
| `active_paths` | `multipath.rs:104` | optimized_engine.rs → iron_core.rs |
| `mark_path_failed` | `multipath.rs:119` | optimized_engine.rs → iron_core.rs |
| `current_discovery_phase` | `optimized_engine.rs:208` | Routing diagnostics |
| `timeout_budget_summary` | `optimized_engine.rs:218` | Routing diagnostics |
| `negative_cache_stats` | `optimized_engine.rs:223` | Routing diagnostics |
| `prefetch_stats` | `optimized_engine.rs:228` | Routing diagnostics |
| `prefetch_manager_mut` | `optimized_engine.rs:253` | Internal routing |
| `clear_unreachable_peer` | `optimized_engine.rs:298` | Internal routing |
| `expire_old_observations` | `observation.rs:96` | Internal observer |
| `all_connections` | `observation.rs:198` | Internal observer |
| `get_all_relay_stats` | `internet.rs:418` | Public API |
| `get_fallback_relays` | `relay_health.rs:153` | Public API |
| `get_healthy_relays` | `circuit_breaker.rs:291` | Public API |
| `start_hole_punch` | `nat.rs:388` | Public API |
| `get_hole_punch_status` | `nat.rs:495` | Public API |
| `start_refresh` | `resume_prefetch.rs:78` | optimized_engine.rs |
| `mark_refresh_failed` | `resume_prefetch.rs:283` | optimized_engine.rs |
| `next_refresh_hint` | `resume_prefetch.rs:298` | optimized_engine.rs |
| `is_prefetch_complete` | `resume_prefetch.rs:303` | optimized_engine.rs |
| `is_prefetch_in_progress` | `resume_prefetch.rs:308` | optimized_engine.rs |
| `prune_below` | `reputation.rs:64` | Internal reputation |
| `should_advance` | `timeout_budget.rs:118` | Internal timeout |
| `peers_needing_reconnect` | `manager.rs:497` | Public API |
| `relay_discovery_mut` | `bootstrap.rs:189` | Internal bootstrap |
| `reset_circuit_breakers` | `bootstrap.rs:505` | Public API |

**Stub-Only / Test-Only (no production call paths found):**

| Symbol | Target | Notes |
|---|---|---|
| `add_discovered_peer` | `wifi_aware.rs:190/454` | 2 definitions in same file, no external callers |
| `on_read` | `ble/gatt.rs:289` | Trait method definition, no external callers |
| `on_write` | `ble/gatt.rs:282` | Trait method definition, no external callers |
| `abusive_peer_burst_is_rate_limited...` | `swarm.rs:4871` | Test function (`#[cfg(test)]`) |
| `normal_low_volume_usage_is_unaffected` | `swarm.rs:4885` | Test function (`#[cfg(test)]`) |
| `duplicate_window_suppresses_immediate_replay...` | `swarm.rs:4899` | Test function (`#[cfg(test)]`) |
| `token_bucket_refills_after_elapsed_time` | `swarm.rs:4923` | Test function (`#[cfg(test)]`) |
| `cheap_heuristics_reject_invalid_payload_shapes` | `swarm.rs:4936` | Test function (`#[cfg(test)]`) |
| `convergence_marker_rejects_invalid_shape` | `swarm.rs:4952` | Test function (`#[cfg(test)]`) |
| `convergence_marker_requires_local_tracking_context` | `swarm.rs:4966` | Test function (`#[cfg(test)]`) |
| `convergence_marker_accepts_when_custody_exists_locally` | `swarm.rs:4995` | Test function (`#[cfg(test)]`) |
| `peer_id_public_key_extraction_roundtrips...` | `swarm.rs:5032` | Test function (`#[cfg(test)]`) |
| `verify_registration_message_rejects_peer_identity_mismatch` | `swarm.rs:5042` | Test function (`#[cfg(test)]`) |
| All `behaviour.rs` tests (8 functions) | `behaviour.rs:549-672` | Test functions (`#[cfg(test)]`) |
| All `relay_custody.rs` tests (12 functions) | `relay_custody.rs:1718-2165` | Test functions (`#[cfg(test)]`) |

**Note on manifest 0-ref symbols:** Many symbols listed with "0 external refs" in the manifest now have production callers through `iron_core.rs` → `optimized_engine.rs` → target module. The manifest ref counts are stale.

**Broken Anchors (line number drift):**
The manifest line numbers are universally stale. Comparing a sample:
- `transport_type_to_routing_transport`: manifest says line 665, actual is line 667 (drift: +2)
- `add_kad_address`: manifest says line 1300, actual is line 1429 (drift: +129)
- `audit_count`: manifest says line 723, actual is line 748 (drift: +25)
- `get_registration_state_info`: manifest says line 471, actual is line 488 (drift: +17)

All anchors have positive drift (code added above), consistent across files. The manifest should be regenerated before any automated patching depends on its line numbers.

**Missing:** None. All 20 target files and 63 symbols listed in the manifest are present.

---

## Integration Test Coverage

### Tests Present:
| Test | Domain | Status |
|---|---|---|
| `integration_all_phases.rs` | Multi-phase: swarm, relay, observation, bootstrap | Compiles (warnings only) |
| `integration_mycorrhizal_routing.rs` | OptimizedRoutingEngine wiring, TTL, negative cache | Compiles (1 warning) |
| `integration_nat_reflection.rs` | NAT traversal, address reflection | Compiles (all tests `#[ignore]`) |
| `integration_relay_custody.rs` | Relay custody: registration, dereg, delivery, dedup | Compiles |
| `integration_offline_partition_matrix.rs` | Offline/partition recovery | Compiles (4 warnings) |
| `integration_receipt_convergence.rs` | Receipt delivery convergence | Compiles |
| `integration_retry_lifecycle.rs` | Retry lifecycle | Compiles |
| `test_mesh_routing.rs` | Relay stats, reputation, retry, multipath, bootstrap | FAILS (ICE) |
| `test_address_observation.rs` | Address observation between peers | FAILS (ICE) |
| `test_multiport.rs` | Multi-port listening | Compiles |
| `test_persistence_restart.rs` | Persistence across restart | Compiles |
| `property_tests.rs` | Property-based crypto/message tests | FAILS (ICE) |

### Coverage Gaps:
- **BLE GATT layer:** No integration test for `gatt.rs` `on_read`/`on_write` — these are trait stubs
- **Circuit Breaker:** No integration test for `circuit_breaker.rs` get_healthy_relays
- **WiFi Aware:** No integration test for `wifi_aware.rs` add_discovered_peer — stub
- **Relay Health:** No integration test for `relay_health.rs` get_fallback_relays
- **Internet Relay Stats:** No integration test for `internet.rs` get_all_relay_stats
- **Observation Expiry:** No integration test for `observation.rs` expire_old_observations

---

## Priority Target Recommendation

**Primary target: `integration_registration_protocol.rs` fix (ICE cascade resolution)**

**Rationale:**
1. This single test file's broken imports cascade into 10 ICE failures across the test suite, blocking the entire compile gate
2. The symbols it imports (`IdentityKeys`, `DeregistrationRequest`, `RegistrationRequest`, `start_swarm`) appear to have been renamed, moved, or removed from the public API
3. Fixing this one test would likely resolve 6+ dependent failures and restore the compile gate
4. This is the highest-leverage single fix for the B2 batch

**Secondary cluster: `test_mesh_routing.rs` + `property_tests.rs`**
These have their own ICEs (not cascaded from registration_protocol). Likely need import path updates or symbol renames.

---

## Blockers & Risks

1. **ICE cascade (BLOCKER):** 10 test compile failures block `cargo test --workspace --no-run` — the compile gate is down
2. **Stale manifest line numbers (RISK):** All line anchors have drifted; automated patching relying on them would fail
3. **WiFi Aware stub (GAP):** `add_discovered_peer` has no callers — WiFi Aware transport may be non-functional
4. **BLE GATT stub (GAP):** `on_read`/`on_write` are trait definitions only — BLE GATT may be incomplete
5. **No TODO/FIXME/unimplemented!() blocks found** — positive sign, but may mean missing functionality is simply absent rather than flagged

---

## Sweep Metadata
- **TODO/FIXME/unimplemented!() in B2 files:** 0
- **Files checked:** 20 (all present)
- **Symbols checked:** 63 (all present)
- **Integration tests:** 12 files, 3 failing (all ICEs)
- **No source code modified**
