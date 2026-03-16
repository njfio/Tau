# Spec: Issue #3581 - Shared Operator Turn/Task State Protocol for TUI and Webchat

Issue: https://github.com/njfio/Tau/issues/3581
Status: Planned

## Objective
Define and introduce a single typed operator state/event contract that the runtime emits and both Tau TUI and webchat consume. The contract must cover turn lifecycle, task lifecycle, tool execution, approvals, artifacts, memory, cortex, jobs, and structured failures so the product no longer depends on ad hoc log parsing and surface-specific state reconstruction.

## Inputs/Outputs
- Inputs:
  - Runtime/provider lifecycle signals from `tau-coding-agent` and provider adapters.
  - Tool execution lifecycle events from agent core and gateway tool runtime.
  - Session/memory/cortex/job updates emitted by local runtime and gateway services.
  - Existing legacy stderr/stdout progress markers and gateway JSON endpoints during migration.
- Outputs:
  - A typed shared event schema for `turn`, `task`, `tool`, `approval`, `artifact`, `memory`, `cortex`, `job`, and `error`.
  - A shared snapshot/state-sync shape that can reconstruct current operator-visible state without replaying raw logs.
  - Runtime emission rules mapping provider/runtime internals to the shared contract.
  - Client consumption rules for TUI and webchat transcript/progress/tool/task rendering.

## Boundaries / Non-goals
- No full TUI visual redesign in this issue.
- No full webchat visual redesign in this issue.
- No swarm orchestration UX implementation beyond the state primitives it depends on.
- No provider prompt redesign except where required to emit better state transitions.
- No silent fallback behavior; legacy compatibility must remain explicit and temporary.

## Failure Modes
1. Provider request times out after partial progress.
   - Expected: shared state records last phase, elapsed time, provider/tool context, partial output presence, and actionable failure reason.
2. Tool execution starts but no completion event is emitted.
   - Expected: shared state marks tool as in-flight/stale and UI renders explicit degraded state rather than silently dropping it.
3. TUI and webchat consume different subsets of runtime state.
   - Expected: parity tests fail if the same turn/task outcome renders differently across clients.
4. Legacy line-based parsing diverges from typed event path during migration.
   - Expected: migration boundary is explicit; dual-path tests catch inconsistencies.
5. Cancel/interrupt is requested mid-turn.
   - Expected: shared state transitions through cancelling/cancelled with visible reason and no orphaned in-flight task state.

## Acceptance Criteria (testable booleans)
- [ ] AC-1: A typed shared state/event schema exists for the operator-visible domains: turn, task, tool, approval, artifact, memory, cortex, job, error.
- [ ] AC-2: Runtime/provider lifecycle events map into the shared contract with deterministic phase and status semantics.
- [ ] AC-3: Partial-output, timeout, cancel, and tool-failure cases preserve actionable state instead of collapsing to opaque text-only failure.
- [ ] AC-4: TUI consumes the shared contract for transcript/progress/tool/task state instead of relying primarily on free-form log parsing.
- [ ] AC-5: Webchat consumes the same shared contract for transcript/progress/tool/task state instead of relying primarily on endpoint-specific JSON dumps.
- [ ] AC-6: Legacy compatibility boundaries are documented and enforced with explicit migration tests.
- [ ] AC-7: End-to-end tests prove the same turn/task scenario produces equivalent operator-visible state in runtime, TUI, and webchat.

## Files To Touch
- `crates/tau-coding-agent/src/runtime_loop.rs`
- `crates/tau-coding-agent/src/events.rs`
- `crates/tau-agent-core/src/process_types.rs`
- `crates/tau-gateway/src/gateway_openresponses/stream_response_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/ws_stream_handlers.rs`
- `crates/tau-gateway/src/gateway_openresponses/types.rs`
- `crates/tau-gateway/src/gateway_openresponses/webchat_page.rs`
- `crates/tau-tui/src/main.rs`
- `crates/tau-runtime/src/runtime_output_runtime.rs`
- `scripts/run/tau-unified.sh`
- `scripts/verify/*` and/or browser/PTY test harness files needed for parity coverage

## Error Semantics
- Shared-state emission failures are hard-fail visible at entrypoints; they must not be silently ignored.
- Runtime/provider failures must include structured machine-readable context: stage, status, code, reason, elapsed/request budget, and last known active task/tool when available.
- Client renderers may summarize state for UX, but they must not invent success/failure semantics not present in the shared contract.
- Legacy fallbacks, where temporarily required, must be explicit in code and test coverage and must emit observable degraded-state markers.

## Test Plan
1. Add schema-level tests for shared event/state serialization covering all operator-visible domains.
2. Add runtime integration tests proving provider start/progress/partial-output/tool/failure/cancel states map into the shared contract.
3. Add TUI tests that consume typed shared state and verify transcript/progress/tool/task rendering for success, timeout, cancel, and tool-failure turns.
4. Add web/browser tests that verify webchat renders the same shared state transitions and artifacts as the TUI for the same scenario.
5. Add migration tests covering dual emission during cutover so legacy markers and typed state do not silently diverge.
6. Add unified smoke coverage proving `tau-unified` startup exposes one coherent session/turn state across runtime, TUI, dashboard, and webchat.

## Rollout Notes
1. Define schema and shared snapshot first.
2. Emit shared state from runtime alongside legacy markers.
3. Migrate TUI to typed shared state.
4. Migrate webchat to typed shared state.
5. Remove legacy-only UI paths once parity tests are green.
