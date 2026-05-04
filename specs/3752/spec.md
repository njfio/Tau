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
- require mission planning to record memory recall proof, either relevant hits
  with plan rationale or an explicit no-memory result
- write final and failure learning records through `tau-memory`
- attach curator review status to mission learning records

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

### AC-5 Mission plan DAG exposes executable readiness

Given a shared mission snapshot with plan nodes and dependencies,
when the harness asks for ready work,
then only pending nodes whose dependencies are completed or skipped are returned,
and invalid DAGs are rejected before execution.

### AC-6 Checkpoints preserve resumable mission work

Given an executing mission with unfinished plan nodes,
when the mission records a checkpoint,
then the checkpoint stores the pending plan node IDs and the recovery state can
refer to the last checkpoint without importing gateway runtime code.

### AC-7 Completion requires plan, verification, and learning proof

Given a mission that is about to complete,
when the harness evaluates completion readiness,
then incomplete plan nodes, non-passing verification gates, and missing final
learning output are reported as deterministic blockers.

### AC-8 Tool calls are attributable to mission and plan node

Given a mission tool execution record,
when it is added to the shared mission ledger,
then the record preserves mission ID, optional plan node ID, tool name, timing,
status, artifact links, and verification-gate links.

### AC-9 Tool budget exhaustion blocks autonomous tool execution

Given a mission tool budget with allowed tools and call/runtime/cost limits,
when a proposed tool call would exceed those limits,
then the core mission contract rejects the call with a deterministic budget
error before the ledger is mutated.

### AC-10 Completion reports include tool trace evidence

Given mission budget consumption,
when the harness evaluates completion readiness,
then missing ledger entries for consumed tool calls are reported as completion
blockers so final reports cannot claim tool execution without evidence.

### AC-11 Planning records memory recall proof

Given a mission planning step,
when the harness consults memory,
then the mission records either relevant memory hits with plan rationale and
plan-node links, or an explicit no-relevant-memory result.

### AC-12 Completion writes final learning through tau-memory

Given a mission with complete plan, verification, tool, and memory proof,
when final learning output is recorded,
then a typed mission learning record is written through `tau-memory` and the
mission references that record before completion can pass.

### AC-13 Failure recovery writes curator-queued learning

Given a blocked or recovering mission,
when failure learning is captured,
then the mission writes a failure learning record through `tau-memory` with root
cause, evidence, rollback guidance, and curator review status.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Unit | `tau-agent-core` mission module | crate is compiled | mission types are exported from the public crate surface |
| C-02 | AC-2 | Unit | mission status `Completed` | transition to `Executing` is requested | transition fails and status remains `Completed` |
| C-03 | AC-3 | Conformance | gateway mission with verifier/completion/iteration data | projected to shared mission snapshot | core identity/status/proof fields match gateway state |
| C-04 | AC-4 | Regression | existing gateway mission runtime/list/detail tests | scoped gateway tests run | existing fields stay green and additive harness projection fields are present |
| C-05 | AC-5 | Unit | mission plan DAG with pending/completed/skipped dependencies | ready node IDs are requested | only executable nodes are returned |
| C-06 | AC-5 | Unit | mission plan DAG with a missing dependency or cycle | DAG validation runs | deterministic validation errors are returned |
| C-07 | AC-6 | Unit | executing mission with unfinished plan nodes | checkpoint is recorded | checkpoint stores pending node IDs and mission enters checkpointed state |
| C-08 | AC-7 | Unit | mission with incomplete plan/verification/learning proof | completion readiness is evaluated | deterministic completion blockers are returned |
| C-09 | AC-8 | Unit | mission tool-call evidence with mission and plan-node IDs | evidence is recorded | ledger preserves attribution and links to artifacts/gates |
| C-10 | AC-9 | Unit | mission budget has one allowed call | second call or disallowed tool is proposed | budget error is returned and ledger does not change |
| C-11 | AC-10 | Unit | mission budget shows consumed tool calls without ledger evidence | completion readiness is evaluated | missing tool evidence blocker is returned |
| C-12 | AC-11 | Unit | mission planning finds a relevant memory hit | memory evidence is recorded | hit key, score, source event, rationale, plan-node links, and recall status are preserved |
| C-13 | AC-12 | Unit | mission has plan/verification/memory proof but no final learning | final learning is written | `tau-memory` stores a final learning record and completion unblocks |
| C-14 | AC-13 | Unit | mission failure learning is proposed before and after recovery state exists | failure learning write runs | missing recovery is rejected, then a curator-queued failure memory record is stored |

## Success Metrics / Observable Signals

- `tau-agent-core` owns the first shared mission contract.
- `tau-gateway` can project current mission supervisor state into shared mission
  snapshots.
- Gateway mission JSON preserves existing fields and adds shared harness
  projection fields in this slice.
- The ownership-inversion ADR exists and links this issue to the harness lane
  plan.
- Mission plan DAG readiness, checkpoint resume state, and completion blockers
  can be evaluated in `tau-agent-core` without importing gateway or dashboard
  code.
- Mission tool evidence and budget exhaustion can be evaluated in
  `tau-agent-core` before adapters write gateway/session projections.
- Mission memory recall proof and learning records can be evaluated in
  `tau-agent-core` while persisting final/failure learning through the existing
  `tau-memory` store API.
