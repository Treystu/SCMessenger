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

# All agents in a single list for easy reference
ALL_AGENTS = [lead_architect, rust_programmer, code_reviewer, security_auditor, test_engineer, technical_writer]

# ═══════════════════════════════════════════════════════════════════════════════
# 4. PIPELINE BUILDER — EXPLICIT PER-AGENT TASKS
# ═══════════════════════════════════════════════════════════════════════════════
# Using explicit per-agent tasks ensures each specialist receives its task directly,
# so agents with write_to_file actually write files to disk instead of the Architect
# generating text that never gets saved.

def build_pipeline_tasks(spec: str) -> list:
    """Build a full pipeline of tasks from a high-level spec string.
    
    Returns a list of Task objects, one per specialist agent, in execution order.
    The spec describes what to build; each task translates it for its agent.
    """
    return [
        Task(
            description=(
                f"Design the architectural specification for the following SCMessenger component: {spec}. "
                "Produce a clear, detailed spec that the Rust Systems Programmer can implement directly. "
                "Include: field names, types, derive macros, required trait impls, and file naming conventions."
            ),
            expected_output="A detailed architectural specification document for the component.",
            agent=lead_architect
        ),
        Task(
            description=(
                f"Implement the following SCMessenger component in Rust: {spec}. "
                "Follow the architectural specification provided by the Lead Architect. "
                "Write idiomatic Rust with proper derive macros, serde support, and doc comments. "
                "YOU MUST use your 'Write to File' tool to save the code to a .rs file on disk. "
                "Do not just output the code as text — actually write it to a file."
            ),
            expected_output="Confirmation that the Rust source file has been written to disk using the Write to File tool.",
            agent=rust_programmer
        ),
        Task(
            description=(
                f"Read the Rust source file(s) that were just created for: {spec}. "
                "Perform a thorough code review covering: correctness, idiomatic Rust patterns, "
                "ownership/borrowing, error handling, unnecessary allocations, dead code, and style. "
                "Provide specific line references and suggested fixes for any issues found. "
                "YOU MUST use your 'Write to File' tool to save your review report to a markdown file (e.g. code_review_report.md). "
                "Do not just output the report as text — actually write it to a file."
            ),
            expected_output="Confirmation that the code review report has been written to disk using the Write to File tool.",
            agent=code_reviewer
        ),
        Task(
            description=(
                f"Read the Rust source file(s) that were just created for: {spec}. "
                "Perform a security audit covering: injection vulnerabilities, integer overflow/underflow, "
                "cryptographic weaknesses, timing attacks, unsafe blocks, serialization exploits, and P2P attack vectors. "
                "Rate each finding by severity (Critical/High/Medium/Low). "
                "YOU MUST use your 'Write to File' tool to save your audit report to a markdown file (e.g. security_audit_report.md). "
                "Do not just output the report as text — actually write it to a file."
            ),
            expected_output="Confirmation that the security audit report has been written to disk using the Write to File tool.",
            agent=security_auditor
        ),
        Task(
            description=(
                f"Read the Rust source file(s) that were just created for: {spec}. "
                "Write comprehensive Rust tests covering: happy paths, edge cases, error conditions, "
                "serde round-trips, boundary values, and property-based tests where appropriate. "
                "YOU MUST use your 'Write to File' tool to save the test file to disk alongside the source. "
                "Do not just output the tests as text — actually write them to a _tests.rs file."
            ),
            expected_output="Confirmation that the test file has been written to disk using the Write to File tool.",
            agent=test_engineer
        ),
        Task(
            description=(
                f"Read the Rust source file(s) that were just created for: {spec}. "
                "Write comprehensive rustdoc documentation including: module-level docs (//!), "
                "item-level docs (///), usage examples, JSON serialization examples, and integration "
                "guidance for observability pipelines (tracing crate, Grafana/Loki, Elasticsearch). "
                "YOU MUST use your 'Write to File' tool to save the documentation to a .md file on disk. "
                "Do not just output the docs as text — actually write them to a file."
            ),
            expected_output="Confirmation that the documentation file has been written to disk using the Write to File tool.",
            agent=technical_writer
        ),
    ]


# Default pipeline spec for the original built-in task
DEFAULT_SPEC = (
    "A structured tracing payload for mandatory relay protocol events in SCMessenger v0.2.1. "
    "The Rust struct must track 'message_id' (String), 'relay_node_hash' (String), and 'latency_ms' (u64). "
    "Save source to 'observability.rs'."
)


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
            '[{"description": "...", "expected_output": "..."}]'
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


def load_pipeline_tasks_from_file(filepath: str) -> list:
    """Load task specs from a JSON file and expand each into a full pipeline.
    
    Format: [{"description": "...", "expected_output": "..."}]
    Each spec gets expanded into 6 per-agent tasks via build_pipeline_tasks().
    """
    with open(filepath, "r", encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, list):
        data = [data]
    all_tasks = []
    for item in data:
        desc = item.get("description", "")
        if desc:
            all_tasks.extend(build_pipeline_tasks(desc))
    return all_tasks


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
        print(f"📂 Loading tasks from: {args.task_file} — building full pipeline per spec")
        tasks = load_pipeline_tasks_from_file(args.task_file)
    elif args.task:
        print(f"🎯 Dynamic task received via --task — building full pipeline")
        tasks = build_pipeline_tasks(args.task)
    else:
        print("📋 No --task or --task-file provided. Using default built-in pipeline.")
        tasks = build_pipeline_tasks(DEFAULT_SPEC)

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