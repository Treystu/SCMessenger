# Build & CI Rules

Re-injected into agent context on every turn.

## Build Verification (Mandatory)

Scoped to what changed, before finalizing any run (prefer the `build-verify` skill):
1. Rust edits: `cargo build --workspace` (record output in HANDOFF notes).
2. Android edits: `cd android && ./gradlew assembleDebug -x lint --quiet`.
3. WASM edits: `cargo build -p scmessenger-wasm --target wasm32-unknown-unknown`.
4. Format: `cargo fmt --all -- --check`.
5. Lint: `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments`.

Compile gate: `cargo test --workspace --no-run` must pass before any task is
considered complete.

## Docs Sync

Run `./scripts/docs_sync_check.sh` (or the `.ps1`) after any documentation
change; resolve failures before finalizing.

## Path Conventions (CI Enforced)

- `iOS/` uppercase-I in ALL path references; XCFramework at `iOS/SCMessengerCore.xcframework/`.
- No `.py` in repo root (use `scripts/`); no build artifacts committed
  (`git ls-files "*.log" "*.pid" "*.logcat"` must be empty).

## Windows

- Incremental compilation disabled (`.cargo/config.toml`); also
  `export CARGO_INCREMENTAL=0` in the shell before cargo commands.
- Never run two build-tool invocations concurrently (see CLAUDE.md
  Windows-Specific Rules -- Gradle can spawn cargo-ndk upstream).
- Shell scripts need Git Bash/WSL; CI is ubuntu/macos only -- Windows builds
  verified locally.

## Model Availability Check (the `swarm` backend ONLY)

Only when using the `swarm` backend (ollama pool): verify the target ollama model
via `bash .claude/model_validation_template.sh` or `https://ollama.com/api/tags`.
Not applicable to the `lanes`, `native`, or `agent` backends -- for `native` the
model truth is `claude --help` aliases; for `lanes` it is the lake registry
`SCM_UNIFIED_LAKE_ORCHESTRATION.md`.
