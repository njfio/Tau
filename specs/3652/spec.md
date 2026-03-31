# Spec: Issue #3652 - Retry mutating gateway turns when the model promises work without using tools

Status: Reviewed

## Problem Statement
The current gateway fix for zero-tool mutating turns fails closed, but it still
does not create a recovery loop. When the model answers an action-oriented
request with a plain-text promise and zero tool executions, the gateway stops
after that single attempt and returns an error. For interactive coding/build
flows, the gateway should instead feed corrective feedback back into the agent,
retry a bounded number of times, and only fail closed after those retries are
exhausted. Failed-attempt assistant text must not leak into the final visible
reply.

## Scope
In scope:
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/3652/spec.md`
- `specs/3652/plan.md`
- `specs/3652/tasks.md`

Out of scope:
- changing the agent-core turn loop contract
- unbounded autonomous retries
- changing non-action conversational request behavior

## Acceptance Criteria
### AC-1 Gateway retries zero-tool mutating turns with corrective feedback
Given a gateway-backed OpenResponses request whose prompt is action-oriented and
mutating,
when the first attempt completes with zero tool execution evidence,
then the gateway appends corrective feedback and retries the turn instead of
stopping immediately.

### AC-2 Retry success returns only the successful attempt output
Given a mutating action request whose first attempt is a plain-text promise and
whose retry performs at least one tool execution,
when the retry succeeds,
then the gateway returns a completed response, preserves the tool side effects,
and does not include the failed-attempt assistant promise in the final output.

### AC-3 Retry exhaustion still fails closed
Given a mutating action request whose bounded retry attempts all complete with
zero tool execution evidence,
when the retry budget is exhausted,
then the gateway responds with a non-success error explaining that action
retries were exhausted without tool execution evidence.

## Conformance Cases
- C-01 / AC-1, AC-2 / Regression:
  first scripted reply is plain assistant text, the next attempt emits a real
  tool call and final assistant text, and the request completes successfully
  after the corrective retry.
- C-02 / AC-2 / Regression:
  after retry success, the final response text excludes the failed-attempt
  assistant promise and only reflects the successful attempt.
- C-03 / AC-3 / Regression:
  every scripted reply is plain assistant text with zero tool executions, and
  the request fails with a retry-exhaustion gateway error after the bounded
  retry budget is consumed.

## Success Metrics / Observable Signals
- Action-oriented interactive requests no longer stop after a single no-op
  assistant promise
- The gateway can recover automatically when the next corrective turn uses tools
- Misleading failed-attempt assistant text does not appear in the successful
  output returned to the operator
