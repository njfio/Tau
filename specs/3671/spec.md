# Spec: Issue #3671 - Add raw gateway payload tracing and reconcile TUI tool lifecycle state

Status: Implemented

## Problem Statement
The latest live `just tui` repro exposed two separate debugging/runtime defects.
First, gateway attempt tracing is still lossy: it preserves summarized prompts,
assistant output, and tool summaries, but it does not persist the send/receive
payloads needed to debug why a given attempt or retry behaved the way it did.
Second, the TUI tool panel treats tool start/completion as separate entries and
never reconciles them, so completed tools remain counted as active and the UI
misreports the real runtime state.

## Scope
In scope:
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `crates/tau-tui/src/interactive/gateway_client.rs`
- `crates/tau-tui/src/interactive/app.rs`
- `crates/tau-tui/src/interactive/tools.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
- `specs/3671/spec.md`
- `specs/3671/plan.md`
- `specs/3671/tasks.md`
- `specs/milestones/m334/index.md`

Out of scope:
- redesigning the provider adapter contract
- changing the Ralph-loop retry policy itself
- adding a new persistent database for trace storage

## Acceptance Criteria
### AC-1 Gateway tracing persists request/response payload evidence per attempt
Given a gateway OpenResponses turn executes one or more inner-loop attempts,
when the runtime persists attempt traces,
then each attempt record includes both outbound request context and inbound
response payload evidence sufficient to reconstruct what the gateway sent and
what the provider returned.

### AC-2 TUI tool lifecycle state reconciles starts with completions
Given the TUI receives streamed `response.tool_execution.started` and
`response.tool_execution.completed` frames for the same tool execution,
when the completion frame arrives,
then the existing running entry is updated to its terminal state instead of
leaving the tool counted as active forever.

### AC-3 Timeout/failure surfaces stay inspectable
Given an attempt times out or fails after some tool activity,
when the run is inspected through trace files or the TUI,
then the operator can see the request/response payload evidence plus a coherent
terminal tool lifecycle state without stale `Running` entries.

## Conformance Cases
- C-01 / AC-1 / Regression:
  a gateway attempt trace persists structured outbound request payload and
  inbound response payload fields for a retried action request.
- C-02 / AC-2 / Regression:
  TUI streamed completions with the same tool name reconcile by `tool_call_id`
  so only the matching running entry is updated and `active_count()` returns to
  zero after both calls complete.
- C-03 / AC-3 / Functional:
  a timeout/failure after tool activity leaves inspectable payload trace
  records and no stale running-tool state in the TUI reducer.

## Success Metrics / Observable Signals
- Fresh gateway mission iteration records show both `request_payload` and
  `response_payload` evidence instead of only lossy summaries.
- A live TUI session no longer shows `N active` after all streamed tool
  completions have been received.
- Gateway/TUI failures can be debugged from local traces without having to
  infer behavior from summary strings alone.

## Files To Touch
- `specs/3671/spec.md`
- `specs/3671/plan.md`
- `specs/3671/tasks.md`
- `specs/milestones/m334/index.md`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `crates/tau-tui/src/interactive/gateway_client.rs`
- `crates/tau-tui/src/interactive/app.rs`
- `crates/tau-tui/src/interactive/tools.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
