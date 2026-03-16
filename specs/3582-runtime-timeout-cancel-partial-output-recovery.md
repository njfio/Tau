# Spec: Issue #3582 - Runtime Timeout, Cancel, and Partial-Output Recovery Semantics

Issue: https://github.com/njfio/Tau/issues/3582
Status: Planned

## Objective
Define and implement reliable timeout, cancellation, retry, and partial-output recovery semantics for provider-backed interactive turns. Operators must be able to understand what failed, what partial work exists, what the runtime was doing at failure time, and what recovery actions are available across runtime, TUI, and webchat.

## Inputs/Outputs
- Inputs:
  - Provider subprocess lifecycle signals from `tau-provider` clients.
  - Runtime turn lifecycle and tool execution events from `tau-coding-agent` and `tau-agent-core`.
  - User interrupt/cancel/retry actions from interactive runtime, TUI, and webchat.
  - Existing request timeout configuration (`request_timeout_ms`, turn timeout, retries).
- Outputs:
  - Explicit timeout policy separating startup/connect timeout, inactivity timeout, and wall timeout.
  - Structured failure state including last known phase, last active tool/task, partial output presence, and reason code.
  - Deterministic cancel/interrupt/retry semantics for turns and runtime/provider subprocesses.
  - Recovery rules that preserve partial context rather than collapsing to opaque text-only failure.

## Boundaries / Non-goals
- No full TUI redesign in this issue.
- No full webchat redesign in this issue.
- No provider prompt redesign except where necessary to support reliable partial-output/state reporting.
- No silent fallback or auto-success semantics.

## Failure Modes
1. Provider process never responds after startup.
   - Expected: startup/connect timeout with explicit stage context and retry guidance.
2. Provider emits progress/partial output, then stalls.
   - Expected: inactivity timeout with partial output preserved and surfaced.
3. Provider runs beyond maximum allowed wall time.
   - Expected: wall-time timeout with last known phase/tool/task and explicit timeout classification.
4. User interrupts during model generation or tool execution.
   - Expected: deterministic cancelling/cancelled state and subprocess cleanup without orphaned in-flight state.
5. Provider returns invalid response after partial work.
   - Expected: invalid-response failure with partial output/tool/task context preserved.
6. Retry/restart is attempted after runtime/provider failure.
   - Expected: explicit recovery path with new turn/runtime state and no hidden carry-over ambiguity.

## Acceptance Criteria (testable booleans)
- [ ] AC-1: Timeout policy distinguishes connect/start timeout, inactivity timeout, and wall timeout.
- [ ] AC-2: Partial assistant output and last-known phase/tool/task are preserved in structured failure state when available.
- [ ] AC-3: User cancel/interrupt transitions through explicit cancelling/cancelled state and cleans up provider/tool execution deterministically.
- [ ] AC-4: Retry/restart semantics after failure are explicit and observable across runtime, TUI, and webchat.
- [ ] AC-5: Provider invalid-response paths surface actionable stage-aware failure details instead of a single opaque timeout/error line.
- [ ] AC-6: Runtime/provider integration tests cover timeout, partial-output, cancel, invalid-response, and retry/recovery scenarios.

## Files To Touch
- `crates/tau-provider/src/codex_cli_client.rs`
- `crates/tau-provider/src/claude_cli_client.rs`
- `crates/tau-provider/src/gemini_cli_client.rs`
- `crates/tau-provider/src/client.rs`
- `crates/tau-coding-agent/src/runtime_loop.rs`
- `crates/tau-coding-agent/src/events.rs`
- `crates/tau-agent-core/src/lib.rs`
- `crates/tau-runtime/src/runtime_output_runtime.rs`
- `crates/tau-tui/src/main.rs`
- `crates/tau-gateway/src/gateway_openresponses/stream_response_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/ws_stream_handlers.rs`

## Error Semantics
- Provider/runtime timeout and cancel paths must emit structured machine-readable context: `code`, `message`, `stage`, `status`, `elapsed_ms`, `request_budget_ms`, and last-known active tool/task when available.
- Partial output must be explicitly marked as partial/degraded; clients must not present it as final success.
- Cancelled and timed-out turns are distinct statuses and must remain distinguishable in all clients and logs.
- Recovery actions may summarize prior failure state, but must not silently suppress or overwrite it.

## Test Plan
1. Add provider client tests for startup/connect timeout, inactivity timeout, wall timeout, and invalid-response-after-partial-output.
2. Add runtime integration tests proving turn state preserves last phase/tool/task and partial output markers across timeout/cancel/failure paths.
3. Add TUI tests proving timeout/cancel/retry states render explicit recovery information instead of only raw provider text.
4. Add web/browser tests proving the same timeout/cancel/retry scenarios are visible and actionable in webchat.
5. Add smoke coverage for interrupt/cancel from interactive runtime and TUI/webchat control surfaces.

## Rollout Notes
1. Implement structured timeout/recovery state in provider/runtime first.
2. Emit dual-path legacy + structured status during migration.
3. Update TUI and webchat consumers once runtime semantics are stable.
4. Remove legacy timeout-only UI assumptions after parity tests are green.
