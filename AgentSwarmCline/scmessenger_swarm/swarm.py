import os
import sys
import json
import argparse
import subprocess
from datetime import datetime
from crewai import Agent, Task, Crew, Process, LLM
from crewai.tools import tool

# ═══════════════════════════════════════════════════════════════════════════════
# 1. CONFIGURE THE CLOUD LLMs VIA THE LOCAL OLLAMA PROXY
# ═══════════════════════════════════════════════════════════════════════════════
# Model assignments based on Ollama Pro tier availability (50x free tier)
# All models route through localhost:11434 with :cloud suffix

# --- Strategic / Managerial ---
# glm-5.1 (1.5T): Best reasoning model — crew manager + lead architect
architect_llm = LLM(
    model="ollama/glm-5.1:cloud",
    base_url="http://localhost:11434"
)

# --- Code Generation ---
# qwen3-coder:480b: Purpose-built for code generation — primary Rust writer
coder_llm = LLM(
    model="ollama/qwen3-coder:480b:cloud",
    base_url="http://localhost:11434"
)

# --- Code Review & QA ---
# deepseek-v3.2 (671B): Strong analytical reasoning — code review, correctness
reviewer_llm = LLM(
    model="ollama/deepseek-v3.2:cloud",
    base_url="http://localhost:11434"
)

# --- Security Auditing ---
# mistral-large-3:675b: Nuanced analysis, security pattern recognition
security_llm = LLM(
    model="ollama/mistral-large-3:675b:cloud",
    base_url="http://localhost:11434"
)

# --- Test Engineering ---
# devstral-2:123b: Mistral coding model — fast, test-oriented code generation
tester_llm = LLM(
    model="ollama/devstral-2:123b:cloud",
    base_url="http://localhost:11434"
)

# --- Documentation ---
# gemma4:31b: Fast, clean prose generation — rustdoc, changelogs, README
docwriter_llm = LLM(
    model="ollama/gemma4:31b:cloud",
    base_url="http://localhost:11434"
)

# --- Platform Integration ---
# gemma4:31b: Fast execution, binding regeneration, platform wiring
platform_engineer_llm = LLM(
    model="ollama/gemma4:31b:cloud",
    base_url="http://localhost:11434"
)

# --- Fast Execution ---
# gemma4:31b: Quick commands, lint fixes, build verification, minor edits
fast_executer_llm = LLM(
    model="ollama/gemma4:31b:cloud",
    base_url="http://localhost:11434"
)

# ═══════════════════════════════════════════════════════════════════════════════
# 2. DEFINE CUSTOM TOOLS
# ═══════════════════════════════════════════════════════════════════════════════

@tool("Write to File")
def write_to_file(filename: str, content: str) -> str:
    """Writes the provided content to a file. Useful for saving Rust code, tests, and docs."""
    with open(filename, "w", encoding="utf-8") as f:
        f.write(content)
    return f"Successfully wrote to {filename}"


@tool("Read File")
def read_file(filename: str) -> str:
    """Reads the contents of a file. Useful for reviewing existing Rust code before auditing."""
    if not os.path.isfile(filename):
        return f"Error: File '{filename}' not found."
    with open(filename, "r", encoding="utf-8") as f:
        return f.read()


@tool("List Files")
def list_files(directory: str = ".") -> str:
    """Lists files in a directory. Useful for discovering what code exists before review."""
    if not os.path.isdir(directory):
        return f"Error: Directory '{directory}' not found."
    files = os.listdir(directory)
    return "\n".join(files)

@tool("Execute Command")
def execute_command(command: str) -> str:
    """Executes a terminal command. STRICTLY limited to 'cargo' commands. Returns stdout/stderr."""
    # Allow cargo commands, rustc, and basic directory navigation before cargo commands
    valid_starts = ["cargo", "rustc", "cd "]
    if not any(command.strip().startswith(prefix) for prefix in valid_starts):
        return "Error: Security violation. You are only allowed to run 'cargo', 'rustc', or 'cd' commands."
    try:
        # 30-second timeout to prevent infinite hangs
        result = subprocess.run(command, shell=True, capture_output=True, text=True, timeout=30)
        output = f"Exit Code: {result.returncode}\n"
        if result.stdout: output += f"STDOUT:\n{result.stdout}\n"
        if result.stderr: output += f"STDERR:\n{result.stderr}\n"
        return output
    except subprocess.TimeoutExpired:
        return "Error: Command timed out after 30 seconds."
    except Exception as e:
        return f"Error executing command: {str(e)}"


# ═══════════════════════════════════════════════════════════════════════════════
# 3. DEFINE THE AGENTS
# ═══════════════════════════════════════════════════════════════════════════════

lead_architect = Agent(
    role="Lead Systems Architect",
    goal="Ensure the structural integrity and performance of the P2P messaging protocol. Delegate work to specialists and synthesize their results.",
    backstory=(
        "You are a brutally strict systems architect. You oversee the development team "
        "and ensure all Rust code meets production standards. You do not write code yourself; "
        "you delegate to specialists — programmers, reviewers, security auditors, test engineers, "
        "and technical writers. You synthesize their outputs and make final architectural decisions."
    ),
    llm=architect_llm,
    allow_delegation=True
)

rust_programmer = Agent(
    role="Rust Systems Programmer",
    goal="Write highly optimized, memory-safe Rust code based on architectural specs.",
    backstory=(
        "You are an expert Rust developer. You specialize in network observability, mesh connectivity, "
        "and P2P protocol implementation. You execute tasks quickly and exactly as instructed. "
        "You write clean, idiomatic Rust with proper error handling, derive macros, and documentation comments."
    ),
    llm=coder_llm,
    tools=[write_to_file, read_file, list_files],
    allow_delegation=False
)

code_reviewer = Agent(
    role="Senior Code Reviewer",
    goal="Review all generated Rust code for correctness, idiomatic patterns, memory safety, and adherence to project standards. Save your review report to a file.",
    backstory=(
        "You are a senior Rust code reviewer with 15 years of systems programming experience. "
        "You check for: ownership/borrowing errors, unnecessary allocations, missing error handling, "
        "non-idiomatic patterns, improper use of unsafe blocks, and dead code. You read files, "
        "analyze them line by line, and provide detailed feedback with specific line references and fixes. "
        "You MUST save your review report to a markdown file using your Write to File tool. You MUST use your 'Execute Command' tool to run 'cargo check' on the code. Do not guess if it compiles. If the terminal returns an error, include the raw compiler error in your review report."
    ),
    llm=reviewer_llm,
    tools=[write_to_file, read_file, list_files, execute_command],
    allow_delegation=False
)

security_auditor = Agent(
    role="Security Auditor",
    goal="Identify vulnerabilities, unsafe patterns, and potential attack vectors in generated Rust code. Save your audit report to a file.",
    backstory=(
        "You are a cybersecurity specialist focused on Rust and distributed systems. "
        "You hunt for: buffer overflows, integer overflow/underflow, cryptographic weaknesses, "
        "timing attacks, improper serialization vulnerabilities, DDoS vectors in P2P protocol handling, "
        "and any usage of 'unsafe' that isn't justified. You read code files and produce a security audit report. "
        "You MUST save your audit report to a markdown file using your Write to File tool."
    ),
    llm=security_llm,
    tools=[write_to_file, read_file, list_files],
    allow_delegation=False
)

test_engineer = Agent(
    role="Test Engineer",
    goal="Write comprehensive Rust unit tests and integration tests for all generated code.",
    backstory=(
        "You are a Rust test engineering specialist. You write thorough #[cfg(test)] modules, "
        "property-based tests where appropriate, and integration tests. You cover: happy paths, "
        "edge cases, error conditions, serialization round-trips, and boundary values. "
        "You save test files alongside the source code they test. You MUST use your 'Execute Command' tool to run 'cargo test' to verify your tests pass. If they fail, fix them before moving on."
    ),
    llm=tester_llm,
    tools=[write_to_file, read_file, list_files, execute_command],
    allow_delegation=False
)

technical_writer = Agent(
    role="Technical Writer",
    goal="Generate rustdoc comments, module-level documentation, and changelog entries for all produced code.",
    backstory=(
        "You are a technical writer specializing in Rust documentation. You produce: module-level doc comments (//!), "
        "item-level doc comments (///), examples in doc comments, README sections, and CHANGELOG entries. "
        "You read the generated code first, then write documentation files that are clear, accurate, and follow rustdoc conventions."
    ),
    llm=docwriter_llm,
    tools=[write_to_file, read_file, list_files],
    allow_delegation=False
)

platform_engineer = Agent(
    role="Platform Integration Engineer",
    goal="Update UniFFI interface definitions (api.udl) and regenerate platform bindings (Kotlin, Swift) after core API changes.",
    backstory=(
        "You are a platform integration specialist for SCMessenger. Your job is to update the UniFFI interface "
        "definition file (core/src/api.udl) when new Rust methods are added, then run the binding generators "
        "(cargo run --bin gen_kotlin and cargo run --bin gen_swift) to regenerate the Kotlin and Swift stubs. "
        "You understand the .udl format, UniFFI type mappings, and how platform bindings map to Rust types. "
        "You MUST read the existing api.udl before making changes. You MUST run cargo check after updating "
        "the UDL to verify the scaffolding compiles. You MUST run the gen_kotlin and gen_swift binaries "
        "to regenerate bindings."
    ),
    llm=platform_engineer_llm,
    tools=[write_to_file, read_file, list_files, execute_command],
    allow_delegation=False
)

fast_executer = Agent(
    role="Fast Executer",
    goal="Execute quick terminal commands, run build verification, fix lint issues, and perform minor edits under 50 LOC.",
    backstory=(
        "You are a fast execution agent. You run cargo check, cargo test, cargo clippy, and other verification "
        "commands quickly. You can fix trivial syntax errors, typos, and 1-2 line adjustments. You do NOT make "
        "changes to crypto algorithms or identity model. You are the CI gatekeeper who verifies that code compiles "
        "and tests pass. You MUST use your Execute Command tool to run verification commands."
    ),
    llm=fast_executer_llm,
    tools=[write_to_file, read_file, list_files, execute_command],
    allow_delegation=False
)

# All agents in a single list for easy reference
ALL_AGENTS = [lead_architect, rust_programmer, code_reviewer, security_auditor, test_engineer, technical_writer, platform_engineer, fast_executer]

AGENT_MAP = {
    "Lead Systems Architect": lead_architect,
    "Rust Systems Programmer": rust_programmer,
    "Senior Code Reviewer": code_reviewer,
    "Security Auditor": security_auditor,
    "Test Engineer": test_engineer,
    "Technical Writer": technical_writer,
    "Platform Integration Engineer": platform_engineer,
    "Fast Executer": fast_executer
}

# ═══════════════════════════════════════════════════════════════════════════════
# 4. DYNAMIC TASK LOADING
# ═══════════════════════════════════════════════════════════════════════════════
# Tasks are now dynamically routed to specific agents via the "agent" key in JSON.
# The AGENT_MAP dictionary maps agent name strings to Agent objects.

def load_dynamic_tasks_from_file(filepath: str) -> list:
    """Load task specs and assign them to specific agents based on the JSON.
    
    Format: [{"agent": "Rust Systems Programmer", "description": "...", "expected_output": "..."}]
    Each task is routed directly to the named agent via AGENT_MAP.
    """
    with open(filepath, "r", encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, list):
        data = [data]
    tasks = []
    for item in data:
        agent_name = item.get("agent")
        desc = item.get("description", "")
        expected = item.get("expected_output", "Task completed successfully.")
        
        if agent_name in AGENT_MAP and desc:
            tasks.append(Task(
                description=desc,
                expected_output=expected,
                agent=AGENT_MAP[agent_name]
            ))
        else:
            print(f"⚠️ Warning: Skipping task. Invalid agent '{agent_name}' or missing description.")
    return tasks


def build_task(description: str, expected_output: str = "Task completed successfully.") -> Task:
    """Build a single CrewAI Task dynamically (for --task mode)."""
    return Task(
        description=description,
        expected_output=expected_output,
        agent=lead_architect
    )


def build_crew(tasks: list) -> Crew:
    """Assemble the full swarm with all agents."""
    return Crew(
        agents=ALL_AGENTS,
        tasks=tasks,
        process=Process.sequential,
        verbose=True
    )


# ═══════════════════════════════════════════════════════════════════════════════
# 6. CLI INTERFACE
# ═══════════════════════════════════════════════════════════════════════════════

def parse_args():
    parser = argparse.ArgumentParser(
        description="SCMessenger CrewAI Swarm — Multi-agent Rust code production pipeline"
    )
    parser.add_argument(
        "--task",
        type=str,
        default=None,
        help="A single task prompt for the swarm (wrap in quotes)."
    )
    parser.add_argument(
        "--expected-output",
        type=str,
        default="Task completed successfully.",
        help="What the swarm should produce (used with --task)."
    )
    parser.add_argument(
        "--task-file",
        type=str,
        default=None,
        help=(
            "Path to a JSON file containing tasks. Format: "
            '[{"agent": "...", "description": "...", "expected_output": "..."}]'
        )
    )
    parser.add_argument(
        "--list-agents",
        action="store_true",
        help="Print all agent info and exit (no execution)."
    )
    parser.add_argument(
        "--list-models",
        action="store_true",
        help="Print model assignments and exit (no execution)."
    )
    return parser.parse_args()


# ═══════════════════════════════════════════════════════════════════════════════
# 7. IGNITE
# ═══════════════════════════════════════════════════════════════════════════════

if __name__ == "__main__":
    args = parse_args()

    # --list-models: show LLM assignments
    if args.list_models:
        print("=== SCMessenger Swarm — Model Assignments ===")
        print(f"  Architect/Manager : ollama/glm-5.1:cloud          (1.5T — strategic reasoning)")
        print(f"  Rust Programmer  : ollama/qwen3-coder:480b:cloud (480B — code generation)")
        print(f"  Code Reviewer    : ollama/deepseek-v3.2:cloud    (671B — analytical review)")
        print(f"  Security Auditor : ollama/mistral-large-3:675b:cloud (675B — security analysis)")
        print(f"  Test Engineer    : ollama/devstral-2:123b:cloud  (123B — test generation)")
        print(f"  Tech Writer      : ollama/gemma4:31b:cloud       (31B — documentation)")
        sys.exit(0)

    # --list-agents: introspection mode
    if args.list_agents:
        print("=== SCMessenger Swarm Agents ===")
        for agent in ALL_AGENTS:
            print(f"  Role: {agent.role}")
            print(f"  Goal: {agent.goal}")
            print(f"  Delegates: {agent.allow_delegation}")
            print(f"  Tools: {[t.name for t in agent.tools] if agent.tools else 'None'}")
            print()
        sys.exit(0)

    # Determine tasks
    if args.task_file:
        print(f"📂 Loading dynamic tasks from: {args.task_file}")
        tasks = load_dynamic_tasks_from_file(args.task_file)
    elif args.task:
        print(f"🎯 Single task received via --task — routing to Lead Architect")
        tasks = [build_task(args.task, args.expected_output)]
    else:
        print("❌ No --task or --task-file provided. Use --task-file for dynamic routing or --task for a single task.")
        print("   Example --task-file format:")
        print('   [')
        print('     {"agent": "Rust Systems Programmer", "description": "Write the struct in src/file.rs", "expected_output": "File written"},')
        print('     {"agent": "Test Engineer", "description": "Write tests in src/file.rs", "expected_output": "Tests written"}')
        print('   ]')
        sys.exit(1)

    if not tasks:
        print("❌ No valid tasks found. Exiting.")
        sys.exit(1)

    # Build and run the crew
    scmessenger_crew = build_crew(tasks)

    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    print(f"🚀 Initiating SCMessenger Swarm at {timestamp}...")
    print(f"   Agents active: {len(ALL_AGENTS)}")
    print(f"   Tasks queued:  {len(tasks)}")
    print()

    result = scmessenger_crew.kickoff()

    print("\n\n🏁 Execution Complete:")
    print(result)

    # Log result to file for traceability
    log_dir = "swarm_logs"
    os.makedirs(log_dir, exist_ok=True)
    log_file = os.path.join(log_dir, f"run_{datetime.now().strftime('%Y%m%d_%H%M%S')}.txt")
    with open(log_file, "w", encoding="utf-8") as f:
        f.write(f"SCMessenger Swarm Run — {timestamp}\n")
        f.write(f"Agents: {len(ALL_AGENTS)}\n")
        f.write(f"Tasks: {len(tasks)}\n")
        f.write("=" * 60 + "\n\n")
        f.write(str(result))
    print(f"\n📝 Result logged to: {log_file}")