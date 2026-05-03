# ADR-009: Move Mission Ownership Into the Tau Agent Harness

Status: Accepted
Date: 2026-05-03
Related:
- `docs/plans/2026-05-03-001-feat-tau-agent-harness-lane-plan.md`
- `specs/3752/spec.md`
- `specs/milestones/m334/index.md`

## Context

The first Ralph-loop mission implementation made gateway state the visible
operator path. That let Tau expose checkpoints, blocked outcomes, completion
signals, and verifier feedback quickly, but it also made gateway look like the
center of mission truth.

The Agent Harness product lane needs mission to be the durable unit across
coding-agent execution, orchestration, sessions, memory, tools, skills, safety,
and adapters. Gateway, channel, dashboard, and UI surfaces should start,
inspect, approve, and render missions, but they should not own canonical mission
state machines.

## Decision

Mission ownership moves to shared Tau Agent Harness primitives, beginning in
`tau-agent-core`.

The first implementation slice will:

- add shared mission lifecycle, verifier, completion, proof, checkpoint,
  artifact, and learning-output types in `tau-agent-core`
- keep gateway persistence and API payloads compatible
- add gateway projection from current `GatewayMissionState` into shared mission
  snapshots through additive `harness_mission` / `harness_missions` response
  fields
- defer removal of gateway-local persistence until compatibility is proven by
  tests and adapter migration work

## Consequences

Positive:

- Mission identity and lifecycle become reusable outside HTTP gateway code.
- Gateway/channel/dashboard surfaces can become adapters over a shared mission
  contract.
- Future plan DAG, tool-budget, memory, verification, checkpoint, and learning
  work has one durable top-level object.

Tradeoffs:

- There will be a temporary dual-model period while gateway state projects into
  shared mission snapshots.
- The shared model must stay conservative enough to avoid freezing incomplete
  gateway-specific assumptions into core.
- Existing mission JSON compatibility must remain a hard regression gate.

## Guardrails

- Do not remove or rename existing `/gateway/missions` fields in the first
  slice; shared mission views must be additive.
- Do not move self-improvement into source-code auto-apply.
- Do not let adapters infer mission success from assistant prose when verifier
  or completion state is available.
- Do not add external dependencies for the mission contract.
