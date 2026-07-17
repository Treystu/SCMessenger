# TASK: ONION_GATING_PART_A_FULL

Please add the following flat variant to `pub enum IronCoreError` inside `core/src/lib.rs` immediately after the `IoError,` variant:
```rust
    #[error("Onion routing disabled")]
    OnionRoutingDisabled,
```

Provide the FULL, completely updated contents of `core/src/lib.rs` using standard Markdown code block with `// core/src/lib.rs` as the first line.
