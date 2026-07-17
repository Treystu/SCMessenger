#!/usr/bin/env python3
import argparse
import os
import subprocess
import urllib.request
import json
import sys

QWEN_URL = "https://ws-2vzz894jwsk3t27r.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1/chat/completions"

def _key_from_env_file(path, names):
    try:
        with open(os.path.expanduser(path), "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if "=" in line and not line.startswith("#"):
                    k, v = line.split("=", 1)
                    if k.strip() in names and v.strip():
                        return v.strip().strip('"').strip("'")
    except OSError:
        pass
    return None

def get_api_key():
    return (os.environ.get("QWEN_API_KEY")
            or os.environ.get("DASHSCOPE_API_KEY")
            or _key_from_env_file("~/.config/scmorc/dashscope.env",
                                  ("QWEN_API_KEY", "DASHSCOPE_API_KEY")))

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--task", required=True)
    args = parser.parse_args()

    api_key = get_api_key()
    if not api_key:
        print("Error: API key is not set.", file=sys.stderr)
        sys.exit(1)

    with open(args.task, "r", encoding="utf-8") as f:
        task_content = f.read()

    # Extract task description up to Target Files
    task_desc = task_content.split("## Target Files")[0].strip()

    # Get list of source files
    try:
        result = subprocess.run(["git", "ls-files", "*.rs", "*.kt", "*.swift", "*.java", "*.toml", "*.gradle", "*.kts"], capture_output=True, text=True, check=True)
        files = result.stdout.strip().split("\n")
    except subprocess.CalledProcessError:
        print("Error running git ls-files", file=sys.stderr)
        sys.exit(1)

    files_str = "\n".join(files)

    prompt = f"""
You are an expert software architect routing tasks to files in a codebase.
I have a task description. I need you to identify EXACTLY which files from the provided file list must be edited to complete this task.

Task Description:
{task_desc}

File List:
{files_str}

Return ONLY a space-separated list of the file paths that need to be edited. Do not explain. Do not include markdown formatting. Just the file paths.
"""

    payload = {
        "model": "qwen-max",
        "messages": [
            {"role": "system", "content": "You are a file routing assistant. Output ONLY space-separated file paths."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.0,
    }

    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json",
    }

    req = urllib.request.Request(QWEN_URL, headers=headers, data=json.dumps(payload).encode("utf-8"))
    
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            resp = json.loads(r.read().decode("utf-8"))
            content = resp["choices"][0]["message"]["content"].strip()
            # Clean up potential markdown formatting just in case
            content = content.replace("```text", "").replace("```", "").strip()
            # Print to stdout so the shell script can capture it
            print(content)
    except Exception as e:
        print(f"Error querying Qwen: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
