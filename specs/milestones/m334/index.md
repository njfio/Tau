# M334 - Tau Ralph loop supervisor architecture

Status: Active

## Context
Tau should not behave like a one-shot assistant that stops when the model emits
 text. The target operating model is a Tau-native Ralph loop: a continuous
 outer supervisor repeatedly drives the existing inner agent/tool loop until
 verifier-backed completion criteria are met, while reusing Tau's existing
 session, memory, cortex, orchestration, and learning surfaces. The system
 should improve over time by feeding loop outcomes back into action history,
 distilled memory, recovery hints, and operator-visible bulletins.

## Issue Hierarchy
- Epic: [#3653](https://github.com/njfio/Tau/issues/3653) Build Tau around a
  continuous Ralph loop with back-pressure, memory, and learning
- Story: [#3654](https://github.com/njfio/Tau/issues/3654) Define the governed
  Tau Ralph supervisor loop across gateway, session, memory, and learning
- Story: [#3655](https://github.com/njfio/Tau/issues/3655) Implement the first
  Tau Ralph supervisor loop slice for gateway missions
- Story: [#3656](https://github.com/njfio/Tau/issues/3656) Wire gateway mission
  loop into Tau action history and learning insights
- Story: [#3657](https://github.com/njfio/Tau/issues/3657) Add structured
  verifier bundles and first back-pressure adapters to gateway missions
- Story: [#3658](https://github.com/njfio/Tau/issues/3658) Add explicit mission
  completion and checkpoint semantics to the gateway Ralph loop
- Story: [#3659](https://github.com/njfio/Tau/issues/3659) Expose gateway
  mission inspection and TUI resume controls for checkpointed Ralph-loop
  missions

## Scope
- Define an explicit outer supervisor/mission loop above current prompt turns
- Use verifier back-pressure rather than assistant-declared completion
- Reuse `tau-session`, `tau-memory`, cortex, and `tau-orchestrator` as loop
  state and feedback mechanisms
- Expose loop progress, checkpoints, resume, and learning to gateway/TUI
- Keep existing one-shot/session flows on a compatibility path during migration

## Exit Criteria
- Tau has a documented canonical outer loop model with explicit continuation and
  completion semantics
- Mission/session/memory/learning surfaces are composed into one inspectable
  work object instead of separate subsystems
- Operator surfaces can see progress, back-pressure, checkpoints, and learning
  state from the same runtime object
- The implementation plan is sliced into executable follow-up stories with
  bounded verification gates

## Delivery Notes
- Compose existing Tau primitives before adding new subsystems
- Favor explicit completion and checkpoint tools over heuristic stopping alone
- Treat back-pressure, learning, and operator visibility as first-class runtime
  contracts
