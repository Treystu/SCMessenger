#!/usr/bin/env python3
"""Lightweight structural cartographer — extracts code architecture from source files
without requiring an LLM. Generates REPO_MAP.jsonl entries with function signatures,
struct/class names, and imports. Much faster than LLM-based cartography for keeping
the REPO_MAP fresh between LLM reindexes.

Usage:
    python .claude/scripts/structural_cartographer.py --files core/src/routing/mod.rs,core/src/routing/engine.rs
    python .claude/scripts/structural_cartographer.py --from-index
    python .claude/scripts/structural_cartographer.py --from-index --stale-only
"""

import os
import json
import argparse
import re
import sys
from pathlib import Path
from datetime import datetime, timezone


def extract_rust_structure(filepath, repo_root):
    """Extract structural information from a Rust source file."""
    try:
        with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
            content = f.read()
    except Exception:
        return None

    lines = content.split('\n')
    total_lines = len(lines)

    # Extract imports
    imports = []
    for line in lines:
        stripped = line.strip()
        if stripped.startswith('use ') and stripped.endswith(';'):
            imports.append(stripped.rstrip(';'))

    # Extract structs, enums, traits
    structs_or_classes = []
    struct_pattern = re.compile(
        r'^\s*(pub\s+)?(struct|enum|trait|type|impl)\s+(\w+)',
        re.MULTILINE
    )
    for match in struct_pattern.finditer(content):
        structs_or_classes.append(match.group(3))

    # Extract functions with line numbers
    funcs = []
    fn_pattern = re.compile(
        r'^\s*(pub\s+)?(?:async\s+)?(?:const\s+)?fn\s+(\w+)\s*(?:<[^>]*>)?\s*\(',
        re.MULTILINE
    )
    for match in fn_pattern.finditer(content):
        line_no = content[:match.start()].count('\n') + 1
        name = match.group(2)
        # Try to find what this function calls (simplified — just direct calls in body)
        # Look ahead ~50 lines for call patterns
        end_line = min(line_no + 50, total_lines)
        body = '\n'.join(lines[line_no:end_line])
        calls = re.findall(r'(?:self\.|super::|crate::|[\w]+::)([\w]+)\s*\(', body)
        calls = list(set(calls))[:10]  # Cap at 10
        funcs.append({
            "name": name,
            "line": line_no,
            "calls_out_to": calls
        })

    # Build summary
    rel_path = str(Path(filepath).relative_to(repo_root)).replace("\\", "/")
    summary_parts = []
    if structs_or_classes:
        summary_parts.append(f"Defines {len(structs_or_classes)} types: {', '.join(structs_or_classes[:5])}")
    if funcs:
        summary_parts.append(f"{len(funcs)} functions")
    if imports:
        summary_parts.append(f"{len(imports)} imports")
    summary = f"{rel_path}: {'; '.join(summary_parts)}" if summary_parts else f"{rel_path}: structural extraction"

    return {
        "file": rel_path,
        "chunk": 1,
        "summary": summary,
        "structs_or_classes": structs_or_classes,
        "imports": imports[:30],  # Cap imports
        "funcs": funcs
    }


def extract_kotlin_structure(filepath, repo_root):
    """Extract structural information from a Kotlin source file."""
    try:
        with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
            content = f.read()
    except Exception:
        return None

    lines = content.split('\n')
    total_lines = len(lines)

    # Extract imports
    imports = []
    for line in lines:
        stripped = line.strip()
        if stripped.startswith('import '):
            imports.append(stripped)

    # Extract classes, interfaces, objects
    structs_or_classes = []
    class_pattern = re.compile(
        r'^\s*(?:public\s+|private\s+|internal\s+|protected\s+)?(?:data\s+|sealed\s+|abstract\s+|open\s+|companion\s+)?(?:class|interface|object|enum\s+class)\s+(\w+)',
        re.MULTILINE
    )
    for match in class_pattern.finditer(content):
        structs_or_classes.append(match.group(1))

    # Extract functions
    funcs = []
    fn_pattern = re.compile(
        r'^\s*(?:public\s+|private\s+|internal\s+|protected\s+|override\s+)?(?:suspend\s+)?(?:inline\s+)?(?:fun\s+)(?:<[^>]*>\s*)?(\w+)\s*\(',
        re.MULTILINE
    )
    for match in fn_pattern.finditer(content):
        line_no = content[:match.start()].count('\n') + 1
        name = match.group(1)
        # Look for calls in body
        end_line = min(line_no + 30, total_lines)
        body = '\n'.join(lines[line_no:end_line])
        calls = re.findall(r'(\w+)\s*\(', body)
        # Filter to only likely function calls (not keywords)
        calls = [c for c in set(calls) if c not in ('if', 'when', 'for', 'while', 'return', 'throw', 'else', 'val', 'var')][:10]
        funcs.append({
            "name": name,
            "line": line_no,
            "calls_out_to": calls
        })

    rel_path = str(Path(filepath).relative_to(repo_root)).replace("\\", "/")
    summary_parts = []
    if structs_or_classes:
        summary_parts.append(f"Defines {len(structs_or_classes)} types: {', '.join(structs_or_classes[:5])}")
    if funcs:
        summary_parts.append(f"{len(funcs)} functions")
    if imports:
        summary_parts.append(f"{len(imports)} imports")
    summary = f"{rel_path}: {'; '.join(summary_parts)}" if summary_parts else f"{rel_path}: structural extraction"

    return {
        "file": rel_path,
        "chunk": 1,
        "summary": summary,
        "structs_or_classes": structs_or_classes,
        "imports": imports[:30],
        "funcs": funcs
    }


def extract_swift_structure(filepath, repo_root):
    """Extract structural information from a Swift source file."""
    try:
        with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
            content = f.read()
    except Exception:
        return None

    lines = content.split('\n')

    imports = []
    for line in lines:
        stripped = line.strip()
        if stripped.startswith('import '):
            imports.append(stripped)

    structs_or_classes = []
    class_pattern = re.compile(
        r'^\s*(?:public\s+|private\s+|internal\s+|open\s+)?(?:class|struct|protocol|enum|actor)\s+(\w+)',
        re.MULTILINE
    )
    for match in class_pattern.finditer(content):
        structs_or_classes.append(match.group(1))

    funcs = []
    fn_pattern = re.compile(
        r'^\s*(?:public\s+|private\s+|internal\s+|override\s+|static\s+)?(?:func\s+)(\w+)\s*\(',
        re.MULTILINE
    )
    for match in fn_pattern.finditer(content):
        line_no = content[:match.start()].count('\n') + 1
        funcs.append({"name": match.group(1), "line": line_no, "calls_out_to": []})

    rel_path = str(Path(filepath).relative_to(repo_root)).replace("\\", "/")
    summary = f"{rel_path}: {len(structs_or_classes)} types, {len(funcs)} functions"

    return {
        "file": rel_path,
        "chunk": 1,
        "summary": summary,
        "structs_or_classes": structs_or_classes,
        "imports": imports[:30],
        "funcs": funcs
    }


def extract_structure(filepath, repo_root):
    """Auto-detect file type and extract structure."""
    ext = Path(filepath).suffix
    if ext == '.rs':
        return extract_rust_structure(filepath, repo_root)
    elif ext == '.kt':
        return extract_kotlin_structure(filepath, repo_root)
    elif ext == '.swift':
        return extract_swift_structure(filepath, repo_root)
    elif ext in ('.py', '.js', '.ts'):
        # Minimal extraction for other types
        try:
            with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
                content = f.read()
        except Exception:
            return None
        rel_path = str(Path(filepath).relative_to(repo_root)).replace("\\", "/")
        lines = content.split('\n')
        return {
            "file": rel_path,
            "chunk": 1,
            "summary": f"{rel_path}: {len(lines)} lines",
            "structs_or_classes": [],
            "imports": [],
            "funcs": []
        }
    return None


def main():
    parser = argparse.ArgumentParser(description='Structural cartographer for REPO_MAP')
    parser.add_argument('--files', type=str, help='Comma-separated list of files to index')
    parser.add_argument('--from-index', action='store_true', help='Index all files in repo_map_index.json')
    parser.add_argument('--stale-only', action='store_true', help='Only re-index stale files (requires --from-index)')
    parser.add_argument('--output', type=str, default='HANDOFF_AUDIT/REPO_MAP.jsonl',
                        help='Output path (default: HANDOFF_AUDIT/REPO_MAP.jsonl)')
    parser.add_argument('--update-index', action='store_true', help='Also update repo_map_index.json timestamps')
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent.parent
    output_path = Path(args.output) if not Path(args.output).is_absolute() else Path(args.output)

    files_to_index = []

    if args.from_index:
        index_path = repo_root / 'HANDOFF_AUDIT' / 'repo_map_index.json'
        if not index_path.exists():
            print(f"ERROR: Index not found at {index_path}")
            sys.exit(1)

        with open(index_path, 'r', encoding='utf-8') as f:
            index = json.load(f)

        for fpath, meta in index.get('files', {}).items():
            abs_path = repo_root / fpath
            if not abs_path.exists():
                continue

            if args.stale_only:
                # Check if stale
                from datetime import datetime
                indexed_at = meta.get('indexed_at', '')
                if indexed_at:
                    try:
                        indexed_time = datetime.fromisoformat(indexed_at.replace('Z', '+00:00'))
                        file_mtime = datetime.fromtimestamp(abs_path.stat().st_mtime, tz=indexed_time.tzinfo)
                        if file_mtime <= indexed_time:
                            continue  # Not stale
                    except Exception:
                        pass  # Re-index if we can't parse timestamps

            files_to_index.append(str(abs_path))

    elif args.files:
        for f in args.files.split(','):
            f = f.strip()
            abs_path = str(repo_root / f) if not Path(f).is_absolute() else f
            if Path(abs_path).exists():
                files_to_index.append(abs_path)
    else:
        print("ERROR: Specify --files or --from-index")
        sys.exit(1)

    if not files_to_index:
        print("No files to index.")
        return

    print(f"Indexing {len(files_to_index)} files...")

    entries = []
    for fpath in files_to_index:
        entry = extract_structure(fpath, repo_root)
        if entry:
            entries.append(entry)

    if not entries:
        print("No entries generated.")
        return

    # Write as proper JSONL (one JSON object per line)
    with open(output_path, 'w', encoding='utf-8') as f:
        for entry in entries:
            f.write(json.dumps(entry, ensure_ascii=False) + '\n')

    print(f"Generated {len(entries)} entries -> {output_path}")

    # Update index timestamps if requested
    if args.update_index:
        index_path = repo_root / 'HANDOFF_AUDIT' / 'repo_map_index.json'
        with open(index_path, 'r', encoding='utf-8') as f:
            index = json.load(f)

        from datetime import datetime as dt_module
        now = dt_module.now(timezone.utc).isoformat()
        for entry in entries:
            fpath = entry['file']
            if fpath in index.get('files', {}):
                index['files'][fpath]['indexed_at'] = now
            else:
                index['files'][fpath] = {
                    "indexed_at": now,
                    "chunks": 1,
                    "lines": 0
                }

        with open(index_path, 'w', encoding='utf-8') as f:
            json.dump(index, f, indent=2)
        print(f"Updated index timestamps for {len(entries)} files")


if __name__ == '__main__':
    main()