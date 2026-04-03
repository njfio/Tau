# Spec: Issue #3670 - Recover read-only gateway timeouts and clear in-flight tools

Status: Implemented

## Problem Statement
The gateway/TUI now shows live Ralph-loop progress, but action turns can still
spend their entire attempt budget in read-only inspection, hit the turn
timeout, and fail terminally instead of using the outer retry path to push the
model toward mutation. When the timeout happens during a tool execution, the
gateway also leaves that tool without a terminal SSE frame, so the TUI can
render it as perpetually `RUNNING`.

## Scope
In scope:
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/3670/spec.md`
- `specs/3670/plan.md`
- `specs/3670/tasks.md`
- `specs/milestones/m334/index.md`

Out of scope:
- changing provider prompt wording outside timeout retry feedback
- redesigning the verifier bundle model
- changing the TUI rendering layout

## Acceptance Criteria
### AC-1 Read-only action timeouts retry through the outer loop
Given an action-oriented request executes read-only tool calls but does not yet
produce mutation evidence,
when the current attempt times out,
then the gateway records the timeout attempt and re-enters the bounded outer
retry path instead of terminally blocking on the first timeout.

### AC-2 Retry exhaustion still fails closed
Given repeated read-only timeout recovery attempts still never produce mutation
or validation evidence,
when the retry budget is exhausted,
then the gateway fails closed with a structured error instead of looping
forever.

### AC-3 Pending tools receive terminal timeout/failure cleanup
Given an attempt times out or aborts while a tool execution start has been
observed without a matching end event,
when the gateway finalizes the failed attempt,
then it emits a terminal tool completion event with timeout/failure semantics
and persists matching trace/history evidence so the UI no longer shows the tool
as still running.

## Conformance Cases
- C-01 / AC-1 / Regression:
  a scripted action attempt performs read-only tool work, times out, then
  retries and succeeds with a mutating tool execution.
- C-02 / AC-2 / Regression:
  repeated timed-out action attempts still fail with a structured terminal error
  once the retry budget is exhausted.
- C-03 / AC-3 / Functional:
  a streaming request that times out during an in-flight tool emits both
  `response.tool_execution.started` and a terminal
  `response.tool_execution.completed` frame with timeout semantics.

## Success Metrics / Observable Signals
- The gateway no longer hard-fails the first time an action attempt times out
  after read-only inspection work.
- The TUI tools panel does not retain a stale `RUNNING` tool after a timeout.
- Attempt traces and action-history records preserve enough timeout cleanup
  evidence to debug future regressions.
