# Spec: Issue #3624 - Restore action-history persistence and telemetry fidelity on primary agent paths

Status: Implemented
Priority: P1
Milestone: M329
Parent: #3623

## Problem Statement
The primary `prompt*` entrypoints have been inconsistent with `continue_turn*`
for action-history persistence, and tool action records still use placeholder
`turn` and `latency_ms` values. Together those gaps make prompt-driven sessions
learn less than continue-turn sessions and poison downstream learning/reporting
with low-fidelity telemetry.

The current working tree already contains an uncommitted `prompt_internal()`
save call in `crates/tau-agent-core/src/lib.rs`; this story must preserve and
validate that fix while completing the missing telemetry fidelity work.

## Scope
- Keep `prompt`, `prompt_json`, and `prompt_with_stream` aligned with
  `continue_turn*` for action-history persistence.
- Replace placeholder action-history telemetry values with real turn/latency
  data for tool executions.
- Add targeted regression/conformance coverage for persistence and telemetry.

## Out of Scope
- MCP tool/runtime honesty fixes.
- Skills-surface parity with `tau-skills`.
- `plan_executor` documentation/runtime alignment.
- New learning heuristics beyond recording correct history.

## Acceptance Criteria
- AC-1: Completed `prompt*` runs persist action history with the same guarantee
  as `continue_turn*` when action history is enabled.
- AC-2: Persisted tool action-history records contain real turn indices and
  non-placeholder latency values.
- AC-3: Scoped tests prove both the prompt-path persistence contract and the
  telemetry-fidelity contract.

## Conformance Cases
- C-01 (AC-1, regression): a `prompt()` run with action history enabled writes a
  durable history file/record without requiring a `continue_turn*` call.
- C-02 (AC-1, conformance): `prompt_with_stream()` and `prompt_json()` preserve
  the same persistence guarantee as `prompt()`.
- C-03 (AC-2, conformance): a tool-executing run records `turn > 0` and
  `latency_ms > 0` in persisted history.
- C-04 (AC-3, regression): scoped `tau-agent-core` tests and targeted
  integration coverage stay green with the new persistence/telemetry contract.

## Success Signals
- Prompt-driven sessions leave durable action-history records on disk.
- Tool history no longer records `turn: 0` / `latency_ms: 0` placeholders for
  real executions.
- The new tests fail on the pre-fix state and pass on the implemented state.
