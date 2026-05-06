# Quickstart: SCMessenger CLI Expert (Ollama)

This environment allows you to drive the SCMessenger CLI using a local Ollama model. The model is primed to be an expert in identity management, peer connectivity, and messaging.

## 1. Prerequisites
- **Ollama**: Installed and running on `localhost:11434`.
- **Python 3**: With `requests` installed (`pip install requests`).
- **Rust/Cargo**: To build the SCMessenger CLI.

## 2. Build the CLI
Ensure the CLI binary is compiled:
```powershell
cargo build -p scmessenger-cli
```

## 3. Create the Expert Model
Use the provided Modelfile to create the `scm-expert` persona:
```powershell
ollama create scm-expert -f ollama_cfg/CLI_Expert.Modelfile
```

## 4. Run the Bridge
Start the bridge script to interact with your expert:
```powershell
python scripts/ollama_cli_bridge.py
```

## 5. Example Interactions
Once the bridge is running, you can ask things like:
- "Initialize my identity."
- "What is my Peer ID?"
- "Start the daemon and check status."
- "Send 'Hello World' to 12D3Koo..." (requires a valid Peer ID).

## Files in this Setup
- **Prompt**: [.claude/prompts/ollama_expert.md](.claude/prompts/ollama_expert.md) (The system instructions).
- **Modelfile**: [ollama_cfg/CLI_Expert.Modelfile](ollama_cfg/CLI_Expert.Modelfile) (Ollama configuration).
- **Bridge**: [scripts/ollama_cli_bridge.py](scripts/ollama_cli_bridge.py) (The glue between Ollama and the CLI).
- **Driver**: [scripts/core_cli_driver.py](scripts/core_cli_driver.py) (The low-level CLI wrapper).
