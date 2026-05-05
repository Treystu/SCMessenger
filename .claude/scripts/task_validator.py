#!/usr/bin/env python3
"""Pre-dispatch task validation for SCMessenger orchestrate.

Checks task_wire_*.md files to detect false-positive wiring targets:
- GOLDEN_ string literal functions (code inside pub const GOLDEN_*: &str)
- #[test] / #[tokio::test] test functions
- #[kani::proof] formal verification harnesses
- proptest! macro functions
- Already-wired functions (grep for callers)

Exit codes: 0=VALID, 1=FALSE_POSITIVE, 2=ALREADY_WIRED, 3=NEEDS_REVIEW
"""

import re
import subprocess
import sys


def find_golden_regions(content):
    """Find all pub const GOLDEN_* raw string literal regions."""
    regions = []
    for m in re.finditer(r'pub\s+const\s+GOLDEN_\w+\s*:\s*[^=]*=\s*r#"', content):
        start = m.start()
        end = content.find('"#', m.end())
        if end != -1:
            regions.append((start, end + 2))
    return regions


def find_proptest_regions(content):
    """Find all proptest! {} macro regions."""
    regions = []
    for m in re.finditer(r'proptest!\s*\{', content):
        start = m.start()
        try:
            brace_pos = content.index('{', m.start())
        except ValueError:
            continue
        depth = 0
        for i in range(brace_pos, len(content)):
            if content[i] == '{':
                depth += 1
            elif content[i] == '}':
                depth -= 1
                if depth == 0:
                    regions.append((start, i + 1))
                    break
    return regions


def find_test_mod_regions(content):
    """Find all #[cfg(test)] mod blocks."""
    regions = []
    for m in re.finditer(r'#\[cfg\(test\)\]', content):
        mod_match = re.search(r'mod\s+\w+\s*\{', content[m.start():])
        if mod_match:
            block_start = m.start() + mod_match.start()
            try:
                brace_pos = content.index('{', block_start)
            except ValueError:
                continue
            depth = 0
            for i in range(brace_pos, len(content)):
                if content[i] == '{':
                    depth += 1
                elif content[i] == '}':
                    depth -= 1
                    if depth == 0:
                        regions.append((m.start(), i + 1))
                        break
    return regions


def validate_task(task_file, target_file, func_name):
    """Run all validation checks. Returns (verdict, detail)."""
    if not target_file or not func_name:
        return "VALID", ""

    try:
        with open(target_file, 'r', encoding='utf-8', errors='replace') as f:
            content = f.read()
    except FileNotFoundError:
        return "NEEDS_REVIEW", f"target_file_missing:{target_file}"

    lines = content.split('\n')

    golden_regions = find_golden_regions(content)
    proptest_regions = find_proptest_regions(content)
    test_mod_regions = find_test_mod_regions(content)

    # Find function definition
    func_pattern = re.compile(
        r'(?:pub\s+)?(?:async\s+)?fn\s+' + re.escape(func_name) + r'\s*[\(<]'
    )

    for fm in func_pattern.finditer(content):
        func_pos = fm.start()
        func_line = content[:func_pos].count('\n')

        # Check: Inside GOLDEN_ string literal?
        for gs, ge in golden_regions:
            if gs < func_pos < ge:
                return "FALSE_POSITIVE", "GOLDEN_STRING_LITERAL"

        # Check: Inside proptest! macro?
        for ps, pe in proptest_regions:
            if ps < func_pos < pe:
                return "FALSE_POSITIVE", "PROPBTEST_HARNESS"

        # Check: Inside #[cfg(test)] mod block?
        for ts, te in test_mod_regions:
            if ts < func_pos < te:
                return "FALSE_POSITIVE", "TEST_FUNCTION"

        # Check: #[test], #[tokio::test], #[kani::proof] annotation above?
        for j in range(max(0, func_line - 5), func_line):
            stripped = lines[j].strip() if j < len(lines) else ''
            if stripped in ('#[test]', '#[tokio::test]') or stripped.startswith('#[tokio::test('):
                return "FALSE_POSITIVE", "TEST_FUNCTION"
            if '#[kani::proof]' in stripped:
                return "FALSE_POSITIVE", "KANI_PROOF"

        # Only check first occurrence
        break

    # Check: Function name starts with proptest_ (heuristic)
    if func_name.startswith('proptest_'):
        return "FALSE_POSITIVE", "PROPBTEST_HARNESS"

    # Check: Function already has callers?
    try:
        result = subprocess.run(
            ['grep', '-r', '--include=*.rs', '--include=*.kt', '--include=*.swift',
             '--exclude-dir=target', '--exclude-dir=.claude', '--exclude-dir=build',
             '-c', func_name, '.'],
            capture_output=True, text=True, timeout=15
        )
        total_refs = 0
        for line in result.stdout.strip().split('\n'):
            if ':' in line:
                count_str = line.split(':')[-1].strip()
                try:
                    total_refs += int(count_str)
                except ValueError:
                    pass
        if total_refs > 4:
            return "ALREADY_WIRED", f"{total_refs}_references"
    except (subprocess.TimeoutExpired, Exception):
        pass

    return "VALID", ""


def main():
    if len(sys.argv) < 3:
        print("Usage: task_validator.py <task_file> <target_file> <func_name>")
        sys.exit(0)

    task_file = sys.argv[1]
    target_file = sys.argv[2]
    func_name = sys.argv[3]

    verdict, detail = validate_task(task_file, target_file, func_name)

    exit_codes = {
        "VALID": 0,
        "FALSE_POSITIVE": 1,
        "ALREADY_WIRED": 2,
        "NEEDS_REVIEW": 3,
    }

    if detail:
        print(f"{verdict}:{detail}")
    else:
        print(verdict)

    sys.exit(exit_codes.get(verdict, 0))


if __name__ == '__main__':
    main()