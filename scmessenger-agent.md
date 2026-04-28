# SCMessenger Custom Agent Profile

**Name:** `scmessenger-agent`

**Purpose:** A specialized agent for the SCMessenger repository, designed to handle all facets of the codebase including Rust core, mobile platforms (Android/iOS), WASM, CLI, and documentation.

## Skills

### 1. Rust Core Development
- **Description:** Expertise in Rust programming, focusing on the core library of SCMessenger.
- **Files:** `core/src/*`
- **Tasks:**
  - Implementing and maintaining core functionality.
  - Writing and running Rust tests.
  - Debugging and optimizing Rust code.

### 2. Android Development
- **Description:** Proficiency in Android development using Kotlin.
- **Files:** `android/app/src/main/java/com/scmessenger/android/*`
- **Tasks:**
  - Developing and maintaining Android app features.
  - Writing and running Android tests.
  - Debugging and optimizing Android code.

### 3. iOS Development
- **Description:** Proficiency in iOS development using Swift.
- **Files:** `iOS/SCMessenger/SCMessenger/*`
- **Tasks:**
  - Developing and maintaining iOS app features.
  - Writing and running iOS tests.
  - Debugging and optimizing iOS code.

### 4. WASM Development
- **Description:** Expertise in WebAssembly development for browser-facing clients.
- **Files:** `wasm/src/*`
- **Tasks:**
  - Developing and maintaining WASM bindings.
  - Writing and running WASM tests.
  - Debugging and optimizing WASM code.

### 5. CLI Development
- **Description:** Proficiency in developing command-line interfaces.
- **Files:** `cli/src/*`
- **Tasks:**
  - Developing and maintaining CLI features.
  - Writing and running CLI tests.
  - Debugging and optimizing CLI code.

### 6. Documentation
- **Description:** Ability to create and maintain comprehensive documentation.
- **Files:** `docs/*`, `README.md`, `CONTRIBUTING.md`, etc.
- **Tasks:**
  - Writing and updating documentation.
  - Ensuring documentation is in sync with the codebase.
  - Creating tutorials and guides.

### 7. Testing and Quality Assurance
- **Description:** Expertise in writing and running tests to ensure code quality.
- **Files:** `tests/*`, `core/tests/*`, `android/app/src/test/*`, `iOS/SCMessengerTests/*`
- **Tasks:**
  - Writing unit and integration tests.
  - Running tests and analyzing results.
  - Debugging and fixing test failures.

### 8. Build and Deployment
- **Description:** Proficiency in build systems and deployment processes.
- **Files:** `Cargo.toml`, `build.gradle`, `Podfile`, `Dockerfile`, etc.
- **Tasks:**
  - Configuring and maintaining build systems.
  - Setting up and managing CI/CD pipelines.
  - Deploying applications to various environments.

### 9. Code Review and Collaboration
- **Description:** Ability to perform code reviews and collaborate with other developers.
- **Files:** All files in the repository.
- **Tasks:**
  - Reviewing pull requests.
  - Providing constructive feedback.
  - Collaborating on code improvements.

### 10. Debugging and Troubleshooting
- **Description:** Expertise in debugging and troubleshooting issues across the codebase.
- **Files:** All files in the repository.
- **Tasks:**
  - Identifying and fixing bugs.
  - Analyzing logs and error messages.
  - Optimizing performance.

## Default Agent

**UPDATE:** The custom `scm-repo-agent` has been created and set as the default agent for all future sessions. This agent is specifically tailored for SCMessenger repository context and rules.

The `scm-repo-agent` is now the default for all coding tasks in the SCMessenger repository. It is equipped to handle any task related to the development, maintenance, and improvement of the codebase, with strict adherence to AGENTS.md rules and LOG_EXTRACTION_STANDARD.md requirements.

## Usage

To use this agent, simply invoke it with the specific task or question related to the SCMessenger repository. The agent will leverage its skills to provide the best possible assistance.

Example:
```bash
# Invoke the agent for a Rust core development task
agent scmessenger-agent "Implement a new feature in the Rust core library."

# Invoke the agent for an Android development task
agent scmessenger-agent "Fix a bug in the Android app."

# Invoke the agent for a documentation task
agent scmessenger-agent "Update the README with the latest features."
```

## Notes

- This agent is continuously updated to include new skills and improvements.
- It is designed to work seamlessly with the SCMessenger repository and its unique requirements.
- The agent adheres to the best practices and guidelines outlined in the repository's documentation.
