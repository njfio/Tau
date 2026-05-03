# Spec: Issue #3752 - Define shared Tau Agent Harness mission contract

Status: Implemented

## Problem Statement

Tau has useful mission and Ralph-loop primitives, but the first durable mission
state is still gateway-local. The Tau Agent Harness lane needs mission to become
the top-level durable unit shared by core runtime, orchestration, coding-agent
execution, memory, skills, safety, and adapter surfaces.

This first slice establishes a shared mission contract without changing public
gateway `mission` response fields. Gateway remains compatible while exposing an
additive `harness_mission` / `harness_missions` projection built from shared Tau
Agent Harness primitives.

## Scope

In scope:

- define shared mission lifecycle/status, verifier, completion, proof, and
  checkpoint primitives in `tau-agent-core`
- reject invalid lifecycle transitions in the shared mission model
- add gateway adapter projection from existing `GatewayMissionState` into the
  shared mission snapshot
- expose additive shared mission projections on gateway mission list/detail
  responses while preserving existing `mission` fields
- document the ownership inversion from gateway-local mission truth to
  harness-owned mission truth
- validate with focused core and gateway tests

Out of scope:

- removing gateway mission persistence in this slice
- removing or renaming existing `/gateway/missions` JSON fields
- adding new dependencies
- adding dashboard/channel UI changes
- enabling source-code self-modification

## Acceptance Criteria

### AC-1 Shared mission primitives exist outside gateway

Given `tau-agent-core`,
when downstream crates need mission status, completion, verifier, proof, and
checkpoint concepts,
then they can use exported shared mission types without importing
`tau-gateway`.

### AC-2 Mission lifecycle rejects invalid transitions

Given a shared mission snapshot in a terminal state,
when code attempts to transition it back into an active state,
then the transition is rejected with a deterministic error and the mission
state remains unchanged.

### AC-3 Gateway mission state projects into the shared contract

Given an existing `GatewayMissionState`,
when the gateway adapter creates a shared mission snapshot,
then mission ID, session key, response ID, goal summary, status, iteration
count, latest verifier, latest completion, checkpoint/recovery state, and
tool-proof placeholders are preserved.

### AC-4 Gateway persistence and existing API fields are preserved

Given existing gateway mission persistence and endpoint tests,
when the shared mission projection is introduced,
then existing gateway mission state serialization and existing endpoint fields
remain compatible, and additive harness projections expose shared mission state.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Unit | `tau-agent-core` mission module | crate is compiled | mission types are exported from the public crate surface |
| C-02 | AC-2 | Unit | mission status `Completed` | transition to `Executing` is requested | transition fails and status remains `Completed` |
| C-03 | AC-3 | Conformance | gateway mission with verifier/completion/iteration data | projected to shared mission snapshot | core identity/status/proof fields match gateway state |
| C-04 | AC-4 | Regression | existing gateway mission runtime/list/detail tests | scoped gateway tests run | existing fields stay green and additive harness projection fields are present |

## Success Metrics / Observable Signals

- `tau-agent-core` owns the first shared mission contract.
- `tau-gateway` can project current mission supervisor state into shared mission
  snapshots.
- Gateway mission JSON preserves existing fields and adds shared harness
  projection fields in this slice.
- The ownership-inversion ADR exists and links this issue to the harness lane
  plan.
