#!/usr/bin/env python3
"""Inventory the rand 0.8 API shapes still in the tree, for `#545`.

`#545` upgrades the workspace from `rand` 0.8 to 0.10 and its acceptance says
"zero rand-0.8 API uses remain". That sentence is checkable without a Rust
toolchain: the 0.8-era API shapes are lexical, so a scan can list them, and a
second scan after the migration can prove the list is empty.

Default mode is a report and always exits 0, so this can be added to CI before
the migration lands. `--enforce` exits 1 while any *production* hit remains, which
is the mode to wire in when the upgrade is ready.

What counts as a remnant, and what does not:

- `rand::thread_rng`        -> `rand::rng()` in 0.10
- `OsRng`                   -> `rand::rngs::SysRng` (feature `sys_rng`, fallible)
                               or an `x25519-dalek` `random()`/`From<[u8;32]>` path
- `RngCore`                 -> `rand::Rng` (infallible) or `rand::TryRng`
- `.gen_range(`             -> `.random_range(` through `RngExt`
- `.gen(`                   -> `.random()` through `RngExt`
- `use rand::Rng;`          -> `use rand::RngExt;` (the method surface moved)
- `from_entropy(`           -> removed; seed through a constructor
- `fill_bytes(`             -> NOT a remnant: still provided by `Rng`/`Fill`.
  Listed in the summary only so the count is visible, never as a defect.

Test-versus-production is a heuristic, not a fact: a hit counts as test code when
it appears after the last `mod tests` / `mod test` declaration in its file. Files
that declare tests in a separate module or behind `#[cfg(test)]` helpers can be
misfiled by this, so the per-hit list prints every hit with its classification
and the reader can correct it. Nothing is truncated (AGENTS.md rule 15).
"""

import argparse
import re
import sys
from pathlib import Path

ROOTS = ("core", "cli", "wasm", "desktop_bridge")
SKIP_DIRS = {"target", "node_modules", "tmp", "build", "dist", ".git"}

PATTERNS = (
    ("thread_rng", re.compile(r"rand::thread_rng\b"), "rand::rng()"),
    (
        "OsRng",
        re.compile(r"\bOsRng\b"),
        "rand::rngs::SysRng (feature sys_rng, fallible) or x25519-dalek random()",
    ),
    ("RngCore", re.compile(r"\bRngCore\b"), "rand::Rng / rand::TryRng"),
    ("gen_range", re.compile(r"\.gen_range\s*\("), ".random_range( via RngExt"),
    ("gen_call", re.compile(r"\.gen\s*(::<[^>]*>)?\s*\("), ".random( via RngExt"),
    ("rand_Rng_import", re.compile(r"use\s+rand::Rng\s*;"), "use rand::RngExt;"),
    ("from_entropy", re.compile(r"from_entropy\s*\("), "removed in 0.10"),
)

INFORMATIONAL = (("fill_bytes", re.compile(r"fill_bytes\s*\(")),)
TEST_DECL = re.compile(r"^\s*(pub\s+)?mod\s+(tests?)\b")


def rust_files(roots):
    for root in roots:
        base = Path(root)
        if not base.is_dir():
            continue
        for path in sorted(base.rglob("*.rs")):
            if any(part in SKIP_DIRS for part in path.parts):
                continue
            yield path


def classify(path, lines, lineno):
    """Test code by path, else production unless after the last `mod tests`."""
    if "tests" in path.parts or "test" in path.name or "proptest" in path.name:
        return "test"
    test_start = 0
    for index, line in enumerate(lines, start=1):
        if TEST_DECL.match(line):
            test_start = index
    return "test" if test_start and lineno > test_start else "prod"


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--enforce",
        action="store_true",
        help="exit 1 while any production rand-0.8 API shape remains",
    )
    parser.add_argument("--roots", nargs="*", default=list(ROOTS))
    args = parser.parse_args(argv)
    roots = tuple(args.roots)

    hits = []
    informational = []
    files_with_hits = set()

    files = list(rust_files(roots))
    for path in files:
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError as exc:
            print(f"[WARNING] cannot read {path}: {exc}")
            continue
        lines = text.splitlines()
        for lineno, line in enumerate(lines, start=1):
            for name, pattern, guidance in PATTERNS:
                if pattern.search(line):
                    hits.append(
                        (str(path), lineno, classify(path, lines, lineno), name,
                         guidance, line.strip())
                    )
                    files_with_hits.add(str(path))
            for name, pattern in INFORMATIONAL:
                if pattern.search(line):
                    informational.append((str(path), lineno, name))

    print(f"scanned {len(files)} Rust files under {', '.join(roots)}")
    print("")
    if hits:
        print("REMNANTS (rand 0.8 API shapes):")
        for path, lineno, kind, name, guidance, snippet in hits:
            print(f"  {path}:{lineno} [{kind}] {name}: {snippet}")
            print(f"      -> {guidance}")
    else:
        print("REMNANTS (rand 0.8 API shapes): none")

    prod = [hit for hit in hits if hit[2] == "prod"]
    test = [hit for hit in hits if hit[2] == "test"]
    print("")
    print(f"total remnant hits: {len(hits)} in {len(files_with_hits)} files")
    print(f"  production: {len(prod)}")
    print(f"  test:       {len(test)}")
    counts = {}
    for _, _, _, name, _, _ in hits:
        counts[name] = counts.get(name, 0) + 1
    for name, _, _ in PATTERNS:
        if name in counts:
            print(f"    {name}: {counts[name]}")
    prod_files = sorted({hit[0] for hit in prod})
    print(f"  production files ({len(prod_files)}): {', '.join(prod_files) or 'none'}")
    print("")
    print(f"fill_bytes occurrences (informational, not a remnant): {len(informational)}")

    if args.enforce:
        if prod:
            print(f"[FAIL] {len(prod)} production rand-0.8 API shape(s) remain")
            return 1
        print("[OK] no production rand-0.8 API shape remains")
        return 0
    print("[INFO] report mode: exit 0 by design. Use --enforce to gate the migration.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
