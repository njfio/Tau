---
title: feat: Introduce Governed Mission Mode
type: feat
status: active
date: 2026-03-23
---

# feat: Introduce Governed Mission Mode

## Overview

Add a first-class `Mission` operating model that becomes the canonical unit of work across Tau. A mission should bind together goal, acceptance criteria, execution plan, approvals, budget, recovery state, artifacts, background execution, and resumability so the system operates on governed work items rather than loosely related prompts and sessions.

## Problem Statement / Motivation

Tau already contains many of the building blocks required for autonomy:

- structured planning and execution scaffolding in `tau-orchestrator`
- session and memory surfaces in gateway/runtime
- recovery and escalation primitives in `tau-agent-core`
- operator-facing shells in `tau-tui`

The current gap is composition. Sessions, plans, memory, and operator controls exist, but they are not represented as a single inspectable work object. This makes the runtime capable but harder to trust, resume, audit, or steer as a coherent mission.

## Proposed Solution

Introduce a mission domain model and lifecycle that sits above existing prompt/session behavior:

1. Add a versioned mission schema with:
   - `mission_id`
   - human goal and acceptance criteria
   - plan reference and execution progress
   - approval checkpoints and policy decisions
   - budget and runtime limits
   - artifact pointers and checkpoint metadata
   - recovery status and escalation history
2. Persist mission state independently from raw chat/session logs while linking back to them.
3. Expose mission-oriented APIs in the gateway and internal runtime bridges.
4. Make TUI and dashboard operate primarily on mission state rather than only prompt/session state.
5. Preserve compatibility by allowing existing session-based flows to run in an implicit single-mission mode during migration.

## Implementation Phases

### Phase 1: Mission Domain Model

- Define `Mission`, `MissionStatus`, `ApprovalCheckpoint`, `MissionBudget`, and `MissionArtifactRef`.
- Choose persistence location and schema versioning strategy.
- Add serialization and migration tests for persisted mission state.

### Phase 2: Runtime Wiring

- Attach mission context to plan-first execution and session runtime flows.
- Record mission lifecycle transitions around planner, executor, delegated steps, and recovery events.
- Link mission state to memory/session keys exposed via gateway surfaces.

### Phase 3: Control Surfaces

- Add gateway endpoints for mission create/read/update/control operations.
- Add TUI and dashboard mission summaries, mission detail views, and checkpoint controls.
- Add operator-safe resume/cancel/retry semantics.

### Phase 4: Migration and Compatibility

- Support implicit mission creation for legacy prompt flows.
- Add migration/compatibility documentation and artifact readers.
- Keep existing session APIs working while mapping them to missions internally.

## Technical Considerations

- `tau-orchestrator` already has `StructuredPlan` and execution reporting, but they need a parent lifecycle object rather than free-standing plan use.
- `tau-coding-agent` gateway surfaces already expose session and memory administration; mission APIs should be layered beside these, not bolted onto ad hoc endpoint growth.
- Mission persistence must be fail-closed and resumable. Partial writes cannot orphan approval state or make recovery ambiguous.
- Approval semantics need explicit policy boundaries. A mission must never silently auto-approve actions that previously required operator intent.

## System-Wide Impact

### Interaction Graph

Operator action in TUI or dashboard starts or resumes a mission, which invokes planner/executor flow, which writes mission progress, which updates gateway state and artifacts, which feeds operator surfaces and recovery controls.

### Error & Failure Propagation

Planner failure, delegated-step failure, recovery escalation, and budget exhaustion must all transition mission state explicitly. Failures cannot remain hidden in per-turn chat history only.

### State Lifecycle Risks

Mission state, session state, memory state, and artifact checkpoints can drift if they are updated independently. The plan should include idempotent mission writes and explicit cleanup of abandoned or cancelled missions.

### API Surface Parity

Equivalent flows must exist across:

- CLI/runtime startup flows
- gateway HTTP surfaces
- TUI mission operations
- dashboard mission operations

### Integration Test Scenarios

- Start a mission, approve a checkpoint, resume after restart, and verify mission/session/memory linkage remains intact.
- Fail a delegated step, trigger recovery, and confirm mission state records both failure and next action.
- Run a legacy prompt-only flow and confirm implicit mission creation keeps old UX intact.

## SpecFlow Notes

### Primary Operator Flows

1. Create a mission from a goal and acceptance criteria.
2. Review the generated plan before execution.
3. Approve, deny, or edit checkpoints during execution.
4. Pause or resume a mission after crash or operator interruption.
5. Inspect artifacts, memory, and recovery actions for a specific mission.

### Important Gaps to Resolve in Implementation

- Whether approvals are step-scoped, tool-scoped, or policy-scoped.
- Whether a mission owns one session lineage or can span multiple branches/sessions.
- Whether mission creation is explicit by default in CLI mode or implicit unless the operator chooses otherwise.

### Default Planning Assumptions

- A mission can own multiple execution attempts but one primary goal.
- Approval history is durable and audit-visible.
- Existing session-key behavior remains supported during migration.

## Acceptance Criteria

- [ ] A versioned mission data model exists and is persisted with backward-compatible migration behavior.
- [ ] Planner/executor flows can attach to a `mission_id` and update mission lifecycle state as work progresses.
- [ ] Gateway surfaces expose mission list/detail/control endpoints or equivalent mission-capable control paths.
- [ ] Mission state records approvals, checkpoints, recovery actions, and budget/utilization signals.
- [ ] Existing session-oriented flows remain functional through an explicit compatibility path.
- [ ] TUI and dashboard can both render mission summary information from the same mission model.
- [ ] Integration tests cover mission resume, failure recovery, and approval checkpoint transitions.
- [ ] Docs describe mission lifecycle, operator expectations, and compatibility boundaries.

## Success Metrics

- Operators can resume interrupted work from mission state without reconstructing context from raw chat logs.
- Mission-related support/debugging flows use a single identifier instead of combining session, memory, and artifact paths manually.
- Approval and recovery actions are observable in both machine-readable state and operator-facing surfaces.

## Dependencies & Risks

### Dependencies

- [ ] Mission state should align with `StructuredPlan` and `PlanExecutionReport`.
- [ ] Gateway and operator surfaces need a shared schema or adapter layer.
- [ ] TUI Mission Control and Unified Runtime State Backbone plans are natural companion work.

### Risks

- Mission state can become an oversized dumping ground if ownership boundaries stay vague.
- Compatibility migration may create dual-write complexity across sessions and missions.
- Approval design can easily regress safety if defaults are not explicit and conservative.

## Sources & References

- Source ideation: `docs/ideation/2026-03-23-autonomous-operator-mission-control-ideation.md`
- Repo contract: `AGENTS.md`
- Planning primitives: `crates/tau-orchestrator/src/plan.rs:12`
- Execution reporting: `crates/tau-orchestrator/src/plan_executor.rs:12`
- Plan-first bridge: `crates/tau-coding-agent/src/orchestrator_bridge.rs:144`
- Runtime hooks: `crates/tau-coding-agent/src/runtime_loop.rs:1025`
- Gateway session and memory surfaces: `docs/tau-coding-agent/code-map.md:96`
- Gateway API references: `docs/guides/gateway-api-reference.md:58`
- TUI shell state surface: `crates/tau-tui/src/lib.rs:572`
- Existing improvement roadmap: `docs/AGENT_IMPROVEMENTS_PLAN.md`
