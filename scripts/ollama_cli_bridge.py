#!/usr/bin/env python3
"""
Ollama CLI Bridge — Allows a local Ollama model to drive the SCMessenger CLI.

This script:
1. Takes user input.
2. Sends it to the 'scm-expert' model in Ollama.
3. Extracts commands from the model's response.
4. Executes them via core_cli_driver.py.
5. Feeds results back to the model.
"""

import json
import subprocess
import sys
import urllib.request
import urllib.error
from pathlib import Path

# Config
OLLAMA_API = "http://localhost:11434/api/generate"
MODEL_NAME = "scm-expert"
REPO_ROOT = Path(__file__).resolve().parent.parent
DRIVER_PATH = REPO_ROOT / "scripts" / "core_cli_driver.py"

def call_ollama(prompt, context=None):
    payload = {
        "model": MODEL_NAME,
        "prompt": prompt,
        "stream": False,
    }
    if context:
        payload["context"] = context
    
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        OLLAMA_API, 
        data=data, 
        headers={"Content-Type": "application/json"}
    )
    
    try:
        with urllib.request.urlopen(req) as response:
            return json.loads(response.read().decode("utf-8"))
    except Exception as e:
        print(f"Error calling Ollama: {e}")
        return None

def execute_cli_command(command_line):
    """Executes a command produced by the model."""
    print(f"Executing: {command_line}")
    # Strip 'python ' prefix if model included it
    cmd = command_line.replace("python ", "").strip()
    
    # Split into arguments
    import shlex
    args = shlex.split(cmd)
    
    # Ensure it starts with the driver path
    if len(args) > 0 and "core_cli_driver.py" in args[0]:
        args[0] = str(DRIVER_PATH)
    else:
        # If it just outputted 'status' etc, prepend the driver
        args = [sys.executable, str(DRIVER_PATH)] + args

    try:
        # Use sys.executable to ensure we use the same python version
        result = subprocess.run([sys.executable] + args, capture_output=True, text=True)
        return result.stdout
    except Exception as e:
        return json.dumps({"status": "error", "reason": str(e)})

def main():
    print(f"--- SCMessenger Ollama Bridge ---")
    print(f"Model: {MODEL_NAME}")
    print(f"Type 'exit' or 'quit' to stop.\n")
    
    context = None
    
    while True:
        try:
            user_input = input("User > ")
        except (EOFError, KeyboardInterrupt):
            break
            
        if user_input.lower() in ["exit", "quit"]:
            break
            
        # 1. Send user request to Ollama
        print("Thinking...")
        resp = call_ollama(user_input, context)
        if not resp:
            continue
            
        model_output = resp.get("response", "")
        context = resp.get("context")
        
        # 2. Extract and execute commands
        lines = model_output.split("\n")
        command_results = []
        
        for line in lines:
            line = line.strip()
            # Detect command patterns
            if "core_cli_driver.py" in line or any(line.startswith(c) for c in ["start", "stop", "status", "identity", "send"]):
                res = execute_cli_command(line)
                command_results.append(res)
        
        # 3. If commands were run, feed results back to model for a final summary
        if command_results:
            feedback_prompt = "Command Results:\n" + "\n".join(command_results) + "\nSummarize the outcome for the user."
            final_resp = call_ollama(feedback_prompt, context)
            if final_resp:
                print(f"Expert > {final_resp.get('response', '')}")
        else:
            print(f"Expert > {model_output}")

if __name__ == "__main__":
    main()
