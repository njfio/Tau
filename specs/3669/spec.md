# Spec: Issue #3669 - Stream gateway Ralph-loop progress into the TUI

Status: Reviewed

## Problem Statement
The interactive TUI currently submits blocking non-streaming `/v1/responses`
requests. When the gateway Ralph loop is active, the TUI waits for a final JSON
response and shows no in-flight progress. Recent live repros prove the gateway
can execute tools and persist attempt traces while the TUI still shows
`Tools (0 active / 0 total)` and no incremental chat updates. That makes the
loop appear dead even when work is happening.

## Scope
In scope:
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `crates/tau-tui/src/interactive/gateway_client.rs`
- `crates/tau-tui/src/interactive/app.rs`
- `crates/tau-tui/src/interactive/chat.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
- `specs/3669/spec.md`
- `specs/3669/plan.md`
- `specs/3669/tasks.md`
- `specs/milestones/m334/index.md`

Out of scope:
- changing the verifier policy for when the outer loop stops
- redesigning the TUI layout
- changing provider prompt content beyond what is required for streaming

## Acceptance Criteria
### AC-1 TUI requests the streaming gateway path
Given the interactive TUI submits a prompt through the gateway client,
when it builds the `/v1/responses` request,
then it sends `stream: true` and reads the SSE response instead of waiting for a
single final JSON body.

### AC-2 Streamed text appears in the TUI during an in-flight turn
Given the gateway emits `response.output_text.delta` events,
when the TUI processes the streaming turn,
then the assistant transcript updates incrementally before final completion.

### AC-3 Tool lifecycle events appear in the TUI tools panel
Given the gateway executes tools during a streaming turn,
when tool execution starts and ends,
then the gateway emits tool lifecycle SSE frames and the TUI tools panel
reflects those events with running/completed status.

### AC-4 Final completion and errors remain compatible
Given the streaming turn completes or fails,
when the TUI finishes processing the SSE stream,
then it still records the final assistant output or a system error message and
returns to the correct agent state.

## Conformance Cases
- C-01 / AC-1 / Regression:
  the TUI gateway request body contains `"stream": true`.
- C-02 / AC-2 / Functional:
  a scripted SSE response with text deltas produces an assistant message in the
  TUI whose content is assembled incrementally and matches the completed text.
- C-03 / AC-3 / Functional:
  scripted tool start/end SSE frames produce visible tool entries with running
  and success status in the TUI.
- C-04 / AC-4 / Regression:
  a streaming error frame yields a system error message and sets the TUI status
  to `ERROR`.

## Success Metrics / Observable Signals
- Live `just tui` sessions visibly stream chat progress instead of appearing
  frozen until timeout.
- The TUI tools panel shows tool executions during gateway Ralph-loop activity.
- Timeouts and runtime failures surface as structured system messages instead of
  opaque transport disconnects.

## Files To Touch
- `specs/3669/spec.md`
- `specs/3669/plan.md`
- `specs/3669/tasks.md`
- `specs/milestones/m334/index.md`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `crates/tau-tui/src/interactive/gateway_client.rs`
- `crates/tau-tui/src/interactive/app.rs`
- `crates/tau-tui/src/interactive/chat.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
