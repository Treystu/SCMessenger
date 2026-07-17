# U2 Topic constants

## Task Description
Define `TOPIC_LOBBY` and `TOPIC_MESH` once in core and import everywhere.
Today, `["sc-lobby", "sc-mesh"]` are hardcoded in `cli/src/main.rs` (lines ~1455 and 2465) and separately in `core/src/transport/swarm.rs`.

Fix direction: define `pub const TOPIC_LOBBY: &str = "sc-lobby";` and `pub const TOPIC_MESH: &str = "sc-mesh";` in `core/src/lib.rs` (or a new `constants.rs`) and import everywhere.

## Target Files
- `core/src/lib.rs`
- `core/src/transport/swarm.rs`
- `cli/src/main.rs`

## Acceptance Criteria
- `TOPIC_LOBBY` and `TOPIC_MESH` are defined exactly once.
- Hardcoded topic strings are removed from CLI and core transport.
- Gate: `cargo check --workspace`
