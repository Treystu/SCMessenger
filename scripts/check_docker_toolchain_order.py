#!/usr/bin/env python3
"""Fail a Dockerfile that runs `rustup` before the repository's toolchain pin is visible.

rustup honours `rust-toolchain.toml` from the directory containing it. A
Dockerfile that does `FROM rust:latest` and then `rustup target add ...` before
copying the repository installs the target into the image's DEFAULT toolchain;
once the pin is later copied in, rustup switches to the pinned channel and the
targets/components are missing from it. That is the defect that kept the Docker
Integration Suite red until the android image was fixed (#447), and it was still
latent in `Dockerfile.rust-test` and `Dockerfile.wasm-test` (#449).

The rule: every `RUN rustup ...` instruction must be preceded by a COPY whose
source names `rust-toolchain.toml` (or copies the whole build context, `COPY . .`),
in the same stage.

Usage:
  python scripts/check_docker_toolchain_order.py            # check docker/
  python scripts/check_docker_toolchain_order.py --self-check
"""

from __future__ import annotations

import argparse
import io
import re
import sys
from pathlib import Path

# Only the rustup COMMAND matters. `curl ... sh.rustup.rs | sh -s -- -y` is the
# installer: it sets a default toolchain and copies nothing from the repo, so it
# is not this rule's subject (the android image installs that way and then adds
# targets after the COPY, which is the fixed shape from #447).
RUSTUP = re.compile(r"^\s*RUN\s+rustup\b", re.IGNORECASE)
COPY = re.compile(r"^\s*(?:COPY|ADD)\s+(.+)$", re.IGNORECASE)
FROM = re.compile(r"^\s*FROM\s+", re.IGNORECASE)


def violations(text: str, label: str) -> list[str]:
    """Return one message per rustup instruction with no visible pin before it."""
    found: list[str] = []
    pin_visible = False
    for number, line in enumerate(text.splitlines(), start=1):
        if FROM.match(line):
            # A new stage starts with an empty filesystem: no pin yet.
            pin_visible = False
            continue
        copy = COPY.match(line)
        if copy:
            sources = copy.group(1).split()
            if any("rust-toolchain.toml" in source or source == "." or source == "./" for source in sources):
                pin_visible = True
            continue
        if RUSTUP.match(line) and not pin_visible:
            found.append(f"{label}:{number}: rustup runs before rust-toolchain.toml is copied in")
    return found


def check_paths(paths: list[Path]) -> list[str]:
    problems: list[str] = []
    for path in paths:
        problems.extend(violations(io.open(path, encoding="utf-8").read(), str(path)))
    return problems


def self_check() -> int:
    good = """FROM rust:latest
COPY rust-toolchain.toml ./
RUN rustup target add aarch64-linux-android
COPY . .
CMD ["cargo", "test"]
"""
    good_context = """FROM rust:latest
COPY . .
RUN rustup target add aarch64-linux-android
"""
    bad = """FROM rust:latest
RUN rustup component add clippy
COPY . .
"""
    bad_two_stages = """FROM rust:latest
COPY rust-toolchain.toml ./
RUN rustup target add x
FROM rust:latest AS build
RUN rustup target add y
"""
    cases = [
        ("pin copied first", good, 0),
        ("whole context copied first", good_context, 0),
        ("rustup before any COPY", bad, 1),
        ("new stage resets the pin", bad_two_stages, 1),
    ]
    failures = 0
    for label, text, expected in cases:
        got = len(violations(text, label))
        if got != expected:
            print(f"[ERROR] self-check {label!r}: expected {expected} violation(s), got {got}")
            failures += 1
    print(f"[{'OK' if failures == 0 else 'FAIL'}] self-check: {failures} failure(s)")
    return failures


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-check", action="store_true", help="verify the detector itself")
    parser.add_argument(
        "--docker-dir",
        default=str(Path(__file__).resolve().parent.parent / "docker"),
        help="directory holding the Dockerfiles to check",
    )
    args = parser.parse_args(argv)

    if args.self_check:
        return 1 if self_check() else 0

    directory = Path(args.docker_dir)
    paths = sorted(p for p in directory.glob("Dockerfile*") if p.is_file())
    if not paths:
        print(f"[ERROR] no Dockerfiles found under {directory}")
        return 2

    problems = check_paths(paths)
    for problem in problems:
        print(f"[ERROR] {problem}")
    # Print the full set, never a truncated head: the count is the verdict.
    print(f"[{'OK' if not problems else 'FAIL'}] docker toolchain order: "
          f"{len(problems)} violation(s) across {len(paths)} file(s)")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
