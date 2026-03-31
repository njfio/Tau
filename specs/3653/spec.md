# Spec: Issue #3653 - Build Tau around a continuous Ralph loop with back-pressure, memory, and learning

Status: Reviewed

## Problem Statement
Tau already contains many of the pieces required for long-horizon autonomous
work: an inner LLM/tool loop, durable session state, action history, memory
stores, cortex bulletins, orchestration, and operator surfaces. The gap is that
these systems do not currently compose into a single continuous outer loop with
objective back-pressure. The model can still stop after a textual answer, while
learning and memory remain secondary instead of defining the default operating
mode. Tau needs a canonical outer supervisor loop that keeps working until
verifier-backed completion criteria are satisfied or a governed stop condition
is reached.

## Scope
In scope:
- define Tau's canonical outer Ralph loop architecture
- define how session, mission, memory, cortex, verifier, and learning state fit
  together
- define migration boundaries from current prompt/session semantics
- identify execution slices for follow-up implementation stories

Out of scope:
- full implementation of all loop slices in this epic
- replacing every existing entrypoint in one change
- uncontrolled self-modification without approval boundaries

## Acceptance Criteria
### AC-1 Tau has a single documented outer-loop operating model
Given Tau's multiple runtime surfaces,
when an operator asks how autonomous work progresses,
then there is one documented canonical loop describing execution, verification,
continuation, checkpointing, and completion.

### AC-2 The outer loop composes existing Tau subsystems
Given Tau's current session, memory, cortex, orchestration, and learning
subsystems,
when the architecture is defined,
then those subsystems are reused as loop inputs/outputs rather than replaced by
parallel abstractions.

### AC-3 Back-pressure and learning are first-class loop contracts
Given a long-running mission,
when execution iterates,
then verifier output, action history, distilled memory, and learning insights
are part of the continuation decision and next-turn context.

### AC-4 The epic breaks into executable follow-up slices
Given the size of the architecture change,
when the epic spec is complete,
then concrete follow-up stories/tasks exist for implementation without requiring
another blank-sheet architecture pass.

## Success Metrics / Observable Signals
- Tau can be described as "outer supervisor loop + inner agent/tool loop"
  without hand-waving over state ownership
- Existing Tau learning and memory infrastructure becomes part of the mainline
  product direction rather than optional side infrastructure
- The architecture supports operator-visible progress, pause/resume, and
  verifier-driven continuation

## Related Follow-up
- Story: `specs/3654/spec.md`
