# Target Cleanup — 2026-08-08

Status: Active
Last updated: 2026-08-08

## Summary

C: drive was at 98% (6.5 GB free), below the repo's 25 GB minimum gate
threshold for a full build sweep. Reclaimed disk space from the Rust build
output tree using `scripts/clean_target.sh` (never `cargo clean` — that
command is blocked by a PreToolUse hook and is known to wipe all of
`target/` even when scoped with `--target <triple>`).

## Free space

- Before: 6.4 GB free (`df -h /c`, 98% used)
- After: 31.4 GB free per the cleanup script's own accounting; final
  `df -h /c` read 32 GB free / 87% used (237G total, 205G used)

## What was removed

| Path | Size reclaimed | Why safe |
|---|---|---|
| `target/aarch64-linux-android/` | 5.2 GB | Cross-compile output, regenerable via `cargo-ndk`; not currently building |
| `target/armv7-linux-androideabi/` | 3.9 GB | Same as above |
| `target/i686-linux-android/` | 4.0 GB | Same as above |
| `target/x86_64-linux-android/` | 5.1 GB | Same as above |
| `target/debug/deps/` | 7.7 GB | Intermediate object files; keeps built binaries in `target/debug/` |
| `target/debug/build/` | 371 MB | Build-script intermediates |
| `target/debug/incremental/` | 3.9 MB | Incremental compilation cache |
| `target/debug/examples/` | 0 (empty) | n/a |

Total reclaimed: ~26.3 GB (18.0 GB from the `--triples` pass, 7.0 GB from
the `--deps` pass, per the script's own before/after `df` accounting).

Both passes were run via `scripts/clean_target.sh` (`--triples` then
`--deps`), which deletes by explicit path rather than invoking `cargo
clean`, and which backs up and verifies `core/target/generated-sources/`
around each run. That directory was intact before and after both passes
(contains `uniffi/`, ~1.2 MB).

## What was deliberately NOT removed

- `core/target/generated-sources/` (1.2 MB) — hard constraint; holds
  UniFFI Kotlin/Swift bindings that `scripts/ffi_surface.sh` silently
  depends on. Untouched.
- `core/target/android-libs/` (1.9 GB) — not covered by
  `scripts/clean_target.sh`, and its role (staged native libs consumed by
  the Android app build) was not confirmed safe to delete. Left alone
  since the 25 GB threshold was already cleared without it.
- `core/target/staged-cdylib/` (23 MB) — same reasoning as above; small
  enough not to matter either way.
- `android/build/` (136 KB) and `android/app/build/` — trivially small,
  not worth the risk/effort once the threshold was cleared.
- `tmp/` — explicitly off-limits (today's field-test logs still being
  analyzed).
- `target/debug/*` binaries (e.g. `scmessenger-cli.exe`) — kept; the live
  node runs from this exact path.
- No `cargo clean` in any form was run or attempted.

## Node process status

The task brief named PID 21340 as the live node. That PID no longer
existed by the time this task ran; investigation (`tasklist | grep -i
scmess`, then `wmic process where "ProcessId=<pid>" get
ExecutablePath,CommandLine`) found a different PID, 10864, running the
same binary and command: `target\debug\scmessenger-cli.exe start`. The
coordinator confirmed mid-task that PID 21340 had crashed independently
at 00:15:20 (reported cause: a libp2p-upnp panic, unrelated to this
cleanup) and a fresh node came up under a new PID.

`scmessenger-cli.exe` (now PID 10864) was verified alive via plain
`tasklist | grep -i scmess` before either deletion pass, and again
afterward — [OK] still running throughout. `target/debug/scmessenger-cli.exe`
itself was confirmed present and unmodified (same size/timestamp) after
both passes.

## Notes for whoever picks this up next

- No build was run and no source files were modified.
- If Android cross-compile targets are needed again, they must be
  rebuilt via `cargo-ndk` (`aarch64-linux-android` and
  `x86_64-linux-android` are the required targets per
  `.claude/rules/android.md`; `armv7-linux-androideabi` and
  `i686-linux-android` are full-coverage/optional).
- `target/debug/deps` will regrow on the next `cargo build`/`cargo check`;
  this is expected and not a regression.
