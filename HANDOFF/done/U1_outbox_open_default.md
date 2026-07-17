# U1 Outbox::open_default() helper

## Task Description
Create a single source of truth for outbox initialization. 
Currently, `Outbox::persistent(...)` is initialized independently in 3 places in the CLI:
`cli/src/main.rs:1318` (`cmd_start`), `cli/src/main.rs:2478` (`cmd_relay`), and `cli/src/main.rs:2932` (`cmd_send_offline`).

Fix direction: Create a single `Outbox::open_default(data_dir: &Path)` helper in `core/src/store/outbox.rs`, called from all 3 CLI sites.

## Target Files
- `core/src/store/outbox.rs`
- `cli/src/main.rs`

## Acceptance Criteria
- `Outbox::open_default` is defined and used by the CLI.
- No duplicate `Outbox::persistent` configuration logic remains in `cli/src/main.rs`.
- Gate: `cargo check --workspace`
