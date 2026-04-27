# Spec: Issue #3654 - Define the governed Tau Ralph supervisor loop across gateway, session, memory, and learning

Status: Reviewed

## Problem Statement
Current Tau flows are still centered on individual prompts and turns. Even when
the model has tools, the runtime can terminate after a single attempt or after
bounded local retries, because the system lacks a first-class outer supervisor
loop that owns the mission, verifier feedback, checkpoint state, and learning
cycle. Tau already has the pieces to do better:

- `tau-session` for durable lineage, branching, undo/redo, and resume
- `tau-memory` action history and durable memory records
- cortex bulletin + observer surfaces for cross-session/operator context
- `tau-orchestrator` for plans and execution reporting
- gateway/TUI operator shells

The missing piece is composition: one governed outer loop that keeps re-entering
the inner agent/tool loop until verifier-backed completion criteria are
satisfied or a governed stop condition is reached, while learning from each
iteration.

## Scope
In scope:
- define the outer Tau supervisor/mission loop contract
- define loop state ownership across mission, session, memory, and learning
- define verifier/back-pressure integration and continuation semantics
- define operator-visible checkpoint, resume, blocked, and completion semantics
- define how learning/memory signals are injected back into subsequent
  iterations

Out of scope:
- implementing every verifier adapter
- replacing existing gateway/TUI code paths immediately
- full self-modifying code application without approval gates

## Acceptance Criteria
### AC-1 Outer supervisor loop governs continuation
Given an operator mission,
when Tau executes work,
then an outer supervisor owns the cycle
`goal -> active task -> inner tool loop -> verifier/back-pressure -> continue/replan/checkpoint/complete`
and assistant text alone cannot declare mission completion.

### AC-2 Loop state reuses existing Tau persistence surfaces
Given Tau's existing runtime components,
when the outer loop persists state,
then it reuses durable session lineage, memory/action history, and mission-style
checkpointing rather than creating disconnected per-feature state.

### AC-3 Learning and memory are integrated into each iteration
Given repeated supervisor iterations,
when one iteration finishes,
then the runtime records action/session outcomes, distills memory/insights, and
injects the relevant learned context into subsequent iterations.

### AC-4 Operator surfaces can inspect and steer the loop
Given the gateway/TUI operator experience,
when work is in progress or blocked,
then operators can inspect current objective, verifier status, checkpoint state,
latest learning/back-pressure signals, and can pause/resume/retry or approve
gated actions.

### AC-5 Migration keeps current prompt/session flows viable
Given existing prompt-driven entrypoints,
when the supervisor architecture lands,
then legacy flows still operate through an explicit compatibility or implicit
single-mission path while the new loop becomes the preferred model.

## Conformance Cases
- C-01 / AC-1 / Architecture:
  a build mission runs an inner turn, a verifier reports failing tests, and the
  outer loop feeds that back into the next iteration instead of stopping on the
  assistant's text.
- C-02 / AC-2 / Architecture:
  an interrupted mission can be resumed from durable mission/session checkpoint
  state without reconstructing context from chat text alone.
- C-03 / AC-3 / Architecture:
  repeated failures on a tool produce action-history and learning signals that
  appear in the next iteration context and in cortex/operator summaries.
- C-03a / AC-3 / Gateway learning handoff:
  checkpointed and blocked `complete_task` outcomes are written to the existing
  `tau-memory` action-history store with session and mission identifiers so a
  later gateway iteration can inject the learned outcome through the learning
  bulletin.
- C-04 / AC-4 / Architecture:
  an operator can inspect current mission progress, see the active verifier
  failure, and choose pause/resume/approve/retry from gateway or TUI surfaces.
- C-04a / AC-4 / Gateway streaming:
  a streamed `complete_task(status="partial")` produces a
  `response.operator_turn_state.snapshot` with a `mission.checkpointed` event,
  while a streamed `complete_task(status="blocked")` produces a blocked
  operator snapshot with `mission_completion_blocked` error context, and both
  still emit legacy `response.completed` compatibility frames.
- C-05 / AC-5 / Compatibility:
  a legacy prompt-only flow runs as an implicit single-mission loop without
  breaking existing session semantics.

## Success Metrics / Observable Signals
- Tau can keep working until verifiers pass instead of until the model says
  "done"
- Operators can point to one identifier/work object for a running mission and
  inspect its plan, progress, checkpoints, and learning state
- Learning surfaces such as action history, failure patterns, and cortex
  bulletins actively steer later iterations

## Files To Touch
- `specs/milestones/m334/index.md`
- `specs/3653/spec.md`
- `specs/3654/spec.md`
- `specs/3654/plan.md`
- `specs/3654/tasks.md`
