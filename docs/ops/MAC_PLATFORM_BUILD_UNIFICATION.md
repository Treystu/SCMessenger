# macOS Platform Build Unification — 2026-06-10

**Status:** ACTIVE — all 5 platforms now buildable on macOS without dual-boot
**Host:** Apple Silicon Mac Mini, 8GB RAM, Darwin 23.5.0
**Last verified:** 2026-06-10

## What this document covers

A one-time Mac-bootstrap that converts the SCMessenger build from "Windows-shaped,
scripted, manual-fixes-needed" to "cross-platform, scripted, adb-install-ready." All
changes preserve Windows compatibility — the source of truth is now Unix, with
Windows-specific paths only in override files that are gitignored.

## 1. Toolchain installed in user-space (`~/`)

| Tool | Version | Location | Install method |
|------|---------|----------|---------------|
| Homebrew | 5.1.15-254 | `~/homebrew/` | curl install script |
| rustup + cargo | 1.95.0 | `~/.cargo/bin/` | pre-existing (verified) |
| rustup targets | 14 targets (aarch64/x86_64 android, ios, darwin, linux-musl, wasm) | rustup component | pre-existing |
| Android command-line tools | 37.0.0 | `~/homebrew/share/android-commandlinetools/` | `brew install --cask android-commandlinetools` |
| Android NDK r26b (build pin) | 26.1.10909125 | `~/Library/Android/sdk/ndk/26.1.10909125/` | sdkmanager |
| Android NDK r29 (fallback) | 29.x | `~/homebrew/share/android-ndk/` | `brew install --cask android-ndk` |
| Android platform-tools (adb) | 37.0.0 | `~/Library/Android/sdk/platform-tools/` | sdkmanager |
| Android build-tools | 35.0.0 | `~/Library/Android/sdk/build-tools/35.0.0/` | sdkmanager |
| Android platform 35 | 35_r02 | `~/Library/Android/sdk/platforms/android-35/` | sdkmanager |
| Java (JBR) | 21.0.10 | `/Applications/Android Studio.app/Contents/jbr/Contents/Home/` | pre-existing |
| cargo-ndk | 4.1.2 | `~/.cargo/bin/cargo-ndk` | `cargo install cargo-ndk` |
| gh CLI | 2.93.0 | `~/homebrew/bin/gh` | `brew install gh` |

## 2. Shell environment (`~/.zprofile`)

```bash
export PATH="$HOME/homebrew/bin:$HOME/.local/bin:$HOME/.cargo/bin:$PATH"
export ANDROID_HOME="$HOME/Library/Android/sdk"
export NODE_OPTIONS="--max-old-space-size=2048"
export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
export PATH="$JAVA_HOME/bin:$PATH"
export ANDROID_NDK_HOME="$HOME/Library/Android/sdk/ndk/26.1.10909125"
export ANDROID_NDK_ROOT="$ANDROID_NDK_HOME"
export PATH="$HOME/Library/Android/sdk/platform-tools:$HOME/Library/Android/sdk/emulator:$PATH"
```

## 3. Cargo config (`~/.cargo/config.toml`)

User-level muzzle: `jobs=4`, `incremental=false` for dev/release, linkers for each
target triple. Repo-level `.cargo/config.toml` is Windows-shaped (MSVC + GNU
fallback) and left untouched for Windows users.

## 4. CRLF → LF conversion (128 files)

The repo's `.sh`, `.bash`, `.py`, `.gradle` files were committed with Windows CRLF
line endings. macOS bash rejects these ("set: pipefail: invalid option name" when
the shebang is `#!/bin/bash\r`). All converted to LF. `.ps1` files intentionally
left as CRLF (PowerShell accepts either).

Conversion command (run from repo root):
```bash
find . -type f \( -name "*.sh" -o -name "*.bash" -o -name "*.py" -o -name "*.gradle" \) \
  -not -path "./.git/*" -not -path "./target/*" -not -path "*/build/*" \
  -not -path "*/.gradle/*" -not -path "./core/target/*" 2>/dev/null \
  | xargs -I {} sh -c 'file "{}" | grep -q CRLF && tr -d "\r" < "{}" > "{}.tmp" && mv "{}.tmp" "{}"'
```

## 5. Cross-platform gradle.properties

**`android/gradle.properties`** source-of-truth changes:
- `org.gradle.jvmargs=-Xmx4g` → `-Xmx3g` (8GB Mac Mini throttle)
- `org.gradle.user.home=E:/build-tools/.gradle` → `${user.home}/.gradle`
- Windows-specific comments kept for context, paths removed

**`android/gradle.properties.local-overrides`** (gitignored): per-machine overrides
with example values for 8GB and 16GB hosts.

## 6. Mac-port of `.claude/scripts/quota_lib.sh`

The bash `quota_lib.sh` used `\s*` regex (Linux/Git-Bash only). On macOS BSD sed/grep
this failed silently, causing `get_quota_context` to leak the entire match string
into the eval'd output, which then triggered `bash: ok: command not found` at
`orchestrator_manager.sh:615`. Replaced with `[ \t]*` + `-E` for ERE. Committed as
`eedbfb58` on the integration branch.

## 7. adb wired to PATH

Both `~/.zprofile` and `~/.zshrc` now export
`~/Library/Android/sdk/platform-tools:$PATH` so `adb` resolves in any new shell.
`adb version` → `Android Debug Bridge version 1.0.41 (37.0.0-14910828)`.

## 8. Debug keystore

Created at `~/.android/debug.keystore` (2,048-bit RSA, 10,000-day validity,
password `android`, alias `androiddebugkey`). The Android `app/build.gradle` has a
fallback block that uses this keystore for release builds when no other signing
config is provided.

## 9. Build verification (5/5 GREEN on integration branch)

| # | Gate | Result | Wall | State file |
|---|------|--------|------|------------|
| 1 | `cargo check --workspace` | PASS | 10.33s | `HANDOFF/STATE/2026-06-10_BUILD_GATE_integration_cargo_check.log` |
| 2 | `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` | PASS | 4.71s | `HANDOFF/STATE/2026-06-10_BUILD_GATE_integration_cargo_check_wasm.log` |
| 3 | `cargo test --workspace --no-run` | PASS | 36.65s | `HANDOFF/STATE/2026-06-10_BUILD_GATE_integration_cargo_test_norun.log` |
| 4 | `cargo build -p scmessenger-cli` | PASS | 24.41s | `HANDOFF/STATE/2026-06-10_BUILD_GATE_integration_cargo_build_cli.log` |
| 5 | `./gradlew :app:compileDebugKotlin` | PASS | 6m 52s | `HANDOFF/STATE/2026-06-10_BUILD_GATE_integration_gradle_compileDebugKotlin.log` |

## 10. APK install procedure (when phone is connected)

```bash
# After adb devices shows your phone:
cd ~/Documents/Github/SCMessenger
./android/install-clean.sh
# or for incremental install (faster, no uninstall):
cd android && ./gradlew installDebug
```

The `install-clean.sh` script handles:
- adb device detection + mDNS reconnect (if wireless)
- Optional uninstall of existing app
- Gradle clean + installDebug
- Grants all runtime permissions (BLE, WiFi Aware, location, notifications)

## 11. Pool launch path now functional

The `bash .claude/orchestrator_manager.sh pool launch <agent> <task>` path now
works on Mac after the quota_lib + CRLF + chmod fixes. The 1 worker currently
running (`rust-coder_1781140384` on P1_CLI_024) is proof of life.

## 12. How to switch back to Windows

1. The `android/gradle.properties.local-overrides` is gitignored — on Windows, you
   can re-create it with `org.gradle.user.home=E:/build-tools/.gradle` and
   `org.gradle.jvmargs=-Xmx4g -XX:MaxMetaspaceSize=1g`.
2. The repo-level `.cargo/config.toml` is already Windows-shaped (MSVC + GNU
   fallback) — it just works on Windows without the user-level `~/.cargo/config.toml`.
3. `.claude/scripts/quota_lib.sh` regex now uses `[ \t]*` + `-E` which is POSIX
   ERE-compatible (works on both macOS and Linux/Git-Bash). No re-merge needed.
4. CRLF → LF conversion: Windows line endings were a bug on Unix shells; the LF
   versions still parse fine on Windows shells (which accept LF or CRLF). The
   change is one-way safe.
5. The `powershell.exe`/`taskkill`/`Get-CimInstance` calls in
   `orchestrator_manager.sh` and `force_kill_pid` have `kill` and `taskkill`
   fallbacks that work on both platforms.

## 13. Verification on a clean Mac

```bash
# After cloning the repo on a fresh Mac:
bash android/verify-build-setup.sh   # checks all prerequisites
# If all checks pass:
cd android && ./gradlew :app:compileDebugKotlin
# If that succeeds, the workspace is unified and ready.
```
