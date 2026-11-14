# REPO_MAP Context for Task: [VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses

**Target function: `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses`**

## core/src/transport/abstraction.rs (1 chunks, 535 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/abstraction.rs: Defines 9 types: TransportType, fmt, TransportCapabilities, TransportCapabilities, TransportEvent; 26 functions; 5 imports

### Structs/Classes
- TransportCapabilities
- TransportCommand
- TransportError
- TransportEvent
- TransportType
- fmt

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `fmt` | 25 |  |
| `new` | 53 |  |
| `for_transport` | 70 |  |
| `fmt` | 144 |  |
| `fmt` | 206 |  |
| `test_transport_type_display` | 267 | for_transport, new |
| `test_transport_type_hash` | 276 | for_transport, new |
| `test_transport_capabilities_creation` | 286 | for_transport, new |
| `test_transport_capabilities_for_ble` | 296 | for_transport |
| `test_transport_capabilities_for_wifi_aware` | 306 | for_transport |
| `test_transport_capabilities_for_wifi_direct` | 314 | for_transport, PeerNotFound |
| `test_transport_capabilities_for_internet` | 322 | TransportNotAvailable, PeerNotFound, SendFailed, for_transport, ConnectionFailed |
| `test_transport_capabilities_for_local` | 329 | InvalidPayload, Internal, TransportIoError, TransportNotAvailable, SerializationError, PeerNotFound, SendFailed, Timeout, for_transport, ConnectionFailed |
| `test_transport_event_display` | 337 | InvalidPayload, Internal, TransportIoError, TransportNotAvailable, SerializationError, PeerNotFound, SendFailed, Timeout, ConnectionFailed |
| `test_transport_command_display` | 350 | InvalidPayload, Internal, TransportIoError, TransportNotAvailable, SerializationError, PeerNotFound, SendFailed, Timeout, ConnectionFailed |
| `test_transport_error_display` | 363 | InvalidPayload, Internal, TransportIoError, TransportNotAvailable, SerializationError, PeerNotFound, SendFailed, Timeout, ConnectionFailed |
| `test_transport_error_types` | 369 | InvalidPayload, Internal, TransportIoError, TransportNotAvailable, SerializationError, SendFailed, Timeout, ConnectionFailed |
| `test_transport_event_peer_discovered` | 381 |  |
| `test_transport_event_data_received` | 404 |  |
| `test_transport_command_send_data` | 427 |  |
| `test_transport_command_connect` | 450 | serialize, deserialize |
| `test_transport_command_disconnect` | 470 | serialize, deserialize |
| `test_serialization_transport_event` | 482 | serialize, for_transport, deserialize, PeerNotFound |
| `test_all_transport_types_distinct` | 506 | for_transport, PeerNotFound |
| `test_transport_capabilities_clone` | 522 | for_transport, PeerNotFound |
| `test_transport_error_clone` | 530 | PeerNotFound |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashSet`
- `use std::fmt`
- `use super::*`
- `use thiserror::Error`
---

## core/src/abuse/auto_block.rs (1 chunks, 336 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/transport/ble/beacon.rs (1 chunks, 372 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/ble/beacon.rs: Defines 7 types: BleBeaconError, BleBeacon, BleBeacon, BeaconBuilder, BeaconBuilder; 22 functions; 4 imports

### Structs/Classes
- BeaconBuilder
- BeaconParser
- BleBeacon
- BleBeaconError

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `size` | 47 | new, now, SystemTimeError |
| `new` | 61 | from_slice, new, now, SystemTimeError |
| `with_rotation_period` | 70 | from_slice, new, now, SystemTimeError |
| `build` | 76 | now, SystemTimeError, new, CreationFailed, from_slice |
| `new` | 139 | new, now, SystemTimeError |
| `with_rotation_period` | 147 | new, now, SystemTimeError |
| `parse` | 153 | new, now, SystemTimeError |
| `is_fresh` | 159 | new, now, SystemTimeError |
| `test_beacon_builder_creates_beacon` | 176 | new |
| `test_beacon_size_matches_contents` | 188 | new |
| `test_beacon_parser_decrypts_valid_beacon` | 200 | new |
| `test_beacon_parser_rejects_wrong_key` | 217 | new |
| `test_beacon_parser_detects_fresh_beacons` | 232 | new |
| `test_beacon_parser_accepts_skewed_epochs` | 248 | new |
| `test_beacon_parser_rejects_stale_epochs` | 271 | new |
| `test_beacon_builder_with_custom_rotation_period` | 288 | new |
| `test_beacon_parser_with_custom_rotation_period` | 300 | new |
| `test_beacon_rotation_different_epochs` | 318 | new |
| `test_beacon_parser_rejects_invalid_format` | 341 | new |
| `test_beacon_service_uuid_constant` | 352 | new |
| `test_default_rotation_period_constant` | 357 | new |
| `test_beacon_builder_default_rotation` | 362 | new |

### Imports
- `use crate::transport::discovery::{decrypt_beacon_with_period, BeaconPayload}`
- `use super::*`
- `use thiserror::Error`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/transport/behaviour.rs (1 chunks, 681 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/behaviour.rs: Defines 19 types: IronCoreBehaviour, MessageRequest, MessageResponse, RelayRequest, RelayResponse; 21 functions; 9 imports

### Structs/Classes
- DeregistrationPayload
- DeregistrationRequest
- IronCoreBehaviour
- LedgerExchangeRequest
- LedgerExchangeResponse
- MessageRequest
- MessageResponse
- RegistrationMessage
- RegistrationPayload
- RegistrationRequest
- RegistrationResponse
- RelayRequest
- RelayResponse
- SharedPeerEntry

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `canonical_bytes` | 128 | verify, serialize |
| `validate_fields` | 131 | verify |
| `new_signed` | 150 | verify, serialize |
| `verify_for_public_key` | 166 | verify, serialize, parse_str |
| `canonical_bytes` | 198 | serialize, parse_str |
| `validate_fields` | 201 | parse_str |
| `new_signed` | 230 | verify |
| `verify_for_public_key` | 246 | verify, parse_str |
| `validate_identity_id` | 281 | encode, parse_str, hash |
| `validate_uuid_v4` | 293 | encode, parse_str, hash |
| `validate_signature_bytes` | 301 | hash, encode |
| `validate_identity_owner` | 308 | hash, encode |
| `new` | 367 | default, from_secs, new |
| `relay_request_carries_ws13_metadata_when_set` | 538 | generate, new_signed, from_str |
| `relay_request_missing_ws13_fields_deserialize_with_defaults` | 554 | generate, new_signed, from_str |
| `registration_payload_canonical_bytes_are_stable` | 571 | generate, new_signed |
| `signed_registration_request_verifies_against_matching_public_key` | 585 | generate, new_signed |
| `signed_registration_request_rejects_tampered_payload` | 599 | generate, new_signed |
| `signed_registration_request_rejects_malformed_identity_id` | 617 | generate, new_signed |
| `signed_deregistration_request_verifies_against_matching_public_key` | 635 | generate, new_signed |
| `signed_deregistration_request_rejects_same_source_and_target_device` | 649 | generate, new_signed |

### Imports
- `use crate::identity::IdentityKeys`
- `use libp2p::mdns`
- `use libp2p::swarm::behaviour::toggle::Toggle`
- `use libp2p::upnp`
- `use super::*`
- `use super::reflection::{AddressReflectionRequest, AddressReflectionResponse}`
- `use uuid::Uuid`
- `use web_time::Duration`
---

## cli/src/ble_daemon.rs (2 chunks, 449 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/ble_daemon.rs: Defines 11 types: BleError, std, std, BleResult, BleStatus; 23 functions; 3 imports cli/src/ble_daemon.rs: Defines 11 types: BleError, std, std, BleResult, BleStatus; 23 functions; 3 imports

### Structs/Classes
- BleAdapterInfo
- BleConfig
- BleDaemon
- BleError
- BleResult
- BleStatus
- Default
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `probe_and_log` | 8 | new |
| `fmt` | 79 | ManagerInitFailed, Other |
| `default` | 126 | Unavailable, ManagerInitFailed, new |
| `new` | 146 | ManagerInitFailed, get_adapter_info, Unavailable, Available, Other, new |
| `initialize` | 161 | ManagerInitFailed, get_adapter_info, Unavailable, Available, Other, new |
| `get_adapter_info` | 212 | is_available, status, Other, Available |
| `is_available` | 225 | is_available, status, Other, Available |
| `status` | 230 | is_available, status, Other |
| `scan_for_advertisements` | 236 | is_available, status, Other |
| `advertise_service` | 265 | is_available, new, Other, default, status |
| `shutdown` | 287 | default, new, from_timestamp |
| `default` | 293 | default, new, from_timestamp |
| `is_ble_available` | 300 | new, from_timestamp |
| `format_bytes` | 323 | Other, from_utf8_lossy, new, from_timestamp |
| `format_timestamp` | 336 | Other, new, default, from_utf8_lossy, from_timestamp |
| `try_enable_bluetooth` | 345 | ManagerInitFailed, Unavailable, Other, new, default, from_utf8_lossy |
| `test_ble_error_display` | 371 | ManagerInitFailed, Unavailable, new, Other, default |
| `test_ble_config_default` | 383 | default, Unavailable, ManagerInitFailed, new |
| `test_ble_status_initialization` | 390 | default, Unavailable, ManagerInitFailed, new |
| `test_ble_status_disabled` | 400 | default, Unavailable, new |
| `test_format_bytes` | 408 | default, Unavailable, new |
| `test_ble_error_variants` | 415 | default, Unavailable, new |
| `test_ble_daemon_fallback_logic` | 432 | default, Unavailable, new |
| `probe_and_log` | 8 | new |
| `fmt` | 79 | ManagerInitFailed, Other |
| `default` | 126 | new, Unavailable, ManagerInitFailed |
| `new` | 146 | new, get_adapter_info, Unavailable, Other, Available, ManagerInitFailed |
| `initialize` | 161 | new, get_adapter_info, Unavailable, Other, Available, ManagerInitFailed |
| `get_adapter_info` | 212 | status, Available, Other, is_available |
| `is_available` | 225 | status, Available, Other, is_available |
| `status` | 230 | status, Other, is_available |
| `scan_for_advertisements` | 236 | status, Other, is_available |
| `advertise_service` | 265 | new, status, Other, is_available, default |
| `shutdown` | 287 | new, from_timestamp, default |
| `default` | 293 | new, from_timestamp, default |
| `is_ble_available` | 300 | new, from_timestamp |
| `format_bytes` | 323 | new, from_timestamp, Other, from_utf8_lossy |
| `format_timestamp` | 336 | new, from_utf8_lossy, Other, from_timestamp, default |
| `try_enable_bluetooth` | 345 | new, from_utf8_lossy, Unavailable, Other, ManagerInitFailed, default |
| `test_ble_error_display` | 371 | new, Unavailable, Other, ManagerInitFailed, default |
| `test_ble_config_default` | 383 | new, Unavailable, ManagerInitFailed, default |
| `test_ble_status_initialization` | 390 | new, Unavailable, ManagerInitFailed, default |
| `test_ble_status_disabled` | 400 | new, Unavailable, default |
| `test_format_bytes` | 408 | new, Unavailable, default |
| `test_ble_error_variants` | 415 | new, Unavailable, default |
| `test_ble_daemon_fallback_logic` | 432 | new, Unavailable, default |

### Imports
- `use btleplug::api::Manager as _`
- `use std::process::Command`
- `use super::*`
---

## core/src/relay/bootstrap.rs (1 chunks, 470 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/transport/bootstrap.rs (1 chunks, 683 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/transport/circuit_breaker.rs (1 chunks, 519 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/circuit_breaker.rs: Defines 8 types: CircuitState, CircuitBreakerConfig, Default, CircuitBreakerEntry, CircuitBreakerEntry; 28 functions; 6 imports

### Structs/Classes
- CircuitBreakerConfig
- CircuitBreakerEntry
- CircuitBreakerManager
- CircuitBreakerStats
- CircuitState
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 42 | from_secs |
| `new` | 75 | default, new |
| `new` | 101 | default, transition_to_half_open, new |
| `with_defaults` | 109 | default, transition_to_half_open, new |
| `allow_request` | 117 | transition_to_half_open, now |
| `record_success` | 160 | now |
| `record_failure` | 202 | now |
| `get_state` | 242 |  |
| `get_failure_count` | 251 |  |
| `get_last_failure_reason` | 260 | default |
| `reset` | 268 | default |
| `reset_all` | 274 | default, now |
| `get_open_circuits` | 281 | default, now |
| `get_healthy_relays` | 291 | default, now |
| `get_stats` | 301 | default, from_secs, now |
| `transition_to_half_open` | 316 | default, from_secs, now, with_defaults |
| `test_circuit_breaker_default_config` | 345 | default, from_secs, with_defaults |
| `test_circuit_starts_closed` | 354 | with_defaults |
| `test_circuit_opens_after_failures` | 361 | with_defaults |
| `test_circuit_does_not_open_before_threshold` | 377 | with_defaults |
| `test_success_resets_closed_circuit` | 389 | with_defaults |
| `test_half_open_success_closes_circuit` | 402 | with_defaults |
| `test_half_open_failure_reopens_circuit` | 425 | with_defaults |
| `test_reset_specific_relay` | 444 | with_defaults |
| `test_reset_all` | 458 | with_defaults |
| `test_get_stats` | 474 | with_defaults |
| `test_get_open_circuits` | 495 | with_defaults |
| `test_last_failure_reason` | 509 | with_defaults |

### Imports
- `use parking_lot::RwLock`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use super::*`
- `use tracing::{debug, info, warn}`
- `use web_time::{Duration, SystemTime}`
---

## cli/src/cli.rs (2 chunks, 392 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/cli.rs: Defines 7 types: Cli, Commands, BlockAction, IdentityAction, ContactAction; 14 functions; 2 imports cli/src/cli.rs: Defines 7 types: Cli, Commands, BlockAction, IdentityAction, ContactAction; 14 functions; 2 imports

### Structs/Classes
- BlockAction
- Cli
- Commands
- ConfigAction
- ContactAction
- IdentityAction
- SwarmAction

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `test_cli_parse_init` | 8 | parse_from |
| `test_cli_parse_identity_show` | 14 | parse_from |
| `test_cli_parse_contact_add` | 20 | parse_from |
| `test_cli_parse_contact_list` | 36 | parse_from |
| `test_cli_parse_block_add` | 47 | parse_from |
| `test_cli_parse_relay` | 55 | parse_from |
| `test_cli_parse_send` | 63 | parse_from |
| `test_cli_parse_status` | 71 | parse_from |
| `test_cli_parse_identity_export` | 77 | parse_from |
| `test_cli_parse_identity_import` | 93 | parse_from |
| `test_cli_parse_contact_remove` | 109 | parse_from |
| `test_cli_parse_contact_search` | 117 | parse_from |
| `test_cli_parse_block_remove` | 125 | parse_from |
| `test_cli_parse_block_delete` | 133 | parse_from |
| `test_cli_parse_init` | 8 | parse_from |
| `test_cli_parse_identity_show` | 14 | parse_from |
| `test_cli_parse_contact_add` | 20 | parse_from |
| `test_cli_parse_contact_list` | 36 | parse_from |
| `test_cli_parse_block_add` | 47 | parse_from |
| `test_cli_parse_relay` | 55 | parse_from |
| `test_cli_parse_send` | 63 | parse_from |
| `test_cli_parse_status` | 71 | parse_from |
| `test_cli_parse_identity_export` | 77 | parse_from |
| `test_cli_parse_identity_import` | 93 | parse_from |
| `test_cli_parse_contact_remove` | 109 | parse_from |
| `test_cli_parse_contact_search` | 117 | parse_from |
| `test_cli_parse_block_remove` | 125 | parse_from |
| `test_cli_parse_block_delete` | 133 | parse_from |

### Imports
- `use clap::{Parser, Subcommand}`
- `use super::*`
---

## core/src/relay/client.rs (1 chunks, 1042 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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

## cli/src/config.rs (2 chunks, 332 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/config.rs: Defines 5 types: Config, NetworkConfig, Default, Default, Config; 15 functions; 4 imports cli/src/config.rs: Defines 5 types: Config, NetworkConfig, Default, Default, Config; 15 functions; 4 imports

### Structs/Classes
- Config
- Default
- NetworkConfig

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 61 | config_dir, new, data_local_dir, default, create_dir_all |
| `default` | 74 | config_dir, create_dir_all, data_local_dir, from_str, read_to_string, config_file |
| `config_dir` | 86 | config_dir, create_dir_all, data_local_dir, default, to_string_pretty, from_str, read_to_string, config_file |
| `data_dir` | 98 | config_dir, create_dir_all, write, data_local_dir, default, to_string_pretty, from_str, read_to_string, config_file |
| `config_file` | 110 | config_dir, write, default, to_string_pretty, from_str, read_to_string, config_file |
| `load` | 115 | write, default, to_string_pretty, from_str, read_to_string, config_file |
| `save` | 134 | to_string_pretty, write, config_file |
| `set` | 142 | save |
| `get` | 190 |  |
| `list` | 206 | strip_peer_id |
| `strip_peer_id` | 238 | strip_peer_id, default, save |
| `add_bootstrap_node` | 247 | to_string, save, strip_peer_id, default, from_str |
| `remove_bootstrap_node` | 263 | to_string, save, strip_peer_id, default, from_str |
| `test_default_config` | 285 | default, to_string, from_str |
| `test_config_serialization` | 293 | default, to_string, from_str |
| `default` | 61 | new, config_dir, default, data_local_dir, create_dir_all |
| `default` | 74 | read_to_string, config_dir, config_file, from_str, data_local_dir, create_dir_all |
| `config_dir` | 86 | read_to_string, config_dir, to_string_pretty, config_file, from_str, default, data_local_dir, create_dir_all |
| `data_dir` | 98 | read_to_string, config_dir, to_string_pretty, write, config_file, from_str, default, data_local_dir, create_dir_all |
| `config_file` | 110 | read_to_string, config_dir, to_string_pretty, write, config_file, from_str, default |
| `load` | 115 | read_to_string, to_string_pretty, write, config_file, from_str, default |
| `save` | 134 | write, config_file, to_string_pretty |
| `set` | 142 | save |
| `get` | 190 |  |
| `list` | 206 | strip_peer_id |
| `strip_peer_id` | 238 | strip_peer_id, save, default |
| `add_bootstrap_node` | 247 | from_str, to_string, default, strip_peer_id, save |
| `remove_bootstrap_node` | 263 | from_str, to_string, default, strip_peer_id, save |
| `test_default_config` | 285 | from_str, to_string, default |
| `test_config_serialization` | 293 | from_str, to_string, default |

### Imports
- `use anyhow::{Context, Result}`
- `use serde::{Deserialize, Serialize}`
- `use std::path::PathBuf`
- `use super::*`
---

## cli/src/contacts.rs (2 chunks, 254 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/transport/diagnostics.rs (1 chunks, 177 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/diagnostics.rs: Defines 3 types: NetworkDiagnosticsReport, PeerConnectionSummary, ExtendedNetworkDiagnostics; 7 functions; 4 imports

### Structs/Classes
- ExtendedNetworkDiagnostics
- NetworkDiagnosticsReport
- PeerConnectionSummary

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `get_network_diagnostics_report` | 44 |  |
| `get_extended_network_diagnostics` | 83 | now |
| `compute_avg_latency` | 113 | to_string, now |
| `now_ms` | 127 | to_string, new, now |
| `network_diagnostics_report_serializes` | 140 | to_string, new |
| `peer_connection_summary_formats_state` | 157 | new |
| `get_network_diagnostics_report_from_empty_monitor` | 170 | new |

### Imports
- `use crate::transport::health::{ConnectionState, ConnectionStats, TransportHealthMonitor}`
- `use libp2p::PeerId`
- `use serde::{Deserialize, Serialize}`
- `use super::*`
---

## core/src/transport/discovery.rs (1 chunks, 471 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/discovery.rs: Defines 9 types: BeaconError, DiscoveryMode, Default, DiscoveryMode, DiscoveryConfig; 21 functions; 3 imports

### Structs/Classes
- BeaconError
- BeaconPayload
- Default
- DiscoveryConfig
- DiscoveryMode

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 59 |  |
| `allows_mdns` | 66 |  |
| `allows_identify` | 71 |  |
| `advertises_identify` | 76 |  |
| `default` | 93 |  |
| `new` | 104 | from_le_bytes |
| `with_advertise_protocols` | 113 | from_le_bytes |
| `with_accept_unknown_peers` | 119 | from_le_bytes |
| `to_bytes` | 138 | from_le_bytes, new, now, EncryptionError |
| `from_bytes` | 147 | from_le_bytes, new, now, EncryptionError |
| `create_encrypted_beacon` | 174 | from_slice, new, now, EncryptionError |
| `decrypt_beacon` | 230 | from_slice, new, now, EncryptionError |
| `decrypt_beacon_with_period` | 238 | now, from_bytes, new, from_slice, EncryptionError |
| `test_beacon_encrypt_decrypt_roundtrip` | 298 | from_bytes |
| `test_beacon_decrypt_wrong_key_fails` | 313 | to_string, from_bytes, from_str |
| `test_beacon_payload_serialization` | 329 | to_string, from_bytes, from_str |
| `test_discovery_config_serialization` | 345 | to_string, from_str |
| `test_discovery_mode_serialization_all_variants` | 365 | to_string, from_str |
| `test_discovery_mode_properties` | 395 | new |
| `test_discovery_config_builder` | 420 | new |
| `test_epoch_rotation_changes_beacon` | 431 |  |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
---

## core/src/transport/escalation.rs (1 chunks, 667 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/escalation.rs: Defines 6 types: EscalationPolicy, Default, EscalationError, EscalationState, EscalationEngine; 38 functions; 9 imports

### Structs/Classes
- Default
- EscalationEngine
- EscalationError
- EscalationPolicy
- EscalationState

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 29 | new |
| `new` | 70 | select_best_transport, new, find_better_transport |
| `set_capabilities` | 79 | select_best_transport, find_better_transport |
| `init_peer` | 85 | select_best_transport, find_better_transport, now |
| `should_escalate` | 111 | find_worse_transport, find_better_transport, now |
| `escalate` | 122 | find_worse_transport, now |
| `deescalate` | 143 | find_worse_transport |
| `current_transport` | 166 | for_transport, escalation_score |
| `update_available_transports` | 172 | for_transport, escalation_score |
| `select_best_transport` | 187 | for_transport, escalation_score |
| `escalation_score` | 208 | for_transport, escalation_score |
| `find_better_transport` | 249 | escalation_score |
| `find_worse_transport` | 276 | escalation_score |
| `cleanup_peer` | 303 | new |
| `all_states` | 309 | new |
| `create_peer_id` | 321 | new |
| `test_escalation_engine_creation` | 329 | new |
| `test_init_peer_empty_transports` | 335 | new |
| `test_init_peer_success` | 344 | new |
| `test_select_best_transport_high_bandwidth` | 356 | new |
| `test_select_best_transport_low_latency` | 369 | new |
| `test_select_best_transport_low_power` | 382 | new |
| `test_select_best_transport_balanced` | 395 | new |
| `test_escalation_high_bandwidth_policy` | 409 | new |
| `test_escalation_low_latency_policy` | 426 | new |
| `test_escalation_low_power_policy` | 443 | new |
| `test_escalate_to_better_transport` | 464 | new |
| `test_deescalate_to_fallback` | 482 | new |
| `test_should_escalate_true` | 498 | new |
| `test_should_escalate_false` | 519 | new |
| `test_update_available_transports` | 532 | default, new |
| `test_cleanup_peer` | 548 | default, for_transport, new |
| `test_all_states` | 564 | default, for_transport, new |
| `test_escalation_policy_default` | 581 | default, for_transport, new |
| `test_set_capabilities` | 587 | default, for_transport, new |
| `test_escalation_order_high_bandwidth` | 601 | default, new |
| `test_escalation_order_low_latency` | 624 | default, new |
| `test_escalation_order_low_power` | 647 | default, new |

### Imports
- `use crate::transport::abstraction::TransportCapabilities`
- `use crate::transport::abstraction::TransportType`
- `use parking_lot::RwLock`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use super::*`
- `use thiserror::Error`
- `use tracing::debug`
---

## core/src/relay/findmy.rs (1 chunks, 463 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/transport/ble/gatt.rs (1 chunks, 563 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/ble/gatt.rs: Defines 15 types: GattCharacteristic, GattCharacteristic, GattError, GattFragmentHeader, GattFragmentHeader; 46 functions; 4 imports

### Structs/Classes
- GattCharacteristic
- GattClient
- GattError
- GattFragmentHeader
- GattFragmenter
- GattReassembler
- GattServer
- GattWriteQueue
- GattWriteRequest

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `uuid` | 32 | FragmentationError |
| `new` | 74 | FragmentationError, ReassemblyError, new, max_payload_per_write, from_le_bytes |
| `to_bytes` | 87 | FragmentationError, ReassemblyError, new, max_payload_per_write, from_le_bytes |
| `from_bytes` | 95 | FragmentationError, ReassemblyError, new, max_payload_per_write, from_le_bytes |
| `max_payload_per_write` | 112 | FragmentationError, ReassemblyError, new, from_bytes, max_payload_per_write |
| `fragment` | 117 | FragmentationError, ReassemblyError, new, from_bytes, max_payload_per_write |
| `reassemble` | 150 | ReassemblyError, from_bytes, new |
| `new` | 206 | is_full, FragmentationError, new |
| `new` | 227 | is_full, new |
| `new_default` | 235 | is_full, new |
| `is_full` | 240 | is_full |
| `is_empty` | 245 | is_full |
| `len` | 250 | is_full |
| `enqueue` | 255 | is_full |
| `dequeue` | 264 |  |
| `peek` | 269 |  |
| `clear` | 274 |  |
| `on_write` | 282 |  |
| `on_read` | 289 | from_bytes, new |
| `notify` | 292 | from_bytes, new |
| `is_enabled` | 296 | from_bytes, new |
| `write` | 302 | fragment, from_bytes, new |
| `read` | 309 | fragment, from_bytes, new |
| `subscribe` | 312 | fragment, from_bytes, new |
| `unsubscribe` | 315 | fragment, from_bytes, new |
| `is_connected` | 318 | fragment, from_bytes, new |
| `test_gatt_characteristic_uuids` | 326 | fragment, from_bytes, new |
| `test_gatt_fragment_header_roundtrip` | 333 | fragment, from_bytes, new |
| `test_gatt_fragment_header_invalid_index` | 343 | reassemble, fragment, new |
| `test_gatt_fragmenter_small_message` | 349 | reassemble, fragment, new |
| `test_gatt_fragmenter_large_message` | 358 | reassemble, fragment, new |
| `test_gatt_fragmenter_empty_message` | 371 | reassemble, fragment, new |
| `test_gatt_reassembler_single_fragment` | 380 | reassemble, fragment, new |
| `test_gatt_fragmenter_reassembler_roundtrip` | 392 | new, fragment, reassemble |
| `test_gatt_reassembler_wrong_fragment_count` | 401 | reassemble, new_default, new |
| `test_gatt_reassembler_out_of_order` | 412 | reassemble, new_default, new |
| `test_gatt_write_request_creation` | 429 | new_default, new |
| `test_gatt_write_request_max_size` | 439 | new_default, new |
| `test_gatt_write_queue_empty` | 447 | new_default, new |
| `test_gatt_write_queue_enqueue_dequeue` | 456 | new |
| `test_gatt_write_queue_backpressure` | 472 | new |
| `test_gatt_write_queue_peek` | 494 | new, max_payload_per_write |
| `test_gatt_write_queue_clear` | 510 | new, max_payload_per_write |
| `test_gatt_fragment_payload_size` | 526 | new, max_payload_per_write |
| `test_gatt_characteristic_all_variants` | 532 | new |
| `test_gatt_write_queue_fifo_order` | 545 | new |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use std::collections::VecDeque`
- `use super::*`
- `use thiserror::Error`
---

## core/src/transport/health.rs (1 chunks, 765 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/health.rs: Defines 10 types: ConnectionState, ConnectionStats, ConnectionStats, TransportHealthMonitor, std; 44 functions; 8 imports

### Structs/Classes
- ConnectionState
- ConnectionStats
- Default
- GlobalTransportMetrics
- TransportHealthMonitor
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 63 | new, now |
| `update_state` | 86 | now |
| `record_message_success` | 95 | now |
| `record_message_failure` | 113 | now |
| `record_bytes_received` | 122 | now |
| `record_connection_attempt` | 131 | now |
| `record_successful_connection` | 140 | now |
| `record_connection_failure` | 149 | now |
| `update_current_address` | 158 | now |
| `is_healthy` | 170 | now |
| `quality_score` | 190 | now |
| `fmt` | 236 | new |
| `default` | 252 | new |
| `new` | 259 | new |
| `update_connection_state` | 268 | new |
| `record_message_success` | 300 | new |
| `record_message_failure` | 311 | new |
| `record_bytes_received` | 322 | new |
| `record_connection_attempt` | 333 | new |
| `record_successful_connection` | 345 |  |
| `record_connection_failure` | 356 |  |
| `update_current_address` | 367 |  |
| `get_connection_stats` | 375 | new, now |
| `get_all_connection_stats` | 381 | new, now |
| `get_global_metrics` | 387 | new, now |
| `get_healthy_connections` | 393 | new, now |
| `get_unhealthy_connections` | 403 | new, now |
| `register_state_change_callback` | 413 | new, now |
| `cleanup_stale_connections` | 422 | now |
| `default` | 472 | new, now |
| `new` | 479 | new, now |
| `record_connection_attempt` | 501 |  |
| `record_successful_connection` | 506 |  |
| `record_connection_failure` | 515 | now |
| `record_message_success` | 520 | now |
| `record_message_failure` | 526 | now |
| `record_bytes_received` | 531 | now |
| `record_connection_state_change` | 536 | now |
| `health_score` | 560 | now |
| `uptime_seconds` | 601 | generate_ed25519, new, now |
| `test_connection_stats_quality_score` | 616 | generate_ed25519, new |
| `test_connection_stats_unhealthy` | 635 | generate_ed25519, new |
| `test_transport_health_monitor` | 651 | generate_ed25519, new |
| `test_global_metrics_health_score` | 677 | new |

### Imports
- `use libp2p::identity`
- `use libp2p::{Multiaddr, PeerId}`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use std::sync::{Arc, Mutex}`
- `use super::*`
- `use tracing::{debug, error, info, warn}`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## cli/src/history.rs (2 chunks, 322 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/store/history.rs (1 chunks, 516 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/transport/internet.rs (1 chunks, 903 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/internet.rs: Defines 10 types: InternetTransportError, NatStatus, RelayMode, InternetTransportConfig, Default; 40 functions; 9 imports

### Structs/Classes
- Default
- InternetRelay
- InternetTransportConfig
- InternetTransportError
- NatStatus
- PeerRelayInfo
- RelayMode
- RelayStats

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 87 |  |
| `new` | 144 | new, ConfigError |
| `get_nat_status` | 166 |  |
| `set_nat_status` | 171 |  |
| `connect_to_relay` | 177 |  |
| `connect_to_relay_via_swarm` | 254 | connect_to_relay, dial, ConnectionFailed |
| `relay_for_peer` | 296 | RelayPeerNotFound, BandwidthExceeded |
| `disconnect_relay` | 349 |  |
| `register_relay_peer` | 363 |  |
| `get_peer_relay_info` | 403 | get_active_relay_count |
| `get_relay_peers` | 408 | get_active_relay_count |
| `get_relay_stats` | 413 | get_active_relay_count |
| `get_all_relay_stats` | 418 | get_active_relay_count |
| `get_active_relay_count` | 423 | get_active_relay_count |
| `can_accept_relay` | 428 | get_active_relay_count |
| `cleanup_stale_relays` | 433 | RelayPeerNotFound |
| `shutdown` | 452 | RelayPeerNotFound, Other |
| `attempt_hole_punch` | 466 | RelayPeerNotFound, Other |
| `establish_relay_circuit` | 511 | RelayPeerNotFound, Other |
| `current_unix_timestamp` | 570 | default, new, now |
| `test_internet_relay_creation` | 586 | default, random, new |
| `test_invalid_listen_port` | 593 | default, random, new |
| `test_invalid_max_connections` | 602 | default, random, new |
| `test_nat_status_update` | 611 | default, random, new |
| `test_connect_to_relay` | 623 | default, random, new |
| `test_max_relay_connections` | 641 | default, random, new |
| `test_disconnect_relay` | 661 | default, random, new |
| `test_register_relay_peer` | 676 | default, random, new |
| `test_relay_for_peer` | 692 | default, random, new |
| `test_relay_peer_not_found` | 707 | default, random, new |
| `test_client_mode_relay_fails` | 717 | default, random, new |
| `test_relay_stats` | 731 | default, random, new |
| `test_get_relay_peers` | 751 | default, random, new |
| `test_can_accept_relay` | 771 | default, random, new |
| `test_cleanup_stale_relays` | 791 | default, random, new |
| `test_relay_mode_config` | 820 | default, random, new |
| `test_nat_traversal_hole_punch` | 835 | default, random, new |
| `test_nat_traversal_relay_circuit` | 856 | default, random, new |
| `test_invalid_relay_addresses` | 877 | default, random, new |
| `test_shutdown` | 888 | default, random, new |

### Imports
- `use crate::transport::SwarmHandle`
- `use libp2p::{Multiaddr, PeerId}`
- `use parking_lot::RwLock`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use std::time::{SystemTime, UNIX_EPOCH}`
- `use super::*`
- `use thiserror::Error`
- `use tracing::{debug, info, warn}`
---

## core/src/transport/ble/l2cap.rs (1 chunks, 568 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/ble/l2cap.rs: Defines 15 types: ProtocolServiceMultiplexer, ProtocolServiceMultiplexer, ChannelState, L2capConfig, Default; 41 functions; 3 imports

### Structs/Classes
- ChannelState
- Default
- FragmentHeader
- L2capChannel
- L2capConfig
- L2capError
- L2capFragmenter
- L2capReassembler
- ProtocolServiceMultiplexer

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `value` | 18 |  |
| `default` | 48 | InvalidMtu |
| `new` | 59 | InvalidMtu |
| `with_mtu` | 68 | InvalidMtu |
| `with_timeout` | 74 | InvalidMtu |
| `validate` | 80 | InvalidMtu, FragmentationError |
| `new` | 128 | from_le_bytes, FragmentationError, ReassemblyError, new |
| `to_bytes` | 141 | from_le_bytes, ReassemblyError, new |
| `from_bytes` | 149 | from_le_bytes, ReassemblyError, new |
| `new` | 169 | ConnectionFailed |
| `state` | 178 | ConnectionFailed |
| `config` | 183 | ConnectionFailed |
| `is_connected` | 188 | ConnectionFailed |
| `initiate_connection` | 193 | ConnectionFailed |
| `confirm_connection` | 209 | ConnectionFailed |
| `initiate_close` | 222 | max_payload_per_fragment, new |
| `confirm_close` | 234 | max_payload_per_fragment, new, FragmentationError |
| `new` | 252 | max_payload_per_fragment, new, FragmentationError |
| `max_payload_per_fragment` | 258 | max_payload_per_fragment, new, FragmentationError |
| `fragment` | 264 | max_payload_per_fragment, from_bytes, new, FragmentationError |
| `new` | 300 | ReassemblyError, from_bytes, new |
| `reassemble` | 306 | ReassemblyError, from_bytes, new |
| `test_psm_value` | 358 | default, from_bytes, new |
| `test_l2cap_config_default` | 364 | default, from_bytes, new |
| `test_l2cap_config_builder` | 372 | default, from_bytes, new |
| `test_l2cap_config_validation_valid` | 382 | default, from_bytes, new |
| `test_l2cap_config_validation_invalid_mtu` | 388 | default, from_bytes, new |
| `test_l2cap_config_validation_invalid_timeout` | 394 | default, from_bytes, new |
| `test_fragment_header_roundtrip` | 400 | default, from_bytes, new |
| `test_fragment_header_invalid_index` | 410 | default, new |
| `test_l2cap_channel_state_machine` | 416 | default, new |
| `test_l2cap_channel_invalid_double_connect` | 438 | default, new |
| `test_l2cap_fragmenter_small_message` | 448 | default, new |
| `test_l2cap_fragmenter_large_message` | 460 | default, new |
| `test_l2cap_fragmenter_empty_message` | 476 | default, new |
| `test_l2cap_reassembler_single_fragment` | 487 | default, new |
| `test_l2cap_fragmenter_reassembler_roundtrip` | 502 | default, new |
| `test_l2cap_reassembler_wrong_fragment_count` | 515 | default, new |
| `test_l2cap_reassembler_out_of_order` | 529 | default, new |
| `test_channel_state_transitions` | 549 | default, new |
| `test_l2cap_channel_new_validates_config` | 563 | default, new |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
---

## cli/src/ledger.rs (2 chunks, 550 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/ledger.rs: Defines 5 types: LedgerEntry, LedgerEntry, ConnectionLedger, Default, ConnectionLedger; 27 functions; 6 imports cli/src/ledger.rs: Defines 5 types: LedgerEntry, LedgerEntry, ConnectionLedger, Default, ConnectionLedger; 27 functions; 6 imports

### Structs/Classes
- ConnectionLedger
- Default
- LedgerEntry

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 59 | new, now |
| `record_success` | 85 | min, now |
| `record_failure` | 117 | min, now |
| `should_attempt` | 135 | from_str, new, now, read_to_string |
| `add_topic` | 145 | default, new, from_str, read_to_string |
| `default` | 166 | now, write, new, default, to_string_pretty, from_str, read_to_string |
| `load` | 177 | now, write, default, to_string_pretty, from_str, read_to_string |
| `save` | 197 | to_string_pretty, write, new, now |
| `add_bootstrap` | 213 | new |
| `record_connection` | 235 | new |
| `record_topic` | 251 |  |
| `record_failure` | 259 |  |
| `dialable_addresses` | 273 | now |
| `all_known_topics` | 289 | now |
| `find_by_peer_id` | 301 | now |
| `to_shared_entries` | 312 | new, now |
| `merge_shared_entries` | 336 | new |
| `summary` | 390 |  |
| `strip_peer_id` | 413 |  |
| `extract_ip_port` | 422 |  |
| `test_strip_peer_id` | 448 | new |
| `test_extract_ip_port` | 460 | new |
| `test_ledger_entry_backoff` | 472 | new |
| `test_ledger_entry_backoff_overflow_safety` | 496 | default, new |
| `test_ledger_entry_peer_id_tracking` | 508 | default, new |
| `test_ledger_crud` | 525 | default |
| `test_ledger_topic_tracking` | 540 | default |
| `new` | 59 | new, now |
| `record_success` | 85 | min, now |
| `record_failure` | 117 | min, now |
| `should_attempt` | 135 | new, read_to_string, from_str, now |
| `add_topic` | 145 | new, read_to_string, from_str, default |
| `default` | 166 | new, read_to_string, now, to_string_pretty, write, from_str, default |
| `load` | 177 | read_to_string, now, to_string_pretty, write, from_str, default |
| `save` | 197 | new, write, now, to_string_pretty |
| `add_bootstrap` | 213 | new |
| `record_connection` | 235 | new |
| `record_topic` | 251 |  |
| `record_failure` | 259 |  |
| `dialable_addresses` | 273 | now |
| `all_known_topics` | 289 | now |
| `find_by_peer_id` | 301 | now |
| `to_shared_entries` | 312 | new, now |
| `merge_shared_entries` | 336 | new |
| `summary` | 390 |  |
| `strip_peer_id` | 413 |  |
| `extract_ip_port` | 422 |  |
| `test_strip_peer_id` | 448 | new |
| `test_extract_ip_port` | 460 | new |
| `test_ledger_entry_backoff` | 472 | new |
| `test_ledger_entry_backoff_overflow_safety` | 496 | new, default |
| `test_ledger_entry_peer_id_tracking` | 508 | new, default |
| `test_ledger_crud` | 525 | default |
| `test_ledger_topic_tracking` | 540 | default |

### Imports
- `use anyhow::{Context, Result}`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use std::path::Path`
- `use std::time::{SystemTime, UNIX_EPOCH}`
- `use super::*`
---

## cli/src/lib.rs (2 chunks, 18 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/lib.rs: structural extraction cli/src/lib.rs: structural extraction

---

## core/src/transport/manager.rs (1 chunks, 1415 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/manager.rs: Defines 12 types: TransportState, TransportState, PendingSend, OutgoingQueue, OutgoingQueue; 68 functions; 8 imports

### Structs/Classes
- Default
- OutgoingQueue
- PendingSend
- ReconnectionState
- SendResult
- TransportManager
- TransportState

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 30 | Reverse, new |
| `new` | 62 | from_secs, Reverse, new |
| `enqueue` | 67 | new, from_secs, Reverse |
| `dequeue` | 75 | from_secs, new |
| `len` | 84 | from_millis, from_secs, new |
| `is_empty` | 89 | from_millis, from_secs, new |
| `clear` | 94 | from_millis, from_secs, new |
| `default` | 100 | from_millis, from_secs, new |
| `new` | 146 | from_millis, backoff_interval, now |
| `backoff_interval` | 158 | from_millis, backoff_interval, now |
| `record_failure` | 168 | backoff_interval, now |
| `is_exhausted` | 174 | new, now |
| `is_ready` | 179 | new, now |
| `new` | 221 | new, now |
| `set_health_monitor` | 235 | Vacant, new, now |
| `register_transport` | 240 | Vacant, new, now |
| `handle_event` | 251 | new, Vacant, now |
| `send_to_peer` | 325 | encode, now, PeerNotFound, Queued, best_transport_for_peer |
| `best_transport_for_peer` | 355 | min, PeerNotFound, max |
| `connected_peers` | 415 | new |
| `peers_on_transport` | 421 | new |
| `is_peer_connected` | 431 |  |
| `transports_for_peer` | 440 |  |
| `pending_sends` | 449 |  |
| `add_target_peer` | 461 | now |
| `remove_target_peer` | 467 | now |
| `peers_needing_reconnect` | 482 | now |
| `record_reconnect_success` | 524 | now |
| `record_reconnect_failure` | 530 | now |
| `reconnection_queue_len` | 552 | now |
| `tick` | 557 | now |
| `expire_address_observations` | 594 | all_connections |
| `get_healthy_connections` | 602 | all_connections, empty |
| `get_unhealthy_connections` | 611 | all_connections, empty |
| `get_all_connection_stats` | 620 | all_connections, empty |
| `get_all_observed_connections` | 632 | empty |
| `cleanup_health_stale_connections` | 658 | new |
| `disable_transport` | 668 | for_transport, new |
| `default` | 693 | for_transport, new, now |
| `create_peer_id` | 701 | for_transport, new, now |
| `test_transport_state_creation` | 709 | for_transport, new, now |
| `test_outgoing_queue_fifo_with_priority` | 717 | new, now |
| `test_outgoing_queue_len` | 750 | for_transport, new, now |
| `test_outgoing_queue_clear` | 768 | for_transport, new, now |
| `test_transport_manager_creation` | 784 | for_transport, new |
| `test_register_transport` | 790 | for_transport, new |
| `test_peer_discovered_event` | 801 | new |
| `test_peer_disconnected_event` | 819 | for_transport, new |
| `test_multiple_transports_per_peer` | 842 | for_transport, new |
| `test_best_transport_prefers_connected` | 865 | for_transport, new |
| `test_best_transport_prefers_streaming` | 902 | for_transport, new |
| `test_best_transport_prefers_low_latency` | 933 | for_transport, new |
| `test_best_transport_fails_for_unknown_peer` | 964 | for_transport, new |
| `test_is_peer_connected` | 973 | for_transport, new |
| `test_peers_on_transport` | 990 | for_transport, new |
| `test_send_to_peer_queues_data` | 1029 | Queued, for_transport, new |
| `test_pending_sends_priority_ordering` | 1053 | from_secs, for_transport, new, now |
| `test_tick_cleanup` | 1079 | from_secs, new, now |
| `test_transports_for_peer` | 1107 | new |
| `test_connected_peers_deduplication` | 1132 | new |
| `test_target_peer_queued_for_reconnect_on_disconnect` | 1152 | new |
| `test_non_target_peer_not_queued_for_reconnect` | 1179 | new |
| `test_reconnection_backoff_increases` | 1202 | new |
| `test_reconnection_backoff_capped_at_max` | 1217 | new |
| `test_reconnection_exhaustion` | 1229 | new |
| `test_reconnect_success_removes_from_queue` | 1242 | new |
| `test_remove_target_peer_stops_reconnection` | 1268 | Queued, for_transport, new |
| `test_send_result_is_queued_not_delivered` | 1294 | Queued, for_transport, new |

### Imports
- `use crate::transport::health::TransportHealthMonitor`
- `use crate::transport::observation::AddressObserver`
- `use parking_lot::RwLock`
- `use std::collections::{HashMap, HashSet}`
- `use std::sync::Arc`
- `use super::*`
- `use tracing::{debug, info, warn}`
- `use web_time::{Duration, SystemTime}`
---

## core/src/transport/mesh_routing.rs (1 chunks, 699 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/mesh_routing.rs: Defines 20 types: RankedRoute, RouteCursorAdvance, RelayStats, RelayReputation, RelayReputation; 41 functions; 5 imports

### Structs/Classes
- BootstrapCapability
- Default
- DeliveryAttempt
- MultiPathDelivery
- RankedRoute
- RelayCandidate
- RelayReputation
- RelayStats
- ReputationTracker
- RetryStrategy
- RouteCursorAdvance

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `advance_route_cursor` | 43 | now |
| `unix_now_secs` | 64 | now |
| `calculate_score` | 112 | now |
| `default` | 166 | default, new, now |
| `new` | 172 | default, new, now |
| `record_relay_attempt` | 179 | default, now |
| `best_relays` | 214 | default |
| `add_relay` | 231 | default, from_millis, from_secs |
| `is_empty` | 243 | from_millis, from_secs |
| `all_reputations` | 248 | from_millis, from_secs |
| `default` | 274 | from_millis, from_secs |
| `calculate_delay` | 287 | default, from_millis, new, now |
| `should_retry` | 301 | default, new, now |
| `new` | 324 | default, new, now |
| `next_retry_delay` | 339 | new, now |
| `should_retry` | 344 | new, now |
| `record_failure` | 349 | new, now |
| `default` | 375 | record_recipient_seen_via_relay, new |
| `new` | 381 | with_capacity, record_recipient_seen_via_relay, new |
| `start_delivery` | 392 | with_capacity, record_recipient_seen_via_relay, new |
| `add_relay` | 398 | with_capacity, record_recipient_seen_via_relay, new |
| `record_recipient_seen_via_relay` | 405 | with_capacity, record_recipient_seen_via_relay, new |
| `record_recipient_seen_now` | 417 | with_capacity, record_recipient_seen_via_relay, new |
| `ranked_routes` | 422 | with_capacity, new |
| `get_best_paths` | 502 | ranked_routes |
| `record_success` | 510 |  |
| `record_failure` | 529 |  |
| `converge_delivery` | 546 | new |
| `delivery_attempt` | 551 | new |
| `pending_attempts` | 556 | new, now |
| `reputation` | 561 | new, now |
| `best_relays` | 566 | new, now |
| `default` | 585 | default, random, new, now |
| `new` | 591 | default, random, new, now |
| `add_peer` | 599 | default, random, now |
| `get_bootstrap_candidates` | 610 | default, from_millis, random |
| `can_bootstrap_others` | 615 | default, from_millis, random |
| `test_reputation_calculation` | 625 | default, from_millis, random, new |
| `test_retry_strategy` | 649 | default, from_millis, random, new |
| `test_multi_path_delivery` | 668 | random, new |
| `test_converge_delivery_clears_pending_retry_attempt` | 686 | random, new |

### Imports
- `use libp2p::PeerId`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use super::*`
- `use web_time::{Duration, SystemTime, UNIX_EPOCH}`
---

## core/src/transport/multiport.rs (1 chunks, 345 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/multiport.rs: Defines 6 types: MultiPortConfig, Default, BindResult, BindAnalysis, BindAnalysis; 11 functions; 2 imports

### Structs/Classes
- BindAnalysis
- BindResult
- ConnectivityStatus
- Default
- MultiPortConfig

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 34 | new |
| `generate_listen_addresses` | 57 | new |
| `requires_elevated_privileges` | 97 | new |
| `analyze_bind_results` | 109 | new |
| `report` | 175 | new |
| `test_generate_listen_addresses_default` | 244 | default |
| `test_generate_listen_addresses_ipv4_only` | 256 |  |
| `test_generate_listen_addresses_custom_ports` | 276 |  |
| `test_requires_elevated_privileges` | 292 |  |
| `test_analyze_bind_results` | 303 |  |
| `test_bind_analysis_report` | 332 |  |

### Imports
- `use libp2p::Multiaddr`
- `use super::*`
---

## core/src/transport/nat.rs (1 chunks, 854 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/nat.rs: Defines 11 types: NatTraversalError, NatType, PeerAddressDiscovery, PeerAddressDiscovery, HolePunchAttempt; 37 functions; 10 imports

### Structs/Classes
- Default
- HolePunchAttempt
- HolePunchStatus
- NatConfig
- NatTraversal
- NatTraversalError
- NatType
- PeerAddressDiscovery
- RelayCircuit

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `with_peers` | 87 | ProbesFailed, new |
| `detect_nat_type` | 96 | ProbesFailed, new |
| `get_external_address` | 177 | StunError |
| `default` | 309 | new, with_peers, InvalidConfig |
| `new` | 338 | new, with_peers, InvalidConfig |
| `probe_nat` | 355 | with_peers, HolePunchFailed |
| `get_nat_type` | 378 | HolePunchFailed |
| `get_external_address` | 383 | HolePunchFailed |
| `start_hole_punch` | 388 | send_hole_punch_probes, HolePunchFailed |
| `send_hole_punch_probes` | 440 |  |
| `get_hole_punch_status` | 495 | RelayCircuitFailed |
| `establish_relay_circuit` | 508 | RelayCircuitFailed |
| `close_relay_circuit` | 544 |  |
| `get_active_circuits` | 560 |  |
| `get_relay_circuit` | 570 | now |
| `cleanup_old_attempts` | 581 | with_peers, now |
| `shutdown` | 599 | with_peers, now |
| `current_unix_timestamp` | 612 | default, new, with_peers, now |
| `test_peer_discovery_creation` | 628 | default, new, with_peers |
| `test_peer_discovery_no_peers` | 640 | default, new |
| `test_detect_nat_type_with_peers` | 647 | default, new |
| `test_get_external_address_from_peer` | 654 | default, new |
| `test_nat_traversal_creation` | 660 | default, random, new |
| `test_nat_traversal_invalid_config` | 667 | default, random, new |
| `test_probe_nat` | 677 | default, random, new |
| `test_hole_punch_start` | 684 | default, random, new |
| `test_hole_punch_disabled` | 691 | default, random, new |
| `test_get_hole_punch_status` | 698 | default, random, new |
| `test_establish_relay_circuit` | 704 | default, random, new |
| `test_relay_fallback_disabled` | 722 | default, random, new |
| `test_close_relay_circuit` | 741 | default, random, new |
| `test_get_relay_circuit` | 764 | default, random, new |
| `test_cleanup_old_attempts` | 785 | default, random, new |
| `test_shutdown` | 814 | default, random, new |
| `test_nat_type_equality` | 834 | default |
| `test_hole_punch_status_values` | 840 | default |
| `test_nat_config_defaults` | 847 | default |

### Imports
- `use libp2p::PeerId`
- `use parking_lot::RwLock`
- `use std::collections::HashMap`
- `use std::net::SocketAddr`
- `use std::sync::Arc`
- `use super::*`
- `use super::swarm::SwarmHandle`
- `use thiserror::Error`
- `use tracing::{debug, info}`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/transport/observation.rs (1 chunks, 277 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/observation.rs: Defines 8 types: AddressObservation, AddressObserver, Default, AddressObserver, ConnectionEndpoint; 18 functions; 6 imports

### Structs/Classes
- AddressObservation
- AddressObserver
- ConnectionEndpoint
- ConnectionTracker
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 34 | new, now, recalculate_consensus |
| `new` | 41 | new, now, recalculate_consensus |
| `record_observation` | 49 | now, recalculate_consensus |
| `external_addresses` | 81 | Reverse, new, now, recalculate_consensus |
| `primary_external_address` | 86 | Reverse, new, now, recalculate_consensus |
| `all_observations` | 91 | Reverse, new, now, recalculate_consensus |
| `expire_old_observations` | 96 | Reverse, new, now, recalculate_consensus |
| `recalculate_consensus` | 109 | Reverse, new |
| `default` | 149 | new, now |
| `new` | 156 | new, now |
| `add_connection` | 163 | Ip6, now, Tcp, V4, Ip4, V6 |
| `remove_connection` | 188 | Ip6, Tcp, random, new, V4, Ip4, V6, Udp |
| `get_connection` | 193 | Ip6, Tcp, random, new, V4, Ip4, V6, Udp |
| `all_connections` | 198 | Ip6, Tcp, random, new, V4, Ip4, V6, Udp |
| `extract_socket_addr` | 203 | Ip6, Tcp, random, new, V4, Ip4, V6, Udp |
| `test_address_observer_consensus` | 231 | random, new, extract_socket_addr |
| `test_address_confirmation_count` | 256 | random, new, extract_socket_addr |
| `test_extract_socket_addr` | 272 | extract_socket_addr |

### Imports
- `use libp2p::multiaddr::Protocol`
- `use libp2p::{Multiaddr, PeerId}`
- `use std::collections::HashMap`
- `use std::net::SocketAddr`
- `use super::*`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/transport/peer_broadcast.rs (1 chunks, 160 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/peer_broadcast.rs: Defines 4 types: PeerBroadcaster, PeerInfo, Default, PeerBroadcaster; 10 functions; 6 imports

### Structs/Classes
- Default
- PeerBroadcaster
- PeerInfo

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 33 | full_relay, new, now |
| `new` | 40 | full_relay, new, now |
| `peer_connected` | 47 | full_relay, now |
| `peer_disconnected` | 64 | now |
| `create_peer_joined_message` | 70 | now |
| `create_peer_left_message` | 90 | new, now |
| `create_peer_list_response` | 98 | random, new, now |
| `get_peers_except` | 120 | random, new |
| `peer_count` | 129 | random, new |
| `test_peer_broadcaster` | 139 | random, new |

### Imports
- `use crate::relay::protocol::{RelayCapability, RelayMessage, RelayPeerInfoMessage}`
- `use crate::store::blocked::BlockedIdentity as RelayCapability`
- `use libp2p::PeerId`
- `use std::collections::HashMap`
- `use super::*`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/relay/peer_exchange.rs (1 chunks, 489 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/relay/protocol.rs (1 chunks, 379 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/transport/reflection.rs (1 chunks, 360 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/reflection.rs: Defines 8 types: AddressReflectionRequest, AddressReflectionRequest, Default, AddressReflectionResponse, AddressReflectionResponse; 31 functions; 8 imports

### Structs/Classes
- AddressReflectionRequest
- AddressReflectionResponse
- AddressReflectionService
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 34 | new |
| `with_id` | 46 | new |
| `default` | 55 | new |
| `new` | 75 | new |
| `parse_address` | 84 | new |
| `new` | 111 | new |
| `handle_request` | 129 | new |
| `requests_served` | 151 | serialize, new |
| `enable` | 156 | deserialize, serialize, new |
| `disable` | 162 | deserialize, serialize, new |
| `is_enabled` | 168 | deserialize, serialize, new |
| `reset_stats` | 173 | deserialize, serialize, new |
| `default` | 179 | deserialize, serialize, new |
| `encode_request` | 197 | new, with_id, serialize, deserialize |
| `decode_request` | 202 | new, with_id, serialize, deserialize |
| `encode_response` | 207 | new, with_id, serialize, deserialize |
| `decode_response` | 213 | new, with_id, deserialize |
| `test_request_creation` | 227 | with_id, new |
| `test_request_with_id` | 234 | with_id, new |
| `test_response_creation` | 242 | with_id, new |
| `test_response_parse_address` | 253 | with_id, new |
| `test_service_creation` | 263 | with_id, new |
| `test_service_handle_request` | 270 | with_id, new |
| `test_service_multiple_requests` | 283 | with_id, encode_request, new, decode_request |
| `test_service_enable_disable` | 296 | with_id, encode_request, new, decode_response, encode_response, decode_request |
| `test_service_reset_stats` | 308 | with_id, encode_request, new, decode_response, default, encode_response, decode_request |
| `test_codec_request_roundtrip` | 321 | with_id, encode_request, new, decode_response, default, encode_response, decode_request |
| `test_codec_response_roundtrip` | 331 | decode_response, new, default, encode_response, decode_request |
| `test_codec_invalid_data` | 342 | default, decode_response, decode_request |
| `test_request_default` | 349 | default |
| `test_service_default` | 355 | default |

### Imports
- `use anyhow::Result`
- `use rand::RngCore`
- `use serde::{Deserialize, Serialize}`
- `use std::net::SocketAddr`
- `use std::sync::Arc`
- `use std::sync::atomic::{AtomicU64, Ordering}`
- `use super::*`
---

## core/src/transport/relay_health.rs (1 chunks, 395 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/relay_health.rs: Defines 6 types: RelayMetrics, RelayMetrics, RelayDiscovery, RelayDiscovery, RelayFallback; 23 functions; 6 imports

### Structs/Classes
- RelayDiscovery
- RelayFallback
- RelayMetrics

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `priority_score` | 47 | new, now |
| `is_healthy` | 62 | invalidate_cache, from_secs, new, now |
| `new` | 93 | now, invalidate_cache, new, rebuild_cache, from_secs |
| `update_relay_metrics` | 104 | invalidate_cache, rebuild_cache, now |
| `record_success` | 110 | invalidate_cache, rebuild_cache, now |
| `record_failure` | 126 | invalidate_cache, rebuild_cache |
| `get_priority_relays` | 139 | rebuild_cache, now |
| `get_fallback_relays` | 153 | now |
| `add_fallback_relay` | 158 | now |
| `relay_count` | 166 | invalidate_cache, now |
| `healthy_relay_count` | 171 | invalidate_cache, now |
| `get_all_metrics` | 180 | invalidate_cache, now |
| `cleanup_stale_relays` | 185 | invalidate_cache, now |
| `invalidate_cache` | 216 | healthy_relay_count, new, now |
| `rebuild_cache` | 221 | healthy_relay_count, new, now |
| `new` | 254 | generate_ed25519, from_secs, new |
| `should_retry` | 263 | generate_ed25519, from_secs, now |
| `record_attempt` | 271 | generate_ed25519, from_secs, now |
| `get_backoff_delay` | 277 | generate_ed25519, from_secs, new, now |
| `reset_attempts` | 284 | generate_ed25519, new, now |
| `test_relay_metrics_priority_score` | 295 | generate_ed25519, new, now |
| `test_relay_discovery_priority_ordering` | 324 | generate_ed25519, new, now |
| `test_relay_fallback_backoff` | 373 | new |

### Imports
- `use libp2p::identity`
- `use libp2p::{Multiaddr, PeerId}`
- `use std::collections::{HashMap, VecDeque}`
- `use super::*`
- `use tracing::{debug, info}`
- `use web_time::{Duration, SystemTime, UNIX_EPOCH}`
---

## core/src/transport/reputation.rs (1 chunks, 619 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/transport/ble/scanner.rs (1 chunks, 628 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/ble/scanner.rs: Defines 13 types: BleScanConfig, Default, BleScanConfig, ScannerState, BatteryState; 54 functions; 4 imports

### Structs/Classes
- BatteryState
- BleScanConfig
- BleScanner
- Default
- DutyCycleManager
- ScanResult
- ScannerError
- ScannerState

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 23 | InvalidScanConfig |
| `new` | 34 | InvalidScanConfig |
| `with_duty_cycle` | 45 | InvalidScanConfig |
| `validate` | 54 | InvalidScanConfig |
| `from_percentage` | 95 |  |
| `new` | 112 | get_recommended_duty_cycle |
| `set_battery_state` | 120 | get_recommended_duty_cycle |
| `set_wifi_available` | 125 | get_recommended_duty_cycle |
| `get_recommended_duty_cycle` | 130 | get_recommended_duty_cycle |
| `get_mode_description` | 156 | get_recommended_duty_cycle, now |
| `new` | 199 | now, SystemTimeError |
| `estimate_distance` | 219 | new, now, SystemTimeError |
| `new` | 244 | update_timestamp, new, now, SystemTimeError |
| `state` | 260 | update_timestamp |
| `config` | 265 | update_timestamp |
| `is_active` | 270 | update_timestamp |
| `start_scanning` | 275 | update_timestamp |
| `pause_scanning` | 292 | update_timestamp, get_duty_cycle |
| `stop_scanning` | 304 | update_timestamp, get_duty_cycle, now |
| `set_battery_state` | 316 | get_duty_cycle, now |
| `set_wifi_available` | 321 | get_duty_cycle, now |
| `get_duty_cycle` | 326 | default, get_duty_cycle, now |
| `get_mode` | 331 | default, get_duty_cycle, now |
| `calculate_scan_duration_ms` | 336 | default, get_duty_cycle, now |
| `calculate_pause_duration_ms` | 342 | default, get_duty_cycle, now |
| `update_timestamp` | 349 | default, now |
| `time_since_state_change_ms` | 356 | default, now |
| `test_ble_scan_config_default` | 371 | default, from_percentage |
| `test_ble_scan_config_validation_valid` | 379 | default, from_percentage |
| `test_ble_scan_config_validation_window_exceeds_interval` | 385 | default, from_percentage, new |
| `test_ble_scan_config_validation_zero_interval` | 395 | default, from_percentage, new |
| `test_ble_scan_config_with_duty_cycle` | 405 | default, from_percentage, new |
| `test_ble_scan_config_invalid_duty_cycle` | 413 | default, from_percentage, new |
| `test_battery_state_from_percentage` | 419 | from_percentage, new |
| `test_duty_cycle_manager_aggressive_when_charging` | 429 | new |
| `test_duty_cycle_manager_aggressive_with_charging_and_wifi` | 436 | new |
| `test_duty_cycle_manager_standard_good_battery` | 443 | new |
| `test_duty_cycle_manager_reduced_low_battery` | 450 | default, new |
| `test_duty_cycle_manager_minimal_critical_battery` | 457 | default, new |
| `test_duty_cycle_manager_set_battery_state` | 464 | default, new |
| `test_scan_result_creation` | 473 | default, new |
| `test_scan_result_distance_estimate` | 487 | default, new |
| `test_ble_scanner_creation` | 496 | default, new |
| `test_ble_scanner_start_stop` | 505 | default, new |
| `test_ble_scanner_pause_resume` | 519 | default, new |
| `test_ble_scanner_double_start_error` | 532 | default, new |
| `test_ble_scanner_set_battery_state` | 542 | default, new |
| `test_ble_scanner_set_wifi_available` | 553 | default, new |
| `test_ble_scanner_calculate_scan_duration` | 565 | default, new |
| `test_ble_scanner_calculate_pause_duration` | 575 | default, new |
| `test_ble_scanner_mode_description` | 585 | default, new |
| `test_ble_scanner_time_since_state_change` | 593 | default, new |
| `test_ble_scanner_invalid_config` | 602 | new |
| `test_all_battery_states` | 614 | new |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## cli/src/server.rs (2 chunks, 1577 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/server.rs: Defines 8 types: UiEvent, UiOutbound, UiCommand, WebContext, Clone; 5 functions; 9 imports cli/src/server.rs: Defines 8 types: UiEvent, UiOutbound, UiCommand, WebContext, Clone; 5 functions; 9 imports

### Structs/Classes
- Clone
- UiCommand
- UiEvent
- UiOutbound
- WebContext
- WsSender
- WsSenderList
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `clone` | 131 | new, clone |
| `fmt` | 147 | html, new, end |
| `start` | 168 | ws, html, new, path, end |
| `handle_ws_connection` | 231 | to_string, Lagged, spawn, text, Legacy, JsonRpc |
| `handle_jsonrpc_request` | 320 | from_str |
| `clone` | 131 | new, clone |
| `fmt` | 147 | new, end, html |
| `start` | 168 | new, end, path, html, ws |
| `handle_ws_connection` | 231 | Lagged, Legacy, JsonRpc, to_string, text, spawn |
| `handle_jsonrpc_request` | 320 | from_str |

### Imports
- `use futures::StreamExt`
- `use futures_util::SinkExt`
- `use libp2p::PeerId`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use std::time::Instant`
- `use tokio::sync::{broadcast, mpsc, Mutex}`
- `use warp::Filter`
---

## core/src/abuse/spam_detection.rs (1 chunks, 498 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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

## cli/src/transport_api.rs (2 chunks, 33 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/transport_api.rs: Defines 4 types: RegisterPeerRequest, TransportError, std, warp; 1 functions cli/src/transport_api.rs: Defines 4 types: RegisterPeerRequest, TransportError, std, warp; 1 functions

### Structs/Classes
- RegisterPeerRequest
- TransportError
- std
- warp

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `fmt` | 23 |  |
| `fmt` | 23 |  |

---

## core/src/transport/websocket.rs (1 chunks, 306 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/websocket.rs: Defines 3 types: WebSocketTransportError, WebSocketTransport, WebSocketTransport; 15 functions; 7 imports

### Structs/Classes
- WebSocketTransport
- WebSocketTransportError

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 47 | InvalidUrl, new, from_secs, Binary, timeout, Timeout, ConnectionFailed |
| `from_multiaddr` | 56 | InvalidUrl, new, from_secs, Binary, SendFailed, timeout, Timeout, ConnectionFailed |
| `connect` | 63 | InvalidUrl, Binary, from_secs, SendFailed, timeout, Timeout, ConnectionFailed |
| `send` | 91 | Text, ReceiveFailed, from_secs, Binary, SendFailed, timeout, Timeout, ConnectionFailed |
| `recv` | 109 | Text, ReceiveFailed, Binary, from_secs, timeout, Timeout, ConnectionFailed |
| `close` | 135 | Ip6, Dns4, Wss, Ws, Dns6, Dns, Ip4, Tcp, ConnectionFailed |
| `is_connected` | 148 | Ip6, InvalidUrl, Dns4, Wss, Ws, Dns6, Dns, Ip4, Tcp |
| `multiaddr_to_websocket_url` | 154 | Ip6, InvalidUrl, Dns4, Wss, Ws, Dns6, Dns, Ip4, Tcp |
| `diagnose_websocket_error` | 212 | InvalidUrl, ReceiveFailed, SendFailed, ConfigError, Timeout, HandshakeFailed, ConnectionFailed |
| `test_multiaddr_to_websocket_url_ws` | 258 | from_secs, new, ConnectionFailed |
| `test_multiaddr_to_websocket_url_wss` | 265 | from_secs, new, ConnectionFailed |
| `test_multiaddr_to_websocket_url_wss_protocol` | 272 | from_secs, new, ConnectionFailed |
| `test_multiaddr_to_websocket_url_invalid` | 279 | from_secs, new, ConnectionFailed |
| `test_websocket_transport_creation` | 286 | from_secs, new, ConnectionFailed |
| `test_websocket_transport_diagnostics` | 294 | ConnectionFailed |

### Imports
- `use crate::transport::internet::InternetTransportError`
- `use futures::{SinkExt, StreamExt}`
- `use libp2p::Multiaddr`
- `use std::time::Duration`
- `use super::*`
- `use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest}`
- `use tracing::{debug, info, warn}`
---

## core/src/transport/wifi_aware.rs (1 chunks, 796 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/wifi_aware.rs: Defines 12 types: WifiAwareError, WifiAwareConfig, Default, WifiAwareState, DataPathInfo; 53 functions; 9 imports

### Structs/Classes
- DataPathInfo
- Default
- DiscoveredPeer
- MockWifiAwareBridge
- WifiAwareConfig
- WifiAwareError
- WifiAwarePlatformBridge
- WifiAwareState
- WifiAwareTransport

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 56 | new |
| `is_available` | 128 |  |
| `publish_service` | 131 |  |
| `subscribe_to_services` | 138 | new |
| `unpublish_service` | 145 | new |
| `unsubscribe_from_services` | 148 | new |
| `create_data_path` | 151 | new |
| `close_data_path` | 158 | new |
| `set_on_service_discovered` | 161 | new |
| `set_on_message_received` | 164 | new |
| `set_on_data_path_confirmed` | 167 | new |
| `new` | 183 | new |
| `add_discovered_peer` | 189 |  |
| `is_available` | 198 |  |
| `publish_service` | 201 |  |
| `subscribe_to_services` | 213 |  |
| `unpublish_service` | 225 |  |
| `unsubscribe_from_services` | 229 |  |
| `create_data_path` | 233 | InvalidConfig |
| `close_data_path` | 242 | new, InvalidConfig |
| `set_on_service_discovered` | 247 | new, InvalidConfig |
| `set_on_message_received` | 253 | new, InvalidConfig |
| `set_on_data_path_confirmed` | 255 | new, InvalidConfig |
| `new` | 275 | new, get_state, InvalidConfig |
| `get_state` | 295 | get_state, InvalidConfig |
| `initialize` | 300 | get_state, InvalidConfig |
| `publish_service` | 316 | get_state, InvalidConfig |
| `subscribe` | 338 | get_state, InvalidConfig, PeerNotFound |
| `create_data_path` | 366 | get_state, DataPathFailed, PeerNotFound |
| `close_data_path` | 417 | register_peer |
| `get_data_path` | 429 | new, register_peer |
| `get_active_data_paths` | 434 | new, register_peer |
| `get_discovered_peers` | 439 | new, register_peer |
| `register_peer` | 444 | new, register_peer |
| `add_discovered_peer` | 454 | close_data_path, new, register_peer |
| `wire_discovery_callback` | 474 | close_data_path, new |
| `shutdown` | 499 | close_data_path |
| `estimate_bandwidth_from_rssi` | 523 | default, new |
| `test_wifi_aware_initialization` | 549 | default, new |
| `test_wifi_aware_unavailable` | 559 | default, new |
| `test_publish_service` | 569 | default, new |
| `test_subscribe_service` | 580 | default, random, new |
| `test_publish_disabled` | 591 | default, random, new |
| `test_invalid_config` | 604 | default, random, new |
| `test_create_data_path` | 613 | default, random, new |
| `test_data_path_not_found` | 638 | default, random, new |
| `test_close_data_path` | 653 | default, random, new |
| `test_get_active_data_paths` | 676 | default, random, new |
| `test_max_data_paths_limit` | 707 | default, random, new |
| `test_shutdown` | 736 | default, random, new |
| `test_bandwidth_estimation` | 747 | default, random, new |
| `test_get_discovered_peers` | 763 | default, random, new |
| `test_config_validation` | 787 | default, new |

### Imports
- `use async_trait::async_trait`
- `use libp2p::PeerId`
- `use parking_lot::RwLock`
- `use std::collections::HashMap`
- `use std::net::SocketAddr`
- `use std::sync::Arc`
- `use super::*`
- `use thiserror::Error`
- `use tracing::{info, warn}`
---

## cli/src/ble_mesh.rs (1 chunks, 295 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/ble_mesh.rs: 8 functions; 13 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `scm_service_uuid` | 26 | from_u128, to_value, from_bytes, Legacy, JsonRpc |
| `scm_notify_uuid` | 30 | Legacy, to_value, from_bytes, JsonRpc |
| `decode_ble_payload_for_ui` | 36 | Legacy, to_value, from_bytes, JsonRpc |
| `push_message_to_ui` | 58 | Legacy, to_value, JsonRpc |
| `subscribe_ingress_for_peripheral` | 75 |  |
| `run_ble_central_ingress` | 128 | new |
| `run_ble_peripheral_advertising` | 225 | new, sleep, from_secs |
| `decode_rejects_short_buffer` | 245 | new |

### Imports
- `use btleplug::api::bleuuid::uuid_from_u16`
- `use btleplug::platform::{Manager, Peripheral}`
- `use crate::server::{UiEvent, UiOutbound}`
- `use futures_util::StreamExt`
- `use scmessenger_core::IronCore`
- `use scmessenger_core::IronCore as CoreIron`
- `use scmessenger_core::drift::frame::{DriftFrame, FrameType}`
- `use scmessenger_core::wasm_support::rpc::{notif_message_received, MessageReceivedParams}`
- `use std::collections::HashSet`
- `use std::sync::Arc`
- `use super::*`
- `use tokio::sync::Mutex`
- `use uuid::Uuid`
---

## cli/src/bootstrap.rs (1 chunks, 219 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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

## cli/src/transport_bridge.rs (1 chunks, 386 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/transport_bridge.rs: Defines 5 types: TransportPath, serde, TransportBridge, Default, TransportBridge; 26 functions; 7 imports

### Structs/Classes
- Default
- TransportBridge
- TransportPath
- serde

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `serialize` | 22 | new, detect_cli_capabilities |
| `default` | 48 | new, detect_cli_capabilities, get_wasm_transports |
| `new` | 55 | new, is_compatible_path, find_cli_bridge_transport, get_wasm_transports, detect_cli_capabilities |
| `detect_cli_capabilities` | 63 | new, is_compatible_path, estimate_path_latency, find_cli_bridge_transport, get_wasm_transports, get_path_reliability |
| `register_peer` | 75 | new, is_compatible_path, estimate_path_latency, find_cli_bridge_transport, get_wasm_transports, get_path_reliability |
| `find_all_paths` | 86 | new, is_compatible_path, estimate_path_latency, find_cli_bridge_transport, get_wasm_transports, get_path_reliability, find_all_paths |
| `find_best_path` | 129 | find_all_paths |
| `get_wasm_transports` | 135 |  |
| `is_compatible_path` | 143 |  |
| `find_cli_bridge_transport` | 149 | new, find_all_paths |
| `get_path_reliability` | 159 | new, find_all_paths |
| `estimate_path_latency` | 177 | new, find_all_paths |
| `get_available_paths` | 195 | new, find_all_paths |
| `get_cli_capabilities` | 209 |  |
| `get_peer_capabilities` | 214 |  |
| `get_available_peer_capabilities` | 221 | get_peer_capabilities |
| `register_peer_capabilities` | 243 | get_peer_capabilities, find_all_paths |
| `can_forward_for_wasm` | 252 | get_peer_capabilities, find_all_paths |
| `get_forwarding_capability` | 258 | get_peer_capabilities, find_all_paths |
| `can_reach_destination` | 270 | new, get_peer_capabilities, ed25519_from_bytes, find_all_paths |
| `get_best_forwarding_path` | 285 | new, ed25519_from_bytes, find_all_paths |
| `create_test_peer_id` | 300 | new, ed25519_from_bytes |
| `test_transport_bridge_creation` | 315 | new |
| `test_peer_registration` | 327 | new |
| `test_path_finding` | 344 | new |
| `test_path_scoring` | 370 | new |

### Imports
- `use libp2p::PeerId`
- `use libp2p::identity::Keypair`
- `use scmessenger_core::transport::abstraction::TransportType`
- `use serde::ser::SerializeStruct`
- `use std::collections::HashMap`
- `use super::*`
---

## mobile/src/lib.rs (1 chunks, 77 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/relay/invite.rs (1 chunks, 498 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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

## cli/src/main.rs (1 chunks, 3730 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/main.rs: Defines 8 types: Cli, Commands, AuditAction, BlockAction, IdentityAction; 33 functions; 15 imports

### Structs/Classes
- AuditAction
- BlockAction
- Cli
- Commands
- ConfigAction
- ContactAction
- IdentityAction
- SwarmAction

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `load_or_create_headless_network_keypair` | 32 | create_dir_all, read, from_protobuf_encoding, generate_ed25519, set_permissions, from_mode, write |
| `main` | 362 | parse, layer, create_dir_all, hourly, new, non_blocking, try_from_default_env, from, registry, data_dir |
| `cmd_stop` | 439 | load, stop_node_via_api, is_api_available, with_storage, data_dir |
| `cmd_init` | 453 | with_storage, load, data_dir |
| `cmd_identity` | 492 | with_storage, load, write, data_dir |
| `print_full_identity` | 623 |  |
| `cmd_contact` | 681 | add_contact_via_api, is_api_available, validate_ed25519_public_key, with_storage, data_dir |
| `cmd_config` | 924 | with_storage, load, from_str, data_dir |
| `cmd_history` | 1022 | with_storage, data_dir |
| `cmd_start` | 1078 | load, bind, is_api_available, with_storage, from, data_dir |
| `cmd_relay` | 2060 | with_storage, new, data_dir |
| `cmd_send_offline` | 2493 | is_api_available, new_v4, with_storage, new, now, send_message_via_api, persistent, data_dir |
| `cmd_status` | 2575 | is_api_available, get_peers_via_api, data_dir, with_storage, is_ble_available |
| `cmd_mark_sent` | 2678 | with_storage, data_dir |
| `cmd_history_clear` | 2699 | with_storage, data_dir |
| `cmd_history_enforce_retention` | 2712 | with_storage, data_dir |
| `cmd_history_prune_before` | 2729 | with_storage, data_dir |
| `cmd_block` | 2746 | with_storage, data_dir |
| `cmd_history_get` | 2852 | with_storage, data_dir |
| `cmd_history_stats` | 2883 | with_storage, data_dir |
| `cmd_history_count` | 2910 | with_storage, data_dir |
| `cmd_history_mark_delivered` | 2919 | with_storage, data_dir |
| `cmd_history_clear_conversation` | 2935 | with_storage, new, data_dir |
| `cmd_history_delete` | 2960 | with_storage, new, data_dir |
| `cmd_test` | 2972 | new |
| `looks_like_blake3_id` | 3028 | read_dir, remove_file, from_secs, from_timestamp, now |
| `looks_like_libp2p_peer_id` | 3034 | read_dir, remove_file, from_secs, from_timestamp, now |
| `find_contact` | 3037 | read_dir, remove_file, from_secs, from_timestamp, now |
| `format_timestamp` | 3050 | read_dir, remove_file, load, from_secs, from_timestamp, with_storage, now, write, data_dir |
| `prune_logs` | 3059 | read_dir, remove_file, load, from_secs, with_storage, now, write, data_dir |
| `cmd_audit` | 3086 | with_storage, load, write, data_dir |
| `cmd_swarm` | 3129 | get_peers_via_api, is_api_available |
| `cmd_swarm_stats` | 3135 | get_listeners_via_api, get_peers_via_api, is_api_available |

### Imports
- `use anyhow::{Context, Result}`
- `use chrono::{DateTime, Local, Utc}`
- `use clap::{Parser, Subcommand}`
- `use colored::*`
- `use libp2p::{Multiaddr, PeerId}`
- `use scmessenger_core::IronCore`
- `use scmessenger_core::message::{decode_envelope, MessageType}`
- `use scmessenger_core::store::{Contact, ContactManager, MessageDirection, Outbox, QueuedMessage}`
- `use scmessenger_core::transport::abstraction::TransportType`
- `use scmessenger_core::transport::{self, SwarmEvent}`
- `use std::collections::HashMap`
- `use std::os::unix::fs::PermissionsExt`
- `use std::sync::Arc`
- `use tracing_subscriber::prelude::*`
---

## cli/src/api.rs (1 chunks, 941 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/api.rs: Defines 14 types: SendMessageRequest, SendMessageResponse, AddContactRequest, AddContactResponse, PeerEntry; 24 functions; 7 imports

### Structs/Classes
- AddContactRequest
- AddContactResponse
- ApiContext
- ConnectionPathStateResponse
- DriftStatusResponse
- GetExternalAddressResponse
- GetHistoryRequest
- GetHistoryResponse
- GetListenersResponse
- GetPeersResponse
- HistoryMessage
- PeerEntry
- SendMessageRequest
- SendMessageResponse

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `is_api_available` | 94 | to_bytes, connect, from_slice, to_string, new, builder, from |
| `send_message_via_api` | 99 | to_bytes, from_slice, to_string, new, builder, from |
| `add_contact_via_api` | 129 | empty, to_bytes, from_slice, to_string, new, builder, from |
| `get_peers_via_api` | 166 | empty, to_bytes, from_slice, to_string, new, builder, from |
| `get_history_via_api` | 181 | empty, to_bytes, from_slice, to_string, new, builder, from, from_utf8_lossy |
| `get_external_address_via_api` | 203 | empty, to_bytes, from_slice, new, builder, from_utf8_lossy |
| `get_listeners_via_api` | 226 | empty, to_bytes, from_utf8, from_slice, new, builder |
| `get_connection_path_state_via_api` | 239 | empty, to_bytes, from_utf8, from_slice, new, builder |
| `get_drift_state_via_api` | 252 | empty, to_bytes, from_utf8, from_slice, new, builder |
| `export_diagnostics_via_api` | 264 | empty, spawn, to_bytes, from_utf8, from_millis, new, builder, sleep |
| `stop_node_via_api` | 283 | empty, spawn, exit, from_millis, new, builder, from, sleep |
| `handle_request` | 294 | spawn, to_bytes, exit, from_slice, from_millis, builder, from, sleep |
| `handle_send_message` | 335 | to_bytes, from_slice, to_string, builder, from |
| `handle_add_contact` | 379 | to_bytes, from_slice, to_string, new, builder, from |
| `handle_get_peers` | 406 | to_bytes, from_slice, to_string, builder, from |
| `handle_get_listeners` | 431 | to_bytes, from_slice, to_string, builder, from |
| `handle_get_history` | 448 | to_bytes, from_slice, to_string, builder, from |
| `handle_get_external_address` | 487 | from, builder, to_string |
| `get_connection_path_state` | 513 | now |
| `export_diagnostics` | 530 | now |
| `handle_get_connection_path_state` | 563 | from, builder, to_string |
| `handle_export_diagnostics` | 602 | from, builder |
| `handle_get_drift_status` | 645 | bind, to_string, new, builder, from |
| `start_api_server` | 660 | from, new, bind |

### Imports
- `use anyhow::{Context, Result}`
- `use hyper::service::{make_service_fn, service_fn}`
- `use hyper::{Body, Method, Request, Response, Server, StatusCode}`
- `use serde::{Deserialize, Serialize}`
- `use std::convert::Infallible`
- `use std::net::SocketAddr`
- `use std::sync::Arc`
---

## wasm/src/lib.rs (1 chunks, 2357 lines)
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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
Function `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses` not found in REPO_MAP chunks. Full file listing below.

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
