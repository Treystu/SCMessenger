#!/usr/bin/env python3
"""Fail a compose service that generates UniFFI bindings from a stripped host library.

`Cargo.toml` sets `[profile.release] strip = true`. UniFFI carries the metadata
that `gen_kotlin` reads inside the cdylib/build output; stripping removes it, so
`gen_kotlin` exits 0 having emitted an incomplete binding set and
`:app:kspDebugKotlin` then fails with

    e: [ksp] InjectProcessingStep was unable to process
    'AndroidPlatformBridge(...)' because 'error.NonExistentClass' could not be resolved

That is #155. The rule: in any compose command that runs `gen_kotlin` (or
`gen_swift`), a preceding `cargo build ... --release` must disable stripping for
that build with `CARGO_PROFILE_RELEASE_STRIP=false`, unless the workspace's
release profile does not strip in the first place.

Line-based on purpose: compose files are read in CI images that may not have
PyYAML, and the check must work with nothing installed.

Usage:
  python scripts/check_uniffi_binding_build.py
  python scripts/check_uniffi_binding_build.py --self-check
"""

from __future__ import annotations

import argparse
import io
import re
import sys
from pathlib import Path

GENERATOR = re.compile(r"\bgen_(?:kotlin|swift)\b")
RELEASE_BUILD = re.compile(r"\bcargo\s+build\b[^\n]*--release\b")
STRIP_OVERRIDE = "CARGO_PROFILE_RELEASE_STRIP=false"
LOOKBACK = 40


def release_strips(cargo_toml: str) -> bool:
    """True when [profile.release] sets strip to a stripping value."""
    in_release = False
    for raw in cargo_toml.splitlines():
        line = raw.strip()
        if line.startswith("[") and line.endswith("]"):
            in_release = line == "[profile.release]"
            continue
        if in_release and line.startswith("strip"):
            value = line.split("=", 1)[1].strip().strip('"')
            return value not in ("false", "none")
    return False


def violations(text: str, label: str, strips: bool) -> list[str]:
    if not strips:
        return []
    lines = text.splitlines()
    found: list[str] = []
    for index, line in enumerate(lines):
        if not GENERATOR.search(line):
            continue
        for back in range(index - 1, max(-1, index - 1 - LOOKBACK), -1):
            candidate = lines[back]
            if GENERATOR.search(candidate):
                continue
            if RELEASE_BUILD.search(candidate) and STRIP_OVERRIDE not in candidate:
                found.append(
                    f"{label}:{back + 1}: release build feeds a stripped library to "
                    f"{GENERATOR.search(line).group(0)} at line {index + 1}"
                )
                break
    return found


def check_paths(paths: list[Path], strips: bool) -> list[str]:
    problems: list[str] = []
    for path in paths:
        problems.extend(violations(io.open(path, encoding="utf-8").read(), str(path), strips))
    return problems


def self_check() -> int:
    good = """  svc:
    command: |
      bash -c "
        CARGO_PROFILE_RELEASE_STRIP=false cargo build --manifest-path core/Cargo.toml --release &&
        cargo run --manifest-path core/Cargo.toml --bin gen_kotlin --features gen-bindings
      "
"""
    bad = """  svc:
    command: |
      bash -c "
        cargo build --manifest-path core/Cargo.toml --release &&
        cargo run --manifest-path core/Cargo.toml --bin gen_kotlin --features gen-bindings
      "
"""
    dev_profile = """  svc:
    command: |
      bash -c "
        cargo build --manifest-path core/Cargo.toml --lib &&
        cargo run --manifest-path core/Cargo.toml --bin gen_kotlin --features gen-bindings
      "
"""
    cargo_stripping = "[profile.release]\nopt-level = 3\nstrip = true\n"
    cargo_plain = "[profile.release]\nopt-level = 3\n"

    failures = 0
    if not release_strips(cargo_stripping):
        print("[ERROR] self-check: strip = true must be detected as stripping")
        failures += 1
    if release_strips(cargo_plain):
        print("[ERROR] self-check: a profile without strip must not be treated as stripping")
        failures += 1
    cases = [
        ("override present", good, True, 0),
        ("release build without override", bad, True, 1),
        ("dev profile build", dev_profile, True, 0),
        ("release build, workspace does not strip", bad, False, 0),
    ]
    for label, text, strips, expected in cases:
        got = len(violations(text, label, strips))
        if got != expected:
            print(f"[ERROR] self-check {label!r}: expected {expected} violation(s), got {got}")
            failures += 1
    print(f"[{'OK' if failures == 0 else 'FAIL'}] self-check: {failures} failure(s)")
    return failures


def main(argv: list[str] | None = None) -> int:
    root = Path(__file__).resolve().parent.parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-check", action="store_true", help="verify the detector itself")
    parser.add_argument("--docker-dir", default=str(root / "docker"))
    parser.add_argument("--cargo-toml", default=str(root / "Cargo.toml"))
    args = parser.parse_args(argv)

    if args.self_check:
        return 1 if self_check() else 0

    strips = release_strips(io.open(args.cargo_toml, encoding="utf-8").read())
    directory = Path(args.docker_dir)
    paths = sorted(
        p for p in list(directory.glob("*.yml")) + list(directory.glob("*.yaml")) if p.is_file()
    )
    problems = check_paths(paths, strips)
    for problem in problems:
        print(f"[ERROR] {problem}")
    # Full list, never a truncated head; the count is the verdict.
    print(
        f"[{'OK' if not problems else 'FAIL'}] uniffi binding build: "
        f"{len(problems)} violation(s) across {len(paths)} compose file(s) "
        f"(release profile strips: {strips})"
    )
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
