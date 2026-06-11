import os
import json
import argparse
import re
from pathlib import Path

MAX_CONTEXT_FILES = 60
MAX_CONTEXT_TOKENS_ESTIMATE = 2000  # ~500 tokens per function entry, cap at ~2K


def load_jsonl_maybe_pretty(path):
    """Load a JSONL file that may contain pretty-printed JSON objects."""
    entries = []
    with open(path, 'r', encoding='utf-8-sig', errors='replace') as f:
        content = f.read()

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
            single_line_entries = None
            break

    if single_line_entries is not None:
        return single_line_entries

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

    if current:
        chunk = '\n'.join(current).strip().rstrip(',')
        if chunk:
            try:
                data = json.loads(chunk)
                entries.append(data)
            except json.JSONDecodeError:
                pass

    return entries


def normalize_path(p):
    """Normalize file path: backslashes to forward slashes, collapse redundant separators."""
    return p.replace("\\", "/").strip("/")


def parse_task_target(task_content):
    """Extract TARGET file path and function name from a task file."""
    target_file = None
    func_name = None

    # TARGET: core\src\notification.rs or TARGET: core/src/notification.rs
    m = re.search(r'TARGET:\s*(.+)', task_content)
    if m:
        target_file = normalize_path(m.group(1).strip())

    # Function name from "The function 'name'" or task filename pattern task_wire_<name>
    fm = re.search(r"function\s+['\"]?(\w+)['\"]?", task_content)
    if fm:
        func_name = fm.group(1)
    elif not func_name:
        # Fallback: extract from task filename (e.g., task_wire_unregister_endpoint.md)
        pass  # Caller should pass --function explicitly

    return target_file, func_name


def build_function_index(repo_entries):
    """Build a function-name -> [repo_entry] index for targeted lookup."""
    fn_index = {}
    for data in repo_entries:
        for fn in data.get("funcs", []):
            name = fn.get("name", "")
            if name:
                fn_index.setdefault(name, []).append(data)
    return fn_index


def extract_context(repo_root, task_file=None, files=None, output=None, func_name=None):
    handoff_audit_dir = Path(repo_root) / "HANDOFF_AUDIT"
    repo_map_path = handoff_audit_dir / "REPO_MAP.jsonl"
    index_path = handoff_audit_dir / "repo_map_index.json"

    # Build a set of all files known to the REPO_MAP index
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
    extracted_func_name = func_name

    if files:
        target_files.update([normalize_path(f.strip()) for f in files.split(",") if f.strip()])

    if task_file:
        task_path = Path(repo_root) / task_file
        if task_path.exists():
            try:
                with open(task_path, 'r', encoding='utf-8') as f:
                    content = f.read()

                # Parse TARGET line and function name from task content
                target_file_from_task, func_from_task = parse_task_target(content)
                if target_file_from_task:
                    target_files.add(target_file_from_task)
                if func_from_task and not extracted_func_name:
                    extracted_func_name = func_from_task

                # Pattern 1: Full file paths with extensions (forward or backslash)
                matches = re.findall(r'([a-zA-Z0-9_/\-\\]+\.(?:rs|kt|swift|py|js|ts|cpp|c|h))', content)
                for m in matches:
                    p = normalize_path(m)
                    if (Path(repo_root) / p).exists():
                        target_files.add(p)

                # Pattern 2: Directory references
                dir_matches = re.findall(r'([a-zA-Z0-9_/]+/[a-zA-Z0-9_]+/)', content)
                for d in dir_matches:
                    d_norm = d.replace("\\", "/")
                    for fpath in indexed_files:
                        if fpath.startswith(d_norm):
                            target_files.add(fpath)

                # Pattern 3: Short filenames
                short_matches = re.findall(r'\b([A-Za-z0-9_]+\.(?:rs|kt|swift|py|js|ts|cpp|c|h))\b', content)
                for s in short_matches:
                    if s in filename_to_paths:
                        target_files.update(filename_to_paths[s])

            except Exception:
                pass

    # Cap at MAX_CONTEXT_FILES
    if len(target_files) > MAX_CONTEXT_FILES:
        target_files = set(sorted(target_files)[:MAX_CONTEXT_FILES])

    if not target_files:
        return

    # Load REPO_MAP entries
    repo_entries = load_jsonl_maybe_pretty(repo_map_path)
    if not repo_entries:
        return

    # Build function index if we need per-function extraction
    fn_index = {}
    if extracted_func_name:
        fn_index = build_function_index(repo_entries)

    context_blocks = {}

    for data in repo_entries:
        f_path = data.get("file", "")
        if not f_path:
            continue
        f_path_norm = normalize_path(f_path)

        matched = False
        for target in target_files:
            target_norm = normalize_path(target)
            if f_path_norm == target_norm:
                matched = True
                break
            if f_path_norm.endswith("/" + target_norm) or target_norm.endswith("/" + f_path_norm):
                matched = True
                break
            if Path(f_path_norm).name == Path(target_norm).name:
                # Same basename — check if the target is a known full path
                full_paths = filename_to_paths.get(Path(f_path_norm).name, [])
                if target_norm in full_paths or any(fp == target_norm for fp in full_paths):
                    matched = True
                    break

        if not matched:
            continue

        if f_path_norm not in context_blocks:
            context_blocks[f_path_norm] = []
        context_blocks[f_path_norm].append(data)

    md_lines = []
    task_name = Path(task_file).stem if task_file else "Manual Extraction"
    md_lines.append(f"# REPO_MAP Context for Task: {task_name}\n")

    if extracted_func_name:
        md_lines.append(f"**Target function: `{extracted_func_name}`**\n")

    for f_path, chunks in context_blocks.items():
        chunks.sort(key=lambda x: x.get("chunk", 1))
        abs_p = Path(repo_root) / f_path
        total_lines = 0
        if abs_p.exists():
            try:
                with open(abs_p, 'r', encoding='utf-8', errors='ignore') as f:
                    total_lines = sum(1 for _ in f)
            except Exception:
                pass

        # If we have a target function, extract ONLY that function's context
        if extracted_func_name:
            target_func = None
            target_func_line = None
            for c in chunks:
                for fn in c.get("funcs", []):
                    if fn.get("name") == extracted_func_name:
                        target_func = fn
                        target_func_line = fn.get("line", 0)
                        break
                if target_func:
                    break

            if target_func:
                md_lines.append(f"## {f_path} — function `{extracted_func_name}` (line {target_func_line})")
                calls_to = ", ".join(target_func.get("calls_out_to", []))
                md_lines.append(f"- **Calls out to:** {calls_to if calls_to else 'none'}")
                # Include structs/classes from this chunk for context
                for c in chunks:
                    if c.get("structs_or_classes"):
                        md_lines.append(f"- **Structs in chunk:** {', '.join(c['structs_or_classes'][:5])}")
                        break
                md_lines.append("")
            else:
                # Function not found in REPO_MAP — output minimal file info
                md_lines.append(f"## {f_path} ({len(chunks)} chunks, {total_lines} lines)")
                md_lines.append(f"Function `{extracted_func_name}` not found in REPO_MAP chunks. Full file listing below.\n")
                _emit_full_chunks(md_lines, chunks)

        else:
            # No target function — emit full file context (legacy behavior)
            md_lines.append(f"## {f_path} ({len(chunks)} chunks, {total_lines} lines)")
            _emit_full_chunks(md_lines, chunks)

        md_lines.append("---\n")

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


def _emit_full_chunks(md_lines, chunks):
    """Emit full chunk listing (legacy behavior when no target function)."""
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


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--task-file", type=str)
    parser.add_argument("--files", type=str)
    parser.add_argument("--output", type=str)
    parser.add_argument("--function", type=str, help="Target function name for per-function extraction")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent.parent
    extract_context(repo_root, args.task_file, args.files, args.output, func_name=args.function)