# Wiring Patch Manifest (Pre-Implementation)

This file provides exact edit coordinates and patch templates for each wiring task without implementing runtime logic.

Total tasks: **350**

## B1-core-entrypoints

| Task | Function | Target | Definition line | External refs | Patch template |
|---|---|---|---:|---:|---|
| `annotate_identity` | `annotate_identity` | `core/src/mobile_bridge.rs` | 1665 | 12 | `WIRE annotate_identity call path + tests` |
| `compute_ble_adjustment` | `compute_ble_adjustment` | `core/src/mobile_bridge.rs` | 1029 | 11 | `WIRE compute_ble_adjustment call path + tests` |
| `compute_relay_adjustment` | `compute_relay_adjustment` | `core/src/mobile_bridge.rs` | 1049 | 11 | `WIRE compute_relay_adjustment call path + tests` |
| `default_settings` | `default_settings` | `core/src/mobile_bridge.rs` | 1210 | 11 | `WIRE default_settings call path + tests` |
| `drift_activate` | `drift_activate` | `core/src/lib.rs` | 2485 | 11 | `WIRE drift_activate call path + tests` |
| `drift_deactivate` | `drift_deactivate` | `core/src/lib.rs` | 2493 | 11 | `WIRE drift_deactivate call path + tests` |
| `drift_network_state` | `drift_network_state` | `core/src/lib.rs` | 2501 | 11 | `WIRE drift_network_state call path + tests` |
| `drift_store_size` | `drift_store_size` | `core/src/lib.rs` | 2509 | 11 | `WIRE drift_store_size call path + tests` |
| `export_audit_log` | `export_audit_log` | `core/src/lib.rs` | 2466 | 11 | `WIRE export_audit_log call path + tests` |
| `get_audit_events_since` | `get_audit_events_since` | `core/src/lib.rs` | 2455 | 11 | `WIRE get_audit_events_since call path + tests` |
| `get_audit_log` | `get_audit_log` | `core/src/lib.rs` | 2436 | 11 | `WIRE get_audit_log call path + tests` |
| `get_enhanced_peer_reputation` | `get_enhanced_peer_reputation` | `core/src/lib.rs` | 2558 | 10 | `WIRE get_enhanced_peer_reputation call path + tests` |
| `get_peer_reputation` | `get_peer_reputation` | `core/src/lib.rs` | 2552 | 11 | `WIRE get_peer_reputation call path + tests` |
| `get_privacy_config` | `get_privacy_config` | `core/src/lib.rs` | 1649 | 13 | `WIRE get_privacy_config call path + tests` |
| `get_swarm_bridge` | `get_swarm_bridge` | `core/src/mobile_bridge.rs` | 709 | 13 | `WIRE get_swarm_bridge call path + tests` |
| `on_battery_changed` | `on_battery_changed` | `core/src/mobile_bridge.rs` | 951 | 12 | `WIRE on_battery_changed call path + tests` |
| `on_ble_data_received` | `on_ble_data_received` | `core/src/mobile_bridge.rs` | 954 | 12 | `WIRE on_ble_data_received call path + tests` |
| `on_entering_background` | `on_entering_background` | `core/src/mobile_bridge.rs` | 955 | 11 | `WIRE on_entering_background call path + tests` |
| `on_entering_foreground` | `on_entering_foreground` | `core/src/mobile_bridge.rs` | 956 | 11 | `WIRE on_entering_foreground call path + tests` |
| `on_motion_changed` | `on_motion_changed` | `core/src/mobile_bridge.rs` | 953 | 12 | `WIRE on_motion_changed call path + tests` |
| `on_network_changed` | `on_network_changed` | `core/src/mobile_bridge.rs` | 952 | 12 | `WIRE on_network_changed call path + tests` |
| `peel_onion_layer` | `peel_onion_layer` | `core/src/lib.rs` | 1759 | 13 | `WIRE peel_onion_layer call path + tests` |
| `peer_rate_limit_multiplier` | `peer_rate_limit_multiplier` | `core/src/lib.rs` | 2565 | 11 | `WIRE peer_rate_limit_multiplier call path + tests` |
| `peer_spam_score` | `peer_spam_score` | `core/src/lib.rs` | 2570 | 10 | `WIRE peer_spam_score call path + tests` |
| `prepare_onion_message` | `prepare_onion_message` | `core/src/lib.rs` | 1684 | 13 | `WIRE prepare_onion_message call path + tests` |
| `random_port` | `random_port` | `core/src/lib.rs` | 3333 | 10 | `WIRE random_port call path + tests` |
| `ratchet_has_session` | `ratchet_has_session` | `core/src/lib.rs` | 2129 | 11 | `WIRE ratchet_has_session call path + tests` |
| `ratchet_reset_session` | `ratchet_reset_session` | `core/src/lib.rs` | 2136 | 11 | `WIRE ratchet_reset_session call path + tests` |
| `ratchet_session_count` | `ratchet_session_count` | `core/src/lib.rs` | 2124 | 11 | `WIRE ratchet_session_count call path + tests` |
| `relay_jitter_delay` | `relay_jitter_delay` | `core/src/lib.rs` | 1779 | 11 | `WIRE relay_jitter_delay call path + tests` |
| `routing_tick` | `routing_tick` | `core/src/lib.rs` | 2683 | 11 | `WIRE routing_tick call path + tests` |
| `send_ble_packet` | `send_ble_packet` | `core/src/mobile_bridge.rs` | 957 | 11 | `WIRE send_ble_packet call path + tests` |
| `send_message_status` | `send_message_status` | `core/src/mobile_bridge.rs` | 1854 | 14 | `WIRE send_message_status call path + tests` |
| `set_delegate` | `set_delegate` | `core/src/lib.rs` | 2145 | 14 | `WIRE set_delegate call path + tests` |
| `set_privacy_config` | `set_privacy_config` | `core/src/lib.rs` | 1656 | 13 | `WIRE set_privacy_config call path + tests` |
| `validate_audit_chain` | `validate_audit_chain` | `core/src/lib.rs` | 2473 | 11 | `WIRE validate_audit_chain call path + tests` |

## B2-core-transport-routing

| Task | Function | Target | Definition line | External refs | Patch template |
|---|---|---|---:|---:|---|
| `abusive_peer_burst_is_rate_limited_but_other_peer_still_passes` | `abusive_peer_burst_is_rate_limited_but_other_peer_still_passes` | `core/src/transport/swarm.rs` | 4383 | 10 | `WIRE abusive_peer_burst_is_rate_limited_but_other_peer_still_passes call path + tests` |
| `active_paths` | `active_paths` | `core/src/routing/multipath.rs` | 53 | 12 | `WIRE active_paths call path + tests` |
| `add_discovered_peer` | `add_discovered_peer` | `core/src/transport/wifi_aware.rs` | 201 | 10 | `WIRE add_discovered_peer call path + tests` |
| `add_kad_address` | `add_kad_address` | `core/src/transport/swarm.rs` | 1285 | 10 | `WIRE add_kad_address call path + tests` |
| `all_connections` | `all_connections` | `core/src/transport/observation.rs` | 198 | 10 | `WIRE all_connections call path + tests` |
| `audit_count` | `audit_count` | `core/src/store/relay_custody.rs` | 723 | 10 | `WIRE audit_count call path + tests` |
| `best_relays` | `best_relays` | `core/src/transport/mesh_routing.rs` | 214 | 11 | `WIRE best_relays call path + tests` |
| `calculate_dynamic_ttl` | `calculate_dynamic_ttl` | `core/src/routing/adaptive_ttl.rs` | 150 | 12 | `WIRE calculate_dynamic_ttl call path + tests` |
| `can_bootstrap_others` | `can_bootstrap_others` | `core/src/transport/mesh_routing.rs` | 610 | 12 | `WIRE can_bootstrap_others call path + tests` |
| `cheap_heuristics_reject_invalid_payload_shapes` | `cheap_heuristics_reject_invalid_payload_shapes` | `core/src/transport/swarm.rs` | 4448 | 10 | `WIRE cheap_heuristics_reject_invalid_payload_shapes call path + tests` |
| `cleanup_stale_connections` | `cleanup_stale_connections` | `core/src/transport/health.rs` | 409 | 10 | `WIRE cleanup_stale_connections call path + tests` |
| `clear_unreachable_peer` | `clear_unreachable_peer` | `core/src/routing/optimized_engine.rs` | 255 | 10 | `WIRE clear_unreachable_peer call path + tests` |
| `converge_delivered_for_message_removes_matching_pending_records` | `converge_delivered_for_message_removes_matching_pending_records` | `core/src/store/relay_custody.rs` | 1760 | 10 | `WIRE converge_delivered_for_message_removes_matching_pending_records call path + tests` |
| `convergence_marker_accepts_when_custody_exists_locally` | `convergence_marker_accepts_when_custody_exists_locally` | `core/src/transport/swarm.rs` | 4507 | 10 | `WIRE convergence_marker_accepts_when_custody_exists_locally call path + tests` |
| `convergence_marker_rejects_invalid_shape` | `convergence_marker_rejects_invalid_shape` | `core/src/transport/swarm.rs` | 4464 | 10 | `WIRE convergence_marker_rejects_invalid_shape call path + tests` |
| `convergence_marker_requires_local_tracking_context` | `convergence_marker_requires_local_tracking_context` | `core/src/transport/swarm.rs` | 4478 | 10 | `WIRE convergence_marker_requires_local_tracking_context call path + tests` |
| `current_discovery_phase` | `current_discovery_phase` | `core/src/routing/optimized_engine.rs` | 169 | 10 | `WIRE current_discovery_phase call path + tests` |
| `custody_audit_persists_across_restart` | `custody_audit_persists_across_restart` | `core/src/store/relay_custody.rs` | 2076 | 10 | `WIRE custody_audit_persists_across_restart call path + tests` |
| `custody_deduplicates_same_destination_and_message_id` | `custody_deduplicates_same_destination_and_message_id` | `core/src/store/relay_custody.rs` | 1729 | 10 | `WIRE custody_deduplicates_same_destination_and_message_id call path + tests` |
| `custody_transitions_are_recorded` | `custody_transitions_are_recorded` | `core/src/store/relay_custody.rs` | 1686 | 10 | `WIRE custody_transitions_are_recorded call path + tests` |
| `duplicate_window_suppresses_immediate_replay_then_expires` | `duplicate_window_suppresses_immediate_replay_then_expires` | `core/src/transport/swarm.rs` | 4411 | 10 | `WIRE duplicate_window_suppresses_immediate_replay_then_expires call path + tests` |
| `expire_old_observations` | `expire_old_observations` | `core/src/transport/observation.rs` | 96 | 10 | `WIRE expire_old_observations call path + tests` |
| `for_local_peer_prefers_explicit_custody_dir_override` | `for_local_peer_prefers_explicit_custody_dir_override` | `core/src/store/relay_custody.rs` | 2133 | 10 | `WIRE for_local_peer_prefers_explicit_custody_dir_override call path + tests` |
| `force_state_for_test` | `force_state_for_test` | `core/src/store/relay_custody.rs` | 1409 | 11 | `WIRE force_state_for_test call path + tests` |
| `get_activity` | `get_activity` | `core/src/routing/adaptive_ttl.rs` | 125 | 10 | `WIRE get_activity call path + tests` |
| `get_all_connection_stats` | `get_all_connection_stats` | `core/src/transport/health.rs` | 370 | 10 | `WIRE get_all_connection_stats call path + tests` |
| `get_all_relay_stats` | `get_all_relay_stats` | `core/src/transport/internet.rs` | 427 | 10 | `WIRE get_all_relay_stats call path + tests` |
| `get_bootstrap_candidates` | `get_bootstrap_candidates` | `core/src/transport/mesh_routing.rs` | 605 | 11 | `WIRE get_bootstrap_candidates call path + tests` |
| `get_fallback_relays` | `get_fallback_relays` | `core/src/transport/relay_health.rs` | 153 | 10 | `WIRE get_fallback_relays call path + tests` |
| `get_healthy_connections` | `get_healthy_connections` | `core/src/transport/health.rs` | 382 | 10 | `WIRE get_healthy_connections call path + tests` |
| `get_healthy_relays` | `get_healthy_relays` | `core/src/transport/circuit_breaker.rs` | 270 | 10 | `WIRE get_healthy_relays call path + tests` |
| `get_hole_punch_status` | `get_hole_punch_status` | `core/src/transport/nat.rs` | 495 | 10 | `WIRE get_hole_punch_status call path + tests` |
| `get_registration_state_info` | `get_registration_state_info` | `core/src/store/relay_custody.rs` | 471 | 10 | `WIRE get_registration_state_info call path + tests` |
| `get_unhealthy_connections` | `get_unhealthy_connections` | `core/src/transport/health.rs` | 391 | 10 | `WIRE get_unhealthy_connections call path + tests` |
| `is_prefetch_complete` | `is_prefetch_complete` | `core/src/routing/resume_prefetch.rs` | 296 | 10 | `WIRE is_prefetch_complete call path + tests` |
| `is_prefetch_in_progress` | `is_prefetch_in_progress` | `core/src/routing/resume_prefetch.rs` | 301 | 10 | `WIRE is_prefetch_in_progress call path + tests` |
| `mark_path_failed` | `mark_path_failed` | `core/src/routing/multipath.rs` | 61 | 10 | `WIRE mark_path_failed call path + tests` |
| `mark_refresh_failed` | `mark_refresh_failed` | `core/src/routing/resume_prefetch.rs` | 283 | 10 | `WIRE mark_refresh_failed call path + tests` |
| `negative_cache_stats` | `negative_cache_stats` | `core/src/routing/optimized_engine.rs` | 184 | 10 | `WIRE negative_cache_stats call path + tests` |
| `next_refresh_hint` | `next_refresh_hint` | `core/src/routing/resume_prefetch.rs` | 291 | 10 | `WIRE next_refresh_hint call path + tests` |
| `normal_low_volume_usage_is_unaffected` | `normal_low_volume_usage_is_unaffected` | `core/src/transport/swarm.rs` | 4397 | 10 | `WIRE normal_low_volume_usage_is_unaffected call path + tests` |
| `on_read` | `on_read` | `core/src/transport/ble/gatt.rs` | 289 | 10 | `WIRE on_read call path + tests` |
| `on_write` | `on_write` | `core/src/transport/ble/gatt.rs` | 282 | 10 | `WIRE on_write call path + tests` |
| `peer_id_public_key_extraction_roundtrips_for_ed25519_peers` | `peer_id_public_key_extraction_roundtrips_for_ed25519_peers` | `core/src/transport/swarm.rs` | 4544 | 10 | `WIRE peer_id_public_key_extraction_roundtrips_for_ed25519_peers call path + tests` |
| `peers_needing_reconnect` | `peers_needing_reconnect` | `core/src/transport/manager.rs` | 446 | 10 | `WIRE peers_needing_reconnect call path + tests` |
| `prefetch_manager_mut` | `prefetch_manager_mut` | `core/src/routing/optimized_engine.rs` | 214 | 10 | `WIRE prefetch_manager_mut call path + tests` |
| `prefetch_stats` | `prefetch_stats` | `core/src/routing/optimized_engine.rs` | 189 | 10 | `WIRE prefetch_stats call path + tests` |
| `prune_below` | `prune_below` | `core/src/routing/reputation.rs` | 64 | 10 | `WIRE prune_below call path + tests` |
| `register_path` | `register_path` | `core/src/routing/multipath.rs` | 45 | 10 | `WIRE register_path call path + tests` |
| `register_state_change_callback` | `register_state_change_callback` | `core/src/transport/health.rs` | 400 | 10 | `WIRE register_state_change_callback call path + tests` |
| `registration_payload_canonical_bytes_are_stable` | `registration_payload_canonical_bytes_are_stable` | `core/src/transport/behaviour.rs` | 571 | 10 | `WIRE registration_payload_canonical_bytes_are_stable call path + tests` |
| `registration_transitions_for_identity` | `registration_transitions_for_identity` | `core/src/store/relay_custody.rs` | 483 | 10 | `WIRE registration_transitions_for_identity call path + tests` |
| `relay_discovery_mut` | `relay_discovery_mut` | `core/src/transport/bootstrap.rs` | 193 | 10 | `WIRE relay_discovery_mut call path + tests` |
| `relay_request_carries_ws13_metadata_when_set` | `relay_request_carries_ws13_metadata_when_set` | `core/src/transport/behaviour.rs` | 538 | 10 | `WIRE relay_request_carries_ws13_metadata_when_set call path + tests` |
| `relay_request_missing_ws13_fields_deserialize_with_defaults` | `relay_request_missing_ws13_fields_deserialize_with_defaults` | `core/src/transport/behaviour.rs` | 554 | 10 | `WIRE relay_request_missing_ws13_fields_deserialize_with_defaults call path + tests` |
| `reset_circuit_breakers` | `reset_circuit_breakers` | `core/src/transport/bootstrap.rs` | 455 | 10 | `WIRE reset_circuit_breakers call path + tests` |
| `should_advance` | `should_advance` | `core/src/routing/timeout_budget.rs` | 118 | 10 | `WIRE should_advance call path + tests` |
| `signed_deregistration_request_rejects_same_source_and_target_device` | `signed_deregistration_request_rejects_same_source_and_target_device` | `core/src/transport/behaviour.rs` | 649 | 10 | `WIRE signed_deregistration_request_rejects_same_source_and_target_device call path + tests` |
| `signed_deregistration_request_verifies_against_matching_public_key` | `signed_deregistration_request_verifies_against_matching_public_key` | `core/src/transport/behaviour.rs` | 635 | 10 | `WIRE signed_deregistration_request_verifies_against_matching_public_key call path + tests` |
| `signed_registration_request_rejects_malformed_identity_id` | `signed_registration_request_rejects_malformed_identity_id` | `core/src/transport/behaviour.rs` | 617 | 10 | `WIRE signed_registration_request_rejects_malformed_identity_id call path + tests` |
| `signed_registration_request_rejects_tampered_payload` | `signed_registration_request_rejects_tampered_payload` | `core/src/transport/behaviour.rs` | 599 | 10 | `WIRE signed_registration_request_rejects_tampered_payload call path + tests` |
| `signed_registration_request_verifies_against_matching_public_key` | `signed_registration_request_verifies_against_matching_public_key` | `core/src/transport/behaviour.rs` | 585 | 10 | `WIRE signed_registration_request_verifies_against_matching_public_key call path + tests` |
| `start_hole_punch` | `start_hole_punch` | `core/src/transport/nat.rs` | 388 | 10 | `WIRE start_hole_punch call path + tests` |
| `start_refresh` | `start_refresh` | `core/src/routing/resume_prefetch.rs` | 78 | 10 | `WIRE start_refresh call path + tests` |
| `storage_pressure_emergency_mode_rejects_non_critical_and_recovers` | `storage_pressure_emergency_mode_rejects_non_critical_and_recovers` | `core/src/store/relay_custody.rs` | 2012 | 10 | `WIRE storage_pressure_emergency_mode_rejects_non_critical_and_recovers call path + tests` |
| `storage_pressure_purge_prioritizes_non_identity_then_identity` | `storage_pressure_purge_prioritizes_non_identity_then_identity` | `core/src/store/relay_custody.rs` | 1926 | 10 | `WIRE storage_pressure_purge_prioritizes_non_identity_then_identity call path + tests` |
| `storage_pressure_purge_records_audit_transition_before_delete` | `storage_pressure_purge_records_audit_transition_before_delete` | `core/src/store/relay_custody.rs` | 1982 | 11 | `WIRE storage_pressure_purge_records_audit_transition_before_delete call path + tests` |
| `storage_pressure_quota_bands_follow_locked_policy` | `storage_pressure_quota_bands_follow_locked_policy` | `core/src/store/relay_custody.rs` | 1802 | 10 | `WIRE storage_pressure_quota_bands_follow_locked_policy call path + tests` |
| `storage_pressure_state_uses_synthetic_snapshot_when_probe_unavailable` | `storage_pressure_state_uses_synthetic_snapshot_when_probe_unavailable` | `core/src/store/relay_custody.rs` | 2115 | 10 | `WIRE storage_pressure_state_uses_synthetic_snapshot_when_probe_unavailable call path + tests` |
| `timeout_budget_summary` | `timeout_budget_summary` | `core/src/routing/optimized_engine.rs` | 179 | 10 | `WIRE timeout_budget_summary call path + tests` |
| `token_bucket_refills_after_elapsed_time` | `token_bucket_refills_after_elapsed_time` | `core/src/transport/swarm.rs` | 4435 | 10 | `WIRE token_bucket_refills_after_elapsed_time call path + tests` |
| `transport_type_to_routing_transport` | `transport_type_to_routing_transport` | `core/src/transport/swarm.rs` | 664 | 10 | `WIRE transport_type_to_routing_transport call path + tests` |
| `verify_registration_message_rejects_peer_identity_mismatch` | `verify_registration_message_rejects_peer_identity_mismatch` | `core/src/transport/swarm.rs` | 4554 | 10 | `WIRE verify_registration_message_rejects_peer_identity_mismatch call path + tests` |

## B3-android-repository

| Task | Function | Target | Definition line | External refs | Patch template |
|---|---|---|---:|---:|---|
| `autoSubscribeToPeerTopics` | `autoSubscribeToPeerTopics` | `android/app/src/main/java/com/scmessenger/android/data/TopicManager.kt` | 120 | 11 | `WIRE autoSubscribeToPeerTopics call path + tests` |
| `exportDiagnosticsAsync` | `exportDiagnosticsAsync` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 4358 | 12 | `WIRE exportDiagnosticsAsync call path + tests` |
| `filterMessagesByTopic` | `filterMessagesByTopic` | `android/app/src/main/java/com/scmessenger/android/data/TopicManager.kt` | 133 | 10 | `WIRE filterMessagesByTopic call path + tests` |
| `getBlockedCount` | `getBlockedCount` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 3348 | 11 | `WIRE getBlockedCount call path + tests` |
| `getBootstrapNodesForSettings` | `getBootstrapNodesForSettings` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 67 | 10 | `WIRE getBootstrapNodesForSettings call path + tests` |
| `getInboxCount` | `getInboxCount` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 3415 | 10 | `WIRE getInboxCount call path + tests` |
| `getKnownTopicsList` | `getKnownTopicsList` | `android/app/src/main/java/com/scmessenger/android/data/TopicManager.kt` | 163 | 10 | `WIRE getKnownTopicsList call path + tests` |
| `getMessage` | `getMessage` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 4076 | 11 | `WIRE getMessage call path + tests` |
| `getNetworkDiagnosticsSnapshot` | `getNetworkDiagnosticsSnapshot` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 7473 | 10 | `WIRE getNetworkDiagnosticsSnapshot call path + tests` |
| `getNetworkFailureSummary` | `getNetworkFailureSummary` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 7468 | 10 | `WIRE getNetworkFailureSummary call path + tests` |
| `getRetryDelay` | `getRetryDelay` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 623 | 10 | `WIRE getRetryDelay call path + tests` |
| `getSubscribedTopicsList` | `getSubscribedTopicsList` | `android/app/src/main/java/com/scmessenger/android/data/TopicManager.kt` | 156 | 10 | `WIRE getSubscribedTopicsList call path + tests` |
| `getTransportHealthSummary` | `getTransportHealthSummary` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 8016 | 10 | `WIRE getTransportHealthSummary call path + tests` |
| `incrementAttemptCount` | `incrementAttemptCount` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 613 | 10 | `WIRE incrementAttemptCount call path + tests` |
| `loadPendingOutboxAsync` | `loadPendingOutboxAsync` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 5692 | 10 | `WIRE loadPendingOutboxAsync call path + tests` |
| `logMessageDeliveryAttempt` | `logMessageDeliveryAttempt` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 645 | 10 | `WIRE logMessageDeliveryAttempt call path + tests` |
| `markCorrupted` | `markCorrupted` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 472 | 10 | `WIRE markCorrupted call path + tests` |
| `observeNetworkStats` | `observeNetworkStats` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 7626 | 10 | `WIRE observeNetworkStats call path + tests` |
| `observePeers` | `observePeers` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 7611 | 10 | `WIRE observePeers call path + tests` |
| `onPeerDisconnected` | `onPeerDisconnected` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 1444 | 36 | `WIRE onPeerDisconnected call path + tests` |
| `onPeerIdentified` | `onPeerIdentified` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 1225 | 60 | `WIRE onPeerIdentified call path + tests` |
| `onReceiptReceived` | `onReceiptReceived` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 1860 | 31 | `WIRE onReceiptReceived call path + tests` |
| `primeRelayBootstrapConnectionsLegacy` | `primeRelayBootstrapConnectionsLegacy` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 7154 | 10 | `WIRE primeRelayBootstrapConnectionsLegacy call path + tests` |
| `recordConnectionFailure` | `recordConnectionFailure` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 4207 | 11 | `WIRE recordConnectionFailure call path + tests` |
| `recordTransportEvent` | `recordTransportEvent` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 8004 | 10 | `WIRE recordTransportEvent call path + tests` |
| `resetServiceStats` | `resetServiceStats` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 3003 | 11 | `WIRE resetServiceStats call path + tests` |
| `searchContacts` | `searchContacts` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 3277 | 15 | `WIRE searchContacts call path + tests` |
| `setContactNickname` | `setContactNickname` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 3281 | 11 | `WIRE setContactNickname call path + tests` |
| `shouldRetryMessage` | `shouldRetryMessage` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 637 | 10 | `WIRE shouldRetryMessage call path + tests` |
| `testLedgerRelayConnectivity` | `testLedgerRelayConnectivity` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 1036 | 10 | `WIRE testLedgerRelayConnectivity call path + tests` |
| `updateContactDeviceId` | `updateContactDeviceId` | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 3423 | 11 | `WIRE updateContactDeviceId call path + tests` |

## B4-android-ui

| Task | Function | Target | Definition line | External refs | Patch template |
|---|---|---|---:|---:|---|
| `ConnectionQualityIndicator` | `ConnectionQualityIndicator` | `android/app/src/main/java/com/scmessenger/android/ui/components/StatusIndicator.kt` | 122 | 10 | `WIRE ConnectionQualityIndicator call path + tests` |
| `ContactDetailScreen` | `ContactDetailScreen` | `android/app/src/main/java/com/scmessenger/android/ui/contacts/ContactDetailScreen.kt` | 38 | 34 | `WIRE ContactDetailScreen call path + tests` |
| `ErrorState` | `ErrorState` | `android/app/src/main/java/com/scmessenger/android/ui/components/ErrorBanner.kt` | 138 | 10 | `WIRE ErrorState call path + tests` |
| `IdenticonFromHex` | `IdenticonFromHex` | `android/app/src/main/java/com/scmessenger/android/ui/components/Identicon.kt` | 115 | 10 | `WIRE IdenticonFromHex call path + tests` |
| `InfoBanner` | `InfoBanner` | `android/app/src/main/java/com/scmessenger/android/ui/components/ErrorBanner.kt` | 176 | 10 | `WIRE InfoBanner call path + tests` |
| `LabeledCopyableText` | `LabeledCopyableText` | `android/app/src/main/java/com/scmessenger/android/ui/components/CopyableText.kt` | 89 | 10 | `WIRE LabeledCopyableText call path + tests` |
| `MeshSettingsScreen` | `MeshSettingsScreen` | `android/app/src/main/java/com/scmessenger/android/ui/settings/MeshSettingsScreen.kt` | 32 | 33 | `WIRE MeshSettingsScreen call path + tests` |
| `MessageInput` | `MessageInput` | `android/app/src/main/java/com/scmessenger/android/ui/chat/MessageInput.kt` | 20 | 24 | `WIRE MessageInput call path + tests` |
| `PeerListScreen` | `PeerListScreen` | `android/app/src/main/java/com/scmessenger/android/ui/dashboard/PeerListScreen.kt` | 36 | 26 | `WIRE PeerListScreen call path + tests` |
| `PowerSettingsScreen` | `PowerSettingsScreen` | `android/app/src/main/java/com/scmessenger/android/ui/settings/PowerSettingsScreen.kt` | 30 | 27 | `WIRE PowerSettingsScreen call path + tests` |
| `TopologyScreen` | `TopologyScreen` | `android/app/src/main/java/com/scmessenger/android/ui/dashboard/TopologyScreen.kt` | 40 | 26 | `WIRE TopologyScreen call path + tests` |
| `TruncatedCopyableText` | `TruncatedCopyableText` | `android/app/src/main/java/com/scmessenger/android/ui/components/CopyableText.kt` | 130 | 10 | `WIRE TruncatedCopyableText call path + tests` |
| `WarningBanner` | `WarningBanner` | `android/app/src/main/java/com/scmessenger/android/ui/components/ErrorBanner.kt` | 157 | 10 | `WIRE WarningBanner call path + tests` |
| `clearAllHistory` | `clearAllHistory` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ConversationsViewModel.kt` | 198 | 10 | `WIRE clearAllHistory call path + tests` |
| `clearInput` | `clearInput` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt` | 224 | 10 | `WIRE clearInput call path + tests` |
| `clearSearch` | `clearSearch` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ContactsViewModel.kt` | 549 | 10 | `WIRE clearSearch call path + tests` |
| `getNetworkDiagnosticsReport` | `getNetworkDiagnosticsReport` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt` | 595 | 10 | `WIRE getNetworkDiagnosticsReport call path + tests` |
| `loadConversation` | `loadConversation` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ConversationsViewModel.kt` | 114 | 10 | `WIRE loadConversation call path + tests` |
| `loadMoreMessages` | `loadMoreMessages` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt` | 358 | 12 | `WIRE loadMoreMessages call path + tests` |
| `resolveDeliveryState` | `resolveDeliveryState` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ConversationsViewModel.kt` | 395 | 10 | `WIRE resolveDeliveryState call path + tests` |
| `setPeer` | `setPeer` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt` | 76 | 16 | `WIRE setPeer call path + tests` |
| `updateBatteryFloor` | `updateBatteryFloor` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt` | 386 | 14 | `WIRE updateBatteryFloor call path + tests` |
| `updateDiscoveryMode` | `updateDiscoveryMode` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt` | 416 | 14 | `WIRE updateDiscoveryMode call path + tests` |
| `updateInputText` | `updateInputText` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt` | 216 | 12 | `WIRE updateInputText call path + tests` |
| `updateMaxRelayBudget` | `updateMaxRelayBudget` | `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt` | 380 | 12 | `WIRE updateMaxRelayBudget call path + tests` |

## B5-android-transport-service

| Task | Function | Target | Definition line | External refs | Patch template |
|---|---|---|---:|---:|---|
| `acquireWakeLock` | `acquireWakeLock` | `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt` | 251 | 10 | `WIRE acquireWakeLock call path + tests` |
| `applyAdvertiseSettings` | `applyAdvertiseSettings` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt` | 142 | 14 | `WIRE applyAdvertiseSettings call path + tests` |
| `applyScanSettings` | `applyScanSettings` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` | 237 | 15 | `WIRE applyScanSettings call path + tests` |
| `attemptBleRecovery` | `attemptBleRecovery` | `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt` | 454 | 10 | `WIRE attemptBleRecovery call path + tests` |
| `checkAndRecordMessage` | `checkAndRecordMessage` | `android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt` | 214 | 12 | `WIRE checkAndRecordMessage call path + tests` |
| `clearAnrEvents` | `clearAnrEvents` | `android/app/src/main/java/com/scmessenger/android/service/PerformanceMonitor.kt` | 199 | 10 | `WIRE clearAnrEvents call path + tests` |
| `clearPeerCache` | `clearPeerCache` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` | 536 | 10 | `WIRE clearPeerCache call path + tests` |
| `currentCount` | `currentCount` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleQuotaManager.kt` | 50 | 14 | `WIRE currentCount call path + tests` |
| `disableTransport` | `disableTransport` | `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt` | 388 | 10 | `WIRE disableTransport call path + tests` |
| `enableTransport` | `enableTransport` | `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt` | 364 | 10 | `WIRE enableTransport call path + tests` |
| `forceRestartScanning` | `forceRestartScanning` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` | 479 | 11 | `WIRE forceRestartScanning call path + tests` |
| `fromValue` | `fromValue` | `android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt` | 34 | 10 | `WIRE fromValue call path + tests` |
| `getActiveTransports` | `getActiveTransports` | `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt` | 239 | 10 | `WIRE getActiveTransports call path + tests` |
| `getAllAnrEvents` | `getAllAnrEvents` | `android/app/src/main/java/com/scmessenger/android/service/PerformanceMonitor.kt` | 152 | 10 | `WIRE getAllAnrEvents call path + tests` |
| `getAnrStats` | `getAnrStats` | `android/app/src/main/java/com/scmessenger/android/service/PerformanceMonitor.kt` | 134 | 10 | `WIRE getAnrStats call path + tests` |
| `getAvailableTransports` | `getAvailableTransports` | `android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt` | N/A | 11 | `WIRE getAvailableTransports call path + tests` |
| `getAvailableTransportsSorted` | `getAvailableTransportsSorted` | `android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt` | 202 | 11 | `WIRE getAvailableTransportsSorted call path + tests` |
| `getDedupStats` | `getDedupStats` | `android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt` | 258 | 12 | `WIRE getDedupStats call path + tests` |
| `getHealthStatus` | `getHealthStatus` | `android/app/src/main/java/com/scmessenger/android/service/PerformanceMonitor.kt` | 141 | 10 | `WIRE getHealthStatus call path + tests` |
| `getTotalAnrEvents` | `getTotalAnrEvents` | `android/app/src/main/java/com/scmessenger/android/service/AnrWatchdog.kt` | 264 | 10 | `WIRE getTotalAnrEvents call path + tests` |
| `handleBleFailure` | `handleBleFailure` | `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt` | 431 | 10 | `WIRE handleBleFailure call path + tests` |
| `handleScanFailure` | `handleScanFailure` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` | 498 | 10 | `WIRE handleScanFailure call path + tests` |
| `isPortLikelyBlocked` | `isPortLikelyBlocked` | `android/app/src/main/java/com/scmessenger/android/transport/NetworkDetector.kt` | 189 | 10 | `WIRE isPortLikelyBlocked call path + tests` |
| `isServiceHealthy` | `isServiceHealthy` | `android/app/src/main/java/com/scmessenger/android/service/ServiceHealthMonitor.kt` | 340 | 10 | `WIRE isServiceHealthy call path + tests` |
| `notifyBackground` | `notifyBackground` | `android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt` | 399 | 10 | `WIRE notifyBackground call path + tests` |
| `notifyForeground` | `notifyForeground` | `android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt` | 406 | 10 | `WIRE notifyForeground call path + tests` |
| `onAnr` | `onAnr` | `android/app/src/main/java/com/scmessenger/android/service/AnrWatchdog.kt` | 38 | 10 | `WIRE onAnr call path + tests` |
| `onBind` | `onBind` | `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt` | 534 | 10 | `WIRE onBind call path + tests` |
| `onBleDataReceived` | `onBleDataReceived` | `android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt` | 314 | 40 | `WIRE onBleDataReceived call path + tests` |
| `onDiscoveryStarted` | `onDiscoveryStarted` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 141 | 10 | `WIRE onDiscoveryStarted call path + tests` |
| `onDiscoveryStopped` | `onDiscoveryStopped` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 146 | 10 | `WIRE onDiscoveryStopped call path + tests` |
| `onRegistrationFailed` | `onRegistrationFailed` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 113 | 10 | `WIRE onRegistrationFailed call path + tests` |
| `onResolveFailed` | `onResolveFailed` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 183 | 10 | `WIRE onResolveFailed call path + tests` |
| `onScanFailed` | `onScanFailed` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` | 146 | 10 | `WIRE onScanFailed call path + tests` |
| `onScanResult` | `onScanResult` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` | 100 | 10 | `WIRE onScanResult call path + tests` |
| `onServiceFound` | `onServiceFound` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 151 | 10 | `WIRE onServiceFound call path + tests` |
| `onServiceLost` | `onServiceLost` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 161 | 10 | `WIRE onServiceLost call path + tests` |
| `onServiceRegistered` | `onServiceRegistered` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 122 | 10 | `WIRE onServiceRegistered call path + tests` |
| `onServiceResolved` | `onServiceResolved` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 187 | 10 | `WIRE onServiceResolved call path + tests` |
| `onServiceUnregistered` | `onServiceUnregistered` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 127 | 10 | `WIRE onServiceUnregistered call path + tests` |
| `onStartDiscoveryFailed` | `onStartDiscoveryFailed` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 165 | 10 | `WIRE onStartDiscoveryFailed call path + tests` |
| `onStartFailure` | `onStartFailure` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt` | 56 | 10 | `WIRE onStartFailure call path + tests` |
| `onStartSuccess` | `onStartSuccess` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt` | 49 | 10 | `WIRE onStartSuccess call path + tests` |
| `onStopDiscoveryFailed` | `onStopDiscoveryFailed` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 170 | 10 | `WIRE onStopDiscoveryFailed call path + tests` |
| `onUnregistrationFailed` | `onUnregistrationFailed` | `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | 118 | 10 | `WIRE onUnregistrationFailed call path + tests` |
| `recordAnrEvent` | `recordAnrEvent` | `android/app/src/main/java/com/scmessenger/android/service/PerformanceMonitor.kt` | 53 | 10 | `WIRE recordAnrEvent call path + tests` |
| `recordUiTiming` | `recordUiTiming` | `android/app/src/main/java/com/scmessenger/android/service/PerformanceMonitor.kt` | 88 | 10 | `WIRE recordUiTiming call path + tests` |
| `resetHealth` | `resetHealth` | `android/app/src/main/java/com/scmessenger/android/service/ServiceHealthMonitor.kt` | N/A | 11 | `WIRE resetHealth call path + tests` |
| `sendBlePacket` | `sendBlePacket` | `android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt` | 334 | 40 | `WIRE sendBlePacket call path + tests` |
| `setBleComponents` | `setBleComponents` | `android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt` | 87 | 10 | `WIRE setBleComponents call path + tests` |
| `setRotationInterval` | `setRotationInterval` | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt` | 130 | 17 | `WIRE setRotationInterval call path + tests` |
| `shouldUseTransport` | `shouldUseTransport` | `android/app/src/main/java/com/scmessenger/android/transport/TransportHealthMonitor.kt` | 50 | 10 | `WIRE shouldUseTransport call path + tests` |
| `startAll` | `startAll` | `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt` | 82 | 12 | `WIRE startAll call path + tests` |
| `toLogString` | `toLogString` | `android/app/src/main/java/com/scmessenger/android/transport/NetworkDetector.kt` | 334 | 10 | `WIRE toLogString call path + tests` |
| `updateHeartbeat` | `updateHeartbeat` | `android/app/src/main/java/com/scmessenger/android/service/ServiceHealthMonitor.kt` | 328 | 10 | `WIRE updateHeartbeat call path + tests` |

## B6-wasm

| Task | Function | Target | Definition line | External refs | Patch template |
|---|---|---|---:|---:|---|
| `add_rtc_connection` | `add_rtc_connection` | `wasm/src/connection_state.rs` | 278 | 10 | `WIRE add_rtc_connection call path + tests` |
| `add_websocket` | `add_websocket` | `wasm/src/connection_state.rs` | 257 | 10 | `WIRE add_websocket call path + tests` |
| `close_all_notifications` | `close_all_notifications` | `wasm/src/notification_manager.rs` | 308 | 10 | `WIRE close_all_notifications call path + tests` |
| `detect_browser` | `detect_browser` | `wasm/src/notification_manager.rs` | 323 | 10 | `WIRE detect_browser call path + tests` |
| `drain_received_messages` | `drain_received_messages` | `wasm/src/lib.rs` | 859 | 10 | `WIRE drain_received_messages call path + tests` |
| `get_browser_options` | `get_browser_options` | `wasm/src/notification_manager.rs` | 329 | 10 | `WIRE get_browser_options call path + tests` |
| `get_contact_manager` | `get_contact_manager` | `wasm/src/lib.rs` | 1026 | 10 | `WIRE get_contact_manager call path + tests` |
| `get_daemon_socket_url` | `get_daemon_socket_url` | `wasm/src/lib.rs` | 336 | 10 | `WIRE get_daemon_socket_url call path + tests` |
| `get_default_settings` | `get_default_settings` | `wasm/src/lib.rs` | 902 | 10 | `WIRE get_default_settings call path + tests` |
| `get_history_manager` | `get_history_manager` | `wasm/src/lib.rs` | 1033 | 10 | `WIRE get_history_manager call path + tests` |
| `get_identity_from_daemon` | `get_identity_from_daemon` | `wasm/src/lib.rs` | 407 | 11 | `WIRE get_identity_from_daemon call path + tests` |
| `get_identity_wire_shape` | `get_identity_wire_shape` | `wasm/src/daemon_bridge.rs` | 36 | 10 | `WIRE get_identity_wire_shape call path + tests` |
| `get_iron_core_mode` | `get_iron_core_mode` | `wasm/src/lib.rs` | 322 | 10 | `WIRE get_iron_core_mode call path + tests` |
| `get_permission` | `get_permission` | `wasm/src/notification_manager.rs` | 317 | 10 | `WIRE get_permission call path + tests` |
| `get_settings` | `get_settings` | `wasm/src/lib.rs` | 877 | 10 | `WIRE get_settings call path + tests` |
| `initialize_identity_from_daemon` | `initialize_identity_from_daemon` | `wasm/src/lib.rs` | 375 | 11 | `WIRE initialize_identity_from_daemon call path + tests` |
| `is_permission_granted` | `is_permission_granted` | `wasm/src/notification_manager.rs` | 231 | 10 | `WIRE is_permission_granted call path + tests` |
| `notification_roundtrip_for_ui_state` | `notification_roundtrip_for_ui_state` | `wasm/src/daemon_bridge.rs` | 45 | 10 | `WIRE notification_roundtrip_for_ui_state call path + tests` |
| `parse_response` | `parse_response` | `wasm/src/daemon_bridge.rs` | 21 | 10 | `WIRE parse_response call path + tests` |
| `remove_rtc_connection` | `remove_rtc_connection` | `wasm/src/connection_state.rs` | 286 | 10 | `WIRE remove_rtc_connection call path + tests` |
| `remove_websocket` | `remove_websocket` | `wasm/src/connection_state.rs` | 265 | 10 | `WIRE remove_websocket call path + tests` |
| `request_permission` | `request_permission` | `wasm/src/notification_manager.rs` | 110 | 10 | `WIRE request_permission call path + tests` |
| `send_prepared_envelope` | `send_prepared_envelope` | `wasm/src/lib.rs` | 551 | 10 | `WIRE send_prepared_envelope call path + tests` |
| `set_daemon_socket_url` | `set_daemon_socket_url` | `wasm/src/lib.rs` | 330 | 10 | `WIRE set_daemon_socket_url call path + tests` |
| `set_iron_core_mode` | `set_iron_core_mode` | `wasm/src/lib.rs` | 313 | 10 | `WIRE set_iron_core_mode call path + tests` |
| `show_permission_guidance` | `show_permission_guidance` | `wasm/src/notification_manager.rs` | 355 | 10 | `WIRE show_permission_guidance call path + tests` |
| `start_receive_loop` | `start_receive_loop` | `wasm/src/lib.rs` | 833 | 10 | `WIRE start_receive_loop call path + tests` |
| `stop_swarm` | `stop_swarm` | `wasm/src/lib.rs` | 538 | 10 | `WIRE stop_swarm call path + tests` |
| `update_settings` | `update_settings` | `wasm/src/lib.rs` | 885 | 10 | `WIRE update_settings call path + tests` |
| `validate_settings` | `validate_settings` | `wasm/src/lib.rs` | 807 | 10 | `WIRE validate_settings call path + tests` |

## B7-cli

| Task | Function | Target | Definition line | External refs | Patch template |
|---|---|---|---:|---:|---|
| `advertise_service` | `advertise_service` | `cli/src/ble_daemon.rs` | 253 | 10 | `WIRE advertise_service call path + tests` |
| `can_forward_for_wasm` | `can_forward_for_wasm` | `cli/src/transport_bridge.rs` | 246 | 11 | `WIRE can_forward_for_wasm call path + tests` |
| `can_reach_destination` | `can_reach_destination` | `cli/src/transport_bridge.rs` | 264 | 11 | `WIRE can_reach_destination call path + tests` |
| `count_with_peer` | `count_with_peer` | `cli/src/history.rs` | 182 | 10 | `WIRE count_with_peer call path + tests` |
| `decode_rejects_short_buffer` | `decode_rejects_short_buffer` | `cli/src/ble_mesh.rs` | 245 | 10 | `WIRE decode_rejects_short_buffer call path + tests` |
| `find_by_nickname` | `find_by_nickname` | `cli/src/contacts.rs` | 119 | 10 | `WIRE find_by_nickname call path + tests` |
| `find_by_public_key` | `find_by_public_key` | `cli/src/contacts.rs` | 131 | 10 | `WIRE find_by_public_key call path + tests` |
| `formatted_time` | `formatted_time` | `cli/src/history.rs` | 66 | 10 | `WIRE formatted_time call path + tests` |
| `get_available_paths` | `get_available_paths` | `cli/src/transport_bridge.rs` | 189 | 11 | `WIRE get_available_paths call path + tests` |
| `get_best_forwarding_path` | `get_best_forwarding_path` | `cli/src/transport_bridge.rs` | 279 | 11 | `WIRE get_best_forwarding_path call path + tests` |
| `get_forwarding_capability` | `get_forwarding_capability` | `cli/src/transport_bridge.rs` | 252 | 11 | `WIRE get_forwarding_capability call path + tests` |
| `get_history_via_api` | `get_history_via_api` | `cli/src/api.rs` | 169 | 10 | `WIRE get_history_via_api call path + tests` |
| `is_ble_available` | `is_ble_available` | `cli/src/ble_daemon.rs` | 286 | 10 | `WIRE is_ble_available call path + tests` |
| `scan_for_advertisements` | `scan_for_advertisements` | `cli/src/ble_daemon.rs` | 229 | 10 | `WIRE scan_for_advertisements call path + tests` |
| `set_notes` | `set_notes` | `cli/src/contacts.rs` | 165 | 10 | `WIRE set_notes call path + tests` |
| `try_enable_bluetooth` | `try_enable_bluetooth` | `cli/src/ble_daemon.rs` | 331 | 10 | `WIRE try_enable_bluetooth call path + tests` |

## B8-cross-cutting

| Task | Function | Target | Definition line | External refs | Patch template |
|---|---|---|---:|---:|---|
| `add_step` | `add_step` | `core/src/dspy/modules.rs` | 84 | 10 | `WIRE add_step call path + tests` |
| `apply_policy_config` | `apply_policy_config` | `core/src/drift/relay.rs` | 366 | 10 | `WIRE apply_policy_config call path + tests` |
| `blake3_hash` | `blake3_hash` | `core/src/dspy/signatures.rs` | 136 | 10 | `WIRE blake3_hash call path + tests` |
| `blocked_only_peer_ids` | `blocked_only_peer_ids` | `core/src/store/blocked.rs` | 230 | 12 | `WIRE blocked_only_peer_ids call path + tests` |
| `buildForegroundServiceNotification` | `buildForegroundServiceNotification` | `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt` | 175 | 10 | `WIRE buildForegroundServiceNotification call path + tests` |
| `build_optimization_pipeline` | `build_optimization_pipeline` | `core/src/dspy/teleprompt.rs` | 241 | 10 | `WIRE build_optimization_pipeline call path + tests` |
| `build_security_audit_pipeline` | `build_security_audit_pipeline` | `core/src/dspy/modules.rs` | 240 | 10 | `WIRE build_security_audit_pipeline call path + tests` |
| `chain_ratchet_produces_distinct_keys` | `chain_ratchet_produces_distinct_keys` | `core/src/crypto/kani_proofs.rs` | 51 | 10 | `WIRE chain_ratchet_produces_distinct_keys call path + tests` |
| `clearAllRequestNotifications` | `clearAllRequestNotifications` | `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt` | 492 | 10 | `WIRE clearAllRequestNotifications call path + tests` |
| `clearMessageNotifications` | `clearMessageNotifications` | `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt` | 479 | 10 | `WIRE clearMessageNotifications call path + tests` |
| `contact_new_has_no_last_known_device_id` | `contact_new_has_no_last_known_device_id` | `core/src/store/contacts.rs` | 319 | 10 | `WIRE contact_new_has_no_last_known_device_id call path + tests` |
| `contact_roundtrips_through_serde_with_default_device_id` | `contact_roundtrips_through_serde_with_default_device_id` | `core/src/store/contacts.rs` | 358 | 10 | `WIRE contact_roundtrips_through_serde_with_default_device_id call path + tests` |
| `create_basic` | `create_basic` | `core/src/dspy/teleprompt.rs` | 208 | 10 | `WIRE create_basic call path + tests` |
| `create_cot` | `create_cot` | `core/src/dspy/modules.rs` | 212 | 10 | `WIRE create_cot call path + tests` |
| `create_multihop` | `create_multihop` | `core/src/dspy/modules.rs` | 216 | 10 | `WIRE create_multihop call path + tests` |
| `create_optimizer` | `create_optimizer` | `core/src/dspy/modules.rs` | 220 | 10 | `WIRE create_optimizer call path + tests` |
| `create_receiver_session` | `create_receiver_session` | `core/src/crypto/session_manager.rs` | 76 | 10 | `WIRE create_receiver_session call path + tests` |
| `derive_key_always_32_bytes` | `derive_key_always_32_bytes` | `core/src/crypto/kani_proofs.rs` | 37 | 10 | `WIRE derive_key_always_32_bytes call path + tests` |
| `disable_location_background` | `disable_location_background` | `core/src/mobile/ios_strategy.rs` | 260 | 10 | `WIRE disable_location_background call path + tests` |
| `disabled_notifications_suppress_delivery` | `disabled_notifications_suppress_delivery` | `core/src/notification.rs` | 447 | 10 | `WIRE disabled_notifications_suppress_delivery call path + tests` |
| `duplicates_are_suppressed` | `duplicates_are_suppressed` | `core/src/notification.rs` | 462 | 10 | `WIRE duplicates_are_suppressed call path + tests` |
| `ed25519_conversion_produces_32_bytes` | `ed25519_conversion_produces_32_bytes` | `core/src/crypto/kani_proofs.rs` | 28 | 10 | `WIRE ed25519_conversion_produces_32_bytes call path + tests` |
| `emergency_recover` | `emergency_recover` | `core/src/contacts_bridge.rs` | 279 | 10 | `WIRE emergency_recover call path + tests` |
| `encrypt_xchacha20` | `encrypt_xchacha20` | `core/src/dspy/signatures.rs` | 110 | 10 | `WIRE encrypt_xchacha20 call path + tests` |
| `evaluate_all_tracked` | `evaluate_all_tracked` | `core/src/abuse/auto_block.rs` | 226 | 10 | `WIRE evaluate_all_tracked call path + tests` |
| `explicit_request_overrides_known_contact_inference` | `explicit_request_overrides_known_contact_inference` | `core/src/notification.rs` | 437 | 10 | `WIRE explicit_request_overrides_known_contact_inference call path + tests` |
| `federated_nickname` | `federated_nickname` | `core/src/contacts_bridge.rs` | 53 | 11 | `WIRE federated_nickname call path + tests` |
| `force_ratchet` | `force_ratchet` | `core/src/crypto/ratchet.rs` | 420 | 10 | `WIRE force_ratchet call path + tests` |
| `foreground_direct_messages_follow_foreground_toggle` | `foreground_direct_messages_follow_foreground_toggle` | `core/src/notification.rs` | 475 | 10 | `WIRE foreground_direct_messages_follow_foreground_toggle call path + tests` |
| `formatReportForUser` | `formatReportForUser` | `android/app/src/main/java/com/scmessenger/android/network/DiagnosticsReporter.kt` | 108 | 10 | `WIRE formatReportForUser call path + tests` |
| `generate_cover_traffic_if_due` | `generate_cover_traffic_if_due` | `core/src/drift/relay.rs` | 179 | 10 | `WIRE generate_cover_traffic_if_due call path + tests` |
| `getHealthyRelays` | `getHealthyRelays` | `android/app/src/main/java/com/scmessenger/android/utils/CircuitBreaker.kt` | 199 | 10 | `WIRE getHealthyRelays call path + tests` |
| `getLastFailure` | `getLastFailure` | `android/app/src/main/java/com/scmessenger/android/utils/CircuitBreaker.kt` | N/A | 10 | `WIRE getLastFailure call path + tests` |
| `getLastFailureReason` | `getLastFailureReason` | `android/app/src/main/java/com/scmessenger/android/utils/CircuitBreaker.kt` | 178 | 10 | `WIRE getLastFailureReason call path + tests` |
| `getNotificationStats` | `getNotificationStats` | `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt` | 620 | 10 | `WIRE getNotificationStats call path + tests` |
| `getOpenCircuits` | `getOpenCircuits` | `android/app/src/main/java/com/scmessenger/android/utils/CircuitBreaker.kt` | 194 | 10 | `WIRE getOpenCircuits call path + tests` |
| `get_last_profile` | `get_last_profile` | `core/src/mobile/auto_adjust.rs` | 268 | 10 | `WIRE get_last_profile call path + tests` |
| `get_overrides` | `get_overrides` | `core/src/mobile/auto_adjust.rs` | 263 | 10 | `WIRE get_overrides call path + tests` |
| `get_signable_data` | `get_signable_data` | `core/src/relay/invite.rs` | 90 | 10 | `WIRE get_signable_data call path + tests` |
| `get_signature` | `get_signature` | `core/src/dspy/signatures.rs` | 156 | 10 | `WIRE get_signature call path + tests` |
| `hasDnsFailures` | `hasDnsFailures` | `android/app/src/main/java/com/scmessenger/android/utils/NetworkFailureMetrics.kt` | 61 | 10 | `WIRE hasDnsFailures call path + tests` |
| `hasPortBlocking` | `hasPortBlocking` | `android/app/src/main/java/com/scmessenger/android/utils/NetworkFailureMetrics.kt` | 63 | 10 | `WIRE hasPortBlocking call path + tests` |
| `isAtMaxDelay` | `isAtMaxDelay` | `android/app/src/main/java/com/scmessenger/android/utils/BackoffStrategy.kt` | 79 | 10 | `WIRE isAtMaxDelay call path + tests` |
| `isStorageStateCritical` | `isStorageStateCritical` | `android/app/src/main/java/com/scmessenger/android/utils/StorageManager.kt` | 159 | 10 | `WIRE isStorageStateCritical call path + tests` |
| `jsonrpc_get_identity` | `jsonrpc_get_identity` | `core/src/wasm_support/rpc.rs` | 250 | 10 | `WIRE jsonrpc_get_identity call path + tests` |
| `jsonrpc_send_message_roundtrip` | `jsonrpc_send_message_roundtrip` | `core/src/wasm_support/rpc.rs` | 224 | 10 | `WIRE jsonrpc_send_message_roundtrip call path + tests` |
| `known_contact_defaults_to_direct_message` | `known_contact_defaults_to_direct_message` | `core/src/notification.rs` | 428 | 10 | `WIRE known_contact_defaults_to_direct_message call path + tests` |
| `list_endpoints` | `list_endpoints` | `core/src/notification.rs` | 357 | 11 | `WIRE list_endpoints call path + tests` |
| `load_device_id` | `load_device_id` | `core/src/identity/store.rs` | 209 | 10 | `WIRE load_device_id call path + tests` |
| `load_seniority_timestamp` | `load_seniority_timestamp` | `core/src/identity/store.rs` | 223 | 10 | `WIRE load_seniority_timestamp call path + tests` |
| `new_sync` | `new_sync` | `core/src/store/backend.rs` | 188 | 10 | `WIRE new_sync call path + tests` |
| `nonce_length_invariant` | `nonce_length_invariant` | `core/src/crypto/kani_proofs.rs` | 44 | 10 | `WIRE nonce_length_invariant call path + tests` |
| `notif_mesh_topology` | `notif_mesh_topology` | `core/src/wasm_support/rpc.rs` | 205 | 10 | `WIRE notif_mesh_topology call path + tests` |
| `notification_serialization` | `notification_serialization` | `core/src/wasm_support/rpc.rs` | 264 | 10 | `WIRE notification_serialization call path + tests` |
| `overall_score` | `overall_score` | `core/src/abuse/reputation.rs` | 169 | 10 | `WIRE overall_score call path + tests` |
| `override_ble_advertise_interval` | `override_ble_advertise_interval` | `core/src/mobile/auto_adjust.rs` | 243 | 10 | `WIRE override_ble_advertise_interval call path + tests` |
| `override_relay_priority_threshold` | `override_relay_priority_threshold` | `core/src/mobile/auto_adjust.rs` | 253 | 10 | `WIRE override_relay_priority_threshold call path + tests` |
| `proptest_different_ciphertexts_same_plaintext` | `proptest_different_ciphertexts_same_plaintext` | `core/src/crypto/proptest_harness.rs` | 57 | 10 | `WIRE proptest_different_ciphertexts_same_plaintext call path + tests` |
| `proptest_encrypt_decrypt_roundtrip` | `proptest_encrypt_decrypt_roundtrip` | `core/src/crypto/proptest_harness.rs` | 41 | 10 | `WIRE proptest_encrypt_decrypt_roundtrip call path + tests` |
| `proptest_envelope_field_lengths` | `proptest_envelope_field_lengths` | `core/src/crypto/proptest_harness.rs` | 90 | 10 | `WIRE proptest_envelope_field_lengths call path + tests` |
| `proptest_ratchet_forward_secrecy` | `proptest_ratchet_forward_secrecy` | `core/src/crypto/proptest_harness.rs` | 152 | 10 | `WIRE proptest_ratchet_forward_secrecy call path + tests` |
| `proptest_ratchet_roundtrip` | `proptest_ratchet_roundtrip` | `core/src/crypto/proptest_harness.rs` | 118 | 10 | `WIRE proptest_ratchet_roundtrip call path + tests` |
| `proptest_sign_verify_roundtrip` | `proptest_sign_verify_roundtrip` | `core/src/crypto/proptest_harness.rs` | 104 | 10 | `WIRE proptest_sign_verify_roundtrip call path + tests` |
| `proptest_wrong_key_fails` | `proptest_wrong_key_fails` | `core/src/crypto/proptest_harness.rs` | 75 | 10 | `WIRE proptest_wrong_key_fails call path + tests` |
| `provideMeshRepository` | `provideMeshRepository` | `android/app/src/main/java/com/scmessenger/android/di/AppModule.kt` | 26 | 10 | `WIRE provideMeshRepository call path + tests` |
| `providePreferencesRepository` | `providePreferencesRepository` | `android/app/src/main/java/com/scmessenger/android/di/AppModule.kt` | 34 | 10 | `WIRE providePreferencesRepository call path + tests` |
| `read_with_timeout` | `read_with_timeout` | `core/src/drift/frame.rs` | 183 | 10 | `WIRE read_with_timeout call path + tests` |
| `refresh_delegate_routes` | `refresh_delegate_routes` | `core/src/relay/delegate_prewarm.rs` | 215 | 11 | `WIRE refresh_delegate_routes call path + tests` |
| `register_endpoint` | `register_endpoint` | `core/src/notification.rs` | 304 | 11 | `WIRE register_endpoint call path + tests` |
| `resetNotificationStats` | `resetNotificationStats` | `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt` | 627 | 10 | `WIRE resetNotificationStats call path + tests` |
| `run_optimization` | `run_optimization` | `core/src/dspy/modules.rs` | 179 | 10 | `WIRE run_optimization call path + tests` |
| `set_cover_traffic` | `set_cover_traffic` | `core/src/drift/relay.rs` | 156 | 10 | `WIRE set_cover_traffic call path + tests` |
| `set_reputation_manager` | `set_reputation_manager` | `core/src/drift/relay.rs` | 170 | 10 | `WIRE set_reputation_manager call path + tests` |
| `showMeshStatusNotification` | `showMeshStatusNotification` | `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt` | 533 | 10 | `WIRE showMeshStatusNotification call path + tests` |
| `showPeerDiscoveredNotification` | `showPeerDiscoveredNotification` | `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt` | 501 | 10 | `WIRE showPeerDiscoveredNotification call path + tests` |
| `touch_endpoint` | `touch_endpoint` | `core/src/notification.rs` | 365 | 11 | `WIRE touch_endpoint call path + tests` |
| `unknown_method_error` | `unknown_method_error` | `core/src/wasm_support/rpc.rs` | 278 | 10 | `WIRE unknown_method_error call path + tests` |
| `unknown_sender_defaults_to_direct_message_request` | `unknown_sender_defaults_to_direct_message_request` | `core/src/notification.rs` | 418 | 10 | `WIRE unknown_sender_defaults_to_direct_message_request call path + tests` |
| `unregister_endpoint` | `unregister_endpoint` | `core/src/notification.rs` | 345 | 11 | `WIRE unregister_endpoint call path + tests` |
| `update_keepalive` | `update_keepalive` | `core/src/relay/delegate_prewarm.rs` | 100 | 10 | `WIRE update_keepalive call path + tests` |
| `update_last_known_device_id_can_clear` | `update_last_known_device_id_can_clear` | `core/src/store/contacts.rs` | 344 | 10 | `WIRE update_last_known_device_id_can_clear call path + tests` |
| `update_last_known_device_id_ignores_invalid_values` | `update_last_known_device_id_ignores_invalid_values` | `core/src/store/contacts.rs` | 388 | 10 | `WIRE update_last_known_device_id_ignores_invalid_values call path + tests` |
| `update_last_known_device_id_persists_and_is_readable` | `update_last_known_device_id_persists_and_is_readable` | `core/src/store/contacts.rs` | 325 | 10 | `WIRE update_last_known_device_id_persists_and_is_readable call path + tests` |
| `update_last_known_device_id_trims_valid_uuid` | `update_last_known_device_id_trims_valid_uuid` | `core/src/store/contacts.rs` | 369 | 10 | `WIRE update_last_known_device_id_trims_valid_uuid call path + tests` |

