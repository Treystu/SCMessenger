#!/usr/bin/env python3
"""Fail if a binary target that carries unit tests is excluded from test builds.

#678: `cli/Cargo.toml` set `test = false` on the `scmessenger-cli` bin target, and
that target holds a real unit-test battery (dial scheduling, identity-envelope
contact learning, the `scm test` self-test guard). Nothing in CI selected it, so
the battery never compiled, let alone ran, and a regression in it could not fail
a build. This gate makes the exclusion visible: a `[[bin]]` with `test = false`
whose source contains a `#[cfg(test)]` module is a failure.

Run: python3 scripts/check_bin_test_targets.py
"""

import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SKIP_DIRS = {".git", "target", "tmp", "node_modules"}

BIN_HEADER = re.compile(r"^\[\[bin\]\]\s*$", re.MULTILINE)
FIELD = re.compile(r'^(name|path|test)\s*=\s*"?([^"\n]+)"?\s*$', re.MULTILINE)


def manifest_paths():
    for base, dirs, files in os.walk(ROOT):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        if "Cargo.toml" in files:
            yield os.path.join(base, "Cargo.toml")


def bin_blocks(text):
    """Yield the field map of every [[bin]] section in a manifest."""
    headers = [m.start() for m in BIN_HEADER.finditer(text)]
    for index, start in enumerate(headers):
        stop = headers[index + 1] if index + 1 < len(headers) else len(text)
        block = text[start:stop]
        # The section ends at the next table header that follows its own line.
        body_start = block.find("\n") + 1
        next_header = re.search(r"^\[", block[body_start:], re.MULTILINE)
        if next_header:
            block = block[: body_start + next_header.start()]
        fields = {}
        for key, value in FIELD.findall(block):
            fields[key] = value.strip()
        yield fields


def source_has_tests(source):
    try:
        with open(source, encoding="utf-8", errors="replace") as handle:
            return "#[cfg(test)]" in handle.read()
    except OSError:
        return False


def violations():
    bad = []
    for manifest in manifest_paths():
        with open(manifest, encoding="utf-8", errors="replace") as handle:
            text = handle.read()
        for fields in bin_blocks(text):
            if fields.get("test", "true").strip() != "false":
                continue
            name = fields.get("name", "?")
            rel = fields.get("path", "src/main.rs")
            source = os.path.normpath(os.path.join(os.path.dirname(manifest), rel))
            if source_has_tests(source):
                bad.append(
                    (
                        os.path.relpath(manifest, ROOT),
                        name,
                        os.path.relpath(source, ROOT),
                    )
                )
    return bad


def main():
    bad = violations()
    if bad:
        for manifest, name, source in bad:
            print(f"[FAIL] {manifest}: bin `{name}` sets test = false but {source} has tests")
        print(f"[FAIL] {len(bad)} bin target(s) hide their unit tests from every test build")
        return 1
    print("[OK] no binary target with unit tests is excluded from test builds")
    return 0


if __name__ == "__main__":
    sys.exit(main())
