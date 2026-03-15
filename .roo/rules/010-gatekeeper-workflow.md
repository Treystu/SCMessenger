# Gatekeeper Completion Workflow

## Mandatory Pre-Completion Check

**Before using `attempt_completion`, you MUST delegate to gatekeeper mode** to verify the work meets the user's requirements.

## Workflow Pattern

Follow this pattern for EVERY task:

```
1. Do the work (implement, fix, create, etc.)
2. Delegate to gatekeeper for verification
3. Address any feedback from gatekeeper
4. Only then use attempt_completion
```

## How to Delegate to Gatekeeper

Use the `new_task` tool with mode `gatekeeper`:

```
new_task(
  mode: "gatekeeper",
  message: "Verify that [specific deliverable] meets the user's requirements: [quote original request]. Check: [list specific things to verify]"
)
```

## What Gatekeeper Checks

Gatekeeper verifies:
- Does the delivered work match what the user asked for?
- Are there any obvious issues or gaps?
- Does the implementation follow project standards?
- Are there any edge cases not handled?

## Handling Gatekeeper Feedback

- **APPROVED**: Proceed with `attempt_completion`
- **CHANGES REQUESTED**: Address the feedback, then re-delegate to gatekeeper
- **BLOCKED**: Escalate the blocker to the user via `ask_followup_question`

## Exceptions

You may skip gatekeeper delegation ONLY for:
- Simple informational questions (no code changes)
- Questions asking for clarification
- Reading/listing files without making changes

## Examples

### Example 1: Code Implementation
```
User: "Fix the Android BLE connection issue"
1. Read relevant files, identify issue
2. Implement fix
3. new_task(mode: "gatekeeper", message: "Verify the BLE connection fix addresses the user's request to fix Android BLE connection issues. Check: Does the fix handle reconnection? Error cases?")
4. Address any feedback
5. attempt_completion
```

### Example 2: Documentation
```
User: "Create a bug tracker document"
1. Create the document
2. new_task(mode: "gatekeeper", message: "Verify the bug tracker document meets the user's request. Check: Does it track all known bugs? Is it well-organized?")
3. Address any feedback
4. attempt_completion
```

### Example 3: Orchestrator Delegation
```
User: "Implement feature X across all platforms"
1. Create subtasks for each platform
2. new_task for each platform implementation
3. After all subtasks complete, new_task(mode: "gatekeeper", message: "Verify feature X is implemented across all platforms per the user's request...")
4. Address any feedback
5. attempt_completion
```
