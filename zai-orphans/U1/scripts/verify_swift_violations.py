#!/usr/bin/env python3
"""
Lightweight, compiler-free verification for specific SwiftLint rule fixes.
Used as a --verify command for scripts/delegate_task.py so its built-in
retry loop (--max-rounds) has real signal instead of a single unchecked
attempt. No Swift toolchain available on this host, so this checks textual
properties only -- it is a necessary-not-sufficient signal, not a full lint.

Usage:
  python verify_swift_violations.py explicit_type_interface <file> <line> [<line> ...]
  python verify_swift_violations.py line_length <file> [max_len]
"""
import sys


def check_explicit_type_interface(filepath, line_numbers):
    with open(filepath, encoding="utf-8") as f:
        lines = f.read().replace("\r\n", "\n").split("\n")
    remaining = []
    for ln in line_numbers:
        if ln - 1 >= len(lines):
            continue
        content = lines[ln - 1]
        if "=" in content and ":" not in content.split("=")[0]:
            remaining.append(ln)
    if remaining:
        print(f"STILL VIOLATING explicit_type_interface at original line numbers: {remaining}")
        print("(Note: if you inserted/removed lines above these, the line numbers may have")
        print("shifted -- re-check by searching for the property names, not just these numbers.)")
        return 1
    print(f"[OK] all {len(line_numbers)} listed explicit_type_interface lines now have a type annotation")
    return 0


def check_line_length(filepath, max_len=120):
    with open(filepath, encoding="utf-8") as f:
        lines = f.read().replace("\r\n", "\n").split("\n")
    bad = [(i + 1, len(l)) for i, l in enumerate(lines) if len(l) > max_len]
    if bad:
        print(f"STILL TOO LONG ({len(bad)} lines exceed {max_len} chars): {bad[:15]}")
        return 1
    print(f"[OK] no lines exceed {max_len} chars")
    return 0


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    mode = sys.argv[1]
    filepath = sys.argv[2]
    if mode == "explicit_type_interface":
        line_numbers = [int(x) for x in sys.argv[3:]]
        return check_explicit_type_interface(filepath, line_numbers)
    elif mode == "line_length":
        max_len = int(sys.argv[3]) if len(sys.argv) > 3 else 120
        return check_line_length(filepath, max_len)
    else:
        print(f"Unknown mode: {mode}")
        return 2


if __name__ == "__main__":
    sys.exit(main())
