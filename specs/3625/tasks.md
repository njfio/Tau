# Tasks: Issue #3625 - Make MCP lifecycle tools explicit about unsupported runtime operations

Status: Implemented
Milestone: M329
Parent: #3623

1. [x] T1 (RED): add failing `tau-tools` and MCP SDK coverage proving
   lifecycle MCP tools currently report fake success or ambiguous placeholders.
2. [x] T2 (GREEN): replace placeholder lifecycle responses with explicit
   unsupported/runtime-unavailable contracts and update descriptions/docs.
3. [x] T3 (VERIFY): run scoped `tau-tools` and `tau-ops` verification and
   update issue/process artifacts.

## Test Mapping
- AC-1 -> C-01 -> `tau.training_trigger` response-contract tests
- AC-2 -> C-02 -> `tau.agent_*` response-contract tests
- AC-3 -> C-03 -> descriptor / generated docs assertions
- AC-4 -> C-04 -> scoped verification commands
