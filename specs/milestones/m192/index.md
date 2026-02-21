# M192 - Integration Breadth Scenario Expansion

## Context
`tests/integration` currently has a narrow scenario footprint concentrated in a single roundtrip file. We need broader deterministic scenarios that validate channel-scoped routing behavior and ordered multi-step tool-call flows.

## Scope
- Add new integration scenarios in `tests/integration/tests/agent_tool_memory_roundtrip.rs`.
- Add minimal harness support to assert ordered tool payloads across repeated tool calls in one prompt.
- Keep changes scoped to integration tests and issue-linked spec artifacts.

## Linked Issues
- Epic: #3054
- Story: #3056
- Task: #3055
