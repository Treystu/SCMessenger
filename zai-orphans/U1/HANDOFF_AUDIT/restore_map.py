import os

BASE_DIR = r"c:\Users\kanal\Documents\Github\SCMessenger\HANDOFF_AUDIT"
MASTER_MAP = os.path.join(BASE_DIR, "REPO_MAP.jsonl")
OUTPUT_DIR = os.path.join(BASE_DIR, "output")

def restore():
    jsonl_files = [f for f in os.listdir(OUTPUT_DIR) if f.endswith(".jsonl")]
    print(f"Found {len(jsonl_files)} valid output files. Rebuilding REPO_MAP.jsonl...")
    
    with open(MASTER_MAP, "w", encoding="utf-8") as master:
        for filename in jsonl_files:
            path = os.path.join(OUTPUT_DIR, filename)
            try:
                with open(path, "r", encoding="utf-8") as f:
                    content = f.read().strip()
                    if content:
                        master.write(content + "\n")
            except Exception as e:
                print(f"Error reading {filename}: {e}")

    print("Restoration complete.")

if __name__ == "__main__":
    restore()
