# Rust-Specific Rules

## Module Organization
- Core logic in `core/src/`
- Mobile bindings in `mobile/src/`
- WASM bindings in `wasm/src/`

## Async Patterns
- Use tokio runtime for all async operations
- Prefer `async fn` over `impl Future`
- Use `tokio::spawn` for concurrent tasks

## Error Handling
- Use `thiserror` for error types
- Propagate errors with `?` operator
- Never panic in library code

## UniFFI Integration
- All public API functions must be in `api.udl`
- Use `#[uniffi::export]` for exported functions
- Test bindings after any API changes

## Crypto Safety
- Zeroize sensitive data on drop
- Use constant-time comparisons for secrets
- Never log private keys or seeds
