# Task: Fix Windows link.exe Build Blocker

## Problem
Git Bash's `/usr/bin/link` (GNU hard-link utility) shadows the MSVC linker `link.exe`.
`where link.exe` returns `C:\Program Files\Git\usr\bin\link.exe`.
No actual MSVC `link.exe` is found on the system (`/c/Program Files (x86)/Microsoft Visual Studio/2022` is empty).

Current state:
- Rust target: `x86_64-pc-windows-msvc`
- Only installed target: `x86_64-pc-windows-msvc`
- `gcc` / MinGW: NOT available
- `rust-lld.exe` exists in Rust toolchain but cannot find Windows system libs

## Error
```
link: extra operand '...build_script_build...rcgu.o'
```
(Cargo is calling `/usr/bin/link` instead of MSVC linker)

## Goals
1. Get `cargo check --workspace` passing on Windows
2. Get `cargo test --workspace --no-run` passing (compile gate)
3. Ensure Android build (`./gradlew assembleDebug`) can compile the Rust core via cargo-ndk

## Approaches (investigate all)
1. **Install MSVC Build Tools** via `winget install Microsoft.VisualStudio.2022.BuildTools` or VS Installer
2. **Switch to GNU target** (`rustup target add x86_64-pc-windows-gnu`) and use `gcc` if available
3. **PATH fix** - remove `/usr/bin` from PATH during cargo builds, or reorder PATH
4. **Rust-lld with libpath** - configure rust-lld to find Windows SDK libs
5. **Use `lld-link.exe`** from LLVM if installed

## Constraints
- Do NOT use `Stop-Process` or `taskkill` on claude.exe
- Do NOT modify `orchestrator_manager.sh` hygiene sweep
- If installing software, report what was installed and verify it works
- Keep `.cargo/config.toml` changes minimal and well-documented

## Verification
After fix, run:
```bash
cargo check --workspace
cargo test --workspace --no-run
cd android && ./gradlew assembleDebug -x lint --quiet
```

Report: what was changed, build status, and any remaining blockers.
