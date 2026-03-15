# Tool Use Requirements - All Modes

## CRITICAL: Always Use Tools

**NEVER respond with just text when a tool can accomplish the task.** If you find yourself writing "Let me check...", "I'll look at...", "I should read..." - STOP and invoke the actual tool instead.

## The Thinking-to-Action Gap

Models sometimes generate reasoning about what to do without actually doing it. This is a failure mode. Follow this pattern:

❌ **WRONG**: "Let me check the existing documentation to identify if there's a master bug/issue tracker."
✅ **RIGHT**: Invoke `read_file` on `REMAINING_WORK_TRACKING.md`

❌ **WRONG**: "I should look at the file structure to see what documentation exists."
✅ **RIGHT**: Invoke `list_files` on the relevant directory

❌ **WRONG**: "Let me search for relevant code patterns."
✅ **RIGHT**: Invoke `codebase_search` or `search_files`

## Mandatory Tool Use Rules

1. **Reading files**: ALWAYS use `read_file` - never just describe what you'd read
2. **Listing directories**: ALWAYS use `list_files` - never just describe what you'd find
3. **Searching code**: ALWAYS use `codebase_search` or `search_files` - never just describe what you'd search
4. **Running commands**: ALWAYS use `execute_command` - never just describe what you'd run
5. **Creating files**: ALWAYS use `write_to_file` - never just describe what you'd write
6. **Editing files**: ALWAYS use `apply_diff` - never just describe what you'd change

## Tool Invocation Format

When you need to take action, invoke the tool immediately. Do not:
- Describe what you're about to do
- Explain your plan before executing it
- Generate reasoning about which tool to use without using it

Just invoke the tool with the correct parameters.

## Response Pattern

Follow this pattern for every user request:

1. **Understand** the request (brief reasoning is OK)
2. **ACT** - invoke the appropriate tool immediately
3. **Process** the tool result
4. **ACT again** if needed
5. **Complete** when done

## Orchestrator Mode Specific

When in orchestrator mode:
- Use `new_task` to delegate work to other modes
- Use `read_file` to check documentation before delegating
- Use `list_files` to understand project structure
- NEVER just describe what you'll delegate - actually delegate it
- **Before completing**: Delegate to gatekeeper mode to verify all subtasks are complete

## Architect Mode Specific

When in architect mode:
- Use `read_file` to understand existing code
- Use `codebase_search` to find relevant patterns
- Use `write_to_file` to create plans/documentation
- NEVER just describe what you'd design - actually read the code and create the plan
- **Before completing**: Delegate to gatekeeper mode to verify the design meets requirements

## Completion Workflow

**CRITICAL**: Before using `attempt_completion`, you MUST delegate to gatekeeper mode.
See `.roo/rules/010-gatekeeper-workflow.md` for the complete workflow pattern.
