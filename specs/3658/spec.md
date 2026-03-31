# Spec: Issue #3658 - Add explicit mission completion and checkpoint semantics to the gateway Ralph loop

Status: Reviewed

## Problem Statement
The gateway Ralph loop now has mission persistence, learned context, and
structured verifier bundles, but it still infers stop conditions entirely from
runtime/verifier state. There is no first-class way for the agent to declare
"mission complete", "checkpoint this partial progress", or "blocked for a real
reason" as part of the outer-loop contract. Tau needs an explicit completion
primitive so mission state can distinguish verified completion from checkpointed
or intentionally blocked outcomes without depending only on assistant text.

## Scope
In scope:
- `crates/tau-gateway/src/gateway_openresponses/mission_completion_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/3658/spec.md`
- `specs/3658/plan.md`
- `specs/3658/tasks.md`
- `specs/milestones/m334/index.md`

Out of scope:
- changing the public OpenResponses HTTP schema
- requiring every existing action flow to use explicit completion immediately
- operator UI for resume/checkpoint management
- cross-crate checkpoint orchestration in `tau-session` or `tau-orchestrator`

## Acceptance Criteria
### AC-1 Gateway mission loop exposes an explicit completion tool
Given a gateway-backed mission request,
when the model needs to finish, checkpoint, or explicitly block the mission,
then the agent has a `complete_task` tool that records a structured status of
`success`, `partial`, or `blocked` with a summary.

### AC-2 Partial completion persists checkpointed mission state
Given a gateway mission request that calls `complete_task` with `partial`,
when the request finishes,
then the mission state is persisted as checkpointed with the recorded summary
and latest completion signal instead of being marked completed or treated as a
runtime error.

### AC-3 Explicit blocked completion persists mission-blocked state without a runtime failure
Given a gateway mission request that calls `complete_task` with `blocked`,
when the request finishes,
then the mission state is persisted as blocked from an explicit completion
signal and the response returns the completion summary without treating the
outcome as a gateway runtime error.

### AC-4 Explicit success completion is persisted and compatible with verifier-driven completion
Given a gateway mission request whose verifier requirements pass and that calls
`complete_task` with `success`,
when the request finishes,
then the mission state records the explicit completion signal and completes
successfully without breaking the existing verifier-driven compatibility path.

## Conformance Cases
- C-01 / AC-1 / Unit:
  parse a successful `complete_task` tool trace and verify the gateway extracts
  the expected status and summary.
- C-02 / AC-2 / Regression:
  run a mission that calls `complete_task` with `partial` and verify the
  persisted mission state is `checkpointed`.
- C-03 / AC-3 / Regression:
  run a mission that calls `complete_task` with `blocked` and verify the HTTP
  response is success-shaped while the mission state is `blocked`.
- C-04 / AC-4 / Regression:
  run a mission that satisfies verifier requirements and calls `complete_task`
  with `success`, then verify the mission state stores the explicit completion
  signal and remains `completed`.

## Success Metrics / Observable Signals
- Mission history distinguishes checkpointed, explicitly blocked, and completed
  outcomes
- The gateway has a first-class completion primitive instead of relying only on
  inferred stop conditions
- Existing verifier-driven flows remain compatible while the explicit tool is
  introduced

## Files To Touch
- `specs/3658/spec.md`
- `specs/3658/plan.md`
- `specs/3658/tasks.md`
- `specs/milestones/m334/index.md`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/mission_completion_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
