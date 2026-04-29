# Build & CI Rules

Re-injected into agent context on every turn.

## Build Verification (Mandatory)

### Before Finalizing Any Run:
1. **Rust edits:** Run `cargo build --workspace`. Record output in HANDOFF notes.
2. **Android edits:** Run `cd android && ./gradlew assembleDebug -x lint --quiet`.
3. **WASM edits:** Run `cargo build -p scmessenger-wasm --target wasm32-unknown-unknown`.
4. **Format check:** `cargo fmt --all -- --check`
5. **Lint pass:** `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments`

### Compile Gate
Before considering a task complete, `cargo test --workspace --no-run` must succeed (builds all tests without running them).

## Docs Sync

Run `./scripts/docs_sync_check.sh` (or the PowerShell equivalent `.ps1`) after any documentation change. Resolve failures before finalizing.

## Path Conventions (CI Enforced)

- Use `iOS/` (uppercase-I) for ALL path references. Lowercase `ios/` fails path-governance check.
- XCFramework location: `iOS/SCMessengerCore.xcframework/` — never in repo root.
- No `.py` files in repo root — use `scripts/`.
- No build artifacts committed — verify with `git ls-files "*.log" "*.pid" "*.logcat"`.

## Windows-Specific

- Incremental compilation is disabled (`.cargo/config.toml`) — prevents rlib metadata errors during integration test builds.
- Shell scripts require Git Bash or WSL. PowerShell equivalents exist for key scripts.
- CI runs on ubuntu-latest and macos-latest ONLY. Windows builds are local-only.

## Model Availability Check

Before launching any agent, verify the target model is available:
```bash
bash .claude/model_validation_template.sh
```

Or use WebFetch to check `https://ollama.com/api/tags` for the current catalog.
