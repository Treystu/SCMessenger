import os
import json
import argparse
import re
from pathlib import Path

MAX_CONTEXT_FILES = 60

def load_jsonl_maybe_pretty(path):
    """Load a JSONL file that may contain pretty-printed JSON objects.

    Handles both:
    - Standard JSONL: one JSON object per line
    - Pretty-printed: multi-line JSON objects separated by blank lines or }
    """
    entries = []
    with open(path, 'r', encoding='utf-8-sig', errors='replace') as f:
        content = f.read()

    # Try standard JSONL first (one object per line)
    lines = content.split('\n')
    single_line_entries = []
    for line in lines:
        line = line.strip()
        if not line:
            continue
        try:
            data = json.loads(line)
            single_line_entries.append(data)
        except json.JSONDecodeError:
            # Not valid single-line JSON — fall back to multi-line parsing
            single_line_entries = None
            break

    if single_line_entries is not None:
        return single_line_entries

    # Multi-line JSON: split on closing braces followed by commas or newlines
    # Use a brace-counting approach to split objects
    entries = []
    depth = 0
    current = []
    for line in content.split('\n'):
        current.append(line)
        depth += line.count('{') - line.count('}')
        if depth <= 0 and current:
            chunk = '\n'.join(current).strip().rstrip(',')
            if chunk:
                try:
                    data = json.loads(chunk)
                    entries.append(data)
                except json.JSONDecodeError:
                    pass
            current = []
            depth = 0

    # Handle any remaining content
    if current:
        chunk = '\n'.join(current).strip().rstrip(',')
        if chunk:
            try:
                data = json.loads(chunk)
                entries.append(data)
            except json.JSONDecodeError:
                pass

    return entries


def extract_context(repo_root, task_file=None, files=None, output=None):
    handoff_audit_dir = Path(repo_root) / "HANDOFF_AUDIT"
    repo_map_path = handoff_audit_dir / "REPO_MAP.jsonl"
    index_path = handoff_audit_dir / "repo_map_index.json"

    # Build a set of all files known to the REPO_MAP index for directory resolution
    indexed_files = set()
    if index_path.exists():
        try:
            with open(index_path, 'r', encoding='utf-8') as f:
                index = json.load(f)
            indexed_files = set(index.get("files", {}).keys())
        except Exception:
            pass

    # Build a filename -> full-path map for short name resolution
    filename_to_paths = {}
    for fpath in indexed_files:
        basename = Path(fpath).name
        filename_to_paths.setdefault(basename, []).append(fpath)

    target_files = set()
    if files:
        target_files.update([f.strip() for f in files.split(",") if f.strip()])

    if task_file:
        task_path = Path(repo_root) / task_file
        if task_path.exists():
            try:
                with open(task_path, 'r', encoding='utf-8') as f:
                    content = f.read()

                # Pattern 1: Full file paths with extensions (e.g. core/src/routing/mod.rs)
                matches = re.findall(r'([a-zA-Z0-9_/\-\\]+\.(?:rs|kt|swift|py|js|ts|cpp|c|h))', content)
                for m in matches:
                    p = m.replace("\\", "/")
                    if (Path(repo_root) / p).exists():
                        target_files.add(p)

                # Pattern 2: Directory references like "core/src/routing/" or "core/src/store/"
                # Also handle "core/src/routing/ —" (with em-dash after dir)
                dir_matches = re.findall(r'([a-zA-Z0-9_/]+/[a-zA-Z0-9_]+/)', content)
                for d in dir_matches:
                    d_norm = d.replace("\\", "/")
                    # Resolve directory to all indexed files under it
                    for fpath in indexed_files:
                        if fpath.startswith(d_norm):
                            target_files.add(fpath)

                # Pattern 3: Short filenames like "MeshForegroundService.kt" or "signatures.rs"
                short_matches = re.findall(r'\b([A-Za-z0-9_]+\.(?:rs|kt|swift|py|js|ts|cpp|c|h))\b', content)
                for s in short_matches:
                    if s in filename_to_paths:
                        target_files.update(filename_to_paths[s])
            except Exception:
                pass

    # Cap at MAX_CONTEXT_FILES to prevent context explosion from broad directory matches
    if len(target_files) > MAX_CONTEXT_FILES:
        target_files = set(sorted(target_files)[:MAX_CONTEXT_FILES])

    if not target_files:
        return

    # Load REPO_MAP entries (handles both JSONL and pretty-printed JSON)
    repo_entries = load_jsonl_maybe_pretty(repo_map_path)
    if not repo_entries:
        return

    context_blocks = {}

    for data in repo_entries:
        f_path = data.get("file", "")
        if not f_path:
            continue
        # Normalize: some entries use short names like "abstraction.rs"
        # Try to match against both the raw path and the indexed full paths
        matched = False
        for target in target_files:
            if f_path == target or f_path.endswith("/" + target) or target.endswith("/" + f_path):
                matched = True
                break
            # Also match basename
            if Path(f_path).name == Path(target).name and f_path == target:
                matched = True
                break
        if not matched:
            # Direct set membership check
            if f_path not in target_files:
                # Try basename matching for short-name entries
                basename = Path(f_path).name
                full_paths_for_basename = filename_to_paths.get(basename, [])
                if not any(fp in target_files for fp in full_paths_for_basename):
                    continue
                # If basename matches, use the full path from index
                f_path = [fp for fp in full_paths_for_basename if fp in target_files][0] if any(fp in target_files for fp in full_paths_for_basename) else f_path
        if f_path not in context_blocks:
            context_blocks[f_path] = []
        context_blocks[f_path].append(data)

    md_lines = []
    task_name = Path(task_file).stem if task_file else "Manual Extraction"
    md_lines.append(f"# REPO_MAP Context for Task: {task_name}\n")

    for f_path, chunks in context_blocks.items():
        chunks.sort(key=lambda x: x.get("chunk", 1))
        abs_p = Path(repo_root) / f_path
        total_lines = sum(1 for _ in open(abs_p, 'r', encoding='utf-8', errors='ignore')) if abs_p.exists() else 0
        md_lines.append(f"## {f_path} ({len(chunks)} chunks, {total_lines} lines)")

        summaries = []
        structs = set()
        funcs = []
        imports = set()

        for c in chunks:
            if c.get("summary"):
                summaries.append(c.get("summary"))
            if c.get("structs_or_classes"):
                for s in c.get("structs_or_classes"):
                    structs.add(s)
            if c.get("imports"):
                for i in c.get("imports"):
                    imports.add(i)
            if c.get("funcs"):
                funcs.extend(c.get("funcs"))

        md_lines.append("### Summary")
        md_lines.append(" ".join(summaries) + "\n")

        if structs:
            md_lines.append("### Structs/Classes")
            for s in sorted(structs):
                md_lines.append(f"- {s}")
            md_lines.append("")

        if funcs:
            md_lines.append("### Functions")
            md_lines.append("| Function | Line | Calls Out To |")
            md_lines.append("|----------|------|-------------|")
            for fn in funcs:
                name = fn.get("name", "")
                line_no = fn.get("line", "")
                calls = ", ".join(fn.get("calls_out_to", []))
                md_lines.append(f"| `{name}` | {line_no} | {calls} |")
            md_lines.append("")

        if imports:
            md_lines.append("### Imports")
            for i in sorted(imports):
                md_lines.append(f"- `{i}`")
        md_lines.append("\n---\n")

    md_content = "\n".join(md_lines)

    if output:
        with open(output, 'w', encoding='utf-8') as f:
            f.write(md_content)
    elif task_file:
        cache_dir = handoff_audit_dir / ".context_cache"
        cache_dir.mkdir(exist_ok=True)
        cache_file = cache_dir / f"{Path(task_file).stem}.md"
        with open(cache_file, 'w', encoding='utf-8') as f:
            f.write(md_content)

        task_path = Path(repo_root) / task_file
        if task_path.exists():
            try:
                with open(task_path, 'r', encoding='utf-8') as f:
                    existing_content = f.read()
                if "# REPO_MAP Context" not in existing_content:
                    with open(task_path, 'a', encoding='utf-8') as f:
                        f.write("\n\n" + md_content)
            except Exception:
                pass

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--task-file", type=str)
    parser.add_argument("--files", type=str)
    parser.add_argument("--output", type=str)
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent.parent
    extract_context(repo_root, args.task_file, args.files, args.output)