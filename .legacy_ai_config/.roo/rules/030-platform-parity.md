# Platform Parity Rules

## Cross-Platform Requirement
All critical-path behavior MUST be identical across Android, iOS, and Web/WASM per PHIL-006 and PHIL-010.

## Verification Points
- Relay ON/OFF semantics identical
- Identity display/exchange identical
- Send/receive flow identical
- Settings/preferences aligned
- Error handling consistent

## UniFFI Binding Alignment
All platform bindings MUST align with core/src/api.udl contract.

## Testing Requirement
Platform-specific changes MUST be verified on all target platforms before merge.
