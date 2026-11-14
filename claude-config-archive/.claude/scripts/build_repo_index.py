import os
import json
import argparse
from pathlib import Path
from datetime import datetime

def now_iso():
    return datetime.utcnow().isoformat() + "Z"

def count_lines(file_path):
    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            return sum(1 for _ in f)
    except Exception:
        return 0

def build_index(repo_root, full_rebuild=True, files=None):
    handoff_audit_dir = Path(repo_root) / "HANDOFF_AUDIT"
    done_dir = handoff_audit_dir / "done"
    output_dir = handoff_audit_dir / "output"
    index_path = handoff_audit_dir / "repo_map_index.json"
    
    if full_rebuild or not index_path.exists():
        index = {"version": "1.0", "generated_at": now_iso(), "files": {}}
    else:
        try:
            with open(index_path, 'r', encoding='utf-8') as f:
                index = json.load(f)
        except json.JSONDecodeError:
            index = {"version": "1.0", "generated_at": now_iso(), "files": {}}

    tickets = list(done_dir.glob("*.txt"))
    target_files = set()
    if not full_rebuild and files:
        for f in files:
            p = Path(repo_root) / f
            try:
                target_files.add(str(p.resolve()))
            except Exception:
                pass
    
    files_updated = set()
    for ticket in tickets:
        try:
            with open(ticket, 'r', encoding='utf-8-sig') as f:
                content = f.read().splitlines()
                
            file_path = None
            chunk_num = 1
            for line in content:
                if line.startswith("FILE:"):
                    file_path = line.split("FILE:", 1)[1].strip()
                elif line.startswith("CHUNK:"):
                    try:
                        chunk_num = int(line.split("CHUNK:", 1)[1].strip())
                    except ValueError:
                        pass
            
            if not file_path:
                continue
                
            abs_path = Path(file_path)
            try:
                rel_path = str(abs_path.resolve().relative_to(Path(repo_root).resolve())).replace("\\", "/")
            except ValueError:
                continue
                
            if not full_rebuild and files and str(abs_path.resolve()) not in target_files:
                continue

            output_file = output_dir / f"{ticket.stem}.jsonl"
            if not output_file.exists():
                continue
                
            indexed_at = datetime.utcfromtimestamp(output_file.stat().st_mtime).isoformat() + "Z"
            
            if abs_path.exists():
                file_modified_at = datetime.utcfromtimestamp(abs_path.stat().st_mtime).isoformat() + "Z"
                total_lines = count_lines(abs_path)
            else:
                file_modified_at = indexed_at
                total_lines = 0
            
            if rel_path not in index["files"] or rel_path not in files_updated:
                index["files"][rel_path] = {
                    "indexed_at": indexed_at,
                    "file_modified_at": file_modified_at,
                    "chunks": [],
                    "total_lines": total_lines,
                    "status": "complete"
                }
                files_updated.add(rel_path)
                
            if chunk_num not in index["files"][rel_path]["chunks"]:
                index["files"][rel_path]["chunks"].append(chunk_num)
                index["files"][rel_path]["chunks"].sort()
                
            if indexed_at > index["files"][rel_path]["indexed_at"]:
                index["files"][rel_path]["indexed_at"] = indexed_at
                
        except Exception as e:
            pass

    index["generated_at"] = now_iso()
    with open(index_path, 'w', encoding='utf-8') as f:
        json.dump(index, f, indent=2)

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--full-rebuild", action="store_true")
    parser.add_argument("--incremental", action="store_true")
    parser.add_argument("--files", type=str, help="Comma separated list of files")
    args = parser.parse_args()
    
    repo_root = Path(__file__).resolve().parent.parent.parent
    
    if args.incremental and args.files:
        files = [f.strip() for f in args.files.split(",") if f.strip()]
        build_index(repo_root, full_rebuild=False, files=files)
    else:
        build_index(repo_root, full_rebuild=True)
