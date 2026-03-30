# M329 - Runtime integrity closure wave

Status: Active

## Context
An architectural review identified five structural integrity gaps across
`tau-agent-core`, `tau-tools`, `tau-skills`, and `tau-orchestrator`:

1. the primary `prompt*` entrypoints do not persist action history while
   `continue_turn*` does;
2. persisted tool telemetry records placeholder turn/latency values;
3. several MCP stateful tools report success without touching a live runtime;
4. MCP skills handlers bypass the manifest/trust/runtime model in `tau-skills`;
5. `plan_executor` advertises execution/scheduling behavior that its public
   surface does not implement.

These gaps undermine learning fidelity, operator trust, and the repo's stated
agent-runtime contract.

## Issue Hierarchy
- Epic: #3623

## Scope
- Unify action-history persistence so all completed agent turns follow the same
  finalization path.
- Persist real turn numbers and measured tool latency in action-history
  records.
- Make MCP runtime-changing tools either runtime-backed or explicitly
  not-implemented rather than success-shaped facades.
- Route MCP skills list/info/install behavior through `tau-skills` so external
  callers get the same manifest/trust guarantees as internal code.
- Align `plan_executor` code/docs/API language with actual capabilities, with
  truth-in-advertising as the minimum acceptable outcome.

## Exit Criteria
- `prompt`, `prompt_json`, and `prompt_with_stream` persist action history with
  the same guarantees as `continue_turn*`.
- Action-history entries record non-placeholder turn and latency values.
- `tau.training_trigger` and `tau.agent_*` no longer return fake-success
  responses for work they do not perform.
- MCP `skills_*` operations use `tau-skills` metadata/install flows instead of
  ad hoc filesystem copying.
- `plan_executor` either executes plans for real or no longer claims to.
- Scoped verification passes for `tau-agent-core`, `tau-tools`, `tau-skills`,
  and `tau-orchestrator`.

## Delivery Notes
- Delivery should land in small slices:
  - Slice 1: action-history persistence + telemetry fidelity
  - Slice 2: MCP tool honesty/runtime wiring
  - Slice 3: MCP skills parity with `tau-skills`
  - Slice 4: `plan_executor` alignment
- If Slice 4 requires a real executor instead of documentation correction, that
  decision must be made explicitly during implementation rather than emerging
  accidentally inside the PR.
