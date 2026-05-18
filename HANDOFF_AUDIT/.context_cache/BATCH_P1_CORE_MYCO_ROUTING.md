# REPO_MAP Context for Task: BATCH_P1_CORE_MYCO_ROUTING

**Target function: `BATCH_P1_CORE_MYCO_ROUTING`**

## core/src/routing/adaptive_ttl.rs (1 chunks, 250 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/adaptive_ttl.rs: Defines 6 types: ActivityHistory, Default, ActivityHistory, AdaptiveTTLManager, AdaptiveTTLManager; 21 functions; 3 imports

### Structs/Classes
- ActivityHistory
- AdaptiveTTLManager
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 28 | from_secs, new, now, calculate_ttl |
| `new` | 35 | from_secs, now, calculate_ttl |
| `record_message` | 44 | new, now, calculate_ttl |
| `calculate_ttl` | 50 | from_secs, new, calculate_ttl |
| `decay` | 64 | from_secs, new, calculate_ttl |
| `new` | 87 | from_secs, new |
| `with_defaults` | 97 | from_secs, new |
| `calculate_ttl` | 106 |  |
| `record_activity` | 118 | with_defaults |
| `get_activity` | 125 | from_secs, with_defaults |
| `cleanup` | 130 | from_secs, with_defaults |
| `len` | 138 | from_secs, with_defaults |
| `is_empty` | 143 | from_secs, with_defaults |
| `calculate_dynamic_ttl` | 150 | from_secs, with_defaults |
| `default` | 163 | from_secs, with_defaults |
| `test_adaptive_ttl_creation` | 173 | from_millis, from_secs, new, with_defaults |
| `test_inactive_peer_ttl` | 180 | sleep, new, from_millis, from_secs, with_defaults |
| `test_active_peer_ttl` | 187 | sleep, new, from_millis, from_secs, with_defaults |
| `test_moderate_peer_ttl` | 200 | sleep, new, from_nanos, from_millis, from_secs, with_defaults |
| `test_activity_decay` | 213 | sleep, new, from_nanos, from_millis, from_secs, with_defaults |
| `test_cleanup_old_entries` | 236 | from_nanos, with_defaults |

### Imports
- `use std::collections::HashMap`
- `use super::*`
- `use web_time::{Duration, Instant}`
---

## core/src/mobile/auto_adjust.rs (1 chunks, 512 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/mobile/auto_adjust.rs: Defines 13 types: AdjustmentProfile, std, DeviceProfile, MotionState, BleAdjustment; 31 functions; 2 imports

### Structs/Classes
- AdjustmentProfile
- AdjustmentResult
- AutoAdjustEngine
- BleAdjustment
- Default
- DeviceProfile
- ManualOverride
- MotionState
- RelayAdjustment
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `fmt` | 18 |  |
| `default` | 57 |  |
| `default` | 74 |  |
| `new` | 99 |  |
| `get_adjustment_profile` | 107 |  |
| `apply_ble_adjustments` | 167 | default |
| `apply_relay_adjustments` | 205 | default |
| `override_ble_scan_interval` | 238 | default, apply_ble_adjustments, get_adjustment_profile |
| `override_ble_advertise_interval` | 243 | default, apply_ble_adjustments, get_adjustment_profile, apply_relay_adjustments |
| `override_relay_max_per_hour` | 248 | default, apply_ble_adjustments, get_adjustment_profile, apply_relay_adjustments |
| `override_relay_priority_threshold` | 253 | get_adjustment_profile, new, apply_relay_adjustments, default, apply_ble_adjustments |
| `clear_overrides` | 258 | get_adjustment_profile, new, apply_relay_adjustments, default, apply_ble_adjustments |
| `get_overrides` | 263 | apply_ble_adjustments, get_adjustment_profile, new, apply_relay_adjustments |
| `get_last_profile` | 268 | apply_ble_adjustments, get_adjustment_profile, new, apply_relay_adjustments |
| `compute_adjustments` | 282 | apply_ble_adjustments, get_adjustment_profile, new, apply_relay_adjustments |
| `test_critical_battery` | 299 | new |
| `test_screen_on_high_battery` | 314 | new |
| `test_charging_with_wifi` | 329 | new |
| `test_charging_without_wifi` | 344 | new |
| `test_automotive_mode` | 359 | new |
| `test_ble_adjustments_maximum` | 374 | default, new |
| `test_ble_adjustments_minimal` | 383 | default, new |
| `test_relay_adjustments_maximum` | 392 | default, new |
| `test_relay_adjustments_minimal` | 401 | default, new |
| `test_manual_override_ble_scan` | 410 | default, new |
| `test_manual_override_relay` | 421 | default, new |
| `test_override_setter_methods` | 432 | new |
| `test_clear_overrides` | 445 | new |
| `test_compute_adjustments` | 455 | new |
| `test_low_battery_walking` | 472 | new |
| `test_motion_state_still_vs_walking` | 487 | new |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
---

## core/src/abuse/auto_block.rs (1 chunks, 336 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/abuse/auto_block.rs: Defines 7 types: AutoBlockConfig, Default, AutoBlockReason, AutoBlockAuditEntry, AutoBlockResult; 19 functions; 11 imports

### Structs/Classes
- AutoBlockAuditEntry
- AutoBlockConfig
- AutoBlockEngine
- AutoBlockReason
- AutoBlockResult
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 35 | new |
| `new` | 97 | new |
| `evaluate` | 112 | evaluate |
| `evaluate_and_block` | 161 | evaluate, new |
| `exempt_peer` | 195 | evaluate_and_block |
| `unexempt_peer` | 200 | evaluate_and_block, now |
| `is_exempt` | 205 | evaluate_and_block, now |
| `audit_log` | 214 | default, new, evaluate_and_block, now |
| `update_config` | 219 | default, new, evaluate_and_block, now |
| `config` | 224 | default, new, evaluate_and_block, now |
| `evaluate_all_tracked` | 230 | default, new, evaluate_and_block, now |
| `current_epoch_secs` | 243 | default, new, now |
| `make_engine` | 258 | default, new |
| `test_default_config` | 273 | default |
| `test_exempt_peer_not_blocked` | 281 | default |
| `test_unexempt_peer` | 290 | default |
| `test_audit_log_records_block` | 298 | default |
| `test_disabled_auto_block` | 316 | default |
| `test_neutral_peer_not_blocked` | 326 |  |

### Imports
- `use crate::abuse::reputation::EnhancedAbuseReputationManager`
- `use crate::abuse::spam_detection::{SpamDetectionConfig, SpamDetectionEngine}`
- `use crate::store::backend::MemoryStorage`
- `use crate::store::blocked::BlockedManager`
- `use crate::store::contacts::ContactManager`
- `use crate::transport::reputation::ReputationScore`
- `use parking_lot::RwLock`
- `use serde::{Deserialize, Serialize}`
- `use std::sync::Arc`
- `use std::time::SystemTime`
- `use super::*`
---

## core/src/store/backend.rs (1 chunks, 309 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/store/backend.rs: Defines 12 types: ScanResult, StorageBackend, MemoryStorage, MemoryStorage, Default; 31 functions; 7 imports

### Structs/Classes
- Default
- IndexedDbStorage
- MemoryStorage
- ScanResult
- SledStorage
- StorageBackend

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `put` | 10 | new |
| `get` | 11 | new |
| `remove` | 12 | new |
| `scan_prefix` | 13 | new |
| `count_prefix` | 14 | new |
| `flush` | 15 | new |
| `new` | 25 | new |
| `default` | 33 | new |
| `put` | 39 | new |
| `get` | 46 | default, new |
| `remove` | 50 | default, new |
| `scan_prefix` | 55 | default, new |
| `count_prefix` | 65 | default |
| `flush` | 76 | default, new |
| `new` | 89 | default, new |
| `put` | 102 | builder, new |
| `get` | 106 | builder, new |
| `remove` | 111 | builder, new |
| `scan_prefix` | 116 | builder, new |
| `count_prefix` | 125 | builder, new |
| `flush` | 129 | builder, new |
| `new` | 146 | channel, builder, new |
| `new_sync` | 187 | channel, builder, from, new, spawn_local |
| `persist_put` | 207 | from, builder, persist_put, spawn_local |
| `persist_remove` | 227 | builder, persist_put, from, persist_remove, new, spawn_local |
| `put` | 249 | persist_put, new, persist_remove |
| `get` | 257 | persist_remove, new |
| `remove` | 261 | persist_remove, new |
| `scan_prefix` | 267 | new |
| `count_prefix` | 277 |  |
| `flush` | 288 |  |

### Imports
- `use futures::channel::oneshot`
- `use futures::executor::block_on`
- `use js_sys::wasm_bindgen::JsCast`
- `use rexie::*`
- `use std::collections::HashMap`
- `use std::sync::{Arc, RwLock}`
- `use wasm_bindgen_futures::spawn_local`
---

## core/src/crypto/backup.rs (1 chunks, 211 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/crypto/backup.rs: 8 functions; 5 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `derive_key` | 33 | encode, hash, with_capacity, thread_rng, from_slice, new_from_slice |
| `encrypt_backup` | 59 | encode, with_capacity, decode, thread_rng, from_slice, new_from_slice |
| `decrypt_backup` | 97 | decode, from_utf8, new_from_slice, from_slice |
| `test_encrypt_decrypt_roundtrip` | 128 | encode |
| `test_decrypt_wrong_passphrase_fails` | 139 | encode |
| `test_decrypt_invalid_hex_fails` | 150 | encode |
| `test_decrypt_truncated_data_fails` | 156 | encode |
| `test_different_passphrases_produce_different_ciphertexts` | 164 |  |

### Imports
- `use crate::IronCoreError`
- `use pbkdf2::pbkdf2_hmac`
- `use rand::RngCore`
- `use sha2::Sha256`
- `use super::*`
---

## core/src/store/blocked.rs (1 chunks, 807 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/store/blocked.rs: Defines 4 types: BlockedIdentity, BlockedIdentity, BlockedManager, BlockedManager; 19 functions; 6 imports

### Structs/Classes
- BlockedIdentity
- BlockedManager

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 40 | new |
| `full_relay` | 54 | new, to_vec |
| `with_device_id` | 60 | to_vec |
| `with_reason` | 66 | block, new, to_vec |
| `storage_key` | 72 | block, new, to_vec |
| `new` | 87 | block, new, to_vec |
| `block` | 92 | block, new, to_vec |
| `block_and_delete` | 106 | new, block |
| `unblock` | 120 |  |
| `is_blocked` | 132 | from_slice |
| `get` | 161 | new, Reverse, from_slice, list |
| `list` | 185 | Reverse, new, from_slice, list |
| `count` | 203 | from_slice, now, list |
| `is_blocked_and_deleted` | 211 | new, from_slice, now, list |
| `blocked_only_peer_ids` | 230 | new, now, list |
| `current_timestamp` | 241 | new, now |
| `test_block_unblock` | 255 | new |
| `test_device_specific_block` | 277 | new |
| `test_list_blocked` | 296 | new |

### Imports
- `use crate::IronCoreError`
- `use crate::store::backend::MemoryStorage`
- `use crate::store::backend::StorageBackend`
- `use serde::{Deserialize, Serialize}`
- `use std::sync::Arc`
- `use super::*`
---

## core/src/blocked_bridge.rs (1 chunks, 161 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/blocked_bridge.rs: Defines 6 types: BlockedIdentity, From, From, BlockedManager, Default; 14 functions; 2 imports

### Structs/Classes
- BlockedIdentity
- BlockedManager
- Default
- From

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `from` | 24 | new |
| `from` | 37 | new |
| `default` | 55 | from, new |
| `new` | 61 | from, new |
| `block` | 70 | from, new |
| `unblock` | 75 | from, new |
| `is_blocked` | 80 | from, new |
| `get` | 89 | from, new |
| `list` | 101 | new |
| `count` | 107 | new |
| `blocked_identity_new` | 113 | new |
| `blocked_identity_with_device_id` | 116 |  |
| `blocked_identity_with_reason` | 125 |  |
| `blocked_identity_with_notes` | 131 |  |

### Imports
- `use crate::IronCoreError`
- `use std::sync::Arc`
---

## core/src/relay/bootstrap.rs (1 chunks, 470 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/bootstrap.rs: Defines 8 types: BootstrapError, BootstrapMethod, SeedPeer, SeedPeer, InvitePayload; 39 functions; 4 imports

### Structs/Classes
- BootstrapError
- BootstrapManager
- BootstrapMethod
- InvitePayload
- SeedPeer

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 50 | now |
| `new` | 78 | to_string, serialize, now, SerializationError, deserialize |
| `with_group_key` | 95 | to_string, serialize, now, SerializationError, deserialize, from_str |
| `with_expiry` | 101 | to_string, serialize, now, SerializationError, deserialize, from_str |
| `is_valid` | 107 | to_string, serialize, now, SerializationError, new, deserialize, from_str |
| `to_bytes` | 117 | to_string, serialize, SerializationError, new, deserialize, from_str |
| `from_bytes` | 122 | to_string, SerializationError, new, deserialize, from_str |
| `to_json` | 127 | to_string, SerializationError, new, from_str |
| `from_json` | 132 | SerializationError, new, from_str |
| `new` | 151 | generate_invite, new |
| `with_seed_peers` | 161 | generate_invite, accept_invite, from_bytes, new |
| `get_seed_peers` | 167 | generate_invite, accept_invite, from_bytes, new |
| `generate_invite` | 175 | generate_invite, accept_invite, from_bytes, new |
| `generate_qr_data` | 184 | accept_invite, from_bytes, generate_invite |
| `accept_invite` | 190 | new, accept_invite, from_bytes |
| `parse_qr_data` | 203 | new, accept_invite, from_bytes |
| `get_peer_addresses` | 209 | new |
| `set_addresses` | 220 | new |
| `get_addresses` | 225 | new |
| `test_seed_peer` | 237 | new |
| `test_bootstrap_manager` | 245 | new |
| `test_seed_peer_creation` | 255 | from_bytes, new |
| `test_invite_payload_creation` | 262 | from_bytes, new, from_json |
| `test_invite_payload_with_group_key` | 275 | sleep, from_bytes, new, from_json, from_millis |
| `test_invite_payload_serialization` | 288 | sleep, from_bytes, new, from_json, from_millis |
| `test_invite_payload_json_serialization` | 303 | from_millis, sleep, new, from_json |
| `test_invite_payload_expiry` | 317 | from_millis, sleep, new |
| `test_bootstrap_manager_creation` | 330 | new |
| `test_bootstrap_manager_with_seed_peers` | 337 | new |
| `test_get_seed_peers` | 350 | from_bytes |
| `test_get_seed_peers_empty` | 360 | new, from_bytes |
| `test_generate_invite` | 368 | new, from_bytes |
| `test_generate_qr_data` | 380 | new, sleep, from_bytes, from_millis |
| `test_parse_qr_data` | 394 | from_millis, sleep, new |
| `test_accept_invite_valid` | 407 | from_millis, sleep, new |
| `test_accept_invite_expired` | 420 | from_millis, sleep, new |
| `test_set_addresses` | 436 | new |
| `test_get_peer_addresses` | 446 | new |
| `test_bootstrap_method_enum` | 459 |  |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/transport/bootstrap.rs (1 chunks, 728 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/bootstrap.rs: Defines 6 types: BootstrapConfig, Default, BootstrapState, BootstrapNode, BootstrapManager; 37 functions; 10 imports

### Structs/Classes
- BootstrapConfig
- BootstrapManager
- BootstrapNode
- BootstrapState
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 68 | default, from_secs |
| `new` | 126 | default, new |
| `with_defaults` | 164 | default, new |
| `state` | 169 |  |
| `connected_count` | 174 |  |
| `total_nodes` | 179 | next_connectable_node |
| `relay_discovery` | 184 | next_connectable_node, record_failure |
| `relay_discovery_mut` | 189 | backoff_for_node, next_connectable_node, record_failure |
| `add_bootstrap_node` | 194 | backoff_for_node, next_connectable_node, record_failure |
| `bootstrap` | 217 | random, sleep, backoff_for_node, next_connectable_node, record_attempt, record_success, record_failure |
| `record_success` | 349 | from_secs_f64, now |
| `record_failure` | 358 | from_secs_f64, new, now |
| `record_attempt` | 365 | from_secs_f64, new, now |
| `backoff_for_node` | 374 | from_secs_f64, new |
| `next_connectable_node` | 388 | new |
| `discover_fallback_nodes` | 400 | Ws, new, Wss |
| `is_websocket_address` | 440 | Ws, from_multiaddr, Wss |
| `try_websocket_connection` | 451 | from_multiaddr |
| `circuit_breaker` | 477 | var, new |
| `get_healthy_relays` | 484 | var, new |
| `get_all_relay_stats` | 490 | var, new |
| `get_fallback_relay_addresses` | 500 | var, new |
| `reset_circuit_breakers` | 505 | var, new |
| `resolve_env_bootstrap_nodes` | 511 | var, new |
| `discover_dns_bootstrap` | 525 | new |
| `discover_local_peers` | 545 | new |
| `discover_websocket_bootstrap` | 559 | new |
| `discover_hardcoded_backup_relays` | 600 | default, from_secs, new, with_defaults |
| `test_bootstrap_config_defaults` | 631 | now, new, default, from_secs, with_defaults |
| `test_bootstrap_manager_creation` | 642 | now, new, default, from_secs, with_defaults |
| `test_bootstrap_manager_add_node` | 650 | now, new, default, from_secs, with_defaults |
| `test_bootstrap_manager_no_duplicate` | 659 | now, new, default, from_secs, with_defaults |
| `test_exponential_backoff` | 669 | default, from_secs, new, now |
| `test_env_bootstrap_override` | 688 |  |
| `test_dns_discovery` | 697 |  |
| `test_local_discovery` | 706 |  |
| `test_websocket_discovery` | 713 |  |

### Imports
- `use crate::transport::circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager}`
- `use crate::transport::internet::{InternetRelay, InternetTransportError}`
- `use crate::transport::relay_health::{RelayDiscovery, RelayFallback, RelayMetrics}`
- `use crate::transport::swarm::SwarmHandle`
- `use crate::transport::websocket::{diagnose_websocket_error, WebSocketTransport}`
- `use libp2p::{Multiaddr, PeerId}`
- `use std::collections::VecDeque`
- `use super::*`
- `use tracing::{debug, info, warn}`
- `use web_time::{Duration, SystemTime}`
---

## core/src/privacy/circuit.rs (1 chunks, 529 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/privacy/circuit.rs: Defines 11 types: CircuitId, CircuitId, CircuitError, PeerInfo, CircuitPath; 29 functions; 8 imports

### Structs/Classes
- CircuitBuilder
- CircuitConfig
- CircuitError
- CircuitId
- CircuitPath
- Default
- PeerInfo

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `random` | 16 | from_le_bytes, thread_rng, new |
| `public_keys` | 63 | new |
| `hop_count` | 88 |  |
| `default` | 107 | select_hop_count |
| `validate` | 119 | select_hop_count, select_random_hops, select_diverse_hops, random |
| `new` | 143 | select_hop_count, select_random_hops, select_diverse_hops, random, thread_rng |
| `build_circuit` | 149 | select_hop_count, select_random_hops, select_diverse_hops, random, new, thread_rng |
| `select_hop_count` | 172 | thread_rng, new |
| `select_diverse_hops` | 179 | new |
| `select_random_hops` | 244 |  |
| `peers` | 275 | random, serialize, deserialize |
| `update_peer_reliability` | 280 | default, random, serialize, deserialize |
| `create_test_peers` | 299 | default, random, serialize, deserialize |
| `test_circuit_id_random` | 312 | default, random, serialize, deserialize |
| `test_circuit_id_serialization` | 319 | default, random, serialize, deserialize |
| `test_circuit_config_default` | 327 | default, random |
| `test_circuit_config_invalid_min_hops` | 335 | random |
| `test_circuit_config_invalid_order` | 346 | random |
| `test_circuit_path_public_keys` | 357 | default, random, new |
| `test_circuit_path_public_keys_invalid_length` | 374 | default, random, new |
| `test_circuit_path_hop_count` | 388 | default, random, new |
| `test_circuit_builder_new` | 402 | default, new |
| `test_circuit_builder_invalid_config` | 410 | default, new |
| `test_circuit_builder_insufficient_peers` | 423 | default, new |
| `test_circuit_builder_build_circuit` | 445 | default, new |
| `test_circuit_builder_diverse_paths` | 464 | default, new |
| `test_circuit_builder_update_reliability` | 493 | serialize, random, new, default, deserialize |
| `test_circuit_builder_update_reliability_nonexistent` | 503 | serialize, random, new, default, deserialize |
| `test_circuit_path_serialization` | 513 | random, serialize, deserialize |

### Imports
- `use rand::Rng`
- `use rand::RngCore`
- `use rand::seq::SliceRandom`
- `use rand::thread_rng`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use super::*`
- `use thiserror::Error`
---

## core/src/relay/client.rs (1 chunks, 1039 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/client.rs: Defines 9 types: TransportType, RelayClientConfig, Default, ConnectionState, RelayConnection; 41 functions; 15 imports

### Structs/Classes
- ConnectionState
- Default
- RelayClient
- RelayClientConfig
- RelayClientError
- RelayConnection
- TransportType

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 56 | from_secs, new |
| `new` | 102 | now |
| `with_transport` | 114 | now |
| `set_state` | 126 | now |
| `is_connected` | 139 | new, full_relay |
| `new` | 184 | new, connect_tcp, full_relay, connect_quic |
| `set_capabilities` | 198 | connect_tcp, connect_websocket, connect_quic |
| `create_handshake` | 203 | connect_tcp, connect_websocket, connect_quic |
| `connect` | 214 | connect_tcp, connect_websocket, connect_quic |
| `connect_websocket` | 263 | connect_async, IoTimeout, with_transport, create_handshake, into_client_request, ConnectionFailed |
| `connect_websocket` | 362 | new, IoTimeout, send_and_receive_raw, connect, with_transport, complete_handshake, clone, create_handshake, ConnectionFailed |
| `connect_tcp` | 374 | new, IoTimeout, send_and_receive_raw, connect, with_transport, complete_handshake, clone, create_handshake, ConnectionFailed |
| `connect_quic` | 406 | client, try_with_platform_verifier, new, IoTimeout, with_transport, ConnectionFailed |
| `complete_handshake` | 509 | HandshakeFailed, ConnectionFailed |
| `connect_quic` | 541 | send_and_receive_raw, MessageError, ConnectionFailed |
| `push_envelopes` | 551 | send_and_receive_raw, MessageError, ConnectionFailed |
| `pull_envelopes` | 594 | send_and_receive_raw, MessageError, new, ConnectionFailed |
| `active_connections` | 645 | MessageError |
| `add_connection` | 655 | from_millis, min, MessageError |
| `remove_connection` | 660 | from_millis, min, MessageError |
| `send_ping` | 667 | from_millis, min, MessageError, SerializationError |
| `get_relay_addresses` | 694 | SerializationError, IoTimeout, MessageError, from_millis, min, ConnectionFailed |
| `backoff_duration` | 699 | SerializationError, IoTimeout, MessageError, from_bytes, from_millis, min, ConnectionFailed |
| `send_and_receive_raw` | 704 | MessageError, from_bytes, SerializationError, IoTimeout, ConnectionFailed |
| `test_client` | 757 | bind, spawn, new, from_bytes, default, mobile |
| `test_relay_client_creation` | 763 | bind, spawn, full_relay, from_bytes, mobile |
| `test_create_handshake` | 770 | bind, spawn, full_relay, from_bytes, mobile |
| `test_set_capabilities` | 789 | bind, spawn, full_relay, from_bytes, new, mobile |
| `test_connect_to_relay` | 798 | bind, spawn, full_relay, new, from_bytes |
| `test_relay_connection_creation` | 829 | full_relay, new |
| `test_relay_connection_state_transitions` | 837 | full_relay, new |
| `test_complete_handshake_success` | 855 | full_relay, new |
| `test_complete_handshake_version_mismatch` | 872 | full_relay, new |
| `test_push_envelopes_not_connected` | 887 | new |
| `test_pull_envelopes_not_connected` | 899 | spawn, new, bind |
| `test_active_connections` | 911 | bind, spawn, full_relay, from_bytes, new |
| `test_remove_connection` | 928 | bind, spawn, full_relay, from_bytes, new |
| `test_push_pull_and_ping_over_network` | 942 | full_relay, spawn, from_bytes, bind |
| `test_send_ping_not_connected` | 996 | default, new |
| `test_backoff_duration` | 1008 | default, new |
| `test_get_relay_addresses` | 1025 | default, new |

### Imports
- `use futures::{SinkExt, StreamExt}`
- `use quinn`
- `use std::collections::HashMap`
- `use std::net::SocketAddr`
- `use std::sync::Arc`
- `use super::*`
- `use super::protocol::{RelayCapability, RelayMessage, PROTOCOL_VERSION}`
- `use thiserror::Error`
- `use tokio::io::{AsyncReadExt, AsyncWriteExt}`
- `use tokio::net::TcpStream`
- `use tokio::sync::{Mutex, RwLock}`
- `use tokio::time::timeout`
- `use tokio_tungstenite::tungstenite::client::IntoClientRequest`
- `use tokio_tungstenite::tungstenite::protocol::Message`
- `use web_time::{Duration, SystemTime, UNIX_EPOCH}`
---

## core/src/message/codec.rs (1 chunks, 387 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/message/codec.rs: 11 functions; 6 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `validate_payload_size` | 14 | serialize, deserialize |
| `encode_message` | 26 | serialize, deserialize |
| `decode_message` | 43 | serialize, deserialize, text |
| `encode_envelope` | 57 | serialize, deserialize, text |
| `decode_envelope` | 72 | deserialize, text |
| `test_message_roundtrip` | 91 | text |
| `test_reject_oversized_payload` | 101 | thread_rng, text |
| `test_payload_boundary_accepts_8191_and_8192` | 111 | thread_rng, text |
| `test_payload_boundary_rejects_8193` | 122 | thread_rng, text |
| `test_reject_oversized_decode` | 129 | thread_rng, from_bytes |
| `test_envelope_roundtrip` | 136 | thread_rng, from_bytes |

### Imports
- `use anyhow::{bail, Result}`
- `use crate::drift::DriftEnvelope`
- `use crate::message::types::Message`
- `use rand::RngCore`
- `use super::*`
- `use super::types::{Envelope, Message}`
---

## core/src/drift/compress.rs (1 chunks, 106 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/compress.rs: 10 functions; 2 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `compress` | 8 | compress_prepend_size, decompress_size_prepended, DecompressionFailed |
| `decompress` | 15 | decompress_size_prepended, DecompressionFailed |
| `test_compress_decompress_roundtrip` | 25 |  |
| `test_compress_empty_data` | 34 |  |
| `test_compress_single_byte` | 43 |  |
| `test_compress_large_data` | 52 |  |
| `test_decompress_invalid_data` | 62 |  |
| `test_compress_repetitive_data` | 68 |  |
| `test_compress_random_data` | 81 |  |
| `test_compress_text_data` | 95 |  |

### Imports
- `use super::*`
- `use super::DriftError`
---

## cli/src/contacts.rs (2 chunks, 254 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/contacts.rs: Defines 4 types: Contact, Contact, ContactList, ContactList; 18 functions; 5 imports cli/src/contacts.rs: Defines 4 types: Contact, Contact, ContactList, ContactList; 18 functions; 5 imports

### Structs/Classes
- Contact
- ContactList

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 35 | default, to_vec |
| `with_nickname` | 45 | default, from_slice, to_vec |
| `display_name` | 50 | default, from_slice, to_vec |
| `open` | 62 | default, new, from_slice, to_vec |
| `add` | 73 | new, from_slice, to_vec |
| `get` | 85 | new, from_slice |
| `remove` | 96 | get, add, new, from_slice |
| `list` | 102 | get, add, new, from_slice |
| `find_by_nickname` | 119 | get, add, from_slice |
| `find_by_public_key` | 131 | get, add, from_slice |
| `update_last_seen` | 143 | get, add, new, from_slice |
| `set_nickname` | 153 | get, add, new, from_slice |
| `set_notes` | 165 | get, now, add, new, from_slice |
| `count` | 176 | now, new, from_slice |
| `search` | 181 | now, new, from_slice |
| `current_timestamp` | 209 | tempdir, new, now, open |
| `test_contact_creation` | 222 | tempdir, new, open |
| `test_contact_list` | 231 | new, tempdir, open |
| `new` | 35 | to_vec, default |
| `with_nickname` | 45 | from_slice, to_vec, default |
| `display_name` | 50 | from_slice, to_vec, default |
| `open` | 62 | from_slice, to_vec, new, default |
| `add` | 73 | new, to_vec, from_slice |
| `get` | 85 | new, from_slice |
| `remove` | 96 | new, from_slice, add, get |
| `list` | 102 | new, from_slice, add, get |
| `find_by_nickname` | 119 | from_slice, add, get |
| `find_by_public_key` | 131 | from_slice, add, get |
| `update_last_seen` | 143 | new, from_slice, add, get |
| `set_nickname` | 153 | new, from_slice, add, get |
| `set_notes` | 165 | new, now, get, from_slice, add |
| `count` | 176 | new, from_slice, now |
| `search` | 181 | new, from_slice, now |
| `current_timestamp` | 209 | new, tempdir, open, now |
| `test_contact_creation` | 222 | new, tempdir, open |
| `test_contact_list` | 231 | new, tempdir, open |

### Imports
- `use anyhow::{Context, Result}`
- `use serde::{Deserialize, Serialize}`
- `use sled::Db`
- `use std::path::PathBuf`
- `use super::*`
---

## core/src/store/contacts.rs (1 chunks, 411 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/store/contacts.rs: Defines 4 types: Contact, Contact, ContactManager, ContactManager; 26 functions; 8 imports

### Structs/Classes
- Contact
- ContactManager

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 28 | get, add, derive_public_key_from_peer_id, new |
| `with_nickname` | 40 | get, add, derive_public_key_from_peer_id, new |
| `display_name` | 45 | get, add, derive_public_key_from_peer_id, new, decode |
| `new` | 60 | get, add, derive_public_key_from_peer_id, new, from_bytes, decode |
| `reconcile_from_history` | 66 | get, add, derive_public_key_from_peer_id, new, from_bytes, decode |
| `derive_public_key_from_peer_id` | 84 | decode, encode, from_bytes, to_vec |
| `add` | 128 | new, from_slice, to_vec |
| `get` | 137 | new, from_slice, list |
| `remove` | 152 | new, from_slice, list |
| `list` | 160 | get, list, add, new, from_slice |
| `search` | 177 | get, add, list |
| `set_nickname` | 200 | get, add |
| `set_local_nickname` | 216 | get, add, parse_str |
| `update_last_seen` | 232 | get, count, add, parse_str |
| `update_last_known_device_id` | 248 | get, count, add, parse_str |
| `count` | 270 | count, new, now |
| `flush` | 274 | count, new, now |
| `verify_integrity` | 281 | count, new, now |
| `current_timestamp` | 300 | new, now |
| `make_manager` | 313 | new, from_str |
| `contact_new_has_no_last_known_device_id` | 319 | new, from_str |
| `update_last_known_device_id_persists_and_is_readable` | 325 | new, from_str |
| `update_last_known_device_id_can_clear` | 344 | new, from_str |
| `contact_roundtrips_through_serde_with_default_device_id` | 358 | new, from_str |
| `update_last_known_device_id_trims_valid_uuid` | 369 | new |
| `update_last_known_device_id_ignores_invalid_values` | 388 | new |

### Imports
- `use crate::IronCoreError`
- `use crate::store::backend::MemoryStorage`
- `use crate::store::backend::StorageBackend`
- `use crate::store::history::HistoryManager`
- `use serde::{Deserialize, Serialize}`
- `use std::sync::Arc`
- `use super::*`
---

## core/src/privacy/cover.rs (1 chunks, 527 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/privacy/cover.rs: Defines 10 types: CoverTrafficError, CoverConfig, Default, CoverConfig, CoverMessage; 41 functions; 5 imports

### Structs/Classes
- CoverConfig
- CoverMessage
- CoverTrafficError
- CoverTrafficGenerator
- CoverTrafficScheduler
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 31 | InvalidConfig |
| `validate` | 42 | InvalidConfig |
| `message_interval_ms` | 62 | InvalidConfig |
| `is_cover_traffic` | 89 | generate_cover_message, thread_rng, InvalidConfig |
| `new` | 101 | generate_cover_message, thread_rng, InvalidConfig |
| `generate_cover_message` | 107 | generate_cover_message, thread_rng, InvalidConfig |
| `generate_batch` | 137 | new, from_millis, generate_cover_message, now |
| `config` | 142 | from_millis, new, now |
| `new` | 155 | from_millis, new, now |
| `should_generate_cover_traffic` | 166 | from_millis, new, now |
| `generate_and_update` | 181 | default, from_millis, new, now |
| `reset_timer` | 189 | default, from_millis |
| `next_generation_time` | 194 | default, from_millis |
| `config` | 204 | default |
| `is_cover_traffic` | 213 | default |
| `test_cover_config_default` | 224 | default |
| `test_cover_config_validate` | 232 |  |
| `test_cover_config_validate_zero_rate_enabled` | 242 |  |
| `test_cover_config_validate_zero_rate_disabled` | 252 |  |
| `test_cover_config_validate_zero_message_size` | 262 |  |
| `test_cover_config_validate_excessive_message_size` | 272 | default, new |
| `test_cover_config_message_interval` | 282 | default, new |
| `test_cover_config_message_interval_low_rate` | 292 | default, new |
| `test_cover_message_creation` | 302 | default, new |
| `test_cover_traffic_generator_new` | 316 | default, new |
| `test_cover_traffic_generator_invalid_config` | 323 | new |
| `test_cover_traffic_generator_disabled` | 334 | new |
| `test_generate_cover_message` | 346 | new |
| `test_generate_cover_message_uniqueness` | 362 | default, new |
| `test_generate_batch` | 379 | default, new |
| `test_cover_traffic_scheduler_new` | 396 | default, new |
| `test_cover_traffic_scheduler_should_generate_initially` | 403 | default, new |
| `test_cover_traffic_scheduler_disabled` | 415 | default, new |
| `test_cover_traffic_scheduler_generate_and_update` | 426 | default, new |
| `test_cover_traffic_scheduler_reset_timer` | 442 | default, deserialize, serialize, new |
| `test_cover_traffic_scheduler_next_generation_time` | 452 | deserialize, serialize, new |
| `test_cover_traffic_scheduler_next_generation_time_disabled` | 464 | deserialize, serialize, new |
| `test_is_cover_traffic` | 476 | new, serialize, deserialize |
| `test_cover_config_serialization` | 482 | new, serialize, deserialize |
| `test_cover_message_serialization` | 498 | new, serialize, deserialize |
| `test_cover_traffic_various_sizes` | 515 | new |

### Imports
- `use rand::RngCore`
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
- `use web_time::{Duration, SystemTime}`
---

## core/src/store/dedup.rs (1 chunks, 212 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/store/dedup.rs: Defines 6 types: DedupStats, DedupStats, DedupStatsTracker, DedupStatsTracker, Default; 16 functions; 2 imports

### Structs/Classes
- DedupAggregateStats
- DedupStats
- DedupStatsTracker
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 26 | new, now |
| `record_duplicate` | 36 | new, now |
| `new` | 53 | new, now |
| `with_max_entries` | 61 | new, now |
| `record_received` | 71 | new, now |
| `get_dedup_stats` | 105 | new |
| `len` | 110 | new |
| `is_empty` | 115 | new |
| `clear` | 120 | new |
| `aggregate` | 125 | new |
| `default` | 142 | new |
| `dedup_stats_records_first_seen` | 160 | with_max_entries, new |
| `dedup_stats_increments_duplicates` | 169 | with_max_entries, new |
| `get_dedup_stats_returns_none_for_unknown` | 182 | with_max_entries, new |
| `aggregate_stats_are_correct` | 188 | with_max_entries, new |
| `tracker_evicts_oldest_at_capacity` | 201 | with_max_entries |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
---

## core/src/crypto/encrypt.rs (1 chunks, 715 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/crypto/encrypt.rs: 30 functions; 9 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `ed25519_to_x25519_secret` | 37 | from, from_slice, decode |
| `validate_ed25519_public_key` | 61 | derive_key, decode, from, from_slice |
| `ed25519_public_to_x25519` | 95 | derive_key, from, random_from_rng, from_slice |
| `derive_key` | 110 | random_from_rng, from, from_slice, derive_key, new_from_slice |
| `encrypt_message` | 123 | random_from_rng, from, from_slice, new_from_slice |
| `decrypt_message` | 183 | from, from_slice, new_from_slice |
| `decrypt_message_ratcheted` | 245 |  |
| `encrypt_message_ratcheted` | 288 |  |
| `encrypt_with_ratchet_fallback` | 320 | encode |
| `decrypt_with_ratchet_fallback` | 351 | serialize, encode |
| `is_ratcheted_envelope` | 371 | serialize |
| `sign_envelope` | 386 | serialize, from_bytes |
| `verify_envelope` | 415 | serialize, from_bytes |
| `generate_keypair` | 453 | encode, from_bytes |
| `test_validate_ed25519_public_key_valid` | 463 | encode |
| `test_validate_ed25519_public_key_invalid_hex` | 473 |  |
| `test_validate_ed25519_public_key_wrong_length` | 484 |  |
| `test_encrypt_decrypt_roundtrip` | 496 |  |
| `test_wrong_recipient_fails` | 510 |  |
| `test_tampered_ciphertext_fails` | 525 |  |
| `test_different_messages_different_ciphertext` | 543 |  |
| `test_sender_public_key_in_envelope` | 558 |  |
| `test_empty_plaintext` | 572 |  |
| `test_large_plaintext` | 584 |  |
| `test_invalid_envelope_nonce` | 597 |  |
| `test_aad_binding_prevents_sender_spoofing` | 613 |  |
| `test_sign_and_verify_envelope` | 634 |  |
| `test_tampered_envelope_fails_verification` | 651 |  |
| `test_forged_signature_fails_verification` | 674 |  |
| `test_relay_can_verify_without_decrypting` | 695 |  |

### Imports
- `use anyhow::{bail, Result}`
- `use curve25519_dalek::edwards::CompressedEdwardsY`
- `use ed25519_dalek::SigningKey`
- `use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey, Verifier, VerifyingKey}`
- `use rand::RngCore`
- `use super::*`
- `use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret}`
- `use zeroize::Zeroize`
---

## core/src/routing/engine.rs (1 chunks, 733 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/engine.rs: Defines 7 types: NextHop, RoutingLayer, RoutingDecision, RoutingMaintenance, RoutingSummary; 37 functions; 4 imports

### Structs/Classes
- NextHop
- RoutingDecision
- RoutingEngine
- RoutingLayer
- RoutingMaintenance
- RoutingSummary

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 118 | collect_alternative_hops, new |
| `route_message` | 135 | collect_alternative_hops |
| `route_redundant` | 229 | collect_alternative_hops_count, route_message |
| `is_for_us` | 249 | PeerBecameActive |
| `local_cell` | 256 | PeerBecameStale, PeerBecameActive |
| `local_cell_mut` | 261 | PeerBecameStale, PeerBecameActive |
| `neighborhood` | 266 | PeerBecameStale, PeerBecameActive |
| `neighborhood_mut` | 271 | PeerBecameStale, PeerBecameActive |
| `global_routes` | 276 | PeerBecameStale, PeerBecameActive |
| `global_routes_mut` | 281 | PeerBecameStale, PeerBecameActive, count_reachable_hints |
| `tick` | 292 | PeerBecameStale, PeerBecameActive, count_reachable_hints, new |
| `routing_summary` | 319 | count_reachable_hints, new |
| `collect_alternative_hops` | 337 | new |
| `collect_alternative_hops_count` | 380 | new |
| `count_reachable_hints` | 419 | new |
| `make_peer_id` | 440 | new |
| `make_message_id` | 446 | new |
| `make_hint` | 450 | new |
| `test_routing_engine_creation` | 456 | new |
| `test_is_for_us` | 466 | new |
| `test_route_to_unknown_destination` | 478 | new |
| `test_routing_summary` | 494 | new |
| `test_tick_returns_maintenance` | 504 | new |
| `test_layer_access` | 514 | new |
| `test_route_redundant` | 531 | new |
| `test_high_priority_message_with_no_route` | 546 | new |
| `test_routing_layer_enum` | 562 |  |
| `test_next_hop_direct` | 569 |  |
| `test_next_hop_gateway` | 589 |  |
| `test_next_hop_global_route` | 612 |  |
| `test_next_hop_store_and_carry` | 632 |  |
| `test_next_hop_route_discovery` | 638 |  |
| `test_routing_decision_structure` | 645 |  |
| `test_maintenance_structure` | 665 | new |
| `test_routing_summary_structure` | 680 | spawn, new |
| `test_multiple_routing_decisions_independent` | 699 | spawn, new |
| `test_engine_thread_safety` | 718 | spawn, new |

### Imports
- `use super::*`
- `use super::global::GlobalRoutes`
- `use super::local::{LocalCell, PeerId, TransportType}`
- `use super::neighborhood::NeighborhoodTable`
---

## core/src/drift/envelope.rs (1 chunks, 781 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/envelope.rs: Defines 5 types: DriftEnvelope, EnvelopeType, EnvelopeType, DriftEnvelope, DriftEnvelope; 30 functions; 6 imports

### Structs/Classes
- DriftEnvelope
- EnvelopeType

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `from_u8` | 110 | with_capacity, InvalidEnvelopeType, CiphertextTooLarge, compress |
| `as_u8` | 125 | with_capacity, CiphertextTooLarge, compress |
| `to_bytes` | 141 | with_capacity, CiphertextTooLarge, compress |
| `from_bytes` | 200 | InvalidVersion, from_u8, from_le_bytes |
| `hint_from_public_key` | 333 | hash, now |
| `increment_hop` | 343 | hint_from_public_key, IoError, parse_str, now |
| `is_expired` | 350 | hint_from_public_key, IoError, parse_str, now |
| `to_legacy_envelope` | 364 | hint_from_public_key, IoError, parse_str, now |
| `from_legacy_envelope` | 381 | hint_from_public_key, IoError, parse_str, now |
| `sign` | 456 | new |
| `make_test_envelope` | 501 | from_u8 |
| `test_envelope_type_conversion` | 524 | from_u8, from_bytes |
| `test_envelope_serialize_deserialize` | 554 | from_bytes |
| `test_envelope_empty_ciphertext` | 568 | hint_from_public_key, CiphertextTooLarge, from_bytes |
| `test_envelope_large_ciphertext` | 580 | hint_from_public_key, CiphertextTooLarge, from_bytes |
| `test_envelope_max_ciphertext` | 593 | hint_from_public_key, CiphertextTooLarge, from_bytes |
| `test_envelope_ciphertext_too_large` | 603 | hint_from_public_key, CiphertextTooLarge |
| `test_hint_from_public_key_deterministic` | 612 | hint_from_public_key |
| `test_hint_from_public_key_different_keys` | 621 | hint_from_public_key, from_bytes |
| `test_increment_hop` | 632 | from_bytes |
| `test_is_expired_never_expires` | 645 | InvalidVersion, from_bytes |
| `test_is_expired_in_future` | 653 | InvalidVersion, InvalidEnvelopeType, from_bytes |
| `test_is_expired_in_past` | 661 | InvalidVersion, InvalidEnvelopeType, from_bytes |
| `test_buffer_too_short` | 669 | InvalidVersion, InvalidEnvelopeType, from_bytes |
| `test_invalid_version` | 683 | InvalidVersion, InvalidEnvelopeType, from_bytes |
| `test_invalid_envelope_type` | 692 | InvalidEnvelopeType, from_bytes |
| `test_ciphertext_length_exceeds_buffer` | 702 | from_bytes |
| `test_little_endian_timestamps` | 717 | from_bytes |
| `test_compressed_envelope_roundtrip` | 737 | from_bytes |
| `test_compression_flag_preserved_on_roundtrip` | 767 | from_bytes |

### Imports
- `use crate::message::Envelope`
- `use ed25519_dalek::Signer`
- `use super::*`
- `use super::{DriftError, DRIFT_VERSION}`
- `use uuid::Uuid`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/message/ephemeral.rs (1 chunks, 64 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/message/ephemeral.rs: Defines 1 types: TtlConfig; 3 functions; 2 imports

### Structs/Classes
- TtlConfig

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `is_expired` | 22 | now |
| `test_not_expired` | 36 | now |
| `test_expired` | 50 | now |

### Imports
- `use std::time::{SystemTime, UNIX_EPOCH}`
- `use super::*`
---

## core/src/relay/findmy.rs (1 chunks, 463 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/findmy.rs: Defines 9 types: FindMyConfig, Default, FindMyConfig, WakeUpPayload, WakeUpPayload; 31 functions; 4 imports

### Structs/Classes
- Default
- FindMyBeaconManager
- FindMyConfig
- FindMyError
- WakeUpPayload

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 23 |  |
| `new` | 34 |  |
| `with_key` | 43 |  |
| `with_interval` | 49 |  |
| `new` | 68 | new |
| `encode_wakeup` | 101 | new |
| `decode_wakeup` | 142 | new |
| `is_our_wakeup` | 178 |  |
| `new` | 198 |  |
| `should_broadcast` | 206 |  |
| `generate_beacon` | 220 | default, new |
| `process_beacon` | 238 | default, new |
| `is_our_beacon` | 249 | default, new |
| `default` | 261 | default, new |
| `test_key` | 273 | new |
| `test_wakeup_payload_creation` | 279 | new |
| `test_encode_decode_roundtrip` | 287 | new |
| `test_encode_decode_message_flag_false` | 301 | new |
| `test_encode_produces_22_bytes` | 312 | new |
| `test_is_our_wakeup_match` | 321 | new |
| `test_is_our_wakeup_no_match` | 333 | new |
| `test_is_our_wakeup_invalid_length` | 346 | new |
| `test_beacon_manager_creation` | 357 | new |
| `test_beacon_manager_should_broadcast` | 366 | new |
| `test_beacon_manager_generate_beacon` | 378 | default, new |
| `test_beacon_manager_process_beacon` | 390 | default, new |
| `test_beacon_manager_is_our_beacon` | 409 | default, new |
| `test_find_my_config_default` | 426 | default, new |
| `test_find_my_config_builder` | 434 | new |
| `test_different_keys_produce_different_output` | 444 | new |
| `test_beacon_missing_key_error` | 456 | new |

### Imports
- `use blake3::Hasher`
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
---

## core/src/bin/gen_swift.rs (1 chunks, 122 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/bin/gen_swift.rs: 2 functions; 3 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `main` | 2 | var, from, create_dir_all |
| `main` | 122 |  |

### Imports
- `use camino::Utf8PathBuf`
- `use std::fs`
- `use uniffi_bindgen::bindings::{generate, GenerateOptions, TargetLanguage}`
---

## core/src/routing/global.rs (1 chunks, 798 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/global.rs: Defines 5 types: RouteAdvertisement, RouteRequest, GlobalRoutes, GlobalRoutes, Default; 45 functions; 3 imports

### Structs/Classes
- Default
- GlobalRoutes
- RouteAdvertisement
- RouteRequest

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 77 | new |
| `with_limits` | 88 | new |
| `add_route` | 102 |  |
| `routes_for_hint` | 169 |  |
| `best_route_for_hint` | 182 |  |
| `request_route` | 215 |  |
| `is_route_pending` | 228 |  |
| `increment_route_request_attempts` | 235 |  |
| `resolve_route_request` | 245 |  |
| `update_local_advertisements` | 253 |  |
| `get_advertisements` | 277 |  |
| `cleanup` | 284 |  |
| `remove_routes_via` | 311 | new |
| `route_count` | 330 | new |
| `has_route_for` | 335 | new |
| `unique_destination_count` | 340 | new |
| `pending_request_count` | 345 | new |
| `pending_requests` | 350 | new |
| `default` | 356 | new |
| `make_peer_id` | 364 | new |
| `make_hint` | 370 | new |
| `make_route` | 374 | new |
| `test_add_route_basic` | 395 | new |
| `test_add_multiple_routes_same_destination` | 409 | new |
| `test_best_route_selection_by_hops` | 426 | new |
| `test_best_route_selection_by_reliability` | 444 | new |
| `test_best_route_selection_by_recency` | 463 | new |
| `test_reject_stale_route_update` | 481 | with_limits, new |
| `test_accept_newer_route_update` | 500 | with_limits, new |
| `test_max_routes_per_hint` | 519 | with_limits |
| `test_max_routes_per_hint_replacement` | 536 | with_limits, new |
| `test_max_total_routes` | 555 | with_limits, new |
| `test_cleanup_expired_routes` | 577 | new |
| `test_remove_routes_via_peer` | 602 | new |
| `test_route_request_creation` | 626 | new |
| `test_route_request_attempts` | 640 | new |
| `test_route_request_resolution` | 658 | new |
| `test_update_local_advertisements` | 670 | new |
| `test_local_advertisements_cleared_on_update` | 689 | new |
| `test_unique_destination_count` | 702 | new |
| `test_routes_for_unknown_hint` | 717 | default, new |
| `test_pending_requests_cleanup` | 727 | default, with_limits, new |
| `test_pending_requests_getter` | 746 | default, with_limits, new |
| `test_default_construction` | 759 | default, with_limits |
| `test_complex_routing_scenario` | 767 | with_limits |

### Imports
- `use std::collections::HashMap`
- `use super::*`
- `use super::local::PeerId`
---

## cli/src/history.rs (2 chunks, 322 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/history.rs: Defines 6 types: MessageRecord, Direction, MessageRecord, MessageHistory, MessageHistory; 19 functions; 6 imports cli/src/history.rs: Defines 6 types: MessageRecord, Direction, MessageRecord, MessageHistory, MessageHistory; 19 functions; 6 imports

### Structs/Classes
- Direction
- HistoryStats
- MessageHistory
- MessageRecord

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new_sent` | 42 | default, new_v4, from_timestamp |
| `new_received` | 53 | default, new_v4, to_vec, from_timestamp |
| `formatted_time` | 65 | default, from_slice, to_vec, from_timestamp |
| `peer` | 70 | default, from_slice, to_vec |
| `open` | 85 | default, new, from_slice, to_vec |
| `add` | 96 | new, from_slice, to_vec |
| `get` | 111 | new, recent, from_slice |
| `recent` | 123 | recent, new, from_slice |
| `conversation` | 149 | get, recent, new, from_slice |
| `search` | 154 | get, add, new, from_slice |
| `count` | 176 | get, new, add, from_slice |
| `count_with_peer` | 182 | get, new, add, from_slice |
| `mark_delivered` | 198 | get, add, new, default, from_slice |
| `clear` | 208 | default, new, from_slice |
| `clear_conversation` | 215 | default, new, from_slice |
| `stats` | 236 | default, from_slice, now |
| `current_timestamp` | 273 | now, open, new_sent, new_received, tempdir |
| `test_message_record` | 286 | new_sent, new_received, tempdir, open |
| `test_message_history` | 295 | new_sent, new_received, tempdir, open |
| `new_sent` | 42 | from_timestamp, new_v4, default |
| `new_received` | 53 | to_vec, from_timestamp, new_v4, default |
| `formatted_time` | 65 | from_slice, to_vec, from_timestamp, default |
| `peer` | 70 | from_slice, to_vec, default |
| `open` | 85 | from_slice, to_vec, new, default |
| `add` | 96 | new, to_vec, from_slice |
| `get` | 111 | new, from_slice, recent |
| `recent` | 123 | new, from_slice, recent |
| `conversation` | 149 | new, from_slice, get, recent |
| `search` | 154 | new, from_slice, add, get |
| `count` | 176 | new, from_slice, add, get |
| `count_with_peer` | 182 | new, from_slice, add, get |
| `mark_delivered` | 198 | new, get, from_slice, add, default |
| `clear` | 208 | new, from_slice, default |
| `clear_conversation` | 215 | new, from_slice, default |
| `stats` | 236 | from_slice, now, default |
| `current_timestamp` | 273 | new_received, open, new_sent, now, tempdir |
| `test_message_record` | 286 | new_received, tempdir, open, new_sent |
| `test_message_history` | 295 | new_received, tempdir, open, new_sent |

### Imports
- `use anyhow::{Context, Result}`
- `use chrono::{DateTime, Utc}`
- `use serde::{Deserialize, Serialize}`
- `use sled::Db`
- `use std::path::PathBuf`
- `use super::*`
---

## core/src/store/history.rs (1 chunks, 515 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/store/history.rs: Defines 7 types: MessageDirection, MessageRecord, MessageRecord, MessageRecord, HistoryStats; 26 functions; 6 imports

### Structs/Classes
- HistoryManager
- HistoryStats
- MessageDirection
- MessageRecord

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `adjust_legacy_timestamps` | 34 | new_v4, now |
| `new_sent` | 43 | new_v4, now |
| `new_received` | 57 | new_v4, to_vec, now |
| `current_timestamp` | 72 | to_vec, from_slice, now |
| `new` | 94 | recent_internal, from_slice, to_vec |
| `backend` | 99 | recent_internal, from_slice, to_vec |
| `add` | 102 | new, recent_internal, from_slice, to_vec |
| `get` | 111 | new, recent_internal, from_slice |
| `recent` | 126 | recent_internal, new, from_slice |
| `recent_including_hidden` | 137 | Reverse, recent_internal, new, from_slice |
| `recent_internal` | 144 | recent, Reverse, new, from_slice |
| `conversation` | 185 | recent, from_slice, to_vec |
| `unhide_messages_for_peer` | 195 | new, from_slice, to_vec |
| `hide_messages_for_peer` | 219 | new, Reverse, from_slice, to_vec |
| `search` | 241 | Reverse, new, from_slice |
| `remove_conversation` | 270 | get, add, from_slice |
| `mark_delivered` | 291 | get, add, default, from_slice |
| `clear` | 306 | default, from_slice |
| `delete` | 320 | default, from_slice |
| `stats` | 328 | default, from_slice |
| `count` | 353 | from_slice |
| `enforce_retention` | 357 | from_slice |
| `prune_before` | 388 | new, from_slice |
| `flush` | 409 | new |
| `test_case_insensitive_peer_id_matching` | 421 | new |
| `test_remove_conversation_case_insensitive` | 471 | new |

### Imports
- `use crate::IronCoreError`
- `use crate::store::backend::MemoryStorage`
- `use crate::store::backend::StorageBackend`
- `use serde::{Deserialize, Serialize}`
- `use std::sync::Arc`
- `use super::*`
---

## core/src/store/inbox.rs (1 chunks, 496 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/store/inbox.rs: Defines 5 types: ReceivedMessage, InboxBackend, Inbox, Inbox, Default; 24 functions; 8 imports

### Structs/Classes
- Default
- Inbox
- InboxBackend
- ReceivedMessage

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 50 | Persistent, new |
| `persistent_with_storage` | 63 | Persistent |
| `persistent` | 74 | Persistent |
| `trigger_maintenance` | 83 | Persistent |
| `is_duplicate` | 91 | Persistent |
| `receive` | 106 | Persistent, deserialize |
| `messages_from` | 200 | new, from_utf8_lossy, Persistent, deserialize |
| `all_messages` | 221 | new, Persistent, deserialize, total_count |
| `total_count` | 240 | with_capacity, Persistent, new, total_count |
| `sender_count` | 248 | with_capacity, Persistent, new, total_count |
| `get_inbox_count` | 266 | with_capacity, Persistent, total_count |
| `drain_received_messages` | 273 | with_capacity, Persistent, new |
| `clear_messages` | 300 | Persistent, new, now |
| `default` | 322 | new, now |
| `make_received` | 330 | new, now |
| `test_receive_and_query` | 344 | new |
| `test_deduplication` | 358 | persistent, new |
| `test_is_duplicate` | 369 | persistent, new |
| `test_all_messages` | 378 | persistent, new |
| `test_clear_messages` | 388 | persistent, new |
| `test_persistent_inbox` | 400 | persistent, new |
| `test_persistent_inbox_survives_restart` | 426 | persistent, new |
| `test_drain_received_messages` | 454 | new |
| `test_get_inbox_count` | 478 | new |

### Imports
- `use crate::store::backend::StorageBackend`
- `use crate::store::storage::StorageManager`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::{HashMap, HashSet}`
- `use std::sync::Arc`
- `use super::*`
- `use tempfile::tempdir`
---

## core/src/mobile/ios_strategy.rs (1 chunks, 531 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/mobile/ios_strategy.rs: Defines 12 types: BackgroundMode, std, BluetoothState, CoreBluetoothState, CoreBluetoothState; 49 functions; 3 imports

### Structs/Classes
- BackgroundMode
- BluetoothState
- CoreBluetoothState
- Default
- IosBackgroundConfig
- IosBackgroundStrategy
- LocationAccuracy
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `fmt` | 19 |  |
| `is_any_active` | 48 | new, is_any_active |
| `new` | 51 | new, is_any_active |
| `set_central_active` | 59 | new, is_any_active |
| `set_peripheral_active` | 66 | new, is_any_active |
| `set_restricted` | 73 | new, is_any_active |
| `default` | 86 | new |
| `default` | 108 | new |
| `is_mode_enabled` | 125 |  |
| `enable_mode` | 130 | default |
| `disable_mode` | 135 | default |
| `get_enabled_modes` | 140 | default |
| `validate` | 147 | default |
| `new` | 172 | default |
| `get_config` | 182 |  |
| `set_config` | 187 |  |
| `get_bluetooth_state` | 194 |  |
| `set_bluetooth_state` | 199 |  |
| `schedule_background_fetch` | 204 |  |
| `on_background_fetch` | 217 |  |
| `enable_location_background` | 243 | can_run_ble_peripheral_background, new, can_run_ble_central_background |
| `disable_location_background` | 260 | can_run_ble_peripheral_background, new, can_run_ble_central_background |
| `can_run_ble_central_background` | 266 | can_run_ble_peripheral_background, new, can_run_ble_central_background |
| `can_run_ble_peripheral_background` | 273 | can_run_ble_peripheral_background, new, can_run_ble_central_background |
| `get_recommended_profile` | 280 | can_run_ble_peripheral_background, new, can_run_ble_central_background |
| `test_core_bluetooth_state_creation` | 315 | default, new |
| `test_core_bluetooth_central_activation` | 323 | default, new |
| `test_core_bluetooth_both_active` | 332 | default, new |
| `test_core_bluetooth_restriction` | 341 | default, new |
| `test_ios_background_config_default` | 349 | default, new |
| `test_ios_background_config_validation` | 358 | default, new |
| `test_ios_background_config_empty_modes` | 368 | default, new |
| `test_ios_background_config_enable_disable_modes` | 379 | default, new |
| `test_ios_background_strategy_creation` | 389 | default, new |
| `test_ios_background_strategy_invalid_config` | 396 | default, new |
| `test_schedule_background_fetch` | 404 | default, new |
| `test_schedule_background_fetch_disabled` | 411 | default, new |
| `test_on_background_fetch` | 419 | default, new |
| `test_enable_location_background` | 427 | default, new |
| `test_enable_location_background_disabled_mode` | 435 | default, new |
| `test_enable_location_background_disabled_accuracy` | 442 | default, new |
| `test_can_run_ble_central` | 451 | default, new |
| `test_cannot_run_ble_central_when_restricted` | 458 | default, new |
| `test_get_recommended_profile` | 468 | default, new |
| `test_get_enabled_modes` | 477 | default, new |
| `test_set_config_valid` | 485 | default, new |
| `test_set_config_invalid` | 496 | default, new |
| `test_bluetooth_state_deactivation` | 506 | default, new |
| `test_location_accuracy_levels` | 518 | default |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashSet`
- `use super::*`
---

## core/src/iron_core.rs (1 chunks, 3356 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/iron_core.rs: Defines 6 types: CoreDelegate, ConsentState, IronCore, Default, IronCore; 199 functions; 27 imports

### Structs/Classes
- ConsentState
- CoreDelegate
- Default
- IronCore

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `on_peer_discovered` | 86 |  |
| `on_peer_disconnected` | 87 |  |
| `on_peer_identified` | 88 |  |
| `on_message_received` | 89 |  |
| `on_receipt_received` | 97 |  |
| `default` | 204 | default, new_heuristics_only, persistent, new |
| `new` | 213 | temp_dir, new, default, persistent, new_heuristics_only |
| `with_storage` | 281 | new, default, persistent, new_heuristics_only, with_backend |
| `with_storage_and_logs` | 358 | new, default, persistent, new_heuristics_only, with_backend |
| `start` | 433 | default, new, hash |
| `stop` | 444 | default, new, hash |
| `grant_consent` | 450 | default, new, hash |
| `initialize_identity` | 457 | default, new, hash |
| `identity_id` | 504 | decode, new_v4 |
| `device_id` | 509 | decode, new_v4, now |
| `public_key_hex` | 514 | decode, new_v4, encode_message, now |
| `set_delegate` | 521 | decode, new_v4, encode_message, now |
| `prepare_message_internal` | 532 | decode, new_v4, encode_message, now |
| `prepare_message` | 594 | from_utf8_lossy, prepare_message_internal, prepare_message |
| `prepare_message_with_id` | 605 | from_utf8_lossy, mark_message_sent, prepare_message_internal, prepare_message |
| `mark_message_sent` | 618 | from_utf8_lossy, mark_message_sent, prepare_message |
| `send_message_status` | 627 | from_utf8_lossy, mark_message_sent, prepare_message |
| `is_peer_blocked` | 653 |  |
| `blocked_only_peer_ids` | 666 |  |
| `get_peer_reputation` | 674 |  |
| `peer_spam_score` | 679 |  |
| `peer_rate_limit_multiplier` | 687 |  |
| `sign_data` | 692 |  |
| `get_registration_state` | 703 |  |
| `notify_peer_discovered` | 720 |  |
| `notify_peer_disconnected` | 727 |  |
| `record_abuse_signal` | 734 |  |
| `drift_activate` | 763 | now |
| `drift_deactivate` | 772 | now |
| `drift_network_state` | 781 | now |
| `drift_store_size` | 790 | now |
| `relay_jitter_delay` | 795 | now |
| `is_running` | 815 |  |
| `get_identity_info` | 823 |  |
| `get_device_id` | 849 | decode, verify |
| `get_seniority_timestamp` | 853 | decode, verify |
| `set_nickname` | 859 | decode, verify |
| `verify_signature` | 887 | decode, verify, new |
| `outbox_count` | 902 | new |
| `inbox_count` | 906 | new |
| `drain_received_messages` | 914 | new |
| `block_peer` | 925 | new |
| `unblock_peer` | 951 |  |
| `block_and_delete_peer` | 963 | encrypt_backup, encode |
| `list_blocked_peers_bridge` | 982 | decode, encrypt_backup, decrypt_backup, encode |
| `blocked_count` | 991 | decode, encrypt_backup, decrypt_backup, encode |
| `export_identity_backup` | 999 | encode, try_decode_protobuf, decode, encrypt_backup, decrypt_backup |
| `import_identity_backup` | 1008 | encode, decode, decrypt_backup, try_decode_protobuf |
| `extract_public_key_from_peer_id` | 1031 | to_vec, try_decode_protobuf, encode, now |
| `prepare_receipt` | 1053 | classify_notification, to_vec, now |
| `prepare_cover_traffic` | 1070 | classify_notification, from_str |
| `classify_notification` | 1081 | classify_notification, from_str |
| `get_audit_log` | 1094 | encode, hash, decode, extract_public_key_from_peer_id, resolve_identity, from_str |
| `get_audit_events_since` | 1098 | encode, hash, decode, extract_public_key_from_peer_id, resolve_identity, from_str |
| `set_privacy_config` | 1116 | encode, hash, decode, extract_public_key_from_peer_id, resolve_identity, from_str |
| `resolve_identity` | 1129 | encode, hash, decode, extract_public_key_from_peer_id, resolve_identity |
| `resolve_to_identity_id` | 1139 | hash, decode, resolve_identity, encode |
| `perform_maintenance` | 1149 |  |
| `update_disk_stats` | 1190 | to_string_pretty, privacy_config, list_blocked_peers_bridge |
| `record_log` | 1196 | to_string_pretty, privacy_config, list_blocked_peers_bridge |
| `export_logs` | 1200 | to_string_pretty, privacy_config, list_blocked_peers_bridge |
| `export_audit_log` | 1210 | to_string_pretty, privacy_config, list_blocked_peers_bridge |
| `validate_audit_chain` | 1216 | to_string_pretty, privacy_config, list_blocked_peers_bridge |
| `get_privacy_config` | 1224 | to_string_pretty, privacy_config, list_blocked_peers_bridge |
| `list_blocked_peers` | 1231 | list_blocked_peers_bridge |
| `compute_jitter_delay` | 1255 | contacts_manager, new |
| `contacts_manager` | 1295 | contacts_manager, deserialize, get_signature, new |
| `contact_federated_nickname` | 1301 | contacts_manager, get_signature, deserialize, hash |
| `contact_display_name` | 1307 | hash, get_signature, create_basic, deserialize, contacts_manager |
| `contact_update_last_known_device_id` | 1317 | create_cot, hash, get_signature, create_multihop, create_basic, deserialize, contacts_manager |
| `invite_get_signable_data` | 1332 | create_cot, hash, get_signature, create_optimizer, new, create_multihop, create_basic, deserialize |
| `dspy_verify_signature` | 1344 | create_cot, hash, get_signature, create_optimizer, new, create_multihop, create_basic |
| `dspy_blake3_hash` | 1350 | create_cot, hash, create_optimizer, new, create_multihop, create_basic |
| `dspy_create_basic_teleprompter` | 1355 | create_cot, create_optimizer, new, create_multihop, create_basic |
| `dspy_create_cot` | 1360 | create_multihop, create_cot, new, create_optimizer |
| `dspy_create_multihop` | 1365 | to_string, create_multihop, create_optimizer, new |
| `dspy_create_optimizer` | 1370 | to_string, create_optimizer, new |
| `history_manager` | 1376 | to_string, new |
| `custody_audit_count` | 1386 | to_string |
| `custody_get_registration_state_info` | 1391 | to_string, new |
| `custody_registration_transitions` | 1407 | to_string, new |
| `get_audit_events_by_type` | 1420 | new |
| `get_auto_adjust_engine` | 1439 | new |
| `is_consent_granted` | 1448 | from_str |
| `on_app_resume` | 1458 | decode, construct_onion, from_str |
| `on_app_background` | 1471 | decode, serialize, construct_onion, from_str |
| `prepare_onion_message` | 1490 | serialize, peel_layer, construct_onion, decode, deserialize, from_str |
| `peel_onion_layer` | 1516 | from_le_bytes, serialize, peel_layer, deserialize |
| `random_port` | 1546 | from_le_bytes |
| `ratchet_session_count` | 1558 | decode |
| `ratchet_has_session` | 1563 | decode |
| `ratchet_reset_session` | 1568 | decode, routing_peer_seen, new |
| `routing_peer_seen` | 1578 | decode, routing_peer_seen, new, now |
| `routing_update_peer_hints` | 1585 | to_string, now, new, decode, routing_peer_seen |
| `routing_mark_gateway` | 1599 | to_string, now, new, decode, routing_peer_seen |
| `routing_update_reliability` | 1614 | to_string, routing_peer_seen, new, now |
| `routing_tick` | 1623 | to_string, now |
| `routing_summary` | 1639 | to_string |
| `routing_clear_unreachable_peer` | 1651 |  |
| `routing_current_discovery_phase` | 1658 |  |
| `routing_negative_cache_stats` | 1670 |  |
| `routing_prefetch_stats` | 1685 | calculate_dynamic_ttl |
| `routing_timeout_budget_summary` | 1700 | calculate_dynamic_ttl |
| `routing_calculate_dynamic_ttl` | 1719 | calculate_dynamic_ttl |
| `routing_register_path` | 1735 | new |
| `routing_mark_path_failed` | 1746 | new |
| `swarm_get_best_relays` | 1762 | new |
| `swarm_get_bootstrap_candidates` | 1785 | encode, now, hash, new_v4, new |
| `swarm_can_bootstrap_others` | 1806 | hash, new_v4, encode, now |
| `swarm_get_best_paths` | 1811 | hash, new_v4, encode, now |
| `get_libp2p_keypair` | 1852 | now |
| `receive_message` | 1858 | from_utf8, now |
| `build_registration_request` | 1940 | default, new_signed |
| `get_identity_keys` | 1949 | default, new |
| `flush_outbox_for_peer` | 1952 | default, new |
| `contacts_store_manager` | 1955 | default, new |
| `history_store_manager` | 1958 | default, new |
| `list_blocked_peers_raw` | 1961 | default, new |
| `get_enhanced_peer_reputation` | 1966 | default, new |
| `privacy_config` | 1974 | default, new |
| `make_routing_decision` | 1977 | new |
| `routing_engine_handle` | 1989 | new |
| `set_cover_traffic_generator` | 1992 | new |
| `set_timing_jitter` | 2002 | new |
| `set_circuit_builder` | 2012 | new |
| `register_notification_endpoint` | 2026 |  |
| `unregister_notification_endpoint` | 2037 |  |
| `list_notification_endpoints` | 2045 |  |
| `clear_all_request_notifications` | 2048 |  |
| `clear_message_notifications` | 2053 |  |
| `close_all_notifications` | 2058 |  |
| `transport_manager_handle` | 2063 |  |
| `get_healthy_connections` | 2068 |  |
| `expire_address_observations` | 2074 |  |
| `bootstrap_manager_handle` | 2079 |  |
| `peer_exchange_manager_handle` | 2082 |  |
| `get_unhealthy_connections` | 2092 |  |
| `get_all_connection_stats` | 2098 |  |
| `cleanup_stale_connections` | 2107 | new |
| `current_discovery_phase` | 2115 | new |
| `clear_unreachable_peer` | 2125 | new |
| `get_peer_activity` | 2134 | new |
| `get_all_relay_stats` | 2151 | new |
| `get_fallback_relays` | 2163 | calculate_dynamic_ttl, new |
| `can_bootstrap_others` | 2173 | calculate_dynamic_ttl, get_hole_punch_status, new |
| `get_healthy_relays` | 2184 | calculate_dynamic_ttl, get_hole_punch_status, new |
| `custody_audit_count_usize` | 2190 | new, calculate_dynamic_ttl, get_hole_punch_status |
| `get_registration_state_info` | 2195 | get_hole_punch_status, new, clear_unreachable_peer, decode, calculate_dynamic_ttl |
| `calculate_dynamic_ttl` | 2208 | get_hole_punch_status, new, clear_unreachable_peer, decode, calculate_dynamic_ttl |
| `get_hole_punch_status` | 2217 | clear_unreachable_peer, new, decode |
| `get_active_paths` | 2234 | clear_unreachable_peer, new, decode |
| `record_reconnect_success_and_clear_cache` | 2242 | clear_unreachable_peer, decode |
| `peers_needing_reconnect` | 2263 | default, new |
| `reset_circuit_breakers` | 2271 | default, new |
| `disable_transport` | 2289 | default, new |
| `start_hole_punch` | 2299 | default, new |
| `register_state_change_callback` | 2324 | new, now |
| `relay_discovery_mut` | 2339 | decode, now |
| `get_best_forwarding_path` | 2361 | decode, new, now |
| `get_available_paths` | 2381 | decode, new |
| `get_forwarding_capability` | 2397 |  |
| `routing_prefetch_stats_detailed` | 2421 | decode, from |
| `force_ratchet` | 2436 | decode, from |
| `create_receiver_session` | 2448 | decode, from |
| `converge_delivered_for_message` | 2480 | to_string, for_local_peer |
| `registration_transitions_for_identity` | 2495 | to_string, for_local_peer |
| `enforce_storage_pressure` | 2505 | for_local_peer |
| `storage_pressure_state` | 2514 | for_local_peer |
| `custody_store_for_peer` | 2523 | for_local_peer |
| `drift_apply_policy` | 2533 | new |
| `drift_set_cover_traffic` | 2542 | get_auto_adjust_engine, new |
| `drift_set_reputation_manager` | 2551 | get_auto_adjust_engine, new |
| `drift_generate_cover_traffic_if_due` | 2570 | get_auto_adjust_engine, new |
| `new_drift_sync` | 2580 | get_auto_adjust_engine, new |
| `override_ble_advertise_interval` | 2591 | default, get_auto_adjust_engine, from_str |
| `override_relay_priority_threshold` | 2599 | default, get_auto_adjust_engine, from_str |
| `compute_ble_adjustment` | 2606 | default, get_auto_adjust_engine, new, from_str |
| `compute_relay_adjustment` | 2615 | default, get_auto_adjust_engine, new, from_str |
| `apply_policy_config` | 2630 | default, new, from_str |
| `emergency_recover` | 2681 |  |
| `blocked_only_peer_ids_set` | 2695 |  |
| `can_forward_for_wasm` | 2710 | to_string |
| `can_reach_destination` | 2719 | to_string |
| `routing_refresh_delegate_routes` | 2734 | to_string |
| `routing_run_optimization` | 2743 | to_string |
| `routing_evaluate_all_tracked` | 2755 |  |
| `routing_prune_below` | 2765 |  |
| `routing_should_advance` | 2774 |  |
| `routing_mark_refresh_failed` | 2790 |  |
| `routing_next_refresh_hint` | 2799 |  |
| `routing_is_prefetch_complete` | 2809 |  |
| `routing_is_prefetch_in_progress` | 2819 |  |
| `touch_notification_endpoint` | 2829 |  |
| `update_keepalive` | 2840 |  |

### Imports
- `use crate::IronCoreError`
- `use crate::abuse::EnhancedAbuseReputationManager`
- `use crate::abuse::auto_block::{AutoBlockConfig, AutoBlockEngine}`
- `use crate::abuse::spam_detection::{SpamDetectionConfig, SpamDetectionEngine}`
- `use crate::crypto::{decrypt_message, encrypt_message, session_manager::RatchetSessionManager}`
- `use crate::drift::{MeshStore, NetworkState, RelayConfig, RelayEngine}`
- `use crate::identity::IdentityManager`
- `use crate::message::{decode_envelope, decode_message, encode_envelope, Message}`
- `use crate::notification::NotificationEndpointRegistry`
- `use crate::observability::{AuditEventType, AuditLog as AuditLogType}`
- `use crate::relay::{BootstrapManager, PeerExchangeManager}`
- `use crate::routing::local::TransportType`
- `use crate::routing::optimized_engine::OptimizedRoutingEngine`
- `use crate::store::backend::MemoryStorage`
- `use crate::store::backend::SledStorage`
- `use crate::store::blocked::BlockedManager as CoreBlockedManager`
- `use crate::store::logs::LogManager`
- `use crate::transport::behaviour::RegistrationRequest`
- `use crate::transport::manager::TransportManager`
- `use crate::transport::nat::NatTraversal`
- `use crate::transport::reputation::AbuseSignal`
- `use crate::transport::swarm::SwarmManager`
- `use parking_lot::RwLock`
- `use rand::RngCore`
- `use std::sync::Arc`
- `use std::time::{SystemTime, UNIX_EPOCH}`
---

## core/src/identity/keys.rs (1 chunks, 300 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/identity/keys.rs: Defines 4 types: KeyPair, KeyPair, IdentityKeys, IdentityKeys; 15 functions; 6 imports

### Structs/Classes
- IdentityKeys
- KeyPair

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `generate` | 15 | hash, encode, from_bytes |
| `verifying_key` | 25 | hash, encode, from_bytes |
| `generate` | 38 | hash, encode, from_bytes |
| `public_key_hex` | 48 | hash, encode, from_bytes |
| `identity_id` | 53 | hash, to_bytes, encode, from_bytes |
| `sign` | 60 | try_from_bytes, to_bytes, from_bytes |
| `verify` | 66 | try_from_bytes, to_bytes, from_bytes |
| `to_bytes` | 83 | to_libp2p_keypair, from, from_bytes, to_bytes, try_from_bytes |
| `from_bytes` | 88 | to_libp2p_keypair, from, from_bytes, to_bytes, try_from_bytes |
| `to_libp2p_peer_id` | 111 | try_from_bytes, from, to_libp2p_keypair, generate |
| `to_libp2p_keypair` | 127 | try_from_bytes, from, verify, generate |
| `test_key_generation` | 149 | generate, verify, from_bytes |
| `test_signing` | 159 | generate, verify, from_bytes |
| `test_verification` | 168 | generate, verify, from_bytes |
| `test_serialization` | 184 | generate, from_bytes |

### Imports
- `use anyhow::Result`
- `use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey, Verifier, VerifyingKey}`
- `use rand::RngCore`
- `use super::*`
- `use zeroize::Zeroize`
---

## cli/src/lib.rs (2 chunks, 18 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/lib.rs: structural extraction cli/src/lib.rs: structural extraction

---

## core/src/routing/local.rs (1 chunks, 657 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/local.rs: Defines 8 types: PeerId, TransportType, PeerStatus, PeerInfo, CellSummary; 33 functions; 3 imports

### Structs/Classes
- CellSummary
- LocalCell
- PeerEvent
- PeerId
- PeerInfo
- PeerStatus
- TransportType

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 101 | evict_lowest_reliability, new |
| `with_timeouts` | 112 | evict_lowest_reliability, new |
| `peer_seen` | 123 | evict_lowest_reliability, new |
| `update_peer_hints` | 164 |  |
| `mark_as_gateway` | 171 |  |
| `record_sync` | 178 |  |
| `update_reliability` | 199 | new |
| `peers_for_hint` | 207 | new |
| `active_peers` | 221 | PeerBecameStale, PeerBecameDormant, new |
| `gateway_peers` | 236 | PeerBecameStale, PeerBecameDormant, new |
| `tick` | 244 | PeerBecameStale, PeerBecameDormant, new |
| `summarize` | 284 | new |
| `peer_count` | 313 | now |
| `active_count` | 318 | now |
| `get_peer` | 326 | now |
| `evict_lowest_reliability` | 331 | new, now |
| `local_id` | 349 | with_timeouts, new, now |
| `current_timestamp` | 356 | with_timeouts, new, now |
| `make_peer_id` | 366 | PeerBecameStale, PeerBecameDormant, new, with_timeouts |
| `make_hint` | 372 | PeerBecameStale, PeerBecameDormant, new, with_timeouts |
| `test_peer_seen_creates_active_peer` | 378 | PeerBecameStale, PeerBecameDormant, new, with_timeouts |
| `test_active_timeout_progression` | 392 | PeerBecameStale, PeerBecameDormant, new, with_timeouts |
| `test_update_peer_hints` | 424 | new |
| `test_peers_for_hint` | 439 | new |
| `test_reliability_scoring` | 469 | new, with_timeouts |
| `test_gateway_detection` | 497 | new, with_timeouts |
| `test_max_peers_eviction` | 515 | new, with_timeouts |
| `test_record_sync` | 545 | new |
| `test_cell_summary_generation` | 566 | new, with_timeouts |
| `test_active_peers_sorted_by_reliability` | 588 | new, with_timeouts |
| `test_stale_peers_not_in_peers_for_hint` | 614 | new, with_timeouts |
| `test_multiple_transports` | 635 | new |
| `test_local_id_preserved` | 652 | new |

### Imports
- `use std::collections::HashMap`
- `use super::*`
- `use web_time::SystemTime`
---

## core/src/mobile_bridge.rs (1 chunks, 2979 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/mobile_bridge.rs: Defines 37 types: MeshServiceConfig, ServiceState, ConnectionPathState, MotionState, NetworkType; 144 functions; 11 imports

### Structs/Classes
- AdjustmentProfile
- AutoAdjustEngine
- BehaviorAdjustment
- BleAdjustment
- ConnectionPathState
- Default
- DeviceProfile
- DeviceState
- HistoryManager
- HistoryStats
- LedgerEntry
- LedgerManager
- MeshService
- MeshServiceConfig
- MeshServiceCoreDelegate
- MeshSettingsManager
- MessageDirection
- MessageRecord
- MotionState
- NetworkType
- PlatformBridge
- RelayAdjustment
- ServiceState
- ServiceStats
- SwarmBridge
- crate

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `from_profile` | 83 | recommended_behavior |
| `new` | 156 | default, new |
| `with_storage` | 179 | default, new |
| `with_storage_and_logs` | 202 | default, new, with_storage_and_logs |
| `start` | 226 | downgrade, new, with_storage, with_storage_and_logs |
| `set_delegate` | 306 |  |
| `stop` | 309 | get_swarm_bridge |
| `pause` | 337 | default, get_swarm_bridge |
| `resume` | 344 | default, get_swarm_bridge, get_stats |
| `get_state` | 351 | get_stats, get_state, get_connection_path_state, get_nat_status, default, get_swarm_bridge |
| `get_stats` | 355 | get_stats, get_state, get_connection_path_state, get_nat_status, default, get_swarm_bridge |
| `reset_stats` | 362 | get_stats, get_state, get_connection_path_state, get_nat_status, default |
| `set_platform_bridge` | 367 | get_state, get_connection_path_state, get_stats, get_nat_status |
| `get_nat_status` | 373 | get_state, get_connection_path_state, get_stats, get_nat_status |
| `get_connection_path_state` | 376 | get_state, get_connection_path_state, get_stats, get_nat_status |
| `export_diagnostics` | 392 | get_stats, get_state, get_connection_path_state, get_nat_status, resolve_swarm_keypair_and_mode |
| `start_swarm` | 424 | resolve_swarm_keypair_and_mode |
| `get_swarm_bridge` | 772 | from_profile |
| `update_device_state` | 776 | from_profile |
| `recommended_behavior` | 911 | get_core |
| `get_device_state` | 919 | get_core |
| `set_relay_budget` | 922 | get_core |
| `get_auto_adjust_engine` | 945 | get_core |
| `on_peer_discovered` | 948 | get_core |
| `on_peer_disconnected` | 958 | get_core |
| `on_data_received` | 965 | get_core |
| `on_battery_changed` | 1021 | update_device_state, on_data_received, pause, resume |
| `on_network_changed` | 1032 | update_device_state, on_data_received, pause, resume |
| `on_motion_changed` | 1043 | update_device_state, on_data_received, pause, resume |
| `on_entering_background` | 1053 | on_data_received, pause, resume |
| `on_entering_foreground` | 1058 | on_data_received, get_core, new, resume |
| `on_ble_data_received` | 1063 | on_data_received, get_core, new |
| `get_core` | 1071 | get_core, new |
| `prepare_onion_message` | 1080 | get_core, new |
| `peel_onion_layer` | 1093 | get_core, new |
| `random_port` | 1105 | get_core, new |
| `ratchet_session_count` | 1114 | get_core, new |
| `ratchet_has_session` | 1123 | get_core, new |
| `ratchet_reset_session` | 1132 | get_core, new |
| `routing_tick` | 1139 | get_core, new |
| `is_running` | 1148 | get_core, new |
| `get_all_connection_stats` | 1154 | get_core, new |
| `dispatch_ble_packet` | 1185 |  |
| `compute_behavior` | 1198 |  |
| `resolve_swarm_keypair_and_mode` | 1241 | from, load_or_create_headless_network_keypair, generate_ed25519, read, from_protobuf_encoding, create_dir_all |
| `load_or_create_headless_network_keypair` | 1261 | set_permissions, write, from, generate_ed25519, from_mode, read, from_protobuf_encoding, create_dir_all |
| `on_peer_discovered` | 1312 |  |
| `on_peer_disconnected` | 1320 |  |
| `on_peer_identified` | 1329 |  |
| `on_message_received` | 1342 |  |
| `on_receipt_received` | 1363 |  |
| `on_battery_changed` | 1375 |  |
| `on_network_changed` | 1376 |  |
| `on_motion_changed` | 1377 |  |
| `on_ble_data_received` | 1378 |  |
| `on_entering_background` | 1379 | new |
| `on_entering_foreground` | 1380 | new |
| `send_ble_packet` | 1381 | new |
| `default` | 1428 | new |
| `new` | 1436 | new |
| `compute_profile` | 1442 |  |
| `compute_ble_adjustment` | 1457 |  |
| `compute_relay_adjustment` | 1473 |  |
| `override_ble_scan_interval` | 1489 | from |
| `override_ble_advertise_interval` | 1493 | from |
| `override_relay_max_per_hour` | 1504 | default, from, read_to_string, from_str |
| `override_relay_priority_threshold` | 1508 | create_dir_all, from, default, validate, from_str, read_to_string |
| `clear_overrides` | 1517 | create_dir_all, write, from, default, to_string_pretty, validate, from_str, read_to_string |
| `new` | 1536 | create_dir_all, write, from, default, to_string_pretty, validate, from_str, read_to_string |
| `load` | 1541 | write, default, to_string_pretty, validate, from_str, create_dir_all, read_to_string |
| `save` | 1554 | write, to_string_pretty, default, validate, create_dir_all |
| `validate` | 1568 | default |
| `default_settings` | 1595 | default |
| `adjust_legacy_timestamps` | 1629 | default, from, new, to_vec |
| `new` | 1653 | from, recent_internal, new, default, from_slice, to_vec |
| `add` | 1666 | new, recent_internal, from_slice, to_vec |
| `get` | 1675 | new, recent_internal, from_slice |
| `recent` | 1689 | recent_internal, new, from_slice |
| `recent_including_hidden` | 1700 | recent_internal, new, from_slice |
| `recent_internal` | 1707 | recent, new, from_slice |
| `conversation` | 1746 | recent, new, from_slice |
| `remove_conversation` | 1754 | new, from_slice |
| `search` | 1777 | new, from_slice |
| `unhide_messages_for_peer` | 1811 | get, to_vec, new, from_slice |
| `hide_messages_for_peer` | 1836 | get, add, new, from_slice, to_vec |
| `mark_delivered` | 1859 | get, add, new, default, from_slice |
| `clear` | 1867 | default, new, from_slice |
| `clear_conversation` | 1873 | default, new, from_slice |
| `stats` | 1896 | default, with_capacity, from_slice |
| `count` | 1919 | new, with_capacity, Reverse, from_slice |
| `flush` | 1924 | new, with_capacity, Reverse, from_slice |
| `enforce_retention` | 1934 | new, with_capacity, Reverse, from_slice |
| `prune_before` | 1967 | new, from_slice |
| `delete` | 1988 | from, new, from_str, read_to_string |
| `new` | 2022 | write, from, new, to_string_pretty, from_str, create_dir_all, save_with_entries, read_to_string |
| `load` | 2028 | write, new, to_string_pretty, from_str, create_dir_all, save_with_entries, read_to_string |
| `save_with_entries` | 2040 | write, new, to_string_pretty, create_dir_all, save_with_entries |
| `save` | 2052 | new, save_with_entries |
| `record_connection` | 2057 | new, save_with_entries |
| `record_failure` | 2078 | save_with_entries |
| `annotate_identity` | 2086 | new, save_with_entries |
| `dialable_addresses` | 2138 | Reverse |
| `get_preferred_relays` | 2147 | Reverse |
| `all_known_topics` | 2160 | new |
| `summary` | 2168 | new |
| `default` | 2199 | new_multi_thread, new, new_current_thread |
| `get_global_runtime` | 2207 | new_multi_thread, new, new_current_thread |
| `new` | 2240 | get_runtime_handle, dispatch_ble_packet, new, from_str |
| `send_message` | 2253 | get_runtime_handle, dispatch_ble_packet, from_str |
| `send_message_status` | 2291 | get_runtime_handle, dispatch_ble_packet, from_str |
| `send_to_all_peers` | 2332 | get_runtime_handle, from_str, dispatch_ble_packet |
| `dial` | 2366 | get_runtime_handle, new, from_str |
| `get_peers` | 2383 | get_runtime_handle, new |
| `get_listeners` | 2400 | get_runtime_handle, new |
| `get_external_addresses` | 2425 | get_runtime_handle, new |
| `get_topics` | 2441 | get_runtime_handle, new |
| `subscribe_topic` | 2454 | get_runtime_handle |
| `unsubscribe_topic` | 2467 | get_runtime_handle |
| `publish_topic` | 2479 | get_runtime_handle |
| `shutdown` | 2491 | get_runtime_handle, now |
| `set_handle` | 2504 | now |
| `get_runtime_handle` | 2509 | with_storage, new, now |
| `dispatch_ble_packet` | 2516 | with_storage, new, now |
| `set_dispatch_ble_fn` | 2524 | with_storage, new, now |
| `current_timestamp` | 2528 | with_storage, new, now |
| `make_state` | 2544 | new, with_storage |
| `test_fresh_install_without_identity_resolves_headless_mode_with_persisted_key` | 2555 | new, with_storage |
| `test_identity_creation_upgrades_resolved_mode_from_headless_to_full` | 2599 | new, with_storage |
| `test_connection_path_state_disconnected_by_default` | 2636 | compute_behavior, new |
| `test_compute_behavior_minimal_mode` | 2656 | compute_behavior |
| `test_compute_behavior_low_battery` | 2670 | compute_behavior, from_profile |
| `test_compute_behavior_stationary_good_battery` | 2684 | compute_behavior, new, from_profile |
| `test_compute_behavior_charging_always_full` | 2693 | compute_behavior, new, from_profile |
| `test_compute_behavior_normal_operation` | 2701 | compute_behavior, new, from_profile |
| `test_device_state_from_profile` | 2710 | new, from_profile |
| `test_update_device_state_stores_state` | 2727 | new |
| `test_update_device_state_transitions` | 2755 | new |
| `test_connection_path_state_disconnected_without_peers` | 2801 | new, from_str |
| `test_export_diagnostics_contains_state_fields` | 2813 | new, from_str |
| `test_get_swarm_bridge_initialization` | 2827 | new |
| `test_history_manager_persists_across_restart` | 2839 | new |
| `test_history_manager_recent_sorts_by_timestamp_not_key_order` | 2871 | new |
| `test_ledger_preferred_relays` | 2928 | from_millis, sleep, new |
| `test_mesh_settings_default` | 2955 | new |

### Imports
- `use crate::settings::MeshSettings`
- `use crate::transport::SwarmHandle`
- `use libp2p::{Multiaddr, PeerId}`
- `use parking_lot::{Mutex, RwLock}`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashSet`
- `use std::os::unix::fs::PermissionsExt`
- `use std::str::FromStr`
- `use std::sync::Arc`
- `use super::*`
- `use tempfile::tempdir`
---

## core/src/routing/negative_cache.rs (1 chunks, 534 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/negative_cache.rs: Defines 8 types: BloomFilter, BloomFilter, NegativeCacheEntry, NegativeCacheEntry, NegativeCache; 33 functions; 5 imports

### Structs/Classes
- BloomFilter
- Default
- NegativeCache
- NegativeCacheEntry
- NegativeCacheStats

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 41 | with_capacity, new, hash_positions |
| `for_peer_cache` | 58 | with_capacity, new, hash_positions |
| `hash_positions` | 63 | with_capacity, new, hash_positions |
| `insert` | 88 | hash_positions |
| `contains` | 99 | hash_positions, now |
| `false_positive_rate` | 109 | now |
| `count` | 121 | now |
| `clear` | 126 | now |
| `new` | 144 | now |
| `is_expired` | 151 | for_peer_cache, new, now |
| `refresh` | 155 | default, for_peer_cache, new, now |
| `new` | 198 | default, from_secs, new, for_peer_cache |
| `with_defaults` | 209 | evict_oldest, from_secs, new |
| `is_definitely_unreachable` | 217 | evict_oldest, new |
| `record_unreachable` | 240 | evict_oldest, new |
| `clear_unreachable` | 258 |  |
| `cleanup_expired` | 269 |  |
| `evict_oldest` | 278 |  |
| `stats` | 290 | with_defaults |
| `false_positive_rate` | 301 | new, with_defaults |
| `len` | 306 | new, with_defaults |
| `is_empty` | 311 | new, with_defaults |
| `clear` | 316 | new, with_defaults |
| `prune_below_confidence` | 324 | new, with_defaults |
| `default` | 335 | from_secs, new, with_defaults |
| `test_bloom_filter_basic` | 345 | from_secs, new |
| `test_bloom_filter_false_positive_rate` | 364 | from_millis, sleep, from_secs, new |
| `test_negative_cache_basic` | 378 | from_millis, sleep, from_secs, new |
| `test_negative_cache_expiry` | 395 | from_millis, from_secs, sleep, new |
| `test_negative_cache_clear` | 409 | sleep, new, from_millis, from_secs, with_defaults |
| `test_negative_cache_cleanup` | 420 | sleep, new, from_millis, from_secs, with_defaults |
| `test_negative_cache_capacity` | 435 | from_secs, new, with_defaults |
| `test_negative_cache_stats` | 449 | with_defaults |

### Imports
- `use std::collections::HashMap`
- `use std::collections::hash_map::DefaultHasher`
- `use std::hash::{Hash, Hasher}`
- `use super::*`
- `use web_time::{Duration, Instant}`
---

## core/src/routing/neighborhood.rs (1 chunks, 770 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/neighborhood.rs: Defines 6 types: GatewayInfo, NeighborhoodSummary, NeighborhoodGossip, NeighborhoodTable, NeighborhoodTable; 32 functions; 4 imports

### Structs/Classes
- Default
- GatewayInfo
- NeighborhoodGossip
- NeighborhoodSummary
- NeighborhoodTable

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 72 | evict_stalest_gateway, new |
| `with_max_staleness` | 83 | evict_stalest_gateway, new, rebuild_summaries |
| `update_gateway` | 95 | evict_stalest_gateway, rebuild_summaries, gateways_for_hint |
| `gateways_for_hint` | 130 | update_gateway, gateways_for_hint |
| `best_gateway_for_hint` | 138 | update_gateway, gateways_for_hint |
| `process_gossip` | 155 | rebuild_summaries, update_gateway |
| `generate_gossip` | 208 |  |
| `cleanup` | 238 | new |
| `all_gateways` | 253 | new |
| `all_reachable_hints` | 258 | new |
| `gateway_count` | 283 | new |
| `summary_count` | 288 | new, now |
| `rebuild_summaries` | 293 | new, now |
| `evict_stalest_gateway` | 313 | new, now |
| `default` | 331 | new, now |
| `current_timestamp` | 337 | new, now |
| `make_peer_id` | 347 | new |
| `make_hint` | 353 | new |
| `make_cell_summary` | 357 | new |
| `test_update_gateway` | 369 | new |
| `test_gateways_for_hint` | 385 | new |
| `test_best_gateway_for_hint_prefers_fewer_hops` | 426 | new |
| `test_best_gateway_for_hint_prefers_higher_reliability` | 451 | new |
| `test_process_gossip_adds_neighborhood_info` | 476 | with_max_staleness, new |
| `test_process_gossip_respects_max_hops` | 513 | with_max_staleness |
| `test_cleanup_removes_stale_entries` | 543 | with_max_staleness, new |
| `test_generate_gossip` | 565 | with_max_staleness, new |
| `test_all_reachable_hints` | 583 | with_max_staleness, new |
| `test_reject_gossip_with_excessive_hops` | 610 | with_max_staleness, new |
| `test_max_gateways_eviction` | 630 | new |
| `test_gossip_exchange_propagation` | 680 | new |
| `test_deduplication_prefers_fresh_data` | 721 | new |

### Imports
- `use std::collections::HashMap`
- `use super::*`
- `use super::local::{CellSummary, PeerId, TransportType}`
- `use web_time::SystemTime`
---

## core/src/privacy/onion.rs (1 chunks, 508 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/privacy/onion.rs: Defines 4 types: OnionError, OnionLayer, OnionHeader, OnionEnvelope; 17 functions; 6 imports

### Structs/Classes
- OnionEnvelope
- OnionError
- OnionHeader
- OnionLayer

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `construct_onion` | 93 | random_from_rng, from, new, TooManyHops, thread_rng, from_slice |
| `peel_layer` | 235 | from, new, from_slice |
| `derive_layer_key` | 300 | random_from_rng, from, thread_rng, from_slice, derive_key |
| `derive_nonce` | 308 | derive_key, random_from_rng, from, thread_rng |
| `test_construct_onion_single_hop` | 322 | random_from_rng, from, thread_rng, new |
| `test_construct_onion_multiple_hops` | 337 | random_from_rng, from, thread_rng, new |
| `test_construct_onion_max_hops` | 361 | random_from_rng, from, thread_rng, new |
| `test_construct_onion_too_many_hops` | 375 | serialize, random_from_rng, from, new, thread_rng, deserialize |
| `test_construct_onion_empty_path` | 389 | serialize, random_from_rng, from, thread_rng, deserialize |
| `test_peel_layer_single_hop` | 395 | serialize, random_from_rng, from, thread_rng, deserialize |
| `test_onion_layer_serialization` | 414 | serialize, deserialize |
| `test_onion_envelope_serialization` | 433 | serialize, random_from_rng, from, thread_rng, deserialize |
| `test_key_derivation_deterministic` | 454 | random_from_rng, from, thread_rng |
| `test_key_derivation_different_secrets` | 463 | random_from_rng, from, thread_rng, TooManyHops |
| `test_onion_ephemeral_keys_unique` | 474 | random_from_rng, from, thread_rng, TooManyHops |
| `test_construct_onion_payload_preserved` | 489 | random_from_rng, from, thread_rng, TooManyHops |
| `test_onion_error_display` | 501 | TooManyHops |

### Imports
- `use chacha20poly1305::XChaCha20Poly1305`
- `use chacha20poly1305::aead::{Aead, KeyInit, Payload}`
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
- `use x25519_dalek::{EphemeralSecret, PublicKey}`
---

## core/src/privacy/padding.rs (1 chunks, 320 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/privacy/padding.rs: Defines 2 types: PaddingScheme, PaddingError; 25 functions; 4 imports

### Structs/Classes
- PaddingError
- PaddingScheme

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `pad_message` | 46 | MessageTooLarge, with_capacity |
| `unpad_message` | 74 | InvalidConfig |
| `pad_to_next_standard_size` | 98 | Random, MessageTooLarge, thread_rng, InvalidConfig, Fixed |
| `apply_padding_scheme` | 121 | Random, MessageTooLarge, thread_rng, InvalidConfig, Fixed |
| `test_pad_message_exact_size` | 154 |  |
| `test_pad_message_minimum_size` | 166 |  |
| `test_pad_message_too_large` | 174 |  |
| `test_unpad_message_basic` | 181 |  |
| `test_unpad_message_no_padding` | 189 |  |
| `test_unpad_message_invalid_padding` | 196 |  |
| `test_unpad_message_invalid_trailing_bytes` | 204 | Fixed |
| `test_pad_to_next_standard_size_exact` | 212 | Fixed |
| `test_pad_to_next_standard_size_round_up` | 219 | Fixed, Random |
| `test_pad_to_next_standard_size_small` | 226 | Fixed, Random |
| `test_pad_to_next_standard_size_too_large` | 233 | Fixed, Random |
| `test_apply_padding_none` | 240 | Fixed, Random |
| `test_apply_padding_fixed` | 247 | Fixed, Random |
| `test_apply_padding_power_of_two` | 254 | Random |
| `test_apply_padding_random` | 261 | Random |
| `test_apply_padding_random_invalid` | 268 | Random |
| `test_round_trip_padding` | 275 |  |
| `test_padding_multiple_sizes` | 283 |  |
| `test_padding_empty_message` | 295 |  |
| `test_padding_with_embedded_marker` | 305 |  |
| `test_standard_sizes_constant` | 314 |  |

### Imports
- `use rand::Rng`
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
---

## core/src/relay/peer_exchange.rs (1 chunks, 489 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/peer_exchange.rs: Defines 6 types: PeerExchangeError, RelayPeerInfo, RelayPeerInfo, PeerExchangeManager, PeerExchangeManager; 42 functions; 5 imports

### Structs/Classes
- Default
- PeerExchangeError
- PeerExchangeManager
- RelayPeerInfo

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 34 | now |
| `mark_seen` | 50 | now |
| `record_success` | 59 | new |
| `record_failure` | 64 | new |
| `to_message` | 69 | new |
| `from_message` | 80 | new |
| `new` | 103 | new |
| `with_config` | 112 | add_peer, new |
| `add_peer` | 121 | add_peer, now |
| `get_peer` | 138 | add_peer, now |
| `get_all_peers` | 143 | add_peer, now |
| `get_peers_by_reliability` | 148 | add_peer, now |
| `merge_peer_list` | 159 | add_peer, now, merge_peer_list |
| `prune_stale` | 166 | get_peers_by_reliability, now, merge_peer_list |
| `record_success` | 177 | new, get_peers_by_reliability, merge_peer_list |
| `record_failure` | 185 | new, get_peers_by_reliability, merge_peer_list |
| `peer_count` | 192 | full_relay, new, get_peers_by_reliability, merge_peer_list |
| `has_peer` | 197 | full_relay, new, get_peers_by_reliability, merge_peer_list |
| `clear` | 202 | full_relay, new, get_peers_by_reliability, merge_peer_list |
| `exchange_peers` | 207 | get_peers_by_reliability, sleep, full_relay, new, merge_peer_list, from_millis |
| `default` | 220 | from_millis, full_relay, sleep, new |
| `test_peer` | 232 | from_millis, full_relay, sleep, new |
| `test_relay_peer_info_creation` | 242 | from_millis, sleep |
| `test_mark_seen` | 250 | from_millis, sleep |
| `test_record_success` | 261 | new, from_message |
| `test_record_failure` | 271 | new, from_message |
| `test_score_bounds` | 281 | new, from_message |
| `test_peer_message_conversion` | 298 | new, with_config, from_message |
| `test_peer_exchange_manager_creation` | 310 | new, with_config |
| `test_add_peer` | 317 | new, with_config |
| `test_add_peer_duplicate` | 327 | new, with_config |
| `test_add_peer_capacity` | 341 | new, with_config |
| `test_get_peer` | 352 | new |
| `test_get_all_peers` | 364 | new |
| `test_get_peers_by_reliability` | 376 | new |
| `test_merge_peer_list` | 399 | from_millis, sleep, new, with_config |
| `test_record_success_2` | 411 | from_millis, sleep, new, with_config |
| `test_record_failure_2` | 423 | from_millis, sleep, new, with_config |
| `test_prune_stale` | 435 | from_millis, sleep, new, with_config |
| `test_clear` | 448 | new |
| `test_exchange_peers` | 461 | new |
| `test_exchange_peers_truncation` | 474 | new |

### Imports
- `use std::collections::HashMap`
- `use super::*`
- `use super::protocol::{RelayCapability, RelayPeerInfoMessage}`
- `use thiserror::Error`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/drift/policy.rs (1 chunks, 578 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/policy.rs: Defines 6 types: DeviceState, RelayProfile, PolicyError, PolicyEngine, PolicyEngine; 49 functions; 3 imports

### Structs/Classes
- Default
- DeviceState
- PolicyEngine
- PolicyError
- RelayProfile

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 51 | compute_profile |
| `update_device_state` | 61 | relay_budget_per_hour, compute_profile |
| `scan_interval_ms` | 67 | relay_budget_per_hour |
| `relay_budget_per_hour` | 82 | relay_budget_per_hour |
| `to_relay_config` | 97 | relay_budget_per_hour |
| `set_scan_interval_override` | 118 |  |
| `set_relay_budget_override` | 123 | new |
| `set_battery_floor_override` | 132 | new |
| `should_reduce` | 137 | new, now |
| `current_profile` | 145 | new, now |
| `compute_profile` | 150 | new, now |
| `default` | 171 | new, now |
| `make_device_state` | 179 | new, now |
| `test_profile_maximum` | 194 | new |
| `test_profile_high` | 203 | new |
| `test_profile_standard_high_battery` | 212 | new |
| `test_profile_standard_at_boundary` | 221 | new |
| `test_profile_reduced_mid_battery` | 230 | new |
| `test_profile_reduced_at_boundary_high` | 239 | new |
| `test_profile_reduced_at_boundary_low` | 248 | new |
| `test_profile_minimal_low_battery` | 257 | new |
| `test_profile_minimal_critical` | 266 | new |
| `test_scan_interval_maximum` | 275 | new |
| `test_scan_interval_high` | 284 | new |
| `test_scan_interval_standard` | 293 | new |
| `test_scan_interval_reduced` | 302 | new |
| `test_scan_interval_minimal` | 311 | new |
| `test_scan_interval_override` | 320 | new |
| `test_relay_budget_maximum` | 335 | new |
| `test_relay_budget_high` | 344 | new |
| `test_relay_budget_standard` | 353 | new |
| `test_relay_budget_reduced` | 362 | new |
| `test_relay_budget_minimal` | 371 | new |
| `test_relay_budget_override` | 380 | new |
| `test_coupling_relay_budget_cannot_be_zero` | 398 | new |
| `test_battery_floor_maximum` | 410 | new |
| `test_battery_floor_minimal` | 420 | new |
| `test_battery_floor_override` | 430 | new |
| `test_to_relay_config_maximum` | 446 | new |
| `test_to_relay_config_minimal` | 460 | default, new |
| `test_should_reduce_false_for_standard` | 471 | default, new |
| `test_should_reduce_false_for_maximum` | 480 | default, new |
| `test_should_reduce_true_for_reduced` | 489 | default, new |
| `test_should_reduce_true_for_minimal` | 498 | default, new |
| `test_default_policy_engine` | 507 | default, new |
| `test_current_profile_getter` | 513 | new |
| `test_profile_transitions` | 522 | new |
| `test_relay_config_has_correct_defaults` | 552 | new |
| `test_override_combinations` | 562 | new |

### Imports
- `use super::*`
- `use super::relay::RelayConfig`
- `use thiserror::Error`
---

## core/src/relay/protocol.rs (1 chunks, 379 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/protocol.rs: Defines 7 types: RelayCapability, RelayCapability, Default, RelayMessage, RelayPeerInfoMessage; 19 functions; 3 imports

### Structs/Classes
- Default
- RelayCapability
- RelayMessage
- RelayPeerInfoMessage
- RelayProtocolError

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `full_relay` | 21 | full_relay |
| `mobile` | 31 | full_relay |
| `is_relay` | 41 | full_relay |
| `is_store` | 46 | full_relay |
| `default` | 52 | full_relay |
| `to_bytes` | 166 | serialize, SerializationError, full_relay, DeserializationError, deserialize |
| `from_bytes` | 171 | mobile, full_relay, DeserializationError, deserialize |
| `message_type` | 177 | mobile, full_relay |
| `test_capability_full_relay` | 206 | mobile, from_bytes, full_relay |
| `test_capability_mobile` | 217 | mobile, from_bytes, full_relay |
| `test_relay_message_handshake_serialization` | 228 | from_bytes, full_relay |
| `test_relay_message_store_request_serialization` | 242 | full_relay, from_bytes |
| `test_relay_message_pull_request_serialization` | 260 | full_relay, from_bytes |
| `test_relay_message_peer_exchange_serialization` | 282 | from_bytes, full_relay |
| `test_relay_message_disconnect_serialization` | 306 | from_bytes |
| `test_relay_message_ping_pong` | 323 | from_bytes |
| `test_relay_message_store_ack` | 338 | from_bytes |
| `test_relay_message_pull_response` | 357 | from_bytes |
| `test_invalid_deserialization` | 374 | from_bytes |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
---

## core/src/drift/relay.rs (1 chunks, 810 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/relay.rs: Defines 9 types: NetworkState, RelayConfig, Default, RelayDecision, DropReason; 32 functions; 8 imports

### Structs/Classes
- Default
- DropReason
- MaintenanceReport
- NetworkState
- RelayConfig
- RelayDecision
- RelayEngine
- RelayError

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 43 |  |
| `new` | 123 | hint_from_public_key, new, now |
| `set_network_state` | 144 | with_capacity, new |
| `network_state` | 149 | with_capacity, new |
| `set_cover_traffic` | 157 | with_capacity, from_bytes, new |
| `set_reputation_manager` | 171 | with_capacity, from_bytes |
| `generate_cover_traffic_if_due` | 183 | with_capacity, from_bytes |
| `process_incoming` | 199 | from_bytes |
| `prepare_outgoing` | 331 | SerializationFailed |
| `messages_for_sync` | 343 | now |
| `messages_for_recipient` | 349 | now |
| `maintenance` | 354 | now |
| `apply_policy_config` | 371 | now |
| `store` | 376 | now |
| `store_mut` | 381 | now |
| `check_rate_limit` | 386 | now |
| `make_test_envelope` | 405 | default, hint_from_public_key, new, now |
| `test_deliver_local` | 439 | default, hint_from_public_key, new |
| `test_store_and_relay` | 457 | default, new |
| `test_duplicate_detection` | 478 | default, new |
| `test_expired_message_dropped` | 506 | default, new |
| `test_max_hops_exceeded` | 525 | default, new |
| `test_low_priority_dropped` | 544 | default, new |
| `test_network_dormant_drop` | 567 | default, new |
| `test_rate_limiting` | 586 | default, new |
| `test_coupling_cannot_send_when_dormant` | 618 | default, new |
| `test_coupling_can_send_when_active` | 630 | default, new |
| `test_messages_for_sync` | 642 | default, new |
| `test_messages_for_recipient` | 658 | default, new, now |
| `test_maintenance_removes_expired` | 686 | default, new, now |
| `test_network_state_toggle` | 720 | default, new |
| `test_relay_config_default` | 734 | default |

### Imports
- `use crate::privacy::cover::{CoverConfig, CoverTrafficScheduler}`
- `use std::sync::Arc`
- `use super::*`
- `use super::DriftError`
- `use super::envelope::DriftEnvelope`
- `use super::store::{MeshStore, MessageId, StoredEnvelope}`
- `use super::super::envelope::EnvelopeType`
- `use thiserror::Error`
---

## core/src/transport/reputation.rs (1 chunks, 619 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/reputation.rs: Defines 8 types: ReputationScore, ReputationScore, std, AbuseSignal, PeerAbuseStats; 37 functions; 6 imports

### Structs/Classes
- AbuseReputationManager
- AbuseSignal
- PeerAbuseStats
- ReputationScore
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 26 |  |
| `neutral` | 30 |  |
| `value` | 34 |  |
| `is_trusted` | 38 |  |
| `is_suspicious` | 42 |  |
| `is_abusive` | 46 |  |
| `fmt` | 53 |  |
| `new` | 103 | neutral, calculate_score, now |
| `record_signal` | 122 | new, calculate_score, now |
| `calculate_score` | 146 | new |
| `rate_limit_multiplier` | 165 | from_secs, new |
| `new` | 195 | from_secs, new, now |
| `with_backend` | 206 | from_secs, new, now |
| `load_from_storage` | 218 | to_vec, now |
| `persist_peer` | 257 | new, to_vec |
| `remove_peer_from_storage` | 274 | new |
| `apply_decay` | 286 | persist_peer, new |
| `flush_to_storage` | 335 | persist_peer, remove_peer_from_storage, new |
| `record_signal` | 351 | persist_peer, remove_peer_from_storage, new |
| `get_score` | 378 | remove_peer_from_storage, new, now |
| `rate_limit_multiplier` | 387 | remove_peer_from_storage, new, now |
| `all_reputations` | 396 | remove_peer_from_storage, new, now |
| `prune_stale` | 406 | remove_peer_from_storage, new, now, neutral |
| `len` | 429 | neutral, new, now |
| `is_empty` | 434 | neutral, new, now |
| `current_epoch_secs` | 438 | neutral, new, now |
| `test_neutral_score` | 451 | new, neutral |
| `test_successful_delivery_increases_score` | 460 | new |
| `test_rate_limiting_decreases_score` | 470 | from_secs, new |
| `test_rate_limit_multiplier` | 480 | from_secs, new |
| `test_reputation_manager_eviction` | 495 | from_secs, new, with_backend |
| `test_prune_stale` | 508 | from_secs, new, with_backend |
| `test_mixed_signals` | 518 | new, with_backend |
| `test_persistence_roundtrip` | 532 | from_utf8_lossy, new, with_backend |
| `test_persistence_eviction_cleans_storage` | 558 | from_utf8_lossy, new, with_backend |
| `test_decay_moves_toward_neutral` | 580 | new |
| `test_epoch_secs_recorded` | 608 | new |

### Imports
- `use crate::store::backend::StorageBackend`
- `use parking_lot::RwLock`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH}`
- `use super::*`
---

## core/src/mobile/service.rs (1 chunks, 598 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/mobile/service.rs: Defines 13 types: ServiceState, std, ServiceError, MeshServiceConfig, Default; 54 functions; 4 imports

### Structs/Classes
- Default
- MeshService
- MeshServiceConfig
- MockPlatformBridge
- MotionState
- PlatformBridge
- ServiceError
- ServiceState
- ServiceStats
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `fmt` | 21 | new, ConfigError |
| `default` | 57 | new, ConfigError |
| `validate` | 69 | ConfigError |
| `request_ble_scan` | 89 |  |
| `request_ble_advertise` | 90 |  |
| `request_wifi_aware_publish` | 93 | new |
| `request_wifi_aware_subscribe` | 94 | new |
| `show_notification` | 97 | default, new |
| `update_notification` | 98 | default, new |
| `get_battery_level` | 101 | default, new |
| `is_charging` | 102 | default, new |
| `is_on_wifi` | 103 | default, new |
| `get_motion_state` | 104 | default, new |
| `request_background_time` | 107 | default, new |
| `schedule_background_task` | 108 | default, new |
| `new` | 138 | default, PlatformBridgeError, new |
| `set_platform_bridge` | 150 | PlatformBridgeError |
| `start` | 155 | PlatformBridgeError |
| `stop` | 194 |  |
| `pause` | 216 | get_state |
| `resume` | 230 | get_state |
| `get_state` | 244 | get_state |
| `set_background_restricted` | 249 | get_state |
| `on_peer_discovered` | 264 | default, get_state |
| `on_peer_disconnected` | 277 | default, get_state |
| `on_data_received` | 286 | default, get_state |
| `get_service_stats` | 299 | default |
| `reset_stats` | 304 | default |
| `get_config` | 310 |  |
| `request_ble_scan` | 322 |  |
| `request_ble_advertise` | 325 | new |
| `request_wifi_aware_publish` | 329 | new |
| `request_wifi_aware_subscribe` | 333 | new |
| `show_notification` | 337 | new |
| `update_notification` | 341 | new |
| `get_battery_level` | 345 | new |
| `is_charging` | 349 | new |
| `is_on_wifi` | 353 | new |
| `get_motion_state` | 357 | new |
| `request_background_time` | 361 | new |
| `schedule_background_task` | 365 | new |
| `test_config_validation` | 372 | new |
| `test_config_all_disabled` | 384 | new |
| `test_valid_config` | 396 | new |
| `test_service_creation` | 408 | new |
| `test_service_lifecycle` | 421 | new |
| `test_double_start_fails` | 443 | new |
| `test_pause_resume` | 460 | new |
| `test_background_restricted` | 481 | new |
| `test_peer_discovery` | 502 | new |
| `test_peer_disconnection` | 522 | new |
| `test_data_received_updates_stats` | 543 | new |
| `test_reset_stats` | 564 | new |
| `test_operations_when_not_running` | 585 | new |

### Imports
- `use parking_lot::RwLock`
- `use std::sync::Arc`
- `use super::*`
- `use thiserror::Error`
---

## core/src/platform/service.rs (1 chunks, 761 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/platform/service.rs: Defines 12 types: PlatformError, MeshServiceState, std, PlatformType, PlatformCapabilities; 37 functions; 7 imports

### Structs/Classes
- Default
- MeshService
- MeshServiceConfig
- MeshServiceState
- PlatformCapabilities
- PlatformError
- PlatformType
- ServiceStats
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `fmt` | 62 |  |
| `android` | 106 |  |
| `ios` | 118 |  |
| `desktop` | 130 | InvalidConfig |
| `wasm` | 142 | InvalidConfig |
| `validate` | 171 | InvalidConfig |
| `default` | 183 |  |
| `new` | 236 | android, ios, new |
| `with_capabilities` | 262 | InvalidState, new, now |
| `start` | 288 | InvalidState, Internal, now |
| `stop` | 332 | InvalidState |
| `pause` | 374 | InvalidState |
| `resume` | 410 | InvalidState |
| `state` | 444 | now |
| `update_device_state` | 449 | now |
| `service_stats` | 467 | default, now |
| `capabilities` | 485 | default, new |
| `config` | 490 | default, new |
| `test_config_validation` | 504 | default, new |
| `test_service_creation` | 519 | default, new |
| `test_service_start_stop` | 530 | default, new |
| `test_double_start_fails` | 545 | default, new |
| `test_stop_when_stopped_fails` | 557 | default, new |
| `test_pause_resume_cycle` | 568 | default, new |
| `test_pause_when_stopped_fails` | 584 | default, android, ios, new |
| `test_resume_from_stopped_fails` | 595 | new, default, desktop, android, ios |
| `test_uptime_tracking` | 608 | wasm, new, default, desktop, android, ios |
| `test_android_capabilities` | 623 | wasm, with_capabilities, default, desktop, android, ios |
| `test_ios_capabilities` | 633 | wasm, new, with_capabilities, default, desktop, ios |
| `test_desktop_capabilities` | 643 | wasm, new, with_capabilities, default, desktop, ios |
| `test_wasm_capabilities` | 652 | wasm, new, with_capabilities, default, ios |
| `test_service_with_custom_capabilities` | 661 | default, ios, new, with_capabilities |
| `test_service_stats_initialization` | 673 | default, new |
| `test_device_state_update_without_auto_adjust` | 687 | default, new |
| `test_device_state_update_with_auto_adjust` | 710 | default, new |
| `test_state_display` | 733 | default, new |
| `test_mesh_service_state_transitions` | 740 | default, new |

### Imports
- `use crate::platform::auto_adjust::{AdjustmentProfile, DeviceState, SmartAutoAdjust}`
- `use parking_lot::RwLock`
- `use serde::{Deserialize, Serialize}`
- `use std::sync::Arc`
- `use super::*`
- `use thiserror::Error`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/crypto/session_manager.rs (1 chunks, 345 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/crypto/session_manager.rs: Defines 6 types: RatchetSessionManager, Default, RatchetSessionManager, SerializableRatchetSession, ChainState; 18 functions; 12 imports

### Structs/Classes
- ChainState
- Default
- RatchetSessionManager
- SerializableRatchetSession

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 17 | from_utf8, new, serialize_sessions, deserialize_sessions |
| `new` | 23 | from_utf8, new, init_as_sender, deserialize_sessions, serialize_sessions |
| `with_backend` | 31 | from_utf8, new, init_as_sender, deserialize_sessions, serialize_sessions |
| `save` | 39 | from_utf8, init_as_sender, deserialize_sessions, serialize_sessions, init_as_receiver |
| `load` | 50 | init_as_sender, from_utf8, init_as_receiver, deserialize_sessions |
| `get_or_create_session` | 65 | init_as_sender, init_as_receiver |
| `create_receiver_session` | 80 | to_string, from_session, init_as_receiver |
| `get_session` | 93 | to_string, from_str, from_session |
| `get_session_mut` | 98 | to_string, from_str, from_session |
| `remove_session` | 103 | to_string, from_str, from_session |
| `session_count` | 108 | to_string, from_str, from_session |
| `has_session` | 113 | to_string, from_str, from_session |
| `serialize_sessions` | 118 | to_string, from_str, from_session |
| `deserialize_sessions` | 130 | from_str |
| `from_session` | 187 | decode, from, encode |
| `into_session` | 213 | new_with_index, from, from_bytes, decode_to_slice, decode |
| `generate_signing_key` | 303 | new, from, from_bytes, with_backend |
| `test_manager_persistence_roundtrip` | 311 | from, new, with_backend |

### Imports
- `use anyhow::{bail, Result}`
- `use crate::store::backend::MemoryStorage`
- `use crate::store::backend::StorageBackend`
- `use ed25519_dalek::SigningKey`
- `use rand::RngCore`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use super::*`
- `use super::ratchet::{Chain, RatchetKey, RatchetSession}`
- `use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret}`
- `use zeroize::Zeroize`
---

## core/src/routing/smart_retry.rs (1 chunks, 328 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/smart_retry.rs: Defines 2 types: BackoffStrategy, DeliveryTrigger; 13 functions; 3 imports

### Structs/Classes
- BackoffStrategy
- DeliveryTrigger

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `calculate_next_attempt` | 29 | now |
| `get_current_timestamp` | 65 | now |
| `test_first_attempt_no_delay` | 73 |  |
| `test_exponential_backoff_increases` | 94 |  |
| `test_max_delay_enforcement` | 125 |  |
| `test_zero_base_delay` | 152 |  |
| `test_multiplier_of_one` | 173 |  |
| `test_fractional_multiplier` | 194 |  |
| `test_large_attempt_counts` | 217 | RouteUpdated, PeerDiscovered |
| `test_edge_case_max_equals_base` | 239 | RouteUpdated, PeerDiscovered |
| `test_delivery_trigger_variants` | 262 | RouteUpdated, PeerDiscovered |
| `test_delivery_trigger_equality` | 288 | PeerDiscovered |
| `test_backoff_strategy_equality` | 299 |  |

### Imports
- `use std::time::{SystemTime, UNIX_EPOCH}`
- `use super::*`
---

## core/src/abuse/spam_detection.rs (1 chunks, 498 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/abuse/spam_detection.rs: Defines 8 types: SpamDetectionConfig, Default, SpamDetectionResult, PeerMessageTrack, PeerMessageTrack; 23 functions; 9 imports

### Structs/Classes
- Default
- PeerMessageTrack
- SpamDetectionConfig
- SpamDetectionEngine
- SpamDetectionResult
- SpamSignal

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 43 | new |
| `new` | 91 | new |
| `new` | 119 | new |
| `new_heuristics_only` | 135 | new |
| `content_fingerprint` | 147 | new |
| `detect_spam` | 157 | from_secs, new, now |
| `spam_score` | 268 | detect_spam |
| `record_spam_signal` | 279 | content_fingerprint |
| `record_outbound_message` | 304 | content_fingerprint, from_secs, now |
| `is_content_suspicious` | 350 | default, new_heuristics_only, new |
| `prune_stale_peers` | 355 | default, new_heuristics_only, new |
| `make_engine` | 389 | default, content_fingerprint, new_heuristics_only, new |
| `make_heuristics_only_engine` | 396 | default, content_fingerprint, new_heuristics_only |
| `test_default_config` | 402 | default, content_fingerprint |
| `test_no_contacts_is_not_spam` | 409 | content_fingerprint |
| `test_heuristics_only_no_contacts_is_not_spam` | 417 | content_fingerprint |
| `test_content_fingerprint_deterministic` | 425 | content_fingerprint |
| `test_record_spam_signal_accumulates` | 437 |  |
| `test_record_outbound_message` | 446 |  |
| `test_record_outbound_cold_contact` | 456 |  |
| `test_content_suspicious_length` | 465 |  |
| `test_prune_stale_peers` | 474 |  |
| `test_heuristics_only_flooding_detection` | 487 |  |

### Imports
- `use crate::store::backend::MemoryStorage`
- `use crate::store::blocked::BlockedManager`
- `use crate::store::contacts::ContactManager`
- `use parking_lot::RwLock`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use std::time::Instant`
- `use super::*`
---

## core/src/drift/store.rs (1 chunks, 782 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/store.rs: Defines 6 types: MessageId, StoredEnvelope, StoredEnvelope, MeshStore, MeshStore; 35 functions; 2 imports

### Structs/Classes
- Default
- MeshStore
- MessageId
- StoredEnvelope

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `priority_score` | 45 | new, now |
| `new` | 88 | new, evict_if_over_budget |
| `with_capacity` | 96 | new, evict_if_over_budget |
| `insert` | 107 | evict_if_over_budget |
| `merge` | 123 | evict_if_over_budget |
| `get` | 133 | now |
| `contains` | 138 | now |
| `message_ids` | 143 | now |
| `messages_for_recipient` | 148 | now |
| `by_priority` | 156 | now |
| `len` | 167 | new, now |
| `is_empty` | 172 | new, now |
| `remove_expired` | 179 | new, now |
| `evict_if_over_budget` | 191 | new, now |
| `default` | 213 | new, now |
| `make_envelope` | 221 | new, now |
| `test_insert_single_message` | 245 | new |
| `test_insert_duplicate_is_idempotent` | 256 | new |
| `test_get_message` | 271 | new |
| `test_merge_non_overlapping_stores` | 284 | new |
| `test_merge_overlapping_stores` | 305 | new |
| `test_merge_commutativity` | 327 | new |
| `test_merge_idempotency` | 358 | with_capacity, new, now |
| `test_eviction_on_over_capacity` | 380 | with_capacity, now |
| `test_priority_score_newer_higher` | 413 | now |
| `test_priority_score_fewer_hops_higher` | 447 | now |
| `test_priority_score_explicit_priority` | 479 | new, now |
| `test_remove_expired_messages` | 511 | new, now |
| `test_messages_for_recipient` | 570 | new, now |
| `test_by_priority_ordering` | 597 | new, now |
| `test_message_ids` | 649 | with_capacity, new, now |
| `test_empty_store` | 669 | with_capacity, new, now |
| `test_custom_capacity` | 676 | with_capacity, now |
| `test_merge_with_eviction` | 682 | with_capacity, now |
| `test_insert_after_eviction_preserves_crdt` | 714 | with_capacity |

### Imports
- `use std::collections::HashMap`
- `use super::*`
---

## core/src/drift/sync.rs (1 chunks, 711 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/sync.rs: Defines 6 types: SyncMessage, SyncMessage, SyncState, SyncSession, SyncSession; 27 functions; 7 imports

### Structs/Classes
- Default
- SyncMessage
- SyncSession
- SyncState

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `to_bytes` | 40 | serialize, deserialize, DecompressionFailed |
| `from_bytes` | 45 | deserialize, DecompressionFailed |
| `frame_type` | 50 | new |
| `new` | 93 | new, DecompressionFailed |
| `state` | 105 | new, DecompressionFailed |
| `peer_missing_ids` | 110 | new, DecompressionFailed |
| `our_missing_ids` | 115 | new, DecompressionFailed |
| `initiate` | 120 | new, DecompressionFailed |
| `respond` | 152 | with_cells, from_bytes, DecompressionFailed |
| `complete` | 228 | serialize, from_bytes, DecompressionFailed |
| `default` | 303 | new |
| `merge_envelopes` | 309 | new |
| `make_test_envelope` | 323 | new |
| `make_test_id` | 336 | new |
| `test_sync_session_creation` | 342 | new |
| `test_sync_full_workflow_identical_stores` | 348 | new |
| `test_sync_full_workflow_disjoint_stores` | 376 | new |
| `test_sync_overlapping_stores` | 405 | from_bytes, new |
| `test_sync_message_serialization` | 439 | new, from_bytes |
| `test_sync_message_frame_types` | 464 | new |
| `test_merge_envelopes_into_store` | 487 | new |
| `test_merge_envelopes_idempotent` | 502 | new |
| `test_sync_session_state_transitions` | 519 | with_capacity, new |
| `test_sync_initiate_wrong_state_fails` | 531 | with_capacity, new |
| `test_sync_empty_stores` | 543 | with_capacity, new |
| `test_sync_large_symmetric_difference` | 561 | with_capacity, new |
| `test_sync_response_with_envelopes` | 587 | new |

### Imports
- `use bincode`
- `use crate::drift::DriftError`
- `use crate::drift::StoredEnvelope`
- `use super::*`
- `use super::frame::FrameType`
- `use super::sketch::IBLT`
- `use super::store::{MeshStore, MessageId, StoredEnvelope}`
---

## core/src/privacy/timing.rs (1 chunks, 397 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/privacy/timing.rs: Defines 11 types: JitterDistribution, JitterConfig, Default, JitterConfig, MessagePriority; 27 functions; 4 imports

### Structs/Classes
- Default
- JitterConfig
- JitterDistribution
- JitterError
- MessagePriority
- RelayTimingPolicy
- TimingJitter

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 30 | InvalidConfig |
| `validate` | 41 | InvalidConfig |
| `jitter_config` | 69 |  |
| `default` | 102 | new |
| `new` | 124 | from_millis, thread_rng, new |
| `with_priority` | 130 | from_millis, thread_rng, new |
| `compute_jitter` | 135 | default, from_millis, thread_rng |
| `config` | 140 | default, from_millis, thread_rng |
| `compute_jitter` | 155 | default, from_millis, thread_rng |
| `test_jitter_config_default` | 181 | default |
| `test_jitter_config_validate_valid` | 189 |  |
| `test_jitter_config_validate_equal` | 199 | default, new |
| `test_jitter_config_validate_invalid_order` | 209 | default, new |
| `test_jitter_config_validate_zero_max` | 219 | default, with_priority, new |
| `test_message_priority_jitter_config` | 229 | default, with_priority, new |
| `test_timing_jitter_new` | 244 | default, with_priority, new |
| `test_timing_jitter_new_invalid_config` | 251 | with_priority, new |
| `test_timing_jitter_with_priority` | 262 | from_millis, with_priority |
| `test_compute_jitter_uniform` | 269 | from_millis |
| `test_compute_jitter_exponential` | 284 | default, from_millis |
| `test_compute_jitter_equal_bounds` | 299 | default, from_millis, serialize, new |
| `test_compute_jitter_zero_min` | 311 | default, deserialize, serialize, new |
| `test_relay_timing_policy_default` | 326 | default, deserialize, serialize, new |
| `test_timing_jitter_config_access` | 334 | deserialize, serialize, new |
| `test_jitter_distribution_serialization` | 347 | serialize, deserialize |
| `test_jitter_config_serialization` | 360 | serialize, deserialize |
| `test_exponential_distribution_bias` | 373 |  |

### Imports
- `use rand::Rng`
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use web_time::Duration`
---

## core/src/message/types.rs (1 chunks, 227 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/message/types.rs: Defines 8 types: MessageType, DeliveryStatus, Message, Receipt, Envelope; 10 functions; 2 imports

### Structs/Classes
- DeliveryStatus
- Envelope
- Message
- MessageType
- Receipt
- SignedEnvelope

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `text` | 110 | serialize, new_v4, from_utf8, now |
| `receipt` | 125 | serialize, new_v4, from_utf8, now |
| `text_content` | 140 | from_utf8, text, now |
| `is_recent` | 149 | delivered, text, now |
| `delivered` | 160 | delivered, receipt, text, now |
| `test_create_text_message` | 177 | delivered, serialize, receipt, deserialize, text |
| `test_create_receipt` | 193 | delivered, serialize, receipt, deserialize, text |
| `test_receipt_message` | 200 | delivered, serialize, receipt, deserialize, text |
| `test_message_recency` | 209 | serialize, deserialize, text |
| `test_message_serialization` | 219 | serialize, deserialize, text |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
---

## cli/src/bootstrap.rs (1 chunks, 219 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/bootstrap.rs: 10 functions; 2 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default_bootstrap_nodes` | 47 | strip_peer_id, var |
| `promiscuous_bootstrap_addrs` | 92 | new, strip_peer_id |
| `parse_bootstrap_addr` | 102 | new, strip_peer_id |
| `merge_bootstrap_nodes` | 113 | new, strip_peer_id |
| `default_topics` | 137 |  |
| `test_default_bootstrap_nodes` | 146 |  |
| `test_promiscuous_addrs_strip_peer_id` | 164 |  |
| `test_parse_bootstrap_addr` | 181 |  |
| `test_merge_deduplicates_by_ip` | 197 |  |
| `test_default_topics` | 214 |  |

### Imports
- `use crate::ledger`
- `use super::*`
---

## mobile/src/lib.rs (1 chunks, 77 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
mobile/src/lib.rs: 4 functions; 2 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `test_mobile_bindings_lifecycle` | 12 | new |
| `test_mobile_identity` | 24 | new |
| `test_mobile_messaging` | 41 | new |
| `test_swarm_bridge_creation` | 66 | new |

### Imports
- `use scmessenger_core::IronCore`
- `use scmessenger_core::SwarmBridge`
---

## core/src/routing/optimized_engine.rs (1 chunks, 593 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/optimized_engine.rs: Defines 4 types: OptimizedRoutingEngine, OptimizedRoutingEngine, OptimizedRoutingMaintenance, std; 40 functions; 12 imports

### Structs/Classes
- OptimizedRoutingEngine
- OptimizedRoutingMaintenance
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 54 | new, with_defaults, default_500ms, encode |
| `route_message_optimized` | 71 | encode, from_be_bytes |
| `start_discovery_if_needed` | 183 | default_500ms |
| `advance_discovery_phase` | 194 |  |
| `current_discovery_phase` | 210 |  |
| `is_discovery_in_progress` | 215 | from_secs |
| `timeout_budget_summary` | 220 | from_secs |
| `negative_cache_stats` | 225 | from_secs |
| `prefetch_stats` | 230 | from_secs |
| `adaptive_ttl` | 235 | from_secs |
| `base_engine` | 240 | from_secs |
| `base_engine_mut` | 245 | from_secs |
| `prefetch_manager` | 250 | from_secs |
| `prefetch_manager_mut` | 255 | from_secs |
| `tick` | 260 | from_secs |
| `on_app_background` | 275 | new, now |
| `on_app_resume` | 283 | new, from_secs, now |
| `record_message_activity` | 288 | new, from_secs, now |
| `record_unreachable_peer` | 293 | new, from_secs, tick, now |
| `clear_unreachable_peer` | 298 | new, from_secs, tick, now |
| `active_paths` | 305 | new, from_secs, tick, now |
| `active_paths` | 311 | new, from_secs, tick, now |
| `refresh_delegate_routes` | 318 | from_secs, decode, tick, now |
| `run_optimization` | 338 | from_secs, decode, tick, now |
| `evaluate_all_tracked` | 350 | from_secs, decode, should_advance |
| `can_reach_destination` | 359 | decode, should_advance |
| `prune_below` | 380 | from_le_bytes, decode, should_advance |
| `should_advance` | 395 | from_le_bytes, decode |
| `multipath_mark_path_failed` | 401 | from_le_bytes, decode |
| `multipath_register_path` | 407 | from_le_bytes, decode |
| `fmt` | 438 | new |
| `make_peer_id` | 456 | new |
| `make_message_id` | 462 | new |
| `make_hint` | 466 | new |
| `test_optimized_engine_creation` | 472 | new |
| `test_negative_cache_integration` | 482 | new |
| `test_discovery_phase_advancement` | 500 | new |
| `test_app_lifecycle_integration` | 524 | new, from_secs |
| `test_adaptive_ttl_integration` | 548 | new, from_secs |
| `test_maintenance_integration` | 562 | new |

### Imports
- `use hex`
- `use super::*`
- `use super::adaptive_ttl::AdaptiveTTLManager`
- `use super::engine::*`
- `use super::global::RouteAdvertisement`
- `use super::local::PeerId`
- `use super::multipath::DeliveryPath`
- `use super::multipath::MultiPathDelivery`
- `use super::negative_cache::{NegativeCache, NegativeCacheStats}`
- `use super::resume_prefetch::{PrefetchStats, ResumePrefetchManager}`
- `use super::timeout_budget::{BudgetSummary, DiscoveryPhase, TimeoutBudget}`
- `use web_time::Duration`
---

## core/src/notification.rs (1 chunks, 660 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/notification.rs: Defines 14 types: NotificationKind, NotificationMessageContext, NotificationUiState, NotificationDecision, NotificationDecision; 29 functions; 6 imports

### Structs/Classes
- Default
- NotificationDecision
- NotificationEndpoint
- NotificationEndpointCapabilities
- NotificationEndpointError
- NotificationEndpointRegistry
- NotificationKind
- NotificationMessageContext
- NotificationPlatform
- NotificationUiState
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `suppressed` | 45 | suppressed |
| `allow` | 60 | suppressed |
| `classify_notification` | 77 | suppressed |
| `normalize_required` | 198 |  |
| `normalize_optional` | 202 |  |
| `ids_match` | 211 |  |
| `as_str` | 229 |  |
| `fmt` | 271 | new, StorageError |
| `new` | 294 | new |
| `register_endpoint` | 302 |  |
| `unregister_endpoint` | 340 |  |
| `list_endpoints` | 349 |  |
| `touch_endpoint` | 355 |  |
| `clear_all_request_notifications` | 372 |  |
| `clear_message_notifications` | 393 | new, now |
| `close_all_notifications` | 413 | new, default, now |
| `default` | 422 | new, default, now |
| `now_ms` | 426 | now, default |
| `base_message` | 437 | default |
| `base_ui_state` | 452 | default |
| `unknown_sender_defaults_to_direct_message_request` | 461 | default |
| `known_contact_defaults_to_direct_message` | 471 | default |
| `explicit_request_overrides_known_contact_inference` | 480 | default |
| `disabled_notifications_suppress_delivery` | 490 | default |
| `duplicates_are_suppressed` | 505 | new, default |
| `foreground_direct_messages_follow_foreground_toggle` | 518 | new, default |
| `clear_all_request_notifications_removes_request_only_endpoints` | 545 | new |
| `clear_message_notifications_clears_by_device_id` | 592 | new |
| `close_all_notifications_clears_entire_registry` | 631 | new |

### Imports
- `use crate::MeshSettings`
- `use parking_lot::Mutex`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use super::*`
---

## core/src/contacts_bridge.rs (1 chunks, 377 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/contacts_bridge.rs: Defines 4 types: Contact, Contact, ContactManager, ContactManager; 23 functions; 8 imports

### Structs/Classes
- Contact
- ContactManager

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 29 | from, default |
| `with_nickname` | 41 | from, new, default, to_vec |
| `display_name` | 46 | from, new, default, to_vec |
| `federated_nickname` | 53 | from, new, default, to_vec |
| `new` | 69 | from_slice, default, new, to_vec, from |
| `add` | 85 | new, from_slice, to_vec |
| `get` | 100 | new, from_slice |
| `remove` | 116 | new, from_slice |
| `list` | 124 | new, from_slice |
| `search` | 141 | get, new, from_slice, add |
| `set_nickname` | 172 | get, add |
| `set_local_nickname` | 189 | get, add |
| `update_last_seen` | 206 | get, add, new |
| `update_device_id` | 218 | get, add, new |
| `reconcile_from_history` | 234 | get, new, add |
| `count` | 259 | get, new, add, now |
| `flush` | 263 | get, new, add, now |
| `verify_integrity` | 271 | get, new, add, now |
| `emergency_recover` | 285 | new, now, get, add, tempdir |
| `current_timestamp` | 302 | new, now, tempdir |
| `test_contact_creation` | 315 | new, tempdir |
| `test_contact_manager` | 324 | new, tempdir |
| `test_contact_persistence_across_manager_restart` | 356 | new, tempdir |

### Imports
- `use anyhow::{Context, Result}`
- `use crate::mobile_bridge::HistoryManager`
- `use parking_lot::Mutex`
- `use serde::{Deserialize, Serialize}`
- `use sled::Db`
- `use std::path::PathBuf`
- `use std::sync::Arc`
- `use super::*`
---

## core/src/drift/frame.rs (1 chunks, 423 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/frame.rs: Defines 4 types: DriftFrame, FrameType, FrameType, DriftFrame; 20 functions; 4 imports

### Structs/Classes
- DriftFrame
- FrameType

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `from_u8` | 47 | with_capacity, new, InvalidFrameType |
| `as_u8` | 59 | new, with_capacity |
| `to_bytes` | 72 | from_le_bytes, new, with_capacity |
| `from_bytes` | 111 | new, from_le_bytes, from_u8 |
| `read_with_timeout` | 183 | from_le_bytes, from_bytes, IoError, with_capacity |
| `make_test_frame` | 229 | from_bytes, from_u8 |
| `test_frame_type_conversion` | 238 | from_bytes, from_u8 |
| `test_frame_serialize_deserialize` | 251 | from_bytes |
| `test_frame_empty_payload` | 263 | from_bytes |
| `test_frame_large_payload` | 275 | from_bytes |
| `test_frame_crc32_validation` | 287 | from_bytes |
| `test_frame_crc32_tamper_type` | 299 | from_bytes |
| `test_frame_crc32_tamper_crc` | 311 | from_bytes |
| `test_frame_buffer_too_short` | 325 | from_bytes |
| `test_frame_length_mismatch` | 339 | from_bytes |
| `test_frame_invalid_type` | 349 | from_le_bytes, from_bytes |
| `test_frame_all_types` | 362 | from_le_bytes, from_bytes |
| `test_frame_length_calculation` | 384 | from_bytes, from_le_bytes |
| `test_frame_crc32_deterministic` | 403 | from_bytes |
| `test_frame_multiple_roundtrips` | 412 | from_bytes |

### Imports
- `use crc32fast::Hasher`
- `use super::*`
- `use super::DriftError`
- `use tokio::time::timeout`
---

## core/src/routing/resume_prefetch.rs (1 chunks, 556 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/resume_prefetch.rs: Defines 12 types: PrefetchStatus, PrefetchedRoute, PrefetchedRoute, PrefetchConfig, Default; 34 functions; 6 imports

### Structs/Classes
- Default
- FrequentPeer
- PrefetchConfig
- PrefetchStats
- PrefetchStatus
- PrefetchedRoute
- ResumePrefetchManager
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 49 | now |
| `is_fresh` | 61 | now |
| `is_usable` | 66 | now, from_secs |
| `start_refresh` | 78 | now, from_secs, from_millis |
| `complete_refresh` | 84 | now, from_secs, from_millis |
| `fail_refresh` | 90 | from_secs, from_millis |
| `default` | 111 | from_secs, from_millis |
| `new` | 160 | new, now, default |
| `record_message` | 169 | new, now, default |
| `decay` | 176 | new, default |
| `new` | 187 | new, default, now |
| `with_defaults` | 200 | new, default, now, Reverse |
| `on_app_background` | 207 | new, now, Reverse |
| `on_app_resume` | 232 | new, now, Reverse |
| `get_route_early` | 260 | new |
| `update_route` | 271 | new |
| `mark_refresh_failed` | 283 |  |
| `next_refresh_hint` | 291 |  |
| `is_prefetch_complete` | 296 | from_secs |
| `is_prefetch_in_progress` | 301 | from_secs |
| `stats` | 306 | from_secs |
| `record_message` | 343 | new, from_secs, Reverse |
| `top_frequent_peers` | 373 | with_defaults |
| `clear` | 378 | with_defaults, thread_rng |
| `fmt` | 399 | now, with_defaults, thread_rng |
| `default` | 414 | now, with_defaults, thread_rng |
| `create_test_peer_id` | 423 | now, with_defaults, thread_rng |
| `create_test_route` | 430 | now, with_defaults |
| `test_prefetch_basic` | 447 | with_defaults |
| `test_get_route_early` | 462 | with_defaults |
| `test_update_route` | 477 | with_defaults |
| `test_frequent_peer_tracking` | 496 | now, from_secs, with_defaults |
| `test_prefetch_stats` | 512 | now, from_secs, with_defaults |
| `test_frequent_peer_decay` | 529 | now, from_secs, with_defaults |

### Imports
- `use rand::RngCore`
- `use std::collections::{HashMap, VecDeque}`
- `use super::*`
- `use super::global::RouteAdvertisement`
- `use super::local::PeerId`
- `use web_time::{Duration, Instant}`
---

## core/src/relay/invite.rs (1 chunks, 498 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/invite.rs: Defines 8 types: InviteError, InviteToken, InviteToken, InviteChain, InviteChain; 47 functions; 5 imports

### Structs/Classes
- Default
- InviteChain
- InviteError
- InviteSystem
- InviteToken

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 44 | new, now |
| `with_expiry` | 62 | deserialize, new, now, serialize, SerializationError |
| `with_metadata` | 68 | deserialize, new, now, serialize, SerializationError |
| `with_signature` | 74 | deserialize, new, now, serialize, SerializationError |
| `is_valid` | 80 | deserialize, new, now, serialize, SerializationError |
| `get_signable_data` | 90 | deserialize, new, now, serialize, SerializationError |
| `to_bytes` | 105 | deserialize, new, now, serialize, SerializationError |
| `from_bytes` | 110 | new, SerializationError, deserialize, now |
| `new` | 124 | new, now, get_inviter |
| `record_invite` | 131 | get_trust_chain, now, get_inviter |
| `get_inviter` | 142 | get_trust_chain, get_inviter |
| `get_invitees` | 152 | get_trust_chain, get_inviter |
| `get_trust_chain` | 166 | get_trust_chain, new, get_inviter |
| `distance_from_root` | 179 | get_trust_chain, new |
| `invite_count` | 184 | new |
| `clear` | 189 | new |
| `get_direct_invitations` | 194 | new |
| `default` | 209 | new |
| `new` | 226 | new |
| `create_invite_token` | 235 | new |
| `record_invitation` | 240 | new |
| `get_trust_chain` | 247 | new |
| `get_invitees` | 252 | new |
| `get_inviter` | 257 | new |
| `is_direct_connection` | 262 | new |
| `get_connected_peers` | 270 | new |
| `test_token` | 288 | sleep, new, from_bytes, from_millis |
| `test_invite_token_creation` | 294 | sleep, new, from_bytes, from_millis |
| `test_invite_token_with_expiry` | 302 | sleep, new, from_bytes, from_millis |
| `test_invite_token_with_metadata` | 308 | sleep, new, from_bytes, from_millis |
| `test_invite_token_validity` | 314 | sleep, new, from_bytes, from_millis |
| `test_invite_token_expiry_check` | 323 | sleep, new, from_bytes, from_millis |
| `test_invite_token_serialization` | 331 | new, from_bytes |
| `test_invite_chain_creation` | 342 | new |
| `test_record_invite` | 348 | new |
| `test_get_invitees` | 357 | new |
| `test_get_trust_chain` | 373 | new |
| `test_distance_from_root` | 385 | new |
| `test_get_direct_invitations` | 402 | new |
| `test_invite_system_creation` | 416 | new |
| `test_create_invite_token` | 422 | new |
| `test_record_invitation` | 431 | new |
| `test_get_inviter` | 440 | new |
| `test_is_direct_connection` | 455 | new |
| `test_get_connected_peers` | 464 | new |
| `test_get_trust_chain_via_system` | 476 | new |
| `test_chain_clear` | 490 | new |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use super::*`
- `use thiserror::Error`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/relay/delegate_prewarm.rs (1 chunks, 426 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/delegate_prewarm.rs: Defines 10 types: DelegateInfo, DelegateInfo, WarmConnection, WarmConnection, DelegatePrewarmManager; 26 functions; 5 imports

### Structs/Classes
- Default
- DelegateInfo
- DelegatePrewarmConfig
- DelegatePrewarmManager
- DelegatePrewarmStats
- WarmConnection
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 39 | now |
| `update` | 52 | now |
| `has_capacity` | 59 | now |
| `score` | 64 | now |
| `new` | 88 | now |
| `update_keepalive` | 100 | now |
| `mark_registered` | 105 | from_secs |
| `is_stale` | 110 | from_secs |
| `default` | 148 | select_best_delegates, from_secs, Vacant, default, new, now |
| `new` | 162 | select_best_delegates, Vacant, default, new, now |
| `with_defaults` | 173 | new, select_best_delegates, default, Vacant |
| `add_delegate` | 178 | new, Vacant, select_best_delegates |
| `prewarm_for_background` | 185 | new, Vacant, select_best_delegates, now |
| `refresh_delegate_routes` | 215 | new, default, now |
| `select_best_delegates` | 237 | default |
| `tick` | 256 | now, default |
| `stats` | 289 | now |
| `active_connection_count` | 303 | generate_ed25519, new, from |
| `registered_delegate_count` | 308 | generate_ed25519, new, from |
| `fmt` | 330 | generate_ed25519, new, from, with_defaults |
| `create_test_delegate` | 345 | generate_ed25519, new, from, with_defaults |
| `test_delegate_creation` | 359 | new, with_defaults, now |
| `test_delegate_selection` | 366 | new, with_defaults, now |
| `test_prewarm_for_background` | 380 | new, with_defaults, now |
| `test_tick_maintenance` | 398 | new, with_defaults, now |
| `test_stats` | 415 | with_defaults |

### Imports
- `use libp2p::PeerId`
- `use libp2p::identity::Keypair`
- `use std::collections::{HashMap, VecDeque}`
- `use super::*`
- `use web_time::{Duration, Instant}`
---

## core/src/dspy/signatures.rs (1 chunks, 222 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/dspy/signatures.rs: Defines 8 types: ArchitectSignature, ArchitectSignature, CoderSignature, CoderSignature, VerifierSignature; 15 functions; 8 imports

### Structs/Classes
- ArchitectSignature
- AuditorSignature
- CoderSignature
- VerifierSignature

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 21 |  |
| `new` | 39 |  |
| `new` | 57 | from, new |
| `new` | 75 | from, new |
| `generate_keypair` | 91 | from, new, hash |
| `encrypt_xchacha20` | 110 | new, hash |
| `blake3_hash` | 134 | new, hash |
| `signature_fingerprint` | 139 | new, to_string, from_str |
| `blake3_hash` | 147 | new, to_string, from_str |
| `get_signature` | 167 | new, to_string, from_str |
| `test_architect_signature_serialization` | 179 | new, to_string, from_str |
| `test_golden_examples_exist` | 194 |  |
| `test_blake3_hash_deterministic` | 201 |  |
| `test_blake3_hash_different_inputs` | 210 |  |
| `test_signature_fingerprint_format` | 217 |  |

### Imports
- `use blake3::hash`
- `use chacha20::ChaCha20`
- `use chacha20::cipher::{KeyIVInit, StreamCipher}`
- `use poly1305::Poly1305`
- `use ring::eddsa::KeyPair`
- `use ring::rand::SecureRandom`
- `use serde::{Deserialize, Serialize}`
- `use super::*`
---

## wasm/src/lib.rs (1 chunks, 2347 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
wasm/src/lib.rs: Defines 30 types: DiscoveryMode, MeshSettings, Default, From, MeshSettingsManager; 137 functions; 8 imports

### Structs/Classes
- Default
- DiscoveryMode
- From
- IronCore
- IronCoreMode
- MeshSettings
- MeshSettingsManager
- WasmContactManager
- WasmHistoryManager
- WasmHistoryStats
- WasmIdentityInfo
- WasmMeshSettings
- WasmMessage
- WasmNotificationDecision
- WasmNotificationMessageContext
- WasmNotificationUiState
- WasmPreparedMessage
- WasmRegistrationStateInfo
- WasmSignatureResult

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 50 |  |
| `from` | 76 | from, from_str, read_to_string |
| `new` | 110 | from_str, create_dir_all, default, validate, to_string_pretty, read_to_string, from, write |
| `load` | 113 | from_str, create_dir_all, default, validate, to_string_pretty, read_to_string, from, write |
| `save` | 130 | set_as_global_default, create_dir_all, validate, new, to_string_pretty, set_once, from, write |
| `validate` | 149 | set_as_global_default, new, set_once |
| `init_logging` | 172 | set_as_global_default, new, set_once |
| `new` | 215 | with_storage, new, default |
| `with_storage` | 246 | with_storage, new, default |
| `with_storage` | 277 | new, default |
| `with_storage_async` | 308 | to_value, default, new, from_value, from |
| `start` | 329 | from, to_value, from_value |
| `stop` | 335 | from, to_value, from_value |
| `is_running` | 341 | from, to_value, from_value |
| `get_identity_info` | 346 | from, to_value, from_value |
| `set_iron_core_mode` | 355 | to_value, from_value |
| `get_iron_core_mode` | 364 | to_value |
| `set_daemon_socket_url` | 372 | to_value |
| `get_daemon_socket_url` | 378 | to_value |
| `initialize_identity` | 390 |  |
| `initialize_identity_from_daemon` | 417 |  |
| `get_identity_from_daemon` | 452 | from, to_value |
| `sign_data` | 475 | from, to_value |
| `verify_signature` | 483 |  |
| `prepare_message` | 500 | to_value |
| `receive_message` | 536 | to_value, clone |
| `outbox_count` | 554 | clone |
| `flush_outbox_for_peer` | 561 | clone |
| `inbox_count` | 566 | clone |
| `start_swarm` | 574 | clone |
| `stop_swarm` | 588 |  |
| `send_prepared_envelope` | 601 | to_value |
| `get_peers` | 626 | to_value |
| `get_external_addresses` | 645 | to_value |
| `get_connection_path_state` | 664 | outbox_count, inbox_count, is_running, new, now, get_connection_path_state |
| `export_diagnostics` | 690 | outbox_count, inbox_count, is_running, new, now, get_connection_path_state |
| `subscribe_topic` | 722 |  |
| `unsubscribe_topic` | 740 |  |
| `publish_topic` | 758 |  |
| `dial` | 778 | new |
| `send_to_all_peers` | 798 | new |
| `get_listeners` | 838 | to_value |
| `get_nat_status` | 860 | to_value, from_value, to_string, from_str |
| `get_drift_state` | 865 | to_value, from_value, to_string, from_str |
| `get_drift_store_size` | 870 | to_value, from_value, to_string, from_str |
| `get_audit_log` | 875 | to_value, from_value, to_string, from_str |
| `get_audit_events_since` | 881 | to_value, from_value, to_string, from_str |
| `get_peer_reputation` | 887 | to_value, from_value, to_string, from_str |
| `get_enhanced_peer_reputation` | 892 | to_value, from_value, to_string, from_str |
| `get_privacy_config` | 898 | to_value, from_value, to_string, from_str |
| `set_privacy_config` | 904 | clone, from_value, to_string, from_str |
| `validate_settings` | 916 | clone, from_value |
| `start_receive_loop` | 942 | to_value, clone, new, from |
| `drain_received_messages` | 968 | from, to_value, new, from_value |
| `get_settings` | 986 | from, to_value, from_value, default |
| `update_settings` | 994 | from, to_value, from_value, default |
| `get_default_settings` | 1019 | from, to_value, from_value, default |
| `classify_notification` | 1032 | from, to_value, from_value |
| `set_nickname` | 1053 |  |
| `export_identity_backup` | 1061 | to_value |
| `import_identity_backup` | 1070 | to_value |
| `extract_public_key_from_peer_id` | 1082 | to_value |
| `prepare_message_with_id` | 1093 | to_value |
| `prepare_receipt` | 1119 |  |
| `prepare_cover_traffic` | 1134 |  |
| `mark_message_sent` | 1143 |  |
| `get_contact_manager` | 1148 |  |
| `get_history_manager` | 1155 |  |
| `resolve_identity` | 1166 |  |
| `resolve_to_identity_id` | 1174 |  |
| `block_peer` | 1184 | set, new, from_str |
| `unblock_peer` | 1192 | set, new, from_str |
| `block_and_delete_peer` | 1201 | set, new, from_str, from_f64 |
| `is_peer_blocked` | 1213 | set, new, from_str, from_f64 |
| `list_blocked_peers` | 1221 | from_str, set, new, from_f64, from_bool |
| `blocked_count` | 1274 | to_value |
| `get_device_id` | 1284 | to_value |
| `get_seniority_timestamp` | 1290 | to_value |
| `get_registration_state` | 1296 | to_value |
| `blake3_hash` | 1311 |  |
| `perform_maintenance` | 1320 | from_value |
| `update_disk_stats` | 1328 | from_value |
| `record_log` | 1334 | to_value, from_value |
| `export_logs` | 1340 | to_value, from_value |
| `notify_peer_discovered` | 1350 | to_value, from_value, new |
| `notify_peer_disconnected` | 1356 | to_value, from_value, new |
| `add` | 1369 | to_value, from_value, new |
| `get` | 1378 | to_value, new |
| `remove` | 1387 | to_value, new |
| `list` | 1394 | to_value, new |
| `count` | 1407 | to_value, new |
| `set_local_nickname` | 1412 | to_value, new |
| `search` | 1424 | to_value, new |
| `set_nickname` | 1438 |  |
| `update_last_seen` | 1446 | from_value |
| `update_device_id` | 1454 | from_value |
| `flush` | 1466 | to_value, new, from_value |
| `add` | 1487 | to_value, new, from_value |
| `recent` | 1497 | to_value, new |
| `conversation` | 1514 | to_value, new |
| `clear` | 1527 | to_value |
| `stats` | 1534 | to_value, new |
| `count` | 1549 | to_value, new |
| `enforce_retention` | 1554 | to_value, new |
| `prune_before` | 1561 | to_value, new |
| `get` | 1569 | to_value, new |
| `search` | 1579 | to_value, new, from_str |
| `mark_delivered` | 1593 | new, from_str, from_value |
| `clear_conversation` | 1602 | new, from_str, from_value |
| `delete` | 1610 | new, from_str, from_value |
| `flush` | 1618 | new, from_str, from_value |
| `js_value_from_str` | 1622 | new, from_str, from_value |
| `parse_bootstrap_addrs` | 1634 | new, from_value |
| `relay_url_to_multiaddr` | 1643 |  |
| `ensure_mesh_participation_enabled` | 1696 | start_swarm_with_config, downgrade, channel |
| `start_swarm_runtime` | 1705 | clone, downgrade, channel, spawn_local, start_swarm_with_config |
| `resolve_swarm_keypair_and_mode` | 1819 | generate_ed25519 |
| `from` | 1851 |  |
| `from` | 1872 |  |
| `from` | 1931 |  |
| `from` | 1961 |  |
| `from` | 2006 |  |
| `from` | 2030 | new |
| `from` | 2050 | with_storage, new |
| `test_wasm_core_creation` | 2078 | with_storage, new |
| `test_wasm_identity` | 2087 | with_storage, new |
| `test_desktop_identity_flow_exposes_metadata_after_init` | 2096 | with_storage |
| `test_relay_url_to_multiaddr_ws_defaults` | 2129 | new |
| `test_relay_url_to_multiaddr_wss_defaults` | 2135 | with_storage, new |
| `test_relay_url_to_multiaddr_ipv4_port` | 2141 | with_storage, new |
| `test_relay_url_to_multiaddr_rejects_http` | 2147 | with_storage, new |
| `test_desktop_role_resolution_defaults_to_relay_only_without_identity` | 2153 | with_storage, new |
| `test_desktop_relay_only_flow_blocks_outbound_message_prepare` | 2171 | with_storage, new |
| `test_desktop_contacts_and_messaging_interaction_flow` | 2181 | with_storage, new |
| `test_desktop_mesh_dashboard_stats_update_with_message_flow` | 2218 | create_dir_all, with_storage, new, now, temp_dir |
| `test_notification_manager_creation` | 2255 | new, create_dir_all, temp_dir, now |
| `temp_storage_path` | 2259 | now, create_dir_all, temp_dir |

### Imports
- `use anyhow::Error`
- `use libp2p::{Multiaddr, PeerId}`
- `use parking_lot::Mutex`
- `use scmessenger_core::store::{Contact, MessageDirection, MessageRecord}`
- `use std::sync::Arc`
- `use super::*`
- `use wasm_bindgen::prelude::*`
- `use wasm_bindgen_test::*`
---

## core/src/abuse/reputation.rs (1 chunks, 274 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/abuse/reputation.rs: Defines 4 types: EnhancedAbuseReputationManager, EnhancedAbuseReputationManager, EnhancedReputationScore, EnhancedReputationScore; 24 functions; 9 imports

### Structs/Classes
- EnhancedAbuseReputationManager
- EnhancedReputationScore

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 20 | new, with_backend |
| `with_backend` | 29 | with_backend |
| `apply_decay` | 41 |  |
| `flush_to_storage` | 46 |  |
| `record_signal` | 52 |  |
| `record_spam_signal` | 71 |  |
| `record_outbound_message` | 101 |  |
| `get_enhanced_score` | 112 |  |
| `get_score` | 124 |  |
| `rate_limit_multiplier` | 129 |  |
| `all_enhanced_scores` | 134 |  |
| `spam_detector` | 153 | new |
| `base_manager` | 158 | new, default |
| `overall_score` | 177 | new, default |
| `is_suspicious` | 183 | new, default |
| `is_abusive` | 188 | new, default |
| `make_manager` | 201 | new, default |
| `test_neutral_peer_has_neutral_score` | 212 |  |
| `test_positive_signals_increase_score` | 219 |  |
| `test_negative_signals_decrease_score` | 229 |  |
| `test_enhanced_score_combines_base_and_spam` | 239 |  |
| `test_spam_signal_recording` | 248 |  |
| `test_all_enhanced_scores` | 257 |  |
| `test_outbound_message_tracking` | 266 |  |

### Imports
- `use crate::abuse::spam_detection::SpamDetectionConfig`
- `use crate::abuse::spam_detection::{SpamDetectionEngine, SpamSignal}`
- `use crate::store::backend::MemoryStorage`
- `use crate::store::blocked::BlockedManager`
- `use crate::store::contacts::ContactManager`
- `use crate::transport::reputation::{AbuseReputationManager, AbuseSignal, ReputationScore}`
- `use std::sync::Arc`
- `use super::*`
---

## core/src/dspy/modules.rs (1 chunks, 330 lines)
Function `BATCH_P1_CORE_MYCO_ROUTING` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/dspy/modules.rs: Defines 26 types: DSPyModule, Input, Output, ModuleMetadata, ModuleMetadata; 29 functions; 2 imports

### Structs/Classes
- ChainOfThought
- DSPyError
- DSPyModule
- Input
- ModuleComplexity
- ModuleFactory
- ModuleMetadata
- MultiHopRecall
- OptimizerPipeline
- Output
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `execute` | 20 | ExecutionError, ValidationError, OptimizerError, blake3_hash, new |
| `validate_input` | 23 | ExecutionError, ValidationError, OptimizerError, blake3_hash, new |
| `get_metadata` | 26 | ExecutionError, ValidationError, OptimizerError, blake3_hash, new |
| `fingerprint` | 38 | ExecutionError, ValidationError, OptimizerError, blake3_hash, new |
| `fmt` | 66 | ValidationError, ExecutionError, OptimizerError |
| `new` | 88 |  |
| `add_step` | 98 |  |
| `execute` | 107 |  |
| `validate_input` | 113 | ValidationError, recall |
| `get_metadata` | 117 | ValidationError, recall |
| `new` | 134 | ValidationError, recall |
| `recall` | 144 | ValidationError, recall |
| `execute` | 155 | ValidationError, recall |
| `validate_input` | 162 |  |
| `get_metadata` | 166 |  |
| `new` | 183 | new |
| `run_optimization` | 193 | new |
| `execute` | 204 | new |
| `validate_input` | 209 | new |
| `get_metadata` | 213 | new |
| `create_cot` | 227 | new |
| `create_multihop` | 230 | new |
| `create_optimizer` | 234 | new |
| `build_rust_feature_pipeline` | 240 | build_rust_feature_pipeline, new |
| `build_security_audit_pipeline` | 255 | build_rust_feature_pipeline, new |
| `test_chain_of_thought_module` | 274 | build_rust_feature_pipeline, new |
| `test_multihop_recall` | 281 | build_rust_feature_pipeline, new |
| `test_rust_feature_pipeline` | 288 | build_rust_feature_pipeline |
| `test_module_metadata_fingerprint` | 294 |  |

### Imports
- `use crate::dspy::signatures`
- `use super::*`
---
