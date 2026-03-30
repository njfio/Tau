# Tasks: Issue #3623 - Close agent runtime integrity gaps across persistence, telemetry, MCP, skills, and orchestration

Status: Implemented

Milestone: specs/milestones/m329/index.md

## Ordered Tasks
1. [x] T1 (RED): add failing `tau-agent-core` coverage proving `prompt*`
   entrypoints do not persist action history and persisted tool telemetry still
   carries placeholder turn/latency values.
2. [x] T2 (GREEN): refactor `tau-agent-core` to use one action-history
   finalization path for `prompt*` and `continue_turn*`, then thread real turn
   and latency values into persisted tool records.
3. [x] T3 (VERIFY): run scoped `tau-agent-core` and action-history integration
   tests, then inspect the persisted history contract for non-placeholder
   values.
4. [x] T4 (RED): add failing `tau-tools` coverage showing `tau.training_trigger`
   and `tau.agent_*` return success-shaped responses without runtime-backed side
   effects, and that MCP skills handlers diverge from `tau-skills`.
5. [x] T5 (GREEN): make MCP training/agent lifecycle tools runtime-backed or
   explicitly not implemented, and route MCP skills list/info/install behavior
   through `tau-skills`.
6. [x] T6 (VERIFY): run scoped `tau-tools` and `tau-skills` tests, including
   negative trust/install cases and MCP response-contract checks.
7. [x] T7 (RED): add failing `tau-orchestrator` coverage that captures the
   current mismatch between `plan_executor` claims and its implemented public
   surface.
8. [x] T8 (GREEN): align `plan_executor` docs/API naming with implemented
   behavior, or implement real execution/scheduling only if that choice is made
   explicitly during implementation.
9. [x] T9 (VERIFY): run the cross-crate verification bundle (`tau-agent-core`,
   `tau-tools`, `tau-skills`, `tau-orchestrator`, `cargo fmt --check`,
   `cargo clippy -- -D warnings`) and update issue closeout/process artifacts.

## Test Mapping
- AC-1 -> C-01 -> new/updated `tau-agent-core` persistence tests +
  `agent_tool_memory_roundtrip`
- AC-2 -> C-02 -> `tau-agent-core` tool-history record tests
- AC-3 -> C-03 -> `tau-tools` MCP stateful-tool contract tests
- AC-4 -> C-04 -> `tau-tools` + `tau-skills` catalog/install parity tests
- AC-5 -> C-05 -> `tau-orchestrator` `plan_executor` surface/doc tests

## Test Tier Intent
| Tier | Planned |
| --- | --- |
| Unit | `tau-agent-core`, `tau-tools`, `tau-skills`, `tau-orchestrator` targeted tests |
| Functional | targeted action-history and MCP contract flows |
| Conformance | AC/C-case-mapped tests for persistence, telemetry, MCP honesty, skills parity, executor alignment |
| Integration | `agent_tool_memory_roundtrip` or equivalent targeted integration coverage |
| Regression | facade-tool and placeholder-telemetry regressions |
| Property | N/A unless existing telemetry/history helpers already use property tests |
| Contract/DbC | N/A unless touched public APIs already carry DbC coverage |
| Snapshot | N/A |
| Fuzz | N/A |
| Mutation | N/A for planning; evaluate during implementation if critical-path changes warrant it |
| Performance | N/A unless executor implementation is chosen and adds scheduling overhead |
