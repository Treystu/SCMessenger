#!/usr/bin/env python3
"""Generate the desktop bridge's Kotlin bindings and prove the output is real.

Why this exists (#590). The `MT`/`D1` verification task asked for three things:
build `desktop_bridge`, run its tests, and verify UniFFI Kotlin generation for
the desktop target. Two of the three were already covered and one was not:

- `cargo test --workspace --features test-utils` (.circleci/config.yml, job
  `test_ubuntu`) compiles and runs the crate's unit tests, because
  `desktop_bridge` is a workspace member.
- Kotlin generation was attempted only by `.github/workflows/desktop.yml`,
  whose `build` job has been failing at job level with **zero steps** and no
  uploaded log (run 44938258938, 2026-11-21T18:56:51Z -> 18:56:54Z). A job
  that fails before its first step proves nothing about the bindings, and its
  red is invisible to the CircleCI contexts this repository gates on.
- Nothing on the CircleCI rail ever ran `gen_kotlin` for `desktop_bridge`. The
  `gen_kotlin` binary is declared with `required-features = ["gen-bindings"]`,
  so `cargo test --workspace` never builds it: without this check the target
  could stop compiling and the whole repository would stay green.

The check is not "the command exited 0". `gen_kotlin` is precisely the tool
that exited 0 while emitting an incomplete binding set in #155, so a green exit
is the least trustworthy signal here. It asserts the generated Kotlin names the
exported type and its methods, which is what a downstream KSP consumer needs.

Usage:
  python3 scripts/check_desktop_bridge_bindings.py
  python3 scripts/check_desktop_bridge_bindings.py --self-check
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

PACKAGE = "scmessenger-desktop-bridge"
BINARY = "gen_kotlin"
FEATURE = "gen-bindings"
CDYLIB_STEM = "scmessenger_desktop_bridge"
CDYLIB_SUFFIXES = (".so", ".dylib", ".dll")

# A real binding set for this crate is several hundred lines: one interface
# with ~20 methods plus an enum/record per exported type. A floor catches the
# #155 shape (exit 0, near-empty output) without pinning an exact size that
# would fail on every harmless regeneration.
MIN_TOTAL_LINES = 200

# Names the generated Kotlin must contain, each tied to something the crate
# actually exports. If a rename lands, this list is the one place to update.
REQUIRED_MARKERS = (
    ("interface DesktopBridge", "the exported object at desktop_bridge/src/desktop_bridge.rs:17"),
    ("fun getXdgPaths", "DesktopBridge::get_xdg_paths (desktop_bridge.rs:64)"),
    ("fun start", "DesktopBridge::start (desktop_bridge.rs:78)"),
    ("fun sendNotification", "DesktopBridge::send_notification (desktop_bridge.rs:128)"),
)


def target_directory(root: Path) -> Path:
    """Cargo's own answer for where build output went.

    Asked rather than assumed: build #6122 proved `target/debug` is not a
    reliable guess on this image even though the build succeeded, and a
    CARGO_TARGET_DIR override or a profile setting can move it again.
    """
    completed = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=str(root),
        capture_output=True,
        text=True,
    )
    if completed.returncode == 0:
        try:
            return Path(json.loads(completed.stdout)["target_directory"])
        except (KeyError, ValueError):
            pass
    return root / "target"


def find_cdylib(target_dir: Path) -> Path | None:
    """The built cdylib for this crate, searched anywhere under the target dir."""
    # The `lib` prefix is platform-dependent (`libfoo.so`, `foo.dll`), so match
    # the stem anywhere in the name rather than assuming one spelling.
    for path in sorted(target_dir.rglob(f"*{CDYLIB_STEM}.*")):
        if path.is_file() and path.suffix in CDYLIB_SUFFIXES:
            return path
    return None


def kotlin_sources(out_dir: Path) -> list[Path]:
    """Every generated Kotlin file, sorted, with no truncation."""
    if not out_dir.is_dir():
        return []
    return sorted(p for p in out_dir.rglob("*.kt") if p.is_file())


def validate(files: list[tuple[str, str]]) -> list[str]:
    """Problems with a generated binding set, given (name, content) pairs.

    Pure so `--self-check` can prove the detector fires on the #155 shape
    (an empty or symbol-less set that still exited 0) as well as passing a
    realistic one.
    """
    problems: list[str] = []
    if not files:
        return ["generated no Kotlin files at all (gen_kotlin can exit 0 having emitted nothing)"]

    combined = "\n".join(content for _, content in files)
    total_lines = sum(content.count("\n") + (0 if content.endswith("\n") else 1) for _, content in files)

    if total_lines < MIN_TOTAL_LINES:
        problems.append(
            f"generated {total_lines} line(s) across {len(files)} file(s), "
            f"below the {MIN_TOTAL_LINES}-line floor for this crate"
        )

    for marker, why in REQUIRED_MARKERS:
        if marker not in combined:
            problems.append(f"missing `{marker}` - would break the consumer of {why}")

    return problems


def run(command: list[str], cwd: Path, env: dict[str, str] | None = None) -> int:
    print(f"[INFO] {' '.join(command)}", flush=True)
    completed = subprocess.run(command, cwd=str(cwd), env=env)
    return completed.returncode


def self_check() -> bool:
    """Prove the detector fails the #155 shape and passes a realistic set."""
    realistic_body = "\n".join(
        ["interface DesktopBridge {"] + [f"    fun method{i}()" for i in range(220)] + ["}"]
    )
    realistic = [("DesktopBridge.kt", realistic_body + "\n    fun getXdgPaths()\n    fun start()\n    fun sendNotification()\n")]

    cases: list[tuple[str, list[tuple[str, str]], bool]] = [
        ("realistic binding set passes", realistic, True),
        ("empty output fails", [], False),
        ("symbol-less output fails", [("DesktopBridge.kt", "// nothing here\n")], False),
        ("truncated set fails", [("DesktopBridge.kt", "interface DesktopBridge {\n}\n")], False),
    ]

    ok = True
    for name, files, want_pass in cases:
        problems = validate(files)
        passed = not problems
        verdict = "OK" if passed == want_pass else "FAIL"
        if passed != want_pass:
            ok = False
        print(f"[{verdict}] self-check: {name} (problems={len(problems)})")

    # The realistic case must be missing nothing, or the markers above drifted.
    for problem in validate(realistic):
        ok = False
        print(f"[ERROR] self-check: realistic fixture rejected: {problem}")
    return ok


def main(argv: list[str] | None = None) -> int:
    root = Path(__file__).resolve().parent.parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-check", action="store_true", help="verify the detector itself")
    parser.add_argument(
        "--out-dir",
        default=str(root / "desktop_bridge" / "target" / "generated-sources" / "uniffi" / "kotlin"),
        help="where gen_kotlin writes (its own default, from desktop_bridge/src/bin/gen_kotlin.rs)",
    )
    args = parser.parse_args(argv)

    if args.self_check:
        return 0 if self_check() else 1

    build = run(["cargo", "build", "-p", PACKAGE, "--features", FEATURE], root)
    if build != 0:
        print(f"[FAIL] desktop bridge bindings: `cargo build -p {PACKAGE} --features {FEATURE}` exited {build}")
        return 1

    search_dir = target_directory(root)
    cdylib = find_cdylib(search_dir)
    if cdylib is None:
        print(
            f"[FAIL] desktop bridge bindings: no {CDYLIB_STEM}{{so,dylib,dll}} under "
            f"{search_dir} after a successful build"
        )
        return 1
    print(f"[INFO] built cdylib: {cdylib}")

    # Pass the library explicitly: gen_kotlin otherwise guesses from `HOST` and
    # relative paths, and a wrong guess panics with a message about a build the
    # job has just completed.
    env = dict(os.environ)
    env["SCMESSENGER_DESKTOP_CDYLIB_PATH"] = str(cdylib)
    generate = run(
        ["cargo", "run", "-p", PACKAGE, "--bin", BINARY, "--features", FEATURE],
        root,
        env,
    )
    if generate != 0:
        print(f"[FAIL] desktop bridge bindings: `{BINARY}` exited {generate}")
        return 1

    out_dir = Path(args.out_dir)
    paths = kotlin_sources(out_dir)
    files = [(str(p.relative_to(out_dir)), p.read_text(encoding="utf-8", errors="replace")) for p in paths]

    # Full list, never truncated: the file inventory is part of the evidence.
    print(f"[INFO] generated {len(files)} Kotlin file(s) under {out_dir}")
    for name, content in files:
        lines = content.count("\n") + (0 if content.endswith("\n") else 1)
        print(f"[INFO]   {name}: {lines} line(s)")

    problems = validate(files)
    for problem in problems:
        print(f"[ERROR] {problem}")

    total = sum(c.count("\n") + (0 if c.endswith("\n") else 1) for _, c in files)
    print(
        f"[{'OK' if not problems else 'FAIL'}] desktop bridge bindings: "
        f"{len(files)} file(s), {total} line(s), {len(problems)} problem(s)"
    )
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
