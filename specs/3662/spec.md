# Spec: Issue #3662 - Bound no-tool gateway retry attempts so TUI does not time out first

Status: Implemented

## Problem Statement
The gateway Ralph loop correctly retries action-oriented requests when no tool
execution evidence is observed, but later retry attempts still inherit the full
turn timeout. In practice that allows a third no-tool attempt to run long
enough that the interactive TUI client times out first and reports a transport
error, even though the gateway has mission/verifier state and is still looping.

## Scope
In scope:
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/3662/spec.md`
- `specs/3662/plan.md`
- `specs/3662/tasks.md`
- `specs/milestones/m334/index.md`

Out of scope:
- changing launcher/runtime bootstrap behavior
- redesigning verifier semantics or retry-count policy
- changing TUI transport/UI rendering

## Acceptance Criteria
### AC-1 Later no-tool action retries use a bounded retry timeout
Given an action-oriented gateway request has already completed at least one
no-tool retry cycle,
when the next retry attempt begins,
then the gateway caps that retry attempt to a shorter bounded timeout instead
of reusing the full turn timeout.

### AC-2 Gateway fails closed before the client budget expires
Given a later no-tool retry attempt exceeds the bounded retry timeout,
when the gateway stops that attempt,
then it returns a gateway timeout/failure response before the interactive TUI
client-side request timeout is exhausted.

### AC-3 Mission state records the blocked timeout outcome
Given a no-tool retry attempt times out under the bounded retry budget,
when mission state is persisted,
then the mission is marked blocked with a timeout verifier record for the timed
out retry attempt.

### AC-4 Existing successful retry flow remains intact
Given an action-oriented request succeeds on a later retry by actually calling a
tool,
when the gateway runs the outer loop,
then the existing two-attempt success path still completes successfully.

## Conformance Cases
- C-01 / AC-1 / Regression:
  use a delayed fixture LLM client and verify the third no-tool retry attempt is
  cut short by the bounded retry timeout rather than the full turn timeout.
- C-02 / AC-2 / Regression:
  assert the gateway returns a timeout/server error response for the bounded
  retry attempt before the client-side request times out.
- C-03 / AC-3 / Regression:
  load persisted mission state after bounded retry timeout and verify the final
  verifier record is `gateway_timeout` and the mission status is blocked.
- C-04 / AC-4 / Regression:
  keep the existing successful no-tool-then-tool retry scenario green.

## Success Metrics / Observable Signals
- Interactive TUI action requests no longer sit until the client reports a
  transport timeout after repeated no-tool attempts
- Mission state preserves a server-side blocked reason instead of leaving the
  mission indefinitely running when the client gives up first
- Existing successful retry paths remain unchanged

## Files To Touch
- `specs/3662/spec.md`
- `specs/3662/plan.md`
- `specs/3662/tasks.md`
- `specs/milestones/m334/index.md`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
