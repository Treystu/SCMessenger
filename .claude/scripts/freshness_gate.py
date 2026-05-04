import os
import json
import sys
import argparse
from pathlib import Path
from datetime import datetime

def check_freshness(repo_root, files):
    index_path = Path(repo_root) / "HANDOFF_AUDIT" / "repo_map_index.json"
    result = {
        "verdict": "FRESH",
        "fresh_files": [],
        "stale_files": [],
        "missing_files": [],
        "context_payload_path": ""
    }
    
    if not index_path.exists():
        result["verdict"] = "STALE"
        result["missing_files"] = [str(f) for f in files]
        return result
        
    try:
        with open(index_path, 'r', encoding='utf-8') as f:
            index = json.load(f)
    except json.JSONDecodeError:
        result["verdict"] = "STALE"
        result["missing_files"] = [str(f) for f in files]
        return result

    for file_path in files:
        abs_path = Path(repo_root) / file_path
        if not abs_path.exists():
            continue
            
        try:
            rel_path = str(abs_path.relative_to(repo_root)).replace("\\", "/")
        except ValueError:
            continue

        file_modified_at = datetime.utcfromtimestamp(abs_path.stat().st_mtime)

        if rel_path not in index.get("files", {}):
            result["missing_files"].append(rel_path)
            result["verdict"] = "STALE"
        else:
            file_meta = index["files"][rel_path]
            try:
                indexed_at_str = file_meta["indexed_at"].replace("Z", "+00:00")
                indexed_at = datetime.fromisoformat(indexed_at_str).replace(tzinfo=None)
            except Exception:
                indexed_at = datetime.min
                
            delta = (file_modified_at - indexed_at).total_seconds()
            
            if delta > 1.0: # 1 second buffer
                result["stale_files"].append({
                    "path": rel_path,
                    "indexed_at": file_meta["indexed_at"],
                    "modified_at": file_modified_at.isoformat() + "Z",
                    "delta_hours": delta / 3600.0
                })
                result["verdict"] = "STALE"
            else:
                result["fresh_files"].append(rel_path)

    return result

def get_agent_domains(repo_root, task_file):
    files = set()
    task_path = Path(repo_root) / task_file
    if not task_path.exists():
        return list(files)
        
    try:
        with open(task_path, 'r', encoding='utf-8') as f:
            content = f.read()
            import re
            matches = re.findall(r'([a-zA-Z0-9_/\-\\]+\.(?:rs|kt|swift|py|js|ts|cpp|c|h))', content)
            for m in matches:
                p = m.replace("\\", "/")
                if (Path(repo_root) / p).exists():
                    files.add(p)
    except Exception:
        pass
        
    return list(files)

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--files", type=str)
    parser.add_argument("--task-file", type=str)
    parser.add_argument("--agent-domains", type=str)
    args = parser.parse_args()
    
    repo_root = Path(__file__).resolve().parent.parent.parent
    
    files = []
    if args.files:
        files.extend([f.strip() for f in args.files.split(",") if f.strip()])
    
    if args.task_file:
        files.extend(get_agent_domains(repo_root, args.task_file))
        
    result = check_freshness(repo_root, set(files))
    print(json.dumps(result, indent=2))
    
    if result["verdict"] == "STALE":
        sys.exit(2)
    sys.exit(0)
