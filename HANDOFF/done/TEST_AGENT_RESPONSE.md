# TEST: Agent Response Check

## Purpose
Verify that sub-agents are properly checking the HANDOFF/todo directory every 5 minutes.

## Action Required
If this task is picked up by a sub-agent, it should be moved to HANDOFF/done within 5 minutes.

## Test Criteria
- ✅ Sub-agent detects new task in todo directory
- ✅ Sub-agent processes task within 5-minute loop
- ✅ Task moved to done directory upon completion

**Status**: Test in progress