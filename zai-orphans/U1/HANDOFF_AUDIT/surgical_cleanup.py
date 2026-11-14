import os
import json
import shutil
import re

BASE_DIR = r"c:\Users\kanal\Documents\Github\SCMessenger\HANDOFF_AUDIT"
MASTER_MAP = os.path.join(BASE_DIR, "REPO_MAP.jsonl")
OUTPUT_DIR = os.path.join(BASE_DIR, "output")
DONE_DIR = os.path.join(BASE_DIR, "done")
TODO_DIR = os.path.join(BASE_DIR, "todo")

def is_lazy(data):
    # Check for placeholder summaries
    summary = data.get("summary", "")
    if "Detailed summary" in summary or summary == "REPLACE_WITH_SUMMARY":
        return True
    
    # Check for placeholder class lists
    classes = data.get("structs_or_classes", [])
    if any(x in ["List", "of", "classes"] for x in classes):
        return True
    
    # Check for placeholder function calls or line 0
    funcs = data.get("funcs", [])
    for f in funcs:
        if f.get("line") == 0:
            return True
        calls = f.get("calls_out_to", [])
        if any(x in ["funcs", "it", "calls"] for x in calls):
            return True
        if f.get("name") in ["function_name", "REPLACE_WITH_NAME"]:
            return True
            
    return False

def reset_task(filename, chunk):
    if not filename or not chunk:
        return
        
    print(f"  -> Resetting task: {filename} chunk {chunk}")
    # 1. Delete output file
    out_name = f"{filename}_chunk{chunk}.jsonl"
    out_path = os.path.join(OUTPUT_DIR, out_name)
    if os.path.exists(out_path):
        os.remove(out_path)
    
    # 2. Move done ticket to todo
    ticket_name = f"{filename}_chunk{chunk}.txt"
    done_path = os.path.join(DONE_DIR, ticket_name)
    todo_path = os.path.join(TODO_DIR, ticket_name)
    if os.path.exists(done_path):
        shutil.move(done_path, todo_path)
    elif not os.path.exists(todo_path):
        with open(todo_path, "w", encoding="utf-8") as tf:
            tf.write(f"FILE: (unknown)\nCHUNK: {chunk}")

def cleanup():
    if not os.path.exists(MASTER_MAP):
        print("No REPO_MAP.jsonl found.")
        return

    good_entries = []
    purged_count = 0

    with open(MASTER_MAP, "r", encoding="utf-8") as f:
        content = f.read()
        
    blocks = re.findall(r'\{[^{}]*(?:\{[^{}]*\}[^{}]*)*\}', content, re.DOTALL)
    print(f"Found {len(blocks)} blocks in REPO_MAP.")

    for block in blocks:
        block_clean = block.strip()
        if not block_clean:
            continue
            
        try:
            data = json.loads(block_clean)
            if is_lazy(data):
                purged_count += 1
                filename = data.get("file")
                chunk = data.get("chunk")
                reset_task(filename, chunk)
            else:
                good_entries.append(block_clean)
        except Exception as e:
            print(f"Error parsing block (Purging corrupt data): {e}")
            file_match = re.search(r'"file":\s*"([^"]+)"', block_clean)
            chunk_match = re.search(r'"chunk":\s*(\d+)', block_clean)
            if file_match and chunk_match:
                print(f"  -> Identified corrupt entry: {file_match.group(1)} chunk {chunk_match.group(1)}")
                reset_task(file_match.group(1), chunk_match.group(1))
            purged_count += 1

    # Rewrite MASTER_MAP
    with open(MASTER_MAP, "w", encoding="utf-8") as f:
        for entry in good_entries:
            f.write(entry + "\n")

    print(f"Cleanup complete. Purged {purged_count} lazy/corrupt entries.")

if __name__ == "__main__":
    cleanup()
