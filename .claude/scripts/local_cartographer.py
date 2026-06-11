#!/usr/bin/env python3
"""Local model REPO_MAP cartographer — uses ollama to generate architecture entries
for source files. Chunks files at 700 lines and processes them through the local model.

Usage:
    python .claude/scripts/local_cartographer.py --files core/src/routing/adaptive_ttl.rs,core/src/routing/engine.rs
    python .claude/scripts/local_cartographer.py --from-index --stale-only
    python .claude/scripts/local_cartographer.py --from-index

The script:
1. Reads each file in 700-line chunks
2. Sends each chunk to the local ollama model for architecture extraction
3. Validates the JSON output (rejects placeholders)
4. Appends valid entries to HANDOFF_AUDIT/REPO_MAP.jsonl
5. Updates repo_map_index.json timestamps
"""

import json
import argparse
import subprocess
import sys
import time
import re
import urllib.request
import urllib.error
from pathlib import Path
from datetime import datetime, timezone

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
DEFAULT_MODEL = "qwen2.5-coder:7b"
CHUNK_SIZE = 700
OLLAMA_URL = "http://127.0.0.1:11434/api/generate"
MAX_RETRIES = 2

CARTOGRAPHER_PROMPT = """You are an expert code cartographer. Analyze this source code chunk and extract its architecture.

Output RAW JSON ONLY. No markdown. No backticks. No explanation. Just the JSON object.

Schema:
{{
  "file": "RELATIVE_PATH_FROM_REPO_ROOT",
  "chunk": CHUNK_NUMBER,
  "summary": "One sentence summary of this chunk's purpose",
  "structs_or_classes": ["list of struct/class/enum/trait names defined in this chunk"],
  "imports": ["list of import/use statements"],
  "funcs": [
    {{
      "name": "function_name",
      "line": LINE_NUMBER,
      "calls_out_to": ["list of other functions this function calls"]
    }}
  ]
}}

CRITICAL RULES:
- Use EXACT line numbers from the code (the number before the colon)
- Do NOT use placeholder values like "REPLACE_WITH_NAME" or "function_name"
- Do NOT include markdown code fences
- The "file" field must be the relative path: {file_path}
- The "chunk" field must be: {chunk_number}

CODE (Lines {start_line} - {end_line}):
{code}"""


def validate_entry(data, file_path, chunk_num):
    """Validate a cartographer entry. Returns True if valid."""
    if not isinstance(data, dict):
        return False
    if data.get("file", "") in ("", "REPLACE_WITH_PATH"):
        data["file"] = file_path
    if not data.get("summary") or data["summary"] in ("REPLACE_WITH_SUMMARY", "Detailed summary"):
        return False
    funcs = data.get("funcs", [])
    if funcs:
        for fn in funcs:
            name = fn.get("name", "")
            if name in ("REPLACE_WITH_NAME", "function_name", ""):
                return False
            if fn.get("line") == 0 or fn.get("line") is None:
                return False
    if "List" in data.get("structs_or_classes", []):
        return False
    if "List" in data.get("imports", []):
        return False
    return True


def call_ollama(prompt, model=DEFAULT_MODEL, timeout=300):
    """Call the local ollama model and return the response."""
    body = json.dumps({
        "model": model,
        "prompt": prompt,
        "stream": False,
        "options": {
            "num_ctx": 8192,
            "temperature": 0.1,
            "num_thread": 4
        }
    })

    req = urllib.request.Request(
        OLLAMA_URL,
        data=body.encode(),
        headers={"Content-Type": "application/json"}
    )

    try:
        resp = urllib.request.urlopen(req, timeout=timeout)
        result = json.loads(resp.read().decode())
        return result.get("response", "")
    except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError) as e:
        print(f"  Ollama error: {e}")
        return ""


def parse_json_response(response_text):
    """Parse JSON from the model response, handling markdown fences and other artifacts."""
    text = response_text.strip()
    # Strip markdown code fences
    if text.startswith("```json"):
        text = text[7:]
    elif text.startswith("```"):
        text = text[3:]
    if text.endswith("```"):
        text = text[:-3]
    text = text.strip()

    # Find JSON object boundaries
    start = text.find("{")
    end = text.rfind("}")
    if start >= 0 and end > start:
        text = text[start:end + 1]

    try:
        return json.loads(text)
    except json.JSONDecodeError:
        return None


def process_file(file_path, model=DEFAULT_MODEL):
    """Process a single file through the cartographer, returning entries."""
    rel_path = str(Path(file_path).relative_to(REPO_ROOT)).replace("\\", "/")

    try:
        with open(file_path, "r", encoding="utf-8", errors="replace") as f:
            lines = f.readlines()
    except Exception as e:
        print(f"  ERROR reading {rel_path}: {e}")
        return []

    total_lines = len(lines)
    if total_lines == 0:
        return []

    total_chunks = (total_lines + CHUNK_SIZE - 1) // CHUNK_SIZE
    entries = []

    for chunk_idx in range(total_chunks):
        chunk_num = chunk_idx + 1
        start = chunk_idx * CHUNK_SIZE
        end = min(start + CHUNK_SIZE, total_lines)
        chunk_lines = lines[start:end]

        # Format with line numbers
        numbered = "\n".join(f"{start + i + 1}: {line.rstrip()}" for i, line in enumerate(chunk_lines))

        prompt = CARTOGRAPHER_PROMPT.format(
            file_path=rel_path,
            chunk_number=chunk_num,
            start_line=start + 1,
            end_line=end,
            code=numbered
        )

        retry_error = None
        for attempt in range(MAX_RETRIES + 1):
            current_prompt = prompt
            if retry_error:
                current_prompt = f"CRITICAL JSON ERROR PREVIOUSLY: {retry_error}\n\nFIX THIS AND RE-OUTPUT PERFECT JSON.\n\n" + prompt

            response = call_ollama(current_prompt, model=model)
            if not response:
                retry_error = "Empty response from model"
                continue

            data = parse_json_response(response)
            if data is None:
                retry_error = "Failed to parse JSON"
                continue

            data["file"] = rel_path
            data["chunk"] = chunk_num

            if validate_entry(data, rel_path, chunk_num):
                entries.append(data)
                break
            else:
                retry_error = f"Validation failed: summary={data.get('summary', '')[:50]}"
                continue

        if len(entries) < chunk_num:
            print(f"  CHUNK {chunk_num}/{total_chunks} FAILED after {MAX_RETRIES + 1} attempts")

    return entries


def main():
    parser = argparse.ArgumentParser(description="Local model REPO_MAP cartographer")
    parser.add_argument("--files", type=str, help="Comma-separated list of files to index")
    parser.add_argument("--from-index", action="store_true", help="Index all files in repo_map_index.json")
    parser.add_argument("--stale-only", action="store_true", help="Only re-index stale files (requires --from-index)")
    parser.add_argument("--model", type=str, default=DEFAULT_MODEL, help=f"Ollama model to use (default: {DEFAULT_MODEL})")
    parser.add_argument("--output", type=str, default="HANDOFF_AUDIT/REPO_MAP.jsonl",
                        help="Output JSONL path (default: HANDOFF_AUDIT/REPO_MAP.jsonl)")
    parser.add_argument("--update-index", action="store_true", help="Update repo_map_index.json timestamps")
    parser.add_argument("--limit", type=int, default=0, help="Max files to process (0 = no limit)")
    args = parser.parse_args()

    output_path = REPO_ROOT / args.output

    files_to_process = []

    if args.from_index:
        index_path = REPO_ROOT / "HANDOFF_AUDIT" / "repo_map_index.json"
        if not index_path.exists():
            print(f"ERROR: Index not found at {index_path}")
            sys.exit(1)

        with open(index_path, "r", encoding="utf-8") as f:
            index = json.load(f)

        for fpath, meta in index.get("files", {}).items():
            abs_path = REPO_ROOT / fpath
            if not abs_path.exists():
                continue

            if args.stale_only:
                indexed_at = meta.get("indexed_at", "")
                if indexed_at:
                    try:
                        indexed_time = datetime.fromisoformat(indexed_at.replace("Z", "+00:00"))
                        file_mtime = datetime.fromtimestamp(abs_path.stat().st_mtime, tz=indexed_time.tzinfo)
                        if file_mtime <= indexed_time:
                            continue
                    except Exception:
                        pass

            files_to_process.append(str(abs_path))

    elif args.files:
        for f in args.files.split(","):
            f = f.strip()
            abs_path = str(REPO_ROOT / f) if not Path(f).is_absolute() else f
            if Path(abs_path).exists():
                files_to_process.append(abs_path)
    else:
        print("ERROR: Specify --files or --from-index")
        sys.exit(1)

    if args.limit > 0:
        files_to_process = files_to_process[:args.limit]

    if not files_to_process:
        print("No files to process.")
        return

    print(f"Processing {len(files_to_process)} files with {args.model}...")

    # Check ollama is reachable
    try:
        req = urllib.request.Request("http://127.0.0.1:11434/api/tags")
        resp = urllib.request.urlopen(req, timeout=5)
        models = json.loads(resp.read().decode()).get("models", [])
        model_names = [m.get("name", "") for m in models]
        if not any(args.model in m for m in model_names):
            print(f"WARNING: Model {args.model} not found in available models: {model_names}")
            print("Attempting to pull...")
            pull_req = urllib.request.Request(
                "http://127.0.0.1:11434/api/pull",
                data=json.dumps({"name": args.model, "stream": False}).encode(),
                headers={"Content-Type": "application/json"}
            )
            urllib.request.urlopen(pull_req, timeout=300)
    except Exception as e:
        print(f"WARNING: Could not verify ollama availability: {e}")

    all_entries = []
    total = len(files_to_process)
    for i, fpath in enumerate(files_to_process):
        rel = str(Path(fpath).relative_to(REPO_ROOT)).replace("\\", "/")
        print(f"[{i+1}/{total}] {rel}...")
        entries = process_file(fpath, model=args.model)
        all_entries.extend(entries)
        print(f"  -> {len(entries)} chunks extracted")

    if not all_entries:
        print("No entries generated.")
        return

    # Write as proper JSONL (one JSON object per line)
    with open(output_path, "w", encoding="utf-8") as f:
        for entry in all_entries:
            f.write(json.dumps(entry, ensure_ascii=False) + "\n")

    print(f"\nGenerated {len(all_entries)} entries -> {output_path}")

    # Update index timestamps if requested
    if args.update_index:
        index_path = REPO_ROOT / "HANDOFF_AUDIT" / "repo_map_index.json"
        with open(index_path, "r", encoding="utf-8") as f:
            index = json.load(f)

        now = datetime.now(timezone.utc).isoformat()
        for entry in all_entries:
            fpath = entry["file"]
            if fpath in index.get("files", {}):
                index["files"][fpath]["indexed_at"] = now
            else:
                index["files"][fpath] = {
                    "indexed_at": now,
                    "chunks": entry.get("chunk", 1),
                    "lines": 0
                }

        with open(index_path, "w", encoding="utf-8") as f:
            json.dump(index, f, indent=2)
        print(f"Updated index timestamps for {len(all_entries)} entries")


if __name__ == "__main__":
    main()