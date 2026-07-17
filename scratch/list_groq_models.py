import urllib.request
import json
import os

# Read API key from groq.env
api_key = os.environ.get("GROQ_API_KEY")
if not api_key:
    try:
        with open(os.path.expanduser("~/.config/scmorc/groq.env"), "r") as f:
            for line in f:
                if line.strip().startswith("GROQ_API_KEY="):
                    api_key = line.split("=", 1)[1].strip().strip('"').strip("'")
    except Exception:
        pass

if not api_key:
    print("Error: No GROQ_API_KEY found")
    exit(1)

req = urllib.request.Request(
    "https://api.groq.com/openai/v1/models",
    headers={
        "Authorization": f"Bearer {api_key}",
        "User-Agent": "curl/8.5.0"
    }
)

try:
    with urllib.request.urlopen(req) as r:
        data = json.loads(r.read().decode())
        for model in data["data"]:
            print(model["id"])
except Exception as e:
    print(f"Error: {e}")
