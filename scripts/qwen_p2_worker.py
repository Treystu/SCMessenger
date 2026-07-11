import urllib.request, json, os, re, sys

key = "sk-ws-H.YXMYHR.DVqB.MEUCIQD5eHzEkD6tQ0zdAx38ou06RCQNpAHhn9w11hc8yEBVGgIgeSUBtEFtyKEYnYBKzKS98i4D5lgjTjsQ6bfAWom6Zd4"
base = "https://ws-2vzz894jwsk3t27r.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1/chat/completions"

if not os.path.exists("HANDOFF/todo/PQC_05_HYBRID_KEM_MODULE.md"):
    print("Error: PQC-05 task file not found.")
    sys.exit(1)

with open("HANDOFF/todo/PQC_05_HYBRID_KEM_MODULE.md", "r", encoding="utf-8") as f:
    task_content = f.read()

prompt = f"""
You are a senior Rust systems engineer and cryptographer. Your task is to implement PQC-05: Hybrid X25519+ML-KEM-768 KEM module.

Here is the exact task requirement:
{task_content}

Please provide the FULL, completely updated/new contents of the following files using standard Markdown code blocks with the exact filename as the first line inside the code block (e.g. `// core/src/crypto/pq/hybrid.rs` or `// core/src/crypto/pq/mod.rs`). DO NOT output partial files or snippets. Output the ENTIRE file content.

1. `core/src/crypto/pq/hybrid.rs`:
   - Implement the hybrid KEM module exactly as specified.
   - Use `x25519-dalek` and `libcrux-ml-kem`.
   - Ensure the rejection semantics and exact combiner layout are implemented and documented.
   - Ensure all 5 required tests are present.
   - For `subtle` usage, use `x25519_dalek::StaticSecret` and `SharedSecret::was_contributory()` if available, otherwise check if the shared secret bytes are all zero.
   
2. `core/src/crypto/pq/mod.rs`:
   - Add `pub mod hybrid;`

Make sure to zeroize `ss_x25519`, `ss_mlkem`, and `ikm`. 
For `x25519_dalek::StaticSecret`, use `rand_core::OsRng` to generate ephemeral secrets.
"""

files_to_read = [
    "core/src/crypto/pq/mod.rs",
    "core/src/crypto/mod.rs",
    "core/src/crypto/ratchet.rs"
]

prompt += "\n\n### Current Relevant File Contents:\n"
for f in files_to_read:
    try:
        with open(f, "r", encoding="utf-8") as file:
            prompt += f"\n--- {f} ---\n```\n{file.read()}\n```\n"
    except Exception as e:
        print(f"Warning: Could not read {f}: {e}")

data = {
    "model": "qwen-max",
    "messages": [
        {"role": "system", "content": "You are a senior Rust/Kotlin engineer. strictly provide full file contents in code blocks starting with // filename."},
        {"role": "user", "content": prompt}
    ],
    "temperature": 0.1
}

req = urllib.request.Request(base, headers={
    "Authorization": "Bearer " + key,
    "Content-Type": "application/json"
}, data=json.dumps(data).encode("utf-8"))

print("Sending request to Qwen-Max API...")
try:
    with urllib.request.urlopen(req) as r:
        resp = json.loads(r.read().decode("utf-8"))
        content = resp["choices"][0]["message"]["content"]
        
        os.makedirs("tmp", exist_ok=True)
        with open("tmp/qwen_p2_05_response.md", "w", encoding="utf-8") as f:
            f.write(content)
            
        print("Response received and saved!")
        
        blocks = re.findall(r"```(?:rust|)\n(.*?)\n```", content, re.DOTALL)
        for block in blocks:
            lines = block.split("\n")
            first_line = lines[0].strip()
            if first_line.startswith("// ") or first_line.startswith("# "):
                filename = first_line.replace("// ", "").replace("# ", "").strip()
                print(f"Writing updated {filename}...")
                os.makedirs(os.path.dirname(filename), exist_ok=True)
                with open(filename, "w", encoding="utf-8") as f:
                    f.write(block)
                        
except Exception as e:
    print("ERROR:", e)
