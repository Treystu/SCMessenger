# W2-T1 — wiring triage batch 1: iron_core + wasm surface (155 fns)

- **Branch**: `zai/w2t1-wiring-triage-1` (base 3fe1b8f0) · **Record**: 2026-08-17
- **Input**: `docs/zai/metrics/unwired_functions.json` as generated on
  `zai/w2m-burndown-pipeline` (NOT merged into this branch's base — the W2-M
  pipeline commits live on that branch; the list was extracted read-only via
  `git show` and verified to match: iron_core 72 + wasm/lib 58 +
  wasm/daemon_bridge 25 = 155).
- **Classification counts**: WIRE 80 / EXPOSE 57 / RETIRE 18 (executed).

## Method

The graph's `is_unwired = in_degree==0 AND out_degree==0` only resolves
bare-identifier calls; `self.`/`.method()`/`::` calls, wasm-bindgen JS calls
(camelCase `js_name`), and UniFFI FFI consumers are invisible to it. Each of
the 155 was therefore checked against, in order:

1. Rust callers repo-wide (`.rs`, method and bare syntax, definition excluded),
2. `core/src/api.udl` + committed generated bindings
   (`iOS/SCMessenger/SCMessenger/Generated/api.swift` — UniFFI proc-macro
   exports of the `#[uniffi::export]` IronCore impl block land here),
3. hand-written Kotlin/Swift (excluding Generated/),
4. JS consumers (`ui/app.js`, `ui/index.html`, `headless/main.js`,
   `wasm/test/*.js`) using the camelCase `js_name`,
5. in-file test callers (`#[test]`/`#[wasm_bindgen_test]`).

Receiver-type collisions (e.g. `TransportManager::get_healthy_connections` vs
`IronCore::get_healthy_connections`, `monitor.` vs `core.` receivers) were
resolved by inspecting each hit's receiver.

## A. core/src/iron_core.rs — 72 functions

### A1. WIRE — real callers the graph missed (41; no code change)

| Function | Evidence (call sites) |
|---|---|
| on_peer_discovered | CoreDelegate trait cb; invoked iron_core.rs:1167; impls mobile_bridge.rs:2060, integration_core_ffi_node1.rs:51, integration_ironcore_roundtrip.rs:301 |
| on_peer_disconnected | trait cb; iron_core.rs:1174; same impls |
| on_peer_identified | trait cb; mobile_bridge.rs:1053 |
| on_message_received | trait cb; iron_core.rs:3522 |
| on_receipt_received | trait cb; iron_core.rs:3447 |
| is_peer_blocked | transport/swarm.rs x3; cli api.rs/main.rs/server.rs; wasm lib.rs:1261; integration tests |
| get_blocked_peer_devices | core/tests/integration_core_ffi_node1.rs |
| get_peer_reputation | cli/api.rs, api_axum.rs; wasm lib.rs:895; tests |
| peer_spam_score | transport/swarm.rs |
| peer_rate_limit_multiplier | transport/swarm.rs; ffi_node1 test |
| sign_data | cli/main.rs; wasm lib.rs:482 |
| get_registration_state | cli/main.rs; wasm lib.rs:1343 |
| notify_peer_discovered | mobile_bridge.rs; wasm lib.rs x2; ffi_node1 test |
| notify_peer_disconnected | mobile_bridge.rs; wasm lib.rs x2; ffi_node1 test |
| record_abuse_signal | mobile_bridge.rs; ffi_node1 x11 |
| run_maintenance_cycle | mobile_bridge.rs; ffi_node1 + drift_mule tests |
| drift_network_state | cli api.rs/api_axum.rs x2 ea; mobile_bridge; wasm:873; routing_tick; tests |
| drift_store_size | same set; wasm:878; retry_lifecycle test |
| relay_jitter_delay | mobile_bridge.rs |
| is_running | mobile/lib.rs x3; wasm x5; tests |
| get_identity_info | 20+ sites: cli (main/server/ble_windows/api), wasm, mobile, desktop_bridge, wasm_support/mesh.rs, 10 test files |
| get_device_id | cli/main.rs; wasm:1331 |
| outbox_count | cli api.rs/api_axum.rs; wasm x2; tests |
| inbox_count | cli api.rs/api_axum.rs; wasm x2; tests |
| unblock_peer | cli main.rs/server.rs; wasm:1240; tests |
| block_and_delete_peer | cli main.rs; wasm:1253; contact_block test |
| get_audit_log | cli/server.rs x2; wasm:883 |
| get_audit_events_since | cli/main.rs; wasm:889 |
| contact_federated_nickname | wasm lib.rs:1194 |
| custody_audit_count | cli/api.rs; api_axum.rs |
| is_consent_granted | ffi_node1 test x2 |
| random_port | mobile_bridge.rs:1803 (`core.random_port()`); transport/swarm.rs:2740 |
| get_libp2p_keypair | cli/main.rs x4; mobile_bridge x3 |
| receive_message | cli (main/ble_mesh); mobile_bridge x2; wasm x2; mobile; 10 test files |
| unregister_notification_endpoint | wasm lib.rs:1067 |
| get_all_connection_stats | cli/api.rs:920; cli/main.rs:4463; mobile_bridge.rs:1852 (all `core.` receivers) |
| get_forwarding_capability | in-file tests iron_core.rs:5010-5011 |
| can_reach_destination | cli/main.rs x2; cli + core integration tests x3 |
| routing_mark_refresh_failed | cli/server.rs |
| routing_next_refresh_hint | cli/server.rs |
| routing_is_prefetch_complete | cli/server.rs |

### A2. EXPOSE — keep, platform-reachable or platform-plausible (13; no removal)

| Function | Evidence / rationale |
|---|---|
| register_blocked_device | inside `#[uniffi::export]` impl block; generated api.swift `registerBlockedDevice` present; no hand-written platform caller found — platform-audit flag |
| contact_display_name | exported; api.swift `contactDisplayName`; no hand-written caller — audit flag |
| blocked_only_peer_ids | exported; api.swift `blockedOnlyPeerIds`; IronCore-level callers: 0 (spam_detection.rs/blocked.rs hits are BlockedManager's own method) — audit flag |
| routing_summary | exported; api.swift `routingSummary`; IronCore-level callers: 0 (wired twin is engine-level `routing_summary`) — audit flag |
| on_app_resume | exported; api.swift `onAppResume`; designed app-lifecycle hook; no Rust or hand-written platform caller yet — wire from platform bridges (PHIL-010 lifecycle parity) |
| routing_clear_unreachable_peer | exported; api.swift binding; duplicates wired `clear_unreachable_peer` (line ~3720) — COPIES note |
| routing_current_discovery_phase | exported; api.swift binding; string-form of `current_discovery_phase` |
| routing_negative_cache_stats | exported; api.swift binding; same stats embedded in routing_tick() |
| register_notification_endpoint | plain impl, 0 callers; push-notification registration surface; wired sibling unregister/touch exist on wasm — keep for cluster completion (conservative) |
| list_notification_endpoints | same cluster, 0 callers |
| clear_all_request_notifications | plain impl, 0 IronCore callers; registry-level counterpart tested in notification.rs; Android implements its own NotificationHelper.clearAllRequestNotifications — cross-layer COPIES note |
| clear_message_notifications | same cluster |
| close_all_notifications | same cluster |

### A3. RETIRE — removed (18; commit `refactor(core): retire 18 dead iron_core wrappers`)

All 18: plain (non-exported) impl block, zero callers repo-wide (incl. tests,
UDL, generated bindings — `swift=0` for each camelCase name — and hand-written
Kotlin/Swift), zero in-file references, pure delegation or exact duplicate.

| Function | Why dead (caller-search evidence) |
|---|---|
| list_blocked_wasm | 0 refs; byte-identical twin of list_blocked_peers_wasm; wasm path actually uses list_blocked_peers_raw() |
| list_blocked_peers_wasm | 0 refs; duplicate pair removed together |
| transport_manager_handle | 0 callers; Arc handle not FFI-safe (no platform path); swarm/transport read fields directly |
| bootstrap_manager_handle | 0 callers; wired sibling relay_bootstrap_manager_handle exists (integration_relay_diagnostics.rs:46) |
| peer_exchange_manager_handle | 0 callers; field also unwired — field + initializers + import removed with it |
| get_healthy_connections | IronCore wrapper 0 callers (diagnostics.rs:100/manager.rs:780 hits are monitor/TransportManager receivers) |
| get_unhealthy_connections | same receiver collision resolved — 0 IronCore callers |
| cleanup_stale_connections | manager.rs hits are monitor/TransportManager receivers; perform_maintenance calls transport_manager.tick() instead |
| expire_address_observations | perform_maintenance (2151) calls transport_manager directly, bypassing this wrapper; wan_swarm test hits are `manager.` receivers |
| current_discovery_phase | mycorrhizal test calls `engine.current_discovery_phase()` (engine method), not the wrapper |
| get_peer_activity | 0 callers; adaptive-TTL diagnostics via engine paths |
| get_policy_relay_config | 0 callers; RelayConfig not platform-reachable; drift policy applied internally via drift_apply_policy |
| routing_refresh_delegate_routes | 0 callers; cli server wired only 3 of the 9 "B2 wiring" fns (the 3 kept) |
| routing_run_optimization | 0 callers; same batch |
| routing_evaluate_all_tracked | 0 callers; same batch |
| routing_prune_below | 0 callers; same batch |
| routing_should_advance | 0 callers; same batch |
| prefetch_start_refresh | 0 callers; wired siblings mark_refresh_failed/next_refresh_hint/is_prefetch_complete stay |

## B. wasm/src/lib.rs — 58 functions

### B1. WIRE (27; no code change)

| Function | Evidence |
|---|---|
| start | JS ui/app.js + headless/main.js |
| stop | JS ui/app.js x2 |
| get_identity_info | getIdentityInfo x3 JS |
| initialize_identity | initializeIdentity x4 JS |
| prepare_message_with_id | prepareMessageWithId ui/app.js |
| prepare_receipt | prepareReceipt ui/app.js |
| stop_swarm | stopSwarm ui/app.js x5 |
| send_prepared_envelope | sendPreparedEnvelope x2 |
| get_peers | getPeers x2 JS + 2 Rust integration tests |
| get_external_addresses | getExternalAddresses JS + test_address_observation.rs |
| get_listeners | getListeners JS |
| get_nat_status | getNatStatus JS |
| classify_notification | classifyNotification headless/main.js |
| set_nickname | setNickname x2 JS |
| export_identity_backup | exportIdentityBackup JS |
| import_identity_backup | importIdentityBackup JS |
| extract_public_key_from_peer_id | x2 JS |
| mark_message_sent | markMessageSent JS |
| get_contact_manager | getContactManager JS |
| get_history_manager | getHistoryManager JS |
| blocked_count | blockedCount ui/app.js |
| export_logs | exportLogs ui/app.js x2 + index.html |
| update_last_seen | updateLastSeen JS (WasmContactManager) |
| clear | clear() x2 ui/app.js (WasmHistoryManager) |
| relay_url_to_multiaddr | in-file: start_receive_loop:961 + 4 unit tests |
| resolve_swarm_keypair_and_mode | in-file: start_swarm_runtime:1807 + test:2246 |
| temp_storage_path | in-file test helper x3 |

(27 rows = 24 wasm-bindgen exports with JS callers + 3 free fns with in-file
Rust callers: relay_url_to_multiaddr, resolve_swarm_keypair_and_mode,
temp_storage_path.)

### B2. EXPOSE (31; keep, proposal table)

| Group | Functions (JS name unused today) | Proposal |
|---|---|---|
| Daemon mode (6) | set_iron_core_mode, get_iron_core_mode, set_daemon_socket_url, get_daemon_socket_url, initialize_identity_from_daemon, get_identity_from_daemon | wire a web daemon-mode UI onto the implemented cli daemon protocol, or retire the whole cluster as a unit in a dedicated chunk |
| Crypto/verify parity (2) | sign_data, verify_signature | web parity with CLI sign path; expose in settings/advanced UI |
| Messaging privacy (2) | prepare_message, prepare_cover_traffic | prepare_message duplicates prepareMessageWithId (UI uses superset) — deprecate prepareMessage in the JS API; cover traffic pending privacy UI |
| Gossipsub (3) | subscribe_topic, unsubscribe_topic, publish_topic | topic UI not yet built on web |
| Reputation/audit diagnostics (7) | get_peer_reputation, get_enhanced_peer_reputation, get_overall_score, get_audit_log, get_audit_events_since, get_drift_state, get_drift_store_size | web dashboard parity (core halves already wired via cli/wasm internals) |
| Identity/registration (4) | get_device_id, get_seniority_timestamp, get_privacy_config, contact_federated_nickname | profile/settings parity |
| Invite/resolution (3) | get_invite_signable_data, resolve_identity, resolve_to_identity_id | B2 invite signing + alias UX parity |
| Logging (1) | record_log | exportLogs is wired; recordLog pairs with it |
| Notifications (2) | unregister_notification_endpoint, touch_notification_endpoint | push-notification endpoint lifecycle |
| Contacts (1) | update_device_id | device-rotation parity |

## C. wasm/src/daemon_bridge.rs — 25 functions

### C1. WIRE (12; in-file test callers — no code change)

| Function | Evidence |
|---|---|
| format_get_identity | test :606 |
| format_send_message | tests :615, :625 |
| format_scan_peers | test :634 |
| format_get_topology | test :642 |
| parse_response | test :670 |
| parse_notification | test :657 |
| connect | tests :684, :691 (non-wasm32 sim branch) |
| send_request | test :694 |
| disconnect | test :705 |
| get_identity_wire_shape | #[test] fn (harness-invoked) |
| send_message_wire_shape | #[test] fn |
| send_message_wire_shape_no_msg_id | #[test] fn |

### C2. EXPOSE (13; keep)

- 11 request formatters with zero callers (format_get_contacts,
  format_add_contact, format_remove_contact, format_get_settings,
  format_update_settings, format_get_history, format_get_conversation,
  format_clear_history, format_list_blocked, format_block_peer,
  format_unblock_peer): these are the wasm-client half of the JSON-RPC daemon
  protocol whose SERVER half is fully implemented
  (core/src/wasm_support/rpc.rs `parse_intent` dispatches all 16 methods).
  Plausible platform need is high; wiring the JS side is the missing half.
- on_notification, cancel_request: bridge API completion (callback
  registration, request cancellation); zero callers, same daemon-mode cluster.

No RETIRE in this file: everything is either test-wired or the client half of
an implemented protocol.

## Burndown delta (line-by-line)

| Line | Value | Explanation |
|---|---|---|
| Baseline (W2-M, 2026-08-16T23:23Z) | Unwired: 788 | pre-chunk scoreboard |
| `Unwired:` after W2-T1 rerun (gate w2t1-burndown-graph) | 788 | UNCHANGED — the pipeline counts the static corpus `HANDOFF_AUDIT/REPO_MAP.jsonl`; its ghost filter is FILE-level only. The 18 removed fns are FUNCTION-level ghosts: file exists, `fn` gone (verified: all 18 absent from tree, all 18 still present in regenerated unwired_functions.json) |
| True tree delta attributable to W2-T1 | -18 (788 -> 770) | on a function-existence-honest scoreboard |
| WIRE-classified (80) | stay counted | by design — real callers exist; fixing the resolver to see method-syntax/JS/FFI edges is a graph-builder improvement, out of scope per brief |
| EXPOSE-classified (57) | stay counted | kept code; proposals above |
| Pre-existing function-level ghosts found (NOT this chunk) | 12 more (788 -> 758 honest tree-wide) | cli/ledger.rs strip_peer_id; crypto/encrypt.rs 3 tests; mobile_bridge.rs dialable_addresses + get_preferred_relays (U1's file — untouched); privacy/onion.rs 4 tests; transport/internet.rs attempt_hole_punch; transport/nat.rs get_hole_punch_status |

**Pipeline v2 proposal (for W2-M owner / Integrator):** add a function-level
existence check to the ghost filter (match `fn`/`fun`/`func` + name per
language); it would loudly drop 30 ghosts (18 from this chunk + 12
pre-existing) and end the corpus-staleness overcount. PHIL-016: the current
788 overcounts the tree by 30.

## Gates

| Gate | Result |
|---|---|
| `w2t1-check-core` (cargo check -p scmessenger-core, own target dir) | GATE_EXIT=0 — "Finished `dev` profile ... in 13m 24s", 1 dead_code warning (peer_exchange_manager field) — taken BEFORE the operator disk-pause directive |
| dead-field cleanup (post-warning) | field + 3 initializers + import + stale comment removed; grep-verified zero remaining refs; NOT cargo-reverified — **gates deferred — disk pause** |
| `w2t1-check-wasm` (cargo check -p scmessenger-wasm) | NOT RUN — **gates deferred — disk pause**. Risk: low — all 18 removals had zero wasm-crate callers (grep evidence); core lib checked green at the 18-fn state |
| `w2t1-burndown-graph` (python, no cargo) | GATE_EXIT=0 — "Unwired: 788 / Ghosts drop: 7 files / 72 funcs" |
| `w2t1-burndown-md` (python, no cargo) | GATE_EXIT=0 — "Generated ... 788 functions cataloged across 123 modules"; artifact then restored from HEAD (regenerated scoreboard belongs to the W2-M branch merge, avoiding a duplicate-content conflict) |

## Integrator batch-check list

1. Re-run `cargo check -p scmessenger-core` (post-cleanup verify) and
   `cargo check -p scmessenger-wasm` once disk is cleared.
2. Merge `zai/w2m-burndown-pipeline` BEFORE this branch's scoreboard
   expectations make sense; then regenerate the burndown — expect 788 from
   the corpus until the function-level ghost filter lands.
3. Batch-check U1 overlap: U1 adds UDL/mobile_bridge surface; none of the 18
   removed fns are UDL-declared (verified udl=0) or mobile_bridge-called
   (verified), but U1's diffs should be grepped against the removed names
   before merge.
4. Platform audit for the 8 exported-but-uncalled EXPOSE entries
   (register_blocked_device, contact_display_name, blocked_only_peer_ids,
   routing_summary, on_app_resume, routing_clear_unreachable_peer,
   routing_current_discovery_phase, routing_negative_cache_stats): if no
   Swift/Kotlin product code adopts them by the platform-parity chunk, they
   are the next RETIRE batch (bindings regenerate automatically).
5. Decide the daemon-mode cluster once: wasm mode setters + daemon_bridge
   formatters + bridge methods are one feature, currently half-wired
   (server implemented, no JS client).
