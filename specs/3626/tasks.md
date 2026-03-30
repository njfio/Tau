# Tasks: Issue #3626 - Route MCP skill catalog and install flows through tau-skills

Status: Implemented
Milestone: M329
Parent: #3623

1. [x] T1 (RED): add failing `tau-tools` coverage for nested `SKILL.md`
   naming/lookup drift and missing lockfile writes on MCP skill install.
2. [x] T2 (GREEN): route MCP skills list/info/install through `tau-skills`
   helpers and preserve structured MCP responses.
3. [x] T3 (VERIFY): run scoped `tau-tools` verification and update
   issue/process artifacts.

## Test Mapping
- AC-1 -> C-01 -> nested skill list/info parity tests
- AC-2 -> C-02 -> install + lockfile tests
- AC-3 -> C-03 -> structured payload assertions
- AC-4 -> C-04 -> scoped verification commands
