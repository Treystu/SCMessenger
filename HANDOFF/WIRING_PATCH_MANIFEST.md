# Wiring Patch Manifest (Pre-Implementation)

This file provides exact edit coordinates and patch templates for each wiring task without implementing runtime logic.

Total tasks: **318**

## B1-core-entrypoints

| Task | Function | Resolved symbol | Target | Definition line | External refs | Patch template |
|---|---|---|---|---:|---:|---|
| `prepare_onion_message` | `prepare_onion_message` | `prepare_onion_message` | `core/src/lib.rs` | 1702 | 2 | `WIRE prepare_onion_message call path + tests` |
| `random_port` | `random_port` | `random_port` | `core/src/lib.rs` | 3427 | 0 | `WIRE random_port call path + tests` |
| `ratchet_has_session` | `ratchet_has_session` | `ratchet_has_session` | `core/src/lib.rs` | 2192 | 0 | `WIRE ratchet_has_session call path + tests` |
| `ratchet_reset_session` | `ratchet_reset_session` | `ratchet_reset_session` | `core/src/lib.rs` | 2199 | 0 | `WIRE ratchet_reset_session call path + tests` |
| `ratchet_session_count` | `ratchet_session_count` | `ratchet_session_count` | `core/src/lib.rs` | 2187 | 0 | `WIRE ratchet_session_count call path + tests` |
| `relay_jitter_delay` | `relay_jitter_delay` | `relay_jitter_delay` | `core/src/lib.rs` | 1801 | 0 | `WIRE relay_jitter_delay call path + tests` |
| `routing_tick` | `routing_tick` | `routing_tick` | `core/src/lib.rs` | 2750 | 0 | `WIRE routing_tick call path + tests` |
| `validate_audit_chain` | `validate_audit_chain` | `validate_audit_chain` | `core/src/lib.rs` | 2540 | 1 | `WIRE validate_audit_chain call path + tests` |

## B2-core-transport-routing

| Task | Function | Resolved symbol | Target | Definition line | External refs | Patch template |
|---|---|---|---|---:|---:|---|
| `abusive_peer_burst_is_rate_limited_but_other_peer_still_passes` | `abusive_peer_burst_is_rate_limited_but_other_peer_still_passes` | `abusive_peer_burst_is_rate_limited_but_other_peer_still_passes` | `core/src/transport/swarm.rs` | 4496 | 0 | `WIRE abusive_peer_burst_is_rate_limited_but_other_peer_still_passes call path + tests` |
| `active_paths` | `active_paths` | `active_paths` | `core/src/routing/multipath.rs` | 53 | 0 | `WIRE active_paths call path + tests` |
| `add_discovered_peer` | `add_discovered_peer` | `add_discovered_peer` | `core/src/transport/wifi_aware.rs` | 190 | 0 | `WIRE add_discovered_peer call path + tests` |
| `add_kad_address` | `add_kad_address` | `add_kad_address` | `core/src/transport/swarm.rs` | 1301 | 0 | `WIRE add_kad_address call path + tests` |
| `all_connections` | `all_connections` | `all_connections` | `core/src/transport/observation.rs` | 198 | 0 | `WIRE all_connections call path + tests` |
| `audit_count` | `audit_count` | `audit_count` | `core/src/store/relay_custody.rs` | 723 | 0 | `WIRE audit_count call path + tests` |
| `best_relays` | `best_relays` | `best_relays` | `core/src/transport/mesh_routing.rs` | 214 | 1 | `WIRE best_relays call path + tests` |
| `calculate_dynamic_ttl` | `calculate_dynamic_ttl` | `calculate_dynamic_ttl` | `core/src/routing/adaptive_ttl.rs` | 150 | 2 | `WIRE calculate_dynamic_ttl call path + tests` |
| `can_bootstrap_others` | `can_bootstrap_others` | `can_bootstrap_others` | `core/src/transport/mesh_routing.rs` | 615 | 2 | `WIRE can_bootstrap_others call path + tests` |
| `cheap_heuristics_reject_invalid_payload_shapes` | `cheap_heuristics_reject_invalid_payload_shapes` | `cheap_heuristics_reject_invalid_payload_shapes` | `core/src/transport/swarm.rs` | 4561 | 0 | `WIRE cheap_heuristics_reject_invalid_payload_shapes call path + tests` |
| `cleanup_stale_connections` | `cleanup_stale_connections` | `cleanup_stale_connections` | `core/src/transport/health.rs` | 422 | 0 | `WIRE cleanup_stale_connections call path + tests` |
| `clear_unreachable_peer` | `clear_unreachable_peer` | `clear_unreachable_peer` | `core/src/routing/optimized_engine.rs` | 255 | 0 | `WIRE clear_unreachable_peer call path + tests` |
| `converge_delivered_for_message_removes_matching_pending_records` | `converge_delivered_for_message_removes_matching_pending_records` | `converge_delivered_for_message_removes_matching_pending_records` | `core/src/store/relay_custody.rs` | 1767 | 0 | `WIRE converge_delivered_for_message_removes_matching_pending_records call path + tests` |
| `convergence_marker_accepts_when_custody_exists_locally` | `convergence_marker_accepts_when_custody_exists_locally` | `convergence_marker_accepts_when_custody_exists_locally` | `core/src/transport/swarm.rs` | 4620 | 0 | `WIRE convergence_marker_accepts_when_custody_exists_locally call path + tests` |
| `convergence_marker_rejects_invalid_shape` | `convergence_marker_rejects_invalid_shape` | `convergence_marker_rejects_invalid_shape` | `core/src/transport/swarm.rs` | 4577 | 0 | `WIRE convergence_marker_rejects_invalid_shape call path + tests` |
| `convergence_marker_requires_local_tracking_context` | `convergence_marker_requires_local_tracking_context` | `convergence_marker_requires_local_tracking_context` | `core/src/transport/swarm.rs` | 4591 | 0 | `WIRE convergence_marker_requires_local_tracking_context call path + tests` |
| `current_discovery_phase` | `current_discovery_phase` | `current_discovery_phase` | `core/src/routing/optimized_engine.rs` | 169 | 0 | `WIRE current_discovery_phase call path + tests` |
| `custody_audit_persists_across_restart` | `custody_audit_persists_across_restart` | `custody_audit_persists_across_restart` | `core/src/store/relay_custody.rs` | 2083 | 0 | `WIRE custody_audit_persists_across_restart call path + tests` |
| `custody_deduplicates_same_destination_and_message_id` | `custody_deduplicates_same_destination_and_message_id` | `custody_deduplicates_same_destination_and_message_id` | `core/src/store/relay_custody.rs` | 1736 | 0 | `WIRE custody_deduplicates_same_destination_and_message_id call path + tests` |
| `custody_transitions_are_recorded` | `custody_transitions_are_recorded` | `custody_transitions_are_recorded` | `core/src/store/relay_custody.rs` | 1693 | 0 | `WIRE custody_transitions_are_recorded call path + tests` |
| `duplicate_window_suppresses_immediate_replay_then_expires` | `duplicate_window_suppresses_immediate_replay_then_expires` | `duplicate_window_suppresses_immediate_replay_then_expires` | `core/src/transport/swarm.rs` | 4524 | 0 | `WIRE duplicate_window_suppresses_immediate_replay_then_expires call path + tests` |
| `expire_old_observations` | `expire_old_observations` | `expire_old_observations` | `core/src/transport/observation.rs` | 96 | 0 | `WIRE expire_old_observations call path + tests` |
| `for_local_peer_prefers_explicit_custody_dir_override` | `for_local_peer_prefers_explicit_custody_dir_override` | `for_local_peer_prefers_explicit_custody_dir_override` | `core/src/store/relay_custody.rs` | 2140 | 0 | `WIRE for_local_peer_prefers_explicit_custody_dir_override call path + tests` |
| `get_activity` | `get_activity` | `get_activity` | `core/src/routing/adaptive_ttl.rs` | 125 | 0 | `WIRE get_activity call path + tests` |
| `get_all_connection_stats` | `get_all_connection_stats` | `get_all_connection_stats` | `core/src/transport/health.rs` | 381 | 0 | `WIRE get_all_connection_stats call path + tests` |
| `get_all_relay_stats` | `get_all_relay_stats` | `get_all_relay_stats` | `core/src/transport/internet.rs` | 418 | 0 | `WIRE get_all_relay_stats call path + tests` |
| `get_bootstrap_candidates` | `get_bootstrap_candidates` | `get_bootstrap_candidates` | `core/src/transport/mesh_routing.rs` | 610 | 1 | `WIRE get_bootstrap_candidates call path + tests` |
| `get_fallback_relays` | `get_fallback_relays` | `get_fallback_relays` | `core/src/transport/relay_health.rs` | 153 | 0 | `WIRE get_fallback_relays call path + tests` |
| `get_healthy_connections` | `get_healthy_connections` | `get_healthy_connections` | `core/src/transport/health.rs` | 393 | 0 | `WIRE get_healthy_connections call path + tests` |
| `get_healthy_relays` | `get_healthy_relays` | `get_healthy_relays` | `core/src/transport/circuit_breaker.rs` | 291 | 0 | `WIRE get_healthy_relays call path + tests` |
| `get_hole_punch_status` | `get_hole_punch_status` | `get_hole_punch_status` | `core/src/transport/nat.rs` | 495 | 0 | `WIRE get_hole_punch_status call path + tests` |
| `get_registration_state_info` | `get_registration_state_info` | `get_registration_state_info` | `core/src/store/relay_custody.rs` | 471 | 0 | `WIRE get_registration_state_info call path + tests` |
| `get_unhealthy_connections` | `get_unhealthy_connections` | `get_unhealthy_connections` | `core/src/transport/health.rs` | 403 | 0 | `WIRE get_unhealthy_connections call path + tests` |
| `is_prefetch_complete` | `is_prefetch_complete` | `is_prefetch_complete` | `core/src/routing/resume_prefetch.rs` | 296 | 0 | `WIRE is_prefetch_complete call path + tests` |
| `is_prefetch_in_progress` | `is_prefetch_in_progress` | `is_prefetch_in_progress` | `core/src/routing/resume_prefetch.rs` | 301 | 0 | `WIRE is_prefetch_in_progress call path + tests` |
| `mark_path_failed` | `mark_path_failed` | `mark_path_failed` | `core/src/routing/multipath.rs` | 61 | 0 | `WIRE mark_path_failed call path + tests` |
| `mark_refresh_failed` | `mark_refresh_failed` | `mark_refresh_failed` | `core/src/routing/resume_prefetch.rs` | 283 | 0 | `WIRE mark_refresh_failed call path + tests` |
| `negative_cache_stats` | `negative_cache_stats` | `negative_cache_stats` | `core/src/routing/optimized_engine.rs` | 184 | 0 | `WIRE negative_cache_stats call path + tests` |
| `next_refresh_hint` | `next_refresh_hint` | `next_refresh_hint` | `core/src/routing/resume_prefetch.rs` | 291 | 0 | `WIRE next_refresh_hint call path + tests` |
| `normal_low_volume_usage_is_unaffected` | `normal_low_volume_usage_is_unaffected` | `normal_low_volume_usage_is_unaffected` | `core/src/transport/swarm.rs` | 4510 | 0 | `WIRE normal_low_volume_usage_is_unaffected call path + tests` |
| `on_read` | `on_read` | `on_read` | `core/src/transport/ble/gatt.rs` | 289 | 0 | `WIRE on_read call path + tests` |
| `on_write` | `on_write` | `on_write` | `core/src/transport/ble/gatt.rs` | 282 | 0 | `WIRE on_write call path + tests` |
| `peer_id_public_key_extraction_roundtrips_for_ed25519_peers` | `peer_id_public_key_extraction_roundtrips_for_ed25519_peers` | `peer_id_public_key_extraction_roundtrips_for_ed25519_peers` | `core/src/transport/swarm.rs` | 4657 | 0 | `WIRE peer_id_public_key_extraction_roundtrips_for_ed25519_peers call path + tests` |
| `peers_needing_reconnect` | `peers_needing_reconnect` | `peers_needing_reconnect` | `core/src/transport/manager.rs` | 467 | 0 | `WIRE peers_needing_reconnect call path + tests` |
| `prefetch_manager_mut` | `prefetch_manager_mut` | `prefetch_manager_mut` | `core/src/routing/optimized_engine.rs` | 214 | 0 | `WIRE prefetch_manager_mut call path + tests` |
| `prefetch_stats` | `prefetch_stats` | `prefetch_stats` | `core/src/routing/optimized_engine.rs` | 189 | 0 | `WIRE prefetch_stats call path + tests` |
| `prune_below` | `prune_below` | `prune_below` | `core/src/routing/reputation.rs` | 64 | 0 | `WIRE prune_below call path + tests` |
| `register_path` | `register_path` | `register_path` | `core/src/routing/multipath.rs` | 45 | 0 | `WIRE register_path call path + tests` |
| `register_state_change_callback` | `register_state_change_callback` | `register_state_change_callback` | `core/src/transport/health.rs` | 413 | 0 | `WIRE register_state_change_callback call path + tests` |
| `registration_payload_canonical_bytes_are_stable` | `registration_payload_canonical_bytes_are_stable` | `registration_payload_canonical_bytes_are_stable` | `core/src/transport/behaviour.rs` | 571 | 0 | `WIRE registration_payload_canonical_bytes_are_stable call path + tests` |
| `registration_transitions_for_identity` | `registration_transitions_for_identity` | `registration_transitions_for_identity` | `core/src/store/relay_custody.rs` | 483 | 0 | `WIRE registration_transitions_for_identity call path + tests` |
| `relay_discovery_mut` | `relay_discovery_mut` | `relay_discovery_mut` | `core/src/transport/bootstrap.rs` | 189 | 0 | `WIRE relay_discovery_mut call path + tests` |
| `relay_request_carries_ws13_metadata_when_set` | `relay_request_carries_ws13_metadata_when_set` | `relay_request_carries_ws13_metadata_when_set` | `core/src/transport/behaviour.rs` | 538 | 0 | `WIRE relay_request_carries_ws13_metadata_when_set call path + tests` |
| `relay_request_missing_ws13_fields_deserialize_with_defaults` | `relay_request_missing_ws13_fields_deserialize_with_defaults` | `relay_request_missing_ws13_fields_deserialize_with_defaults` | `core/src/transport/behaviour.rs` | 554 | 0 | `WIRE relay_request_missing_ws13_fields_deserialize_with_defaults call path + tests` |
| `reset_circuit_breakers` | `reset_circuit_breakers` | `reset_circuit_breakers` | `core/src/transport/bootstrap.rs` | 482 | 0 | `WIRE reset_circuit_breakers call path + tests` |
| `should_advance` | `should_advance` | `should_advance` | `core/src/routing/timeout_budget.rs` | 118 | 0 | `WIRE should_advance call path + tests` |
| `signed_deregistration_request_rejects_same_source_and_target_device` | `signed_deregistration_request_rejects_same_source_and_target_device` | `signed_deregistration_request_rejects_same_source_and_target_device` | `core/src/transport/behaviour.rs` | 649 | 0 | `WIRE signed_deregistration_request_rejects_same_source_and_target_device call path + tests` |
| `signed_deregistration_request_verifies_against_matching_public_key` | `signed_deregistration_request_verifies_against_matching_public_key` | `signed_deregistration_request_verifies_against_matching_public_key` | `core/src/transport/behaviour.rs` | 635 | 0 | `WIRE signed_deregistration_request_verifies_against_matching_public_key call path + tests` |
| `signed_registration_request_rejects_malformed_identity_id` | `signed_registration_request_rejects_malformed_identity_id` | `signed_registration_request_rejects_malformed_identity_id` | `core/src/transport/behaviour.rs` | 617 | 0 | `WIRE signed_registration_request_rejects_malformed_identity_id call path + tests` |
| `signed_registration_request_rejects_tampered_payload` | `signed_registration_request_rejects_tampered_payload` | `signed_registration_request_rejects_tampered_payload` | `core/src/transport/behaviour.rs` | 599 | 0 | `WIRE signed_registration_request_rejects_tampered_payload call path + tests` |
| `signed_registration_request_verifies_against_matching_public_key` | `signed_registration_request_verifies_against_matching_public_key` | `signed_registration_request_verifies_against_matching_public_key` | `core/src/transport/behaviour.rs` | 585 | 0 | `WIRE signed_registration_request_verifies_against_matching_public_key call path + tests` |
| `start_hole_punch` | `start_hole_punch` | `start_hole_punch` | `core/src/transport/nat.rs` | 388 | 0 | `WIRE start_hole_punch call path + tests` |
| `start_refresh` | `start_refresh` | `start_refresh` | `core/src/routing/resume_prefetch.rs` | 78 | 0 | `WIRE start_refresh call path + tests` |
| `storage_pressure_emergency_mode_rejects_non_critical_and_recovers` | `storage_pressure_emergency_mode_rejects_non_critical_and_recovers` | `storage_pressure_emergency_mode_rejects_non_critical_and_recovers` | `core/src/store/relay_custody.rs` | 2019 | 0 | `WIRE storage_pressure_emergency_mode_rejects_non_critical_and_recovers call path + tests` |
| `storage_pressure_purge_prioritizes_non_identity_then_identity` | `storage_pressure_purge_prioritizes_non_identity_then_identity` | `storage_pressure_purge_prioritizes_non_identity_then_identity` | `core/src/store/relay_custody.rs` | 1933 | 0 | `WIRE storage_pressure_purge_prioritizes_non_identity_then_identity call path + tests` |
| `storage_pressure_purge_records_audit_transition_before_delete` | `storage_pressure_purge_records_audit_transition_before_delete` | `storage_pressure_purge_records_audit_transition_before_delete` | `core/src/store/relay_custody.rs` | 1989 | 0 | `WIRE storage_pressure_purge_records_audit_transition_before_delete call path + tests` |
| `storage_pressure_quota_bands_follow_locked_policy` | `storage_pressure_quota_bands_follow_locked_policy` | `storage_pressure_quota_bands_follow_locked_policy` | `core/src/store/relay_custody.rs` | 1809 | 0 | `WIRE storage_pressure_quota_bands_follow_locked_policy call path + tests` |
| `storage_pressure_state_uses_synthetic_snapshot_when_probe_unavailable` | `storage_pressure_state_uses_synthetic_snapshot_when_probe_unavailable` | `storage_pressure_state_uses_synthetic_snapshot_when_probe_unavailable` | `core/src/store/relay_custody.rs` | 2122 | 0 | `WIRE storage_pressure_state_uses_synthetic_snapshot_when_probe_unavailable call path + tests` |
| `timeout_budget_summary` | `timeout_budget_summary` | `timeout_budget_summary` | `core/src/routing/optimized_engine.rs` | 179 | 0 | `WIRE timeout_budget_summary call path + tests` |
| `token_bucket_refills_after_elapsed_time` | `token_bucket_refills_after_elapsed_time` | `token_bucket_refills_after_elapsed_time` | `core/src/transport/swarm.rs` | 4548 | 0 | `WIRE token_bucket_refills_after_elapsed_time call path + tests` |
| `transport_type_to_routing_transport` | `transport_type_to_routing_transport` | `transport_type_to_routing_transport` | `core/src/transport/swarm.rs` | 666 | 0 | `WIRE transport_type_to_routing_transport call path + tests` |
| `verify_registration_message_rejects_peer_identity_mismatch` | `verify_registration_message_rejects_peer_identity_mismatch` | `verify_registration_message_rejects_peer_identity_mismatch` | `core/src/transport/swarm.rs` | 4667 | 0 | `WIRE verify_registration_message_rejects_peer_identity_mismatch call path + tests` |

## B3-android-repository

| Task | Function | Resolved symbol | Target | Definition line | External refs | Patch template |
|---|---|---|---|---:|---:|---|
| `autoSubscribeToPeerTopics` | `autoSubscribeToPeerTopics` | `autoSubscribeToPeerTopics` | `android/app/src/main/java/com/scmessenger/android/data/TopicManager.kt` | 120 | 0 | `WIRE autoSubscribeToPeerTopics call path + tests` |
| `exportDiagnosticsAsync` | `exportDiagnosticsAsync` | `exportDiagnosticsAsync` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 4358 | 2 | `WIRE exportDiagnosticsAsync call path + tests` |
| `filterMessagesByTopic` | `filterMessagesByTopic` | `filterMessagesByTopic` | `android/app/src/main/java/com/scmessenger/android/data/TopicManager.kt` | 133 | 0 | `WIRE filterMessagesByTopic call path + tests` |
| `getBlockedCount` | `getBlockedCount` | `getBlockedCount` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 3348 | 0 | `WIRE getBlockedCount call path + tests` |
| `getBootstrapNodesForSettings` | `getBootstrapNodesForSettings` | `getBootstrapNodesForSettings` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 67 | 0 | `WIRE getBootstrapNodesForSettings call path + tests` |
| `getInboxCount` | `getInboxCount` | `getInboxCount` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 3415 | 0 | `WIRE getInboxCount call path + tests` |
| `getKnownTopicsList` | `getKnownTopicsList` | `getKnownTopicsList` | `android/app/src/main/java/com/scmessenger/android/data/TopicManager.kt` | 163 | 0 | `WIRE getKnownTopicsList call path + tests` |
| `getMessage` | `getMessage` | `getMessage` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 4076 | 14 | `WIRE getMessage call path + tests` |
| `getNetworkDiagnosticsSnapshot` | `getNetworkDiagnosticsSnapshot` | `getNetworkDiagnosticsSnapshot` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 7480 | 0 | `WIRE getNetworkDiagnosticsSnapshot call path + tests` |
| `getNetworkFailureSummary` | `getNetworkFailureSummary` | `getNetworkFailureSummary` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 7475 | 0 | `WIRE getNetworkFailureSummary call path + tests` |
| `getRetryDelay` | `getRetryDelay` | `getRetryDelay` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 623 | 0 | `WIRE getRetryDelay call path + tests` |
| `getSubscribedTopicsList` | `getSubscribedTopicsList` | `getSubscribedTopicsList` | `android/app/src/main/java/com/scmessenger/android/data/TopicManager.kt` | 156 | 0 | `WIRE getSubscribedTopicsList call path + tests` |
| `getTransportHealthSummary` | `getTransportHealthSummary` | `getTransportHealthSummary` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 8023 | 0 | `WIRE getTransportHealthSummary call path + tests` |
| `incrementAttemptCount` | `incrementAttemptCount` | `incrementAttemptCount` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 613 | 0 | `WIRE incrementAttemptCount call path + tests` |
| `loadPendingOutboxAsync` | `loadPendingOutboxAsync` | `loadPendingOutboxAsync` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 5699 | 0 | `WIRE loadPendingOutboxAsync call path + tests` |
| `logMessageDeliveryAttempt` | `logMessageDeliveryAttempt` | `logMessageDeliveryAttempt` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 645 | 0 | `WIRE logMessageDeliveryAttempt call path + tests` |
| `markCorrupted` | `markCorrupted` | `markCorrupted` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 472 | 0 | `WIRE markCorrupted call path + tests` |
| `observeNetworkStats` | `observeNetworkStats` | `observeNetworkStats` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 7633 | 0 | `WIRE observeNetworkStats call path + tests` |
| `observePeers` | `observePeers` | `observePeers` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 7618 | 0 | `WIRE observePeers call path + tests` |
| `onPeerDisconnected` | `onPeerDisconnected` | `onPeerDisconnected` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 1444 | 25 | `WIRE onPeerDisconnected call path + tests` |
| `onReceiptReceived` | `onReceiptReceived` | `onReceiptReceived` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 1860 | 17 | `WIRE onReceiptReceived call path + tests` |
| `primeRelayBootstrapConnectionsLegacy` | `primeRelayBootstrapConnectionsLegacy` | `primeRelayBootstrapConnectionsLegacy` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 7161 | 0 | `WIRE primeRelayBootstrapConnectionsLegacy call path + tests` |
| `recordConnectionFailure` | `recordConnectionFailure` | `recordConnectionFailure` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 4207 | 1 | `WIRE recordConnectionFailure call path + tests` |
| `recordTransportEvent` | `recordTransportEvent` | `recordTransportEvent` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 8011 | 0 | `WIRE recordTransportEvent call path + tests` |
| `resetServiceStats` | `resetServiceStats` | `resetServiceStats` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 3003 | 1 | `WIRE resetServiceStats call path + tests` |
| `searchContacts` | `searchContacts` | `searchContacts` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 3277 | 1 | `WIRE searchContacts call path + tests` |
| `setContactNickname` | `setContactNickname` | `setContactNickname` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 3281 | 1 | `WIRE setContactNickname call path + tests` |
| `shouldRetryMessage` | `shouldRetryMessage` | `shouldRetryMessage` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 637 | 0 | `WIRE shouldRetryMessage call path + tests` |
| `testLedgerRelayConnectivity` | `testLedgerRelayConnectivity` | `testLedgerRelayConnectivity` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 1036 | 0 | `WIRE testLedgerRelayConnectivity call path + tests` |
| `updateContactDeviceId` | `updateContactDeviceId` | `updateContactDeviceId` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 3423 | 1 | `WIRE updateContactDeviceId call path + tests` |

## B4-android-ui

| Task | Function | Resolved symbol | Target | Definition line | External refs | Patch template |
|---|---|---|---|---:|---:|---|
| `clearAllHistory` | `clearAllHistory` | `clearAllHistory` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ConversationsViewModel.kt` | 198 | 0 | `WIRE clearAllHistory call path + tests` |
| `clearInput` | `clearInput` | `clearInput` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt` | 224 | 0 | `WIRE clearInput call path + tests` |
| `clearSearch` | `clearSearch` | `clearSearch` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ContactsViewModel.kt` | 549 | 0 | `WIRE clearSearch call path + tests` |
| `ConnectionQualityIndicator` | `ConnectionQualityIndicator` | `ConnectionQualityIndicator` | `android/app/src/main/java/com/scmessenger/android/ui/components/StatusIndicator.kt` | 122 | 0 | `WIRE ConnectionQualityIndicator call path + tests` |
| `ContactDetailScreen` | `ContactDetailScreen` | `ContactDetailScreen` | `android/app/src/main/java/com/scmessenger/android/ui/contacts/ContactDetailScreen.kt` | 38 | 0 | `WIRE ContactDetailScreen call path + tests` |
| `ErrorState` | `ErrorState` | `ErrorState` | `android/app/src/main/java/com/scmessenger/android/ui/components/ErrorBanner.kt` | 138 | 0 | `WIRE ErrorState call path + tests` |
| `getNetworkDiagnosticsReport` | `getNetworkDiagnosticsReport` | `getNetworkDiagnosticsReport` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt` | 614 | 0 | `WIRE getNetworkDiagnosticsReport call path + tests` |
| `IdenticonFromHex` | `IdenticonFromHex` | `IdenticonFromHex` | `android/app/src/main/java/com/scmessenger/android/ui/components/Identicon.kt` | 115 | 0 | `WIRE IdenticonFromHex call path + tests` |
| `InfoBanner` | `InfoBanner` | `InfoBanner` | `android/app/src/main/java/com/scmessenger/android/ui/components/ErrorBanner.kt` | 176 | 0 | `WIRE InfoBanner call path + tests` |
| `LabeledCopyableText` | `LabeledCopyableText` | `LabeledCopyableText` | `android/app/src/main/java/com/scmessenger/android/ui/components/CopyableText.kt` | 89 | 0 | `WIRE LabeledCopyableText call path + tests` |
| `loadConversation` | `loadConversation` | `loadConversation` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ConversationsViewModel.kt` | 114 | 0 | `WIRE loadConversation call path + tests` |
| `loadMoreMessages` | `loadMoreMessages` | `loadMoreMessages` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt` | 358 | 2 | `WIRE loadMoreMessages call path + tests` |
| `MeshSettingsScreen` | `MeshSettingsScreen` | `MeshSettingsScreen` | `android/app/src/main/java/com/scmessenger/android/ui/settings/MeshSettingsScreen.kt` | 32 | 2 | `WIRE MeshSettingsScreen call path + tests` |
| `MessageInput` | `MessageInput` | `MessageInput` | `android/app/src/main/java/com/scmessenger/android/ui/chat/MessageInput.kt` | 20 | 0 | `WIRE MessageInput call path + tests` |
| `PeerListScreen` | `PeerListScreen` | `PeerListScreen` | `android/app/src/main/java/com/scmessenger/android/ui/dashboard/PeerListScreen.kt` | 36 | 0 | `WIRE PeerListScreen call path + tests` |
| `PowerSettingsScreen` | `PowerSettingsScreen` | `PowerSettingsScreen` | `android/app/src/main/java/com/scmessenger/android/ui/settings/PowerSettingsScreen.kt` | 30 | 2 | `WIRE PowerSettingsScreen call path + tests` |
| `resolveDeliveryState` | `resolveDeliveryState` | `resolveDeliveryState` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ConversationsViewModel.kt` | 395 | 0 | `WIRE resolveDeliveryState call path + tests` |
| `setPeer` | `setPeer` | `setPeer` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt` | 76 | 6 | `WIRE setPeer call path + tests` |
| `TopologyScreen` | `TopologyScreen` | `TopologyScreen` | `android/app/src/main/java/com/scmessenger/android/ui/dashboard/TopologyScreen.kt` | 40 | 0 | `WIRE TopologyScreen call path + tests` |
| `TruncatedCopyableText` | `TruncatedCopyableText` | `TruncatedCopyableText` | `android/app/src/main/java/com/scmessenger/android/ui/components/CopyableText.kt` | 130 | 0 | `WIRE TruncatedCopyableText call path + tests` |
| `updateBatteryFloor` | `updateBatteryFloor` | `updateBatteryFloor` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt` | 386 | 2 | `WIRE updateBatteryFloor call path + tests` |
| `updateDiscoveryMode` | `updateDiscoveryMode` | `updateDiscoveryMode` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt` | 416 | 2 | `WIRE updateDiscoveryMode call path + tests` |
| `updateInputText` | `updateInputText` | `updateInputText` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt` | 216 | 2 | `WIRE updateInputText call path + tests` |
| `updateMaxRelayBudget` | `updateMaxRelayBudget` | `updateMaxRelayBudget` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt` | 380 | 0 | `WIRE updateMaxRelayBudget call path + tests` |
| `WarningBanner` | `WarningBanner` | `WarningBanner` | `android/app/src/main/java/com/scmessenger/android/ui/components/ErrorBanner.kt` | 157 | 0 | `WIRE WarningBanner call path + tests` |

## B5-android-transport-service

| Task | Function | Resolved symbol | Target | Definition line | External refs | Patch template |
|---|---|---|---|---:|---:|---|
| `acquireWakeLock` | `acquireWakeLock` | `acquireWakeLock` | `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt` | 251 | 0 | `WIRE acquireWakeLock call path + tests` |
| `applyAdvertiseSettings` | `applyAdvertiseSettings` | `applyAdvertiseSettings` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt` | 142 | 1 | `WIRE applyAdvertiseSettings call path + tests` |
| `applyScanSettings` | `applyScanSettings` | `applyScanSettings` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` | 237 | 2 | `WIRE applyScanSettings call path + tests` |
| `attemptBleRecovery` | `attemptBleRecovery` | `attemptBleRecovery` | `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt` | 454 | 0 | `WIRE attemptBleRecovery call path + tests` |
| `checkAndRecordMessage` | `checkAndRecordMessage` | `checkAndRecordMessage` | `android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt` | 214 | 2 | `WIRE checkAndRecordMessage call path + tests` |
| `clearAnrEvents` | `clearAnrEvents` | `clearAnrEvents` | `android/app/src/main/java/com/scmessenger/android/service/PerformanceMonitor.kt` | 199 | 0 | `WIRE clearAnrEvents call path + tests` |
| `clearPeerCache` | `clearPeerCache` | `clearPeerCache` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` | 536 | 0 | `WIRE clearPeerCache call path + tests` |
| `currentCount` | `currentCount` | `currentCount` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleQuotaManager.kt` | 50 | 4 | `WIRE currentCount call path + tests` |
| `disableTransport` | `disableTransport` | `disableTransport` | `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt` | 388 | 0 | `WIRE disableTransport call path + tests` |
| `enableTransport` | `enableTransport` | `enableTransport` | `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt` | 364 | 0 | `WIRE enableTransport call path + tests` |
| `forceRestartScanning` | `forceRestartScanning` | `forceRestartScanning` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` | 479 | 0 | `WIRE forceRestartScanning call path + tests` |
| `fromValue` | `fromValue` | `fromValue` | `android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt` | 34 | 0 | `WIRE fromValue call path + tests` |
| `getActiveTransports` | `getActiveTransports` | `getActiveTransports` | `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt` | 239 | 0 | `WIRE getActiveTransports call path + tests` |
| `getAllAnrEvents` | `getAllAnrEvents` | `getAllAnrEvents` | `android/app/src/main/java/com/scmessenger/android/service/PerformanceMonitor.kt` | 152 | 0 | `WIRE getAllAnrEvents call path + tests` |
| `getAnrStats` | `getAnrStats` | `getAnrStats` | `android/app/src/main/java/com/scmessenger/android/service/PerformanceMonitor.kt` | 134 | 0 | `WIRE getAnrStats call path + tests` |
| `getAvailableTransports` | `getAvailableTransports` | `getAvailableTransportsSorted` | `android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt` | 202 | 2 | `WIRE getAvailableTransports call path + tests` |
| `getAvailableTransportsSorted` | `getAvailableTransportsSorted` | `getAvailableTransportsSorted` | `android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt` | 202 | 2 | `WIRE getAvailableTransportsSorted call path + tests` |
| `getDedupStats` | `getDedupStats` | `getDedupStats` | `android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt` | 258 | 2 | `WIRE getDedupStats call path + tests` |
| `getHealthStatus` | `getHealthStatus` | `getHealthStatus` | `android/app/src/main/java/com/scmessenger/android/service/PerformanceMonitor.kt` | 141 | 0 | `WIRE getHealthStatus call path + tests` |
| `getTotalAnrEvents` | `getTotalAnrEvents` | `getTotalAnrEvents` | `android/app/src/main/java/com/scmessenger/android/service/AnrWatchdog.kt` | 264 | 0 | `WIRE getTotalAnrEvents call path + tests` |
| `handleBleFailure` | `handleBleFailure` | `handleBleFailure` | `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt` | 431 | 0 | `WIRE handleBleFailure call path + tests` |
| `handleScanFailure` | `handleScanFailure` | `handleScanFailure` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` | 498 | 0 | `WIRE handleScanFailure call path + tests` |
| `isPortLikelyBlocked` | `isPortLikelyBlocked` | `isPortLikelyBlocked` | `android/app/src/main/java/com/scmessenger/android/transport/NetworkDetector.kt` | 189 | 0 | `WIRE isPortLikelyBlocked call path + tests` |
| `isServiceHealthy` | `isServiceHealthy` | `isServiceHealthy` | `android/app/src/main/java/com/scmessenger/android/service/ServiceHealthMonitor.kt` | 340 | 0 | `WIRE isServiceHealthy call path + tests` |
| `notifyBackground` | `notifyBackground` | `notifyBackground` | `android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt` | 399 | 0 | `WIRE notifyBackground call path + tests` |
| `notifyForeground` | `notifyForeground` | `notifyForeground` | `android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt` | 406 | 0 | `WIRE notifyForeground call path + tests` |
| `onAnr` | `onAnr` | `onAnr` | `android/app/src/main/java/com/scmessenger/android/service/AnrWatchdog.kt` | 38 | 0 | `WIRE onAnr call path + tests` |
| `onBind` | `onBind` | `onBind` | `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt` | 534 | 0 | `WIRE onBind call path + tests` |
| `onBleDataReceived` | `onBleDataReceived` | `onBleDataReceived` | `android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt` | 314 | 22 | `WIRE onBleDataReceived call path + tests` |
| `onDiscoveryStarted` | `onDiscoveryStarted` | `onDiscoveryStarted` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 141 | 0 | `WIRE onDiscoveryStarted call path + tests` |
| `onDiscoveryStopped` | `onDiscoveryStopped` | `onDiscoveryStopped` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 146 | 0 | `WIRE onDiscoveryStopped call path + tests` |
| `onRegistrationFailed` | `onRegistrationFailed` | `onRegistrationFailed` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 113 | 0 | `WIRE onRegistrationFailed call path + tests` |
| `onResolveFailed` | `onResolveFailed` | `onResolveFailed` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 183 | 0 | `WIRE onResolveFailed call path + tests` |
| `onScanFailed` | `onScanFailed` | `onScanFailed` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` | 146 | 0 | `WIRE onScanFailed call path + tests` |
| `onScanResult` | `onScanResult` | `onScanResult` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` | 100 | 0 | `WIRE onScanResult call path + tests` |
| `onServiceFound` | `onServiceFound` | `onServiceFound` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 151 | 0 | `WIRE onServiceFound call path + tests` |
| `onServiceLost` | `onServiceLost` | `onServiceLost` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 161 | 0 | `WIRE onServiceLost call path + tests` |
| `onServiceRegistered` | `onServiceRegistered` | `onServiceRegistered` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 122 | 0 | `WIRE onServiceRegistered call path + tests` |
| `onServiceResolved` | `onServiceResolved` | `onServiceResolved` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 187 | 0 | `WIRE onServiceResolved call path + tests` |
| `onServiceUnregistered` | `onServiceUnregistered` | `onServiceUnregistered` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 127 | 0 | `WIRE onServiceUnregistered call path + tests` |
| `onStartDiscoveryFailed` | `onStartDiscoveryFailed` | `onStartDiscoveryFailed` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 165 | 0 | `WIRE onStartDiscoveryFailed call path + tests` |
| `onStartFailure` | `onStartFailure` | `onStartFailure` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt` | 56 | 0 | `WIRE onStartFailure call path + tests` |
| `onStartSuccess` | `onStartSuccess` | `onStartSuccess` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt` | 49 | 0 | `WIRE onStartSuccess call path + tests` |
| `onStopDiscoveryFailed` | `onStopDiscoveryFailed` | `onStopDiscoveryFailed` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 170 | 0 | `WIRE onStopDiscoveryFailed call path + tests` |
| `onUnregistrationFailed` | `onUnregistrationFailed` | `onUnregistrationFailed` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 118 | 0 | `WIRE onUnregistrationFailed call path + tests` |
| `recordAnrEvent` | `recordAnrEvent` | `recordAnrEvent` | `android/app/src/main/java/com/scmessenger/android/service/PerformanceMonitor.kt` | 53 | 0 | `WIRE recordAnrEvent call path + tests` |
| `recordUiTiming` | `recordUiTiming` | `recordUiTiming` | `android/app/src/main/java/com/scmessenger/android/service/PerformanceMonitor.kt` | 88 | 0 | `WIRE recordUiTiming call path + tests` |
| `resetHealth` | `resetHealth` | `resetHealthStats` | `android/app/src/main/java/com/scmessenger/android/service/ServiceHealthMonitor.kt` | 88 | 4 | `WIRE resetHealth call path + tests` |
| `sendBlePacket` | `sendBlePacket` | `sendBlePacket` | `android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt` | 334 | 21 | `WIRE sendBlePacket call path + tests` |
| `setBleComponents` | `setBleComponents` | `setBleComponents` | `android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt` | 87 | 0 | `WIRE setBleComponents call path + tests` |
| `setRotationInterval` | `setRotationInterval` | `setRotationInterval` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt` | 130 | 3 | `WIRE setRotationInterval call path + tests` |
| `shouldUseTransport` | `shouldUseTransport` | `shouldUseTransport` | `android/app/src/main/java/com/scmessenger/android/transport/TransportHealthMonitor.kt` | 50 | 0 | `WIRE shouldUseTransport call path + tests` |
| `startAll` | `startAll` | `startAll` | `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt` | 82 | 0 | `WIRE startAll call path + tests` |
| `toLogString` | `toLogString` | `toLogString` | `android/app/src/main/java/com/scmessenger/android/transport/NetworkDetector.kt` | 334 | 0 | `WIRE toLogString call path + tests` |
| `updateHeartbeat` | `updateHeartbeat` | `updateHeartbeat` | `android/app/src/main/java/com/scmessenger/android/service/ServiceHealthMonitor.kt` | 328 | 0 | `WIRE updateHeartbeat call path + tests` |

## B6-wasm

| Task | Function | Resolved symbol | Target | Definition line | External refs | Patch template |
|---|---|---|---|---:|---:|---|
| `add_rtc_connection` | `add_rtc_connection` | `add_rtc_connection` | `wasm/src/connection_state.rs` | 278 | 0 | `WIRE add_rtc_connection call path + tests` |
| `add_websocket` | `add_websocket` | `add_websocket` | `wasm/src/connection_state.rs` | 257 | 0 | `WIRE add_websocket call path + tests` |
| `close_all_notifications` | `close_all_notifications` | `close_all_notifications` | `wasm/src/notification_manager.rs` | 308 | 0 | `WIRE close_all_notifications call path + tests` |
| `detect_browser` | `detect_browser` | `detect_browser` | `wasm/src/notification_manager.rs` | 323 | 0 | `WIRE detect_browser call path + tests` |
| `drain_received_messages` | `drain_received_messages` | `drain_received_messages` | `wasm/src/lib.rs` | 922 | 0 | `WIRE drain_received_messages call path + tests` |
| `get_browser_options` | `get_browser_options` | `get_browser_options` | `wasm/src/notification_manager.rs` | 329 | 0 | `WIRE get_browser_options call path + tests` |
| `get_contact_manager` | `get_contact_manager` | `get_contact_manager` | `wasm/src/lib.rs` | 1097 | 0 | `WIRE get_contact_manager call path + tests` |
| `get_daemon_socket_url` | `get_daemon_socket_url` | `get_daemon_socket_url` | `wasm/src/lib.rs` | 350 | 0 | `WIRE get_daemon_socket_url call path + tests` |
| `get_history_manager` | `get_history_manager` | `get_history_manager` | `wasm/src/lib.rs` | 1104 | 0 | `WIRE get_history_manager call path + tests` |
| `get_identity_from_daemon` | `get_identity_from_daemon` | `get_identity_from_daemon` | `wasm/src/lib.rs` | 421 | 0 | `WIRE get_identity_from_daemon call path + tests` |
| `get_identity_wire_shape` | `get_identity_wire_shape` | `get_identity_wire_shape` | `wasm/src/daemon_bridge.rs` | 36 | 0 | `WIRE get_identity_wire_shape call path + tests` |
| `get_iron_core_mode` | `get_iron_core_mode` | `get_iron_core_mode` | `wasm/src/lib.rs` | 336 | 0 | `WIRE get_iron_core_mode call path + tests` |
| `get_permission` | `get_permission` | `get_permission` | `wasm/src/notification_manager.rs` | 317 | 0 | `WIRE get_permission call path + tests` |
| `get_settings` | `get_settings` | `get_settings` | `wasm/src/lib.rs` | 940 | 31 | `WIRE get_settings call path + tests` |
| `initialize_identity_from_daemon` | `initialize_identity_from_daemon` | `initialize_identity_from_daemon` | `wasm/src/lib.rs` | 389 | 0 | `WIRE initialize_identity_from_daemon call path + tests` |
| `is_permission_granted` | `is_permission_granted` | `is_permission_granted` | `wasm/src/notification_manager.rs` | 231 | 0 | `WIRE is_permission_granted call path + tests` |
| `notification_roundtrip_for_ui_state` | `notification_roundtrip_for_ui_state` | `notification_roundtrip_for_ui_state` | `wasm/src/daemon_bridge.rs` | 45 | 0 | `WIRE notification_roundtrip_for_ui_state call path + tests` |
| `parse_response` | `parse_response` | `parse_response` | `wasm/src/daemon_bridge.rs` | 21 | 7 | `WIRE parse_response call path + tests` |
| `remove_rtc_connection` | `remove_rtc_connection` | `remove_rtc_connection` | `wasm/src/connection_state.rs` | 286 | 0 | `WIRE remove_rtc_connection call path + tests` |
| `remove_websocket` | `remove_websocket` | `remove_websocket` | `wasm/src/connection_state.rs` | 265 | 0 | `WIRE remove_websocket call path + tests` |
| `request_permission` | `request_permission` | `request_permission` | `wasm/src/notification_manager.rs` | 110 | 0 | `WIRE request_permission call path + tests` |
| `send_prepared_envelope` | `send_prepared_envelope` | `send_prepared_envelope` | `wasm/src/lib.rs` | 565 | 0 | `WIRE send_prepared_envelope call path + tests` |
| `set_daemon_socket_url` | `set_daemon_socket_url` | `set_daemon_socket_url` | `wasm/src/lib.rs` | 344 | 0 | `WIRE set_daemon_socket_url call path + tests` |
| `set_iron_core_mode` | `set_iron_core_mode` | `set_iron_core_mode` | `wasm/src/lib.rs` | 327 | 0 | `WIRE set_iron_core_mode call path + tests` |
| `show_permission_guidance` | `show_permission_guidance` | `show_permission_guidance` | `wasm/src/notification_manager.rs` | 355 | 0 | `WIRE show_permission_guidance call path + tests` |
| `start_receive_loop` | `start_receive_loop` | `start_receive_loop` | `wasm/src/lib.rs` | 896 | 0 | `WIRE start_receive_loop call path + tests` |
| `stop_swarm` | `stop_swarm` | `stop_swarm` | `wasm/src/lib.rs` | 552 | 0 | `WIRE stop_swarm call path + tests` |
| `update_settings` | `update_settings` | `update_settings` | `wasm/src/lib.rs` | 948 | 0 | `WIRE update_settings call path + tests` |
| `validate_settings` | `validate_settings` | `validate_settings` | `wasm/src/lib.rs` | 870 | 0 | `WIRE validate_settings call path + tests` |

## B7-cli

| Task | Function | Resolved symbol | Target | Definition line | External refs | Patch template |
|---|---|---|---|---:|---:|---|
| `advertise_service` | `advertise_service` | `advertise_service` | `cli/src/ble_daemon.rs` | 253 | 0 | `WIRE advertise_service call path + tests` |
| `can_forward_for_wasm` | `can_forward_for_wasm` | `can_forward_for_wasm` | `cli/src/transport_bridge.rs` | 246 | 0 | `WIRE can_forward_for_wasm call path + tests` |
| `can_reach_destination` | `can_reach_destination` | `can_reach_destination` | `cli/src/transport_bridge.rs` | 264 | 0 | `WIRE can_reach_destination call path + tests` |
| `count_with_peer` | `count_with_peer` | `count_with_peer` | `cli/src/history.rs` | 182 | 0 | `WIRE count_with_peer call path + tests` |
| `decode_rejects_short_buffer` | `decode_rejects_short_buffer` | `decode_rejects_short_buffer` | `cli/src/ble_mesh.rs` | 245 | 0 | `WIRE decode_rejects_short_buffer call path + tests` |
| `find_by_nickname` | `find_by_nickname` | `find_by_nickname` | `cli/src/contacts.rs` | 119 | 0 | `WIRE find_by_nickname call path + tests` |
| `find_by_public_key` | `find_by_public_key` | `find_by_public_key` | `cli/src/contacts.rs` | 131 | 0 | `WIRE find_by_public_key call path + tests` |
| `formatted_time` | `formatted_time` | `formatted_time` | `cli/src/history.rs` | 66 | 0 | `WIRE formatted_time call path + tests` |
| `get_available_paths` | `get_available_paths` | `get_available_paths` | `cli/src/transport_bridge.rs` | 189 | 0 | `WIRE get_available_paths call path + tests` |
| `get_best_forwarding_path` | `get_best_forwarding_path` | `get_best_forwarding_path` | `cli/src/transport_bridge.rs` | 279 | 0 | `WIRE get_best_forwarding_path call path + tests` |
| `get_forwarding_capability` | `get_forwarding_capability` | `get_forwarding_capability` | `cli/src/transport_bridge.rs` | 252 | 0 | `WIRE get_forwarding_capability call path + tests` |
| `get_history_via_api` | `get_history_via_api` | `get_history_via_api` | `cli/src/api.rs` | 181 | 0 | `WIRE get_history_via_api call path + tests` |
| `is_ble_available` | `is_ble_available` | `is_ble_available` | `cli/src/ble_daemon.rs` | 286 | 0 | `WIRE is_ble_available call path + tests` |
| `scan_for_advertisements` | `scan_for_advertisements` | `scan_for_advertisements` | `cli/src/ble_daemon.rs` | 229 | 0 | `WIRE scan_for_advertisements call path + tests` |
| `set_notes` | `set_notes` | `set_notes` | `cli/src/contacts.rs` | 165 | 0 | `WIRE set_notes call path + tests` |
| `try_enable_bluetooth` | `try_enable_bluetooth` | `try_enable_bluetooth` | `cli/src/ble_daemon.rs` | 331 | 0 | `WIRE try_enable_bluetooth call path + tests` |

## B8-cross-cutting

| Task | Function | Resolved symbol | Target | Definition line | External refs | Patch template |
|---|---|---|---|---:|---:|---|
| `add_step` | `add_step` | `add_step` | `core/src/dspy/modules.rs` | 84 | 3 | `WIRE add_step call path + tests` |
| `apply_policy_config` | `apply_policy_config` | `apply_policy_config` | `core/src/drift/relay.rs` | 366 | 0 | `WIRE apply_policy_config call path + tests` |
| `blake3_hash` | `blake3_hash` | `blake3_hash` | `core/src/dspy/signatures.rs` | 136 | 0 | `WIRE blake3_hash call path + tests` |
| `blocked_only_peer_ids` | `blocked_only_peer_ids` | `blocked_only_peer_ids` | `core/src/store/blocked.rs` | 230 | 0 | `WIRE blocked_only_peer_ids call path + tests` |
| `build_optimization_pipeline` | `build_optimization_pipeline` | `build_optimization_pipeline` | `core/src/dspy/teleprompt.rs` | 241 | 0 | `WIRE build_optimization_pipeline call path + tests` |
| `build_security_audit_pipeline` | `build_security_audit_pipeline` | `build_security_audit_pipeline` | `core/src/dspy/modules.rs` | 240 | 0 | `WIRE build_security_audit_pipeline call path + tests` |
| `buildForegroundServiceNotification` | `buildForegroundServiceNotification` | `buildForegroundServiceNotification` | `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt` | 175 | 0 | `WIRE buildForegroundServiceNotification call path + tests` |
| `chain_ratchet_produces_distinct_keys` | `chain_ratchet_produces_distinct_keys` | `chain_ratchet_produces_distinct_keys` | `core/src/crypto/kani_proofs.rs` | 51 | 0 | `WIRE chain_ratchet_produces_distinct_keys call path + tests` |
| `clearAllRequestNotifications` | `clearAllRequestNotifications` | `clearAllRequestNotifications` | `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt` | 492 | 0 | `WIRE clearAllRequestNotifications call path + tests` |
| `clearMessageNotifications` | `clearMessageNotifications` | `clearMessageNotifications` | `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt` | 479 | 0 | `WIRE clearMessageNotifications call path + tests` |
| `contact_new_has_no_last_known_device_id` | `contact_new_has_no_last_known_device_id` | `contact_new_has_no_last_known_device_id` | `core/src/store/contacts.rs` | 319 | 0 | `WIRE contact_new_has_no_last_known_device_id call path + tests` |
| `contact_roundtrips_through_serde_with_default_device_id` | `contact_roundtrips_through_serde_with_default_device_id` | `contact_roundtrips_through_serde_with_default_device_id` | `core/src/store/contacts.rs` | 358 | 0 | `WIRE contact_roundtrips_through_serde_with_default_device_id call path + tests` |
| `create_basic` | `create_basic` | `create_basic` | `core/src/dspy/teleprompt.rs` | 208 | 0 | `WIRE create_basic call path + tests` |
| `create_cot` | `create_cot` | `create_cot` | `core/src/dspy/modules.rs` | 212 | 0 | `WIRE create_cot call path + tests` |
| `create_multihop` | `create_multihop` | `create_multihop` | `core/src/dspy/modules.rs` | 216 | 0 | `WIRE create_multihop call path + tests` |
| `create_optimizer` | `create_optimizer` | `create_optimizer` | `core/src/dspy/modules.rs` | 220 | 0 | `WIRE create_optimizer call path + tests` |
| `create_receiver_session` | `create_receiver_session` | `create_receiver_session` | `core/src/crypto/session_manager.rs` | 76 | 0 | `WIRE create_receiver_session call path + tests` |
| `derive_key_always_32_bytes` | `derive_key_always_32_bytes` | `derive_key_always_32_bytes` | `core/src/crypto/kani_proofs.rs` | 37 | 0 | `WIRE derive_key_always_32_bytes call path + tests` |
| `disable_location_background` | `disable_location_background` | `disable_location_background` | `core/src/mobile/ios_strategy.rs` | 260 | 0 | `WIRE disable_location_background call path + tests` |
| `disabled_notifications_suppress_delivery` | `disabled_notifications_suppress_delivery` | `disabled_notifications_suppress_delivery` | `core/src/notification.rs` | 447 | 0 | `WIRE disabled_notifications_suppress_delivery call path + tests` |
| `duplicates_are_suppressed` | `duplicates_are_suppressed` | `duplicates_are_suppressed` | `core/src/notification.rs` | 462 | 0 | `WIRE duplicates_are_suppressed call path + tests` |
| `ed25519_conversion_produces_32_bytes` | `ed25519_conversion_produces_32_bytes` | `ed25519_conversion_produces_32_bytes` | `core/src/crypto/kani_proofs.rs` | 28 | 0 | `WIRE ed25519_conversion_produces_32_bytes call path + tests` |
| `emergency_recover` | `emergency_recover` | `emergency_recover` | `core/src/contacts_bridge.rs` | 279 | 0 | `WIRE emergency_recover call path + tests` |
| `encrypt_xchacha20` | `encrypt_xchacha20` | `encrypt_xchacha20` | `core/src/dspy/signatures.rs` | 110 | 0 | `WIRE encrypt_xchacha20 call path + tests` |
| `evaluate_all_tracked` | `evaluate_all_tracked` | `evaluate_all_tracked` | `core/src/abuse/auto_block.rs` | 226 | 0 | `WIRE evaluate_all_tracked call path + tests` |
| `explicit_request_overrides_known_contact_inference` | `explicit_request_overrides_known_contact_inference` | `explicit_request_overrides_known_contact_inference` | `core/src/notification.rs` | 437 | 0 | `WIRE explicit_request_overrides_known_contact_inference call path + tests` |
| `federated_nickname` | `federated_nickname` | `federated_nickname` | `core/src/contacts_bridge.rs` | 53 | 0 | `WIRE federated_nickname call path + tests` |
| `force_ratchet` | `force_ratchet` | `force_ratchet` | `core/src/crypto/ratchet.rs` | 420 | 0 | `WIRE force_ratchet call path + tests` |
| `foreground_direct_messages_follow_foreground_toggle` | `foreground_direct_messages_follow_foreground_toggle` | `foreground_direct_messages_follow_foreground_toggle` | `core/src/notification.rs` | 475 | 0 | `WIRE foreground_direct_messages_follow_foreground_toggle call path + tests` |
| `formatReportForUser` | `formatReportForUser` | `formatReportForUser` | `android/app/src/main/java/com/scmessenger/android/network/DiagnosticsReporter.kt` | 108 | 0 | `WIRE formatReportForUser call path + tests` |
| `generate_cover_traffic_if_due` | `generate_cover_traffic_if_due` | `generate_cover_traffic_if_due` | `core/src/drift/relay.rs` | 179 | 0 | `WIRE generate_cover_traffic_if_due call path + tests` |
| `get_last_profile` | `get_last_profile` | `get_last_profile` | `core/src/mobile/auto_adjust.rs` | 268 | 0 | `WIRE get_last_profile call path + tests` |
| `get_overrides` | `get_overrides` | `get_overrides` | `core/src/mobile/auto_adjust.rs` | 263 | 0 | `WIRE get_overrides call path + tests` |
| `get_signable_data` | `get_signable_data` | `get_signable_data` | `core/src/relay/invite.rs` | 90 | 0 | `WIRE get_signable_data call path + tests` |
| `get_signature` | `get_signature` | `get_signature` | `core/src/dspy/signatures.rs` | 156 | 3 | `WIRE get_signature call path + tests` |
| `getHealthyRelays` | `getHealthyRelays` | `getHealthyRelays` | `android/app/src/main/java/com/scmessenger/android/utils/CircuitBreaker.kt` | 199 | 0 | `WIRE getHealthyRelays call path + tests` |
| `getLastFailure` | `getLastFailure` | `getLastFailureReason` | `android/app/src/main/java/com/scmessenger/android/utils/CircuitBreaker.kt` | 178 | 1 | `WIRE getLastFailure call path + tests` |
| `getLastFailureReason` | `getLastFailureReason` | `getLastFailureReason` | `android/app/src/main/java/com/scmessenger/android/utils/CircuitBreaker.kt` | 178 | 1 | `WIRE getLastFailureReason call path + tests` |
| `getNotificationStats` | `getNotificationStats` | `getNotificationStats` | `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt` | 620 | 0 | `WIRE getNotificationStats call path + tests` |
| `getOpenCircuits` | `getOpenCircuits` | `getOpenCircuits` | `android/app/src/main/java/com/scmessenger/android/utils/CircuitBreaker.kt` | 194 | 0 | `WIRE getOpenCircuits call path + tests` |
| `hasDnsFailures` | `hasDnsFailures` | `hasDnsFailures` | `android/app/src/main/java/com/scmessenger/android/utils/NetworkFailureMetrics.kt` | 61 | 0 | `WIRE hasDnsFailures call path + tests` |
| `hasPortBlocking` | `hasPortBlocking` | `hasPortBlocking` | `android/app/src/main/java/com/scmessenger/android/utils/NetworkFailureMetrics.kt` | 63 | 0 | `WIRE hasPortBlocking call path + tests` |
| `isAtMaxDelay` | `isAtMaxDelay` | `isAtMaxDelay` | `android/app/src/main/java/com/scmessenger/android/utils/BackoffStrategy.kt` | 79 | 0 | `WIRE isAtMaxDelay call path + tests` |
| `isStorageStateCritical` | `isStorageStateCritical` | `isStorageStateCritical` | `android/app/src/main/java/com/scmessenger/android/utils/StorageManager.kt` | 159 | 0 | `WIRE isStorageStateCritical call path + tests` |
| `jsonrpc_get_identity` | `jsonrpc_get_identity` | `jsonrpc_get_identity` | `core/src/wasm_support/rpc.rs` | 250 | 0 | `WIRE jsonrpc_get_identity call path + tests` |
| `jsonrpc_send_message_roundtrip` | `jsonrpc_send_message_roundtrip` | `jsonrpc_send_message_roundtrip` | `core/src/wasm_support/rpc.rs` | 224 | 0 | `WIRE jsonrpc_send_message_roundtrip call path + tests` |
| `known_contact_defaults_to_direct_message` | `known_contact_defaults_to_direct_message` | `known_contact_defaults_to_direct_message` | `core/src/notification.rs` | 428 | 0 | `WIRE known_contact_defaults_to_direct_message call path + tests` |
| `list_endpoints` | `list_endpoints` | `list_endpoints` | `core/src/notification.rs` | 357 | 0 | `WIRE list_endpoints call path + tests` |
| `new_sync` | `new_sync` | `new_sync` | `core/src/store/backend.rs` | 188 | 0 | `WIRE new_sync call path + tests` |
| `nonce_length_invariant` | `nonce_length_invariant` | `nonce_length_invariant` | `core/src/crypto/kani_proofs.rs` | 44 | 0 | `WIRE nonce_length_invariant call path + tests` |
| `notif_mesh_topology` | `notif_mesh_topology` | `notif_mesh_topology` | `core/src/wasm_support/rpc.rs` | 205 | 0 | `WIRE notif_mesh_topology call path + tests` |
| `notification_serialization` | `notification_serialization` | `notification_serialization` | `core/src/wasm_support/rpc.rs` | 264 | 0 | `WIRE notification_serialization call path + tests` |
| `overall_score` | `overall_score` | `overall_score` | `core/src/abuse/reputation.rs` | 169 | 28 | `WIRE overall_score call path + tests` |
| `override_ble_advertise_interval` | `override_ble_advertise_interval` | `override_ble_advertise_interval` | `core/src/mobile/auto_adjust.rs` | 243 | 0 | `WIRE override_ble_advertise_interval call path + tests` |
| `override_relay_priority_threshold` | `override_relay_priority_threshold` | `override_relay_priority_threshold` | `core/src/mobile/auto_adjust.rs` | 253 | 0 | `WIRE override_relay_priority_threshold call path + tests` |
| `proptest_different_ciphertexts_same_plaintext` | `proptest_different_ciphertexts_same_plaintext` | `proptest_different_ciphertexts_same_plaintext` | `core/src/crypto/proptest_harness.rs` | 57 | 0 | `WIRE proptest_different_ciphertexts_same_plaintext call path + tests` |
| `proptest_encrypt_decrypt_roundtrip` | `proptest_encrypt_decrypt_roundtrip` | `proptest_encrypt_decrypt_roundtrip` | `core/src/crypto/proptest_harness.rs` | 41 | 0 | `WIRE proptest_encrypt_decrypt_roundtrip call path + tests` |
| `proptest_envelope_field_lengths` | `proptest_envelope_field_lengths` | `proptest_envelope_field_lengths` | `core/src/crypto/proptest_harness.rs` | 90 | 0 | `WIRE proptest_envelope_field_lengths call path + tests` |
| `proptest_ratchet_forward_secrecy` | `proptest_ratchet_forward_secrecy` | `proptest_ratchet_forward_secrecy` | `core/src/crypto/proptest_harness.rs` | 152 | 0 | `WIRE proptest_ratchet_forward_secrecy call path + tests` |
| `proptest_ratchet_roundtrip` | `proptest_ratchet_roundtrip` | `proptest_ratchet_roundtrip` | `core/src/crypto/proptest_harness.rs` | 118 | 0 | `WIRE proptest_ratchet_roundtrip call path + tests` |
| `proptest_sign_verify_roundtrip` | `proptest_sign_verify_roundtrip` | `proptest_sign_verify_roundtrip` | `core/src/crypto/proptest_harness.rs` | 104 | 0 | `WIRE proptest_sign_verify_roundtrip call path + tests` |
| `proptest_wrong_key_fails` | `proptest_wrong_key_fails` | `proptest_wrong_key_fails` | `core/src/crypto/proptest_harness.rs` | 75 | 0 | `WIRE proptest_wrong_key_fails call path + tests` |
| `provideMeshRepository` | `provideMeshRepository` | `provideMeshRepository` | `android/app/src/main/java/com/scmessenger/android/di/AppModule.kt` | 26 | 0 | `WIRE provideMeshRepository call path + tests` |
| `providePreferencesRepository` | `providePreferencesRepository` | `providePreferencesRepository` | `android/app/src/main/java/com/scmessenger/android/di/AppModule.kt` | 34 | 0 | `WIRE providePreferencesRepository call path + tests` |
| `read_with_timeout` | `read_with_timeout` | `read_with_timeout` | `core/src/drift/frame.rs` | 183 | 0 | `WIRE read_with_timeout call path + tests` |
| `refresh_delegate_routes` | `refresh_delegate_routes` | `refresh_delegate_routes` | `core/src/relay/delegate_prewarm.rs` | 215 | 0 | `WIRE refresh_delegate_routes call path + tests` |
| `register_endpoint` | `register_endpoint` | `register_endpoint` | `core/src/notification.rs` | 304 | 0 | `WIRE register_endpoint call path + tests` |
| `resetNotificationStats` | `resetNotificationStats` | `resetNotificationStats` | `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt` | 627 | 0 | `WIRE resetNotificationStats call path + tests` |
| `run_optimization` | `run_optimization` | `run_optimization` | `core/src/dspy/modules.rs` | 179 | 0 | `WIRE run_optimization call path + tests` |
| `set_cover_traffic` | `set_cover_traffic` | `set_cover_traffic` | `core/src/drift/relay.rs` | 156 | 0 | `WIRE set_cover_traffic call path + tests` |
| `set_reputation_manager` | `set_reputation_manager` | `set_reputation_manager` | `core/src/drift/relay.rs` | 170 | 0 | `WIRE set_reputation_manager call path + tests` |
| `showMeshStatusNotification` | `showMeshStatusNotification` | `showMeshStatusNotification` | `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt` | 533 | 0 | `WIRE showMeshStatusNotification call path + tests` |
| `showPeerDiscoveredNotification` | `showPeerDiscoveredNotification` | `showPeerDiscoveredNotification` | `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt` | 501 | 0 | `WIRE showPeerDiscoveredNotification call path + tests` |
| `touch_endpoint` | `touch_endpoint` | `touch_endpoint` | `core/src/notification.rs` | 365 | 0 | `WIRE touch_endpoint call path + tests` |
| `unknown_method_error` | `unknown_method_error` | `unknown_method_error` | `core/src/wasm_support/rpc.rs` | 278 | 0 | `WIRE unknown_method_error call path + tests` |
| `unknown_sender_defaults_to_direct_message_request` | `unknown_sender_defaults_to_direct_message_request` | `unknown_sender_defaults_to_direct_message_request` | `core/src/notification.rs` | 418 | 0 | `WIRE unknown_sender_defaults_to_direct_message_request call path + tests` |
| `unregister_endpoint` | `unregister_endpoint` | `unregister_endpoint` | `core/src/notification.rs` | 345 | 0 | `WIRE unregister_endpoint call path + tests` |
| `update_keepalive` | `update_keepalive` | `update_keepalive` | `core/src/relay/delegate_prewarm.rs` | 100 | 0 | `WIRE update_keepalive call path + tests` |
| `update_last_known_device_id_can_clear` | `update_last_known_device_id_can_clear` | `update_last_known_device_id_can_clear` | `core/src/store/contacts.rs` | 344 | 0 | `WIRE update_last_known_device_id_can_clear call path + tests` |
| `update_last_known_device_id_ignores_invalid_values` | `update_last_known_device_id_ignores_invalid_values` | `update_last_known_device_id_ignores_invalid_values` | `core/src/store/contacts.rs` | 388 | 0 | `WIRE update_last_known_device_id_ignores_invalid_values call path + tests` |
| `update_last_known_device_id_persists_and_is_readable` | `update_last_known_device_id_persists_and_is_readable` | `update_last_known_device_id_persists_and_is_readable` | `core/src/store/contacts.rs` | 325 | 0 | `WIRE update_last_known_device_id_persists_and_is_readable call path + tests` |
| `update_last_known_device_id_trims_valid_uuid` | `update_last_known_device_id_trims_valid_uuid` | `update_last_known_device_id_trims_valid_uuid` | `core/src/store/contacts.rs` | 369 | 0 | `WIRE update_last_known_device_id_trims_valid_uuid call path + tests` |

