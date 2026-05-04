import os
import json
import argparse
from pathlib import Path

def extract_context(repo_root, task_file=None, files=None, output=None):
    handoff_audit_dir = Path(repo_root) / "HANDOFF_AUDIT"
    repo_map_path = handoff_audit_dir / "REPO_MAP.jsonl"
    
    target_files = set()
    if files:
        target_files.update([f.strip() for f in files.split(",") if f.strip()])
        
    if task_file:
        task_path = Path(repo_root) / task_file
        if task_path.exists():
            import re
            try:
                with open(task_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                matches = re.findall(r'([a-zA-Z0-9_/\-\\]+\.(?:rs|kt|swift|py|js|ts|cpp|c|h))', content)
                for m in matches:
                    p = m.replace("\\", "/")
                    if (Path(repo_root) / p).exists():
                        target_files.add(p)
            except Exception:
                pass

    if not target_files:
        return
        
    context_blocks = {}
    
    if repo_map_path.exists():
        with open(repo_map_path, 'r', encoding='utf-8') as f:
            for line in f:
                if not line.strip(): continue
                try:
                    data = json.loads(line)
                    f_path = data.get("file", "")
                    if f_path:
                        rel_path = str(Path(f_path).relative_to(repo_root)).replace("\\", "/")
                        if rel_path in target_files:
                            if rel_path not in context_blocks:
                                context_blocks[rel_path] = []
                            context_blocks[rel_path].append(data)
                except Exception:
                    continue
                    
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
        task_path = Path(repo_root) / task_file
        cache_dir = handoff_audit_dir / ".context_cache"
        cache_dir.mkdir(exist_ok=True)
        cache_file = cache_dir / f"{Path(task_file).stem}.md"
        with open(cache_file, 'w', encoding='utf-8') as f:
            f.write(md_content)
            
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
