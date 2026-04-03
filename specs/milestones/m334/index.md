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
- Task: [#3661](https://github.com/njfio/Tau/issues/3661) Recycle stale
  tau-unified runtime after repo/runtime fingerprint changes
- Task: [#3662](https://github.com/njfio/Tau/issues/3662) Bound no-tool gateway
  retry attempts so the TUI does not time out first
- Task: [#3663](https://github.com/njfio/Tau/issues/3663) Persist gateway
  OpenResponses attempt payload traces for Ralph-loop debugging
- Task: [#3664](https://github.com/njfio/Tau/issues/3664) Align tau-unified
  request timeout with CLI provider backend timeouts
- Task: [#3665](https://github.com/njfio/Tau/issues/3665) Keep TUI client
  timeout above gateway runtime and provider budgets
- Task: [#3666](https://github.com/njfio/Tau/issues/3666) Teach CLI provider
  adapters to emit textual tool-call payloads
- Task: [#3667](https://github.com/njfio/Tau/issues/3667) Align tau-unified
  launcher with gateway turn timeout budget
- Task: [#3668](https://github.com/njfio/Tau/issues/3668) Isolate CLI provider
  backends from repo context bleed
- Task: [#3669](https://github.com/njfio/Tau/issues/3669) Stream gateway
  response and tool events into the TUI
- Task: [#3670](https://github.com/njfio/Tau/issues/3670) Recover read-only
  gateway timeouts and clear in-flight tools
- Task: [#3671](https://github.com/njfio/Tau/issues/3671) Add raw gateway
  payload tracing and reconcile TUI tool lifecycle state
- Task: [#3672](https://github.com/njfio/Tau/issues/3672) Break read-only
  timeout spirals in gateway action retries
- Story: [#3673](https://github.com/njfio/Tau/issues/3673) Force tool-required
  retry turns in Ralph-loop recovery
- Task: [#3674](https://github.com/njfio/Tau/issues/3674) Cut off read-only
  exploration spirals and widen mutation recovery budget
- Task: [#3675](https://github.com/njfio/Tau/issues/3675) Force concrete
  mutating tool choice on Ralph-loop recovery retries
- Task: [#3676](https://github.com/njfio/Tau/issues/3676) Add direct OpenAI
  Responses transport with experimental oauth/session mode
- Task: [#3725](https://github.com/njfio/Tau/issues/3725) Define tranche-one
  autonomy benchmark contract and task set
- Task: [#3726](https://github.com/njfio/Tau/issues/3726) Add schema and
  validator for the tranche-one autonomy benchmark contract

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
