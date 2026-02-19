# Plan: Issue #2608 - Integration suite bootstrap under tests/integration

## Approach
1. Add RED integration tests in a new workspace member package (`tests/integration`) with a deterministic queued mock LLM client.
2. Wire the package into the root workspace and keep dependencies path-local to existing crates (`tau-agent-core`, `tau-ai`, `tau-tools`).
3. Implement shared test helpers inside the integration package for:
   - isolated temp workspace/memory directories,
   - deterministic queued `ChatResponse` scripting,
   - tool-result extraction helpers.
4. Implement conformance flow:
   - model returns `memory_write` tool call,
   - model returns `memory_search` tool call,
   - model returns final assistant text,
   - assert search tool result includes the written memory.
5. Run scoped verify gates and map AC -> tests.

## Affected Modules
- `Cargo.toml` (workspace members)
- `tests/integration/Cargo.toml`
- `tests/integration/src/lib.rs`
- `tests/integration/tests/agent_tool_memory_roundtrip.rs`
- `specs/2608/spec.md`
- `specs/2608/plan.md`
- `specs/2608/tasks.md`
- `specs/milestones/m104/index.md`

## Risks / Mitigations
- Risk: root workspace is virtual; incorrect package wiring can break `cargo` commands.
  - Mitigation: add explicit package member and run scoped test target only.
- Risk: non-deterministic filesystem collisions in temp paths.
  - Mitigation: unique per-test directory naming with process id + timestamp and cleanup best-effort.
- Risk: over-coupling to `tau-agent-core` internal test fixtures.
  - Mitigation: define local mock client in integration crate using only public `tau-ai::LlmClient` contract.

## Interfaces / Contracts
- New package contract:
  - `tests/integration` is a standalone workspace member for cross-crate integration tests.
- Public crate contracts consumed:
  - `tau_agent_core::Agent`, `tau_agent_core::AgentConfig`, `tau_agent_core::AgentTool`
  - `tau_tools::tools::{MemoryWriteTool, MemorySearchTool, ToolPolicy}`
  - `tau_ai::{LlmClient, ChatRequest, ChatResponse}`

## ADR
- Not required: no new external dependency, no protocol/wire-format changes, no architectural fork.
