# Spec: Cut Off Read-Only Exploration Spirals And Widen Mutation Recovery Budget

Status: Reviewed

## Problem Statement

Gateway Ralph-loop missions for mutating requests can still spend the entire
first attempt budget in successful read-only tool execution, then enter a
mutation-recovery retry that receives too little time to emit the first tool
call. This leaves the operator with another timeout even though verifier
back-pressure and retry prompting are already in place.

## Acceptance Criteria

- AC-1: For mutation-required requests, the gateway detects repeated successful
  read-only tool executions with no mutation evidence and terminates the active
  attempt early via cooperative cancellation.
- AC-2: Early read-only cancellation is treated as verifier-driven `continue`
  behavior, not a terminal runtime failure, and the mission proceeds into the
  next retry.
- AC-3: Mutation-recovery retries receive a materially larger timeout budget
  than the current 18s-style floor so the model has a realistic chance to emit
  the first mutating tool call.
- AC-4: Regression coverage proves the gateway both cuts off read-only spirals
  and allocates the wider retry budget.

## Scope

In scope:
- Gateway attempt-local read-only saturation detection
- Cooperative cancellation on repeated read-only tool execution
- Timeout policy adjustment for mutation-recovery retries
- Regression tests for the new gateway behavior

Out of scope:
- TUI UI changes
- Provider adapter contract changes beyond already-landed `ToolChoice::Required`
- New tool surfaces or model selection changes

## Conformance Cases

- C-01 / AC-1, AC-2
  Given a mutating request whose first attempt performs only repeated successful
  `ls`/`read` tools, when the configured read-only saturation threshold is hit,
  then the gateway cancels the attempt early and records a `continue` verifier
  outcome instead of burning the full turn timeout.

- C-02 / AC-3
  Given a mutation-recovery retry after a read-only saturated attempt, when the
  gateway computes the retry timeout, then the timeout floor is larger than the
  old short recovery budget and is reflected in the timeout/error path.

- C-03 / AC-4
  Given the live Phaser-style repro shape, when the gateway enters retry after a
  read-only attempt, then regression tests assert both early cutoff semantics
  and the widened retry budget.

## Success Signals

- Attempt traces stop showing 180s read-only discovery spirals before retry.
- Mutation-recovery retries no longer fail solely because they were capped to a
  very short timeout budget.
