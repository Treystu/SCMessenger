# Build Verification Rules

## Mandatory Build Verification
All code, binding, wiring, or generated code changes MUST be verified with appropriate build commands before session end.

## Platform-Specific Build Commands

### Rust Core
```bash
cargo test --workspace
```

### Android
```bash
cd android && ./gradlew assembleDebug
```

### iOS
```bash
cd iOS && ./build-device.sh
```

### WASM
```bash
cd wasm && cargo build --target wasm32-unknown-unknown
```

## Verification Scope
- Small edits are NOT exempt from build verification
- Generated bindings (UniFFI) must be regenerated and verified
- Cross-platform changes require verification on all affected platforms

## Failure Handling
If build verification fails, resolve the issue before concluding the session. Document any blockers with exact command and failure reason.
