# Spec: Issue #3623 - Close agent runtime integrity gaps across persistence, telemetry, MCP, skills, and orchestration

Status: Implemented

Milestone: specs/milestones/m329/index.md

## Problem Statement
A review of the current agent stack found five cross-cutting integrity gaps that
undercut the system's core promises:

- the dominant `prompt*` entrypoints do not persist action history, so
  cross-session learning behavior is incomplete on the main path;
- tool action history stores `turn: 0` and `latency_ms: 0`, so learning and
  reporting consume placeholder telemetry rather than real execution traces;
- MCP tools such as `tau.training_trigger` and `tau.agent_*` return
  success-shaped acknowledgements without actually changing runtime state;
- MCP skills handlers implement their own filesystem-only path instead of using
  the trust/manifest/runtime model in `tau-skills`;
- `plan_executor` presents itself as an execution engine with parallel
  scheduling even though its public surface is reporting/deadlock analysis.

The result is a runtime that appears more integrated and operational than it is
in several key surfaces. This work closes those integrity gaps by making state
recording complete, telemetry truthful, external tools honest, skills behavior
consistent, and orchestration claims accurate.

## Scope
In scope:
- `tau-agent-core` action-history persistence parity between `prompt*` and
  `continue_turn*`.
- Real turn/latency capture for persisted tool action history.
- Honest MCP behavior for stateful training and agent-lifecycle tools.
- MCP skills list/info/install parity with `tau-skills`.
- `plan_executor` code/docs/API alignment with implemented behavior.
- Targeted tests and documentation updates required to verify the above.

Out of scope:
- New long-term memory algorithms beyond recording correct history/telemetry.
- Shipping a full production multi-agent registry unless explicitly required to
  replace facade MCP tools.
- Expanding `plan_executor` into a full DAG executor unless that is chosen as
  the explicit fix instead of documentation/API alignment.
- Unrelated README, example, or UI work.

## Acceptance Criteria
### AC-1 Prompt-path action history persists consistently
Given an agent configured with action-history persistence,
when a caller completes work through `prompt`, `prompt_json`, or
`prompt_with_stream`,
then action history is persisted with the same guarantees as the
`continue_turn*` path rather than being silently skipped on the primary
entrypoints.

### AC-2 Persisted telemetry reflects real execution data
Given an agent turn that executes one or more tools,
when action history is written,
then each recorded tool entry includes the real turn index and a measured
non-placeholder latency value derived from tool execution rather than hardcoded
zeros.

### AC-3 MCP stateful tools are honest about runtime effects
Given an MCP client calling `tau.training_trigger` or `tau.agent_spawn`,
`tau.agent_status`, `tau.agent_cancel`,
when the runtime cannot actually perform the requested state change,
then the tool returns an explicit, machine-checkable not-implemented or
equivalent error contract instead of a fake-success acknowledgement; if runtime
backing is added, tests must prove the side effect instead.

### AC-4 MCP skills operations use the trusted skills surface
Given an MCP client listing, inspecting, or installing skills,
when those operations are handled,
then metadata and install behavior derive from `tau-skills` manifest/trust
flows rather than direct markdown scanning and blind filesystem copying.

### AC-5 Orchestration execution claims match reality
Given the `tau-orchestrator` execution surface,
when developers read `plan_executor` docs or inspect its public API,
then they see behavior that matches the implementation: either real plan
execution/scheduling exists and is tested, or the surface is described as
analysis/reporting only.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance/Regression | action-history-enabled agent | call `prompt*` path and inspect persisted store | the run writes action-history entries just like `continue_turn*` |
| C-02 | AC-2 | Unit/Conformance | tool-executing turn | inspect persisted action-history records | turn index and latency are non-placeholder values tied to the real execution |
| C-03 | AC-3 | Unit/Integration | MCP stateful tool invocation | call training/agent lifecycle tools under current runtime wiring | responses are either side-effect-backed or explicit not-implemented contracts |
| C-04 | AC-4 | Unit/Integration | MCP skills list/info/install calls | compare behavior against `tau-skills` catalog/install paths | skill metadata/install behavior comes from `tau-skills`, including trust failures |
| C-05 | AC-5 | Unit/Conformance | `plan_executor` module surface | inspect docs/public APIs and run targeted tests | execution claims are truthful and supported by tests |

## Success Metrics / Observable Signals
- Prompt-driven sessions leave durable action history in the configured store.
- Learning/reporting surfaces no longer consume `turn: 0` / `latency_ms: 0`
  placeholder data for real tool calls.
- MCP clients no longer receive success-shaped responses for nonexistent
  runtime actions.
- Unsigned or otherwise invalid skill installs fail through the same trust path
  regardless of whether the caller is internal or MCP-based.
- `plan_executor` no longer reads as aspirational API documentation.
