# Plan: Issue #3624 - Restore action-history persistence and telemetry fidelity on primary agent paths

Status: Implemented
Milestone: M329
Parent: #3623

## Approach
1. Preserve the existing uncommitted `prompt_internal()` persistence addition
   and add failing tests that lock in the intended behavior before further
   refactoring.
2. Thread turn/latency metadata into tool-history recording at the tool
   execution boundary so the values reflect actual executions instead of
   placeholders.
3. Reuse the existing action-history save/load helpers and avoid adding another
   parallel persistence path.
4. Verify the contract through scoped unit/regression tests plus the targeted
   integration surface that already exercises prompt-driven tool flows.

## Affected Modules
- `crates/tau-agent-core/src/lib.rs`
- `crates/tau-agent-core/src/session_history.rs`
- `crates/tau-memory/src/action_history.rs`
- `tests/integration/tests/agent_tool_memory_roundtrip.rs` if integration
  coverage needs a persistence assertion

## Risks / Mitigations
- Risk: the current uncommitted prompt-path fix gets duplicated or accidentally
  reverted.
  - Mitigation: treat it as the starting point, add RED coverage first, and
    refactor only after the failing tests are in place.
- Risk: latency measurement could be zero for very fast tool calls.
  - Mitigation: measure elapsed wall time at the execution boundary and assert
    non-placeholder semantics in tests; use `>= 1` ms floor only if the runtime
    resolution requires it.
- Risk: turn numbering may be captured from the wrong scope.
  - Mitigation: derive turn from the existing turn loop context rather than from
    message count heuristics.

## Verification Plan
- Targeted `tau-agent-core` tests for prompt-path persistence and telemetry.
- Any required update to `agent_tool_memory_roundtrip`.
- Final scoped verification:
  - `cargo test -p tau-agent-core`
  - targeted integration command if modified

## Verification Result
- `cargo fmt -p tau-agent-core --check`
- `CARGO_TARGET_DIR=/tmp/tau-target-3624 cargo clippy -p tau-agent-core -- -D warnings`
- `CARGO_TARGET_DIR=/tmp/tau-target-3624 cargo test -p tau-agent-core`
- `CARGO_TARGET_DIR=/tmp/tau-target-3624 cargo test -p tau-integration-tests --test agent_tool_memory_roundtrip`

## ADR
No ADR required. This story tightens an existing runtime contract without
introducing a new public API shape.
