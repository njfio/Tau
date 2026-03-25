---
title: refactor: Unify Runtime State Backbone
type: refactor
status: active
date: 2026-03-23
---

# refactor: Unify Runtime State Backbone

## Overview

Define and adopt a canonical runtime state and event backbone for missions, plans, tool runs, approvals, alerts, recovery, budgets, and background activity. Use that backbone as the shared source for TUI, dashboard, gateway inspection, and persisted artifacts.

## Problem Statement / Motivation

Tau already emits and stores useful state in many places: dashboard JSON files, runtime event logs, action audits, session state, memory state, orchestrator traces, and interactive runtime output. Those surfaces are individually useful but collectively fragmented. Product surfaces end up reading different artifacts, exposing different slices of truth, and drifting in behavior.

## Proposed Solution

Create a single, versioned runtime state model with these properties:

1. One canonical schema for operator-visible runtime state.
2. An append-only event stream plus derived current-state snapshot.
3. Explicit domain sections for:
   - mission status
   - plan execution state
   - tool execution state
   - approval and policy state
   - memory and rationale summaries
   - recovery and alert state
   - budget / cost / degraded-mode signals
4. Clear producer/consumer boundaries so runtime code publishes events and operator surfaces consume derived state.
5. Compatibility adapters for legacy dashboard artifact files and current TUI live-shell readers during migration.

## Implementation Phases

### Phase 1: Inventory and Schema Definition

- Inventory all existing operator-facing state artifacts and endpoints.
- Define canonical event and snapshot schemas.
- Decide ownership boundaries between gateway, coding-agent runtime, orchestrator, and dashboard consumers.

### Phase 2: Producers

- Update runtime components to emit canonical events for plan, tool, memory, approval, and recovery transitions.
- Derive a canonical snapshot from those events.
- Add schema compatibility tests and snapshot replay tests.

### Phase 3: Consumers

- Migrate TUI live-shell, interactive mode, and dashboard views to consume the canonical snapshot/events.
- Maintain adapter reads from legacy artifacts during cutover.
- Expose inspection/debug endpoints over gateway where needed.

### Phase 4: Cleanup

- Deprecate duplicate artifact formats that are no longer needed.
- Update docs, runbooks, and verification scripts to point at the unified backbone.
- Add drift detection so consumers do not silently diverge again.

## Technical Considerations

- The backbone must be cheap to publish and read; operator state cannot require expensive recomputation on every refresh.
- Event schema versioning is mandatory because the repo already relies on persisted artifacts and deterministic replay patterns.
- Avoid mixing raw model transcript storage with operator-grade state. The backbone should summarize what operators need while linking back to deeper artifacts as needed.
- The design should work for both local file-backed state and future remote/control-plane usage.

## System-Wide Impact

### Interaction Graph

Runtime emits canonical events, state reducer builds snapshot, gateway exposes snapshot/events, TUI and dashboard render from shared state, operator actions flow back through runtime control paths, and resulting state transitions are published back into the backbone.

### Error & Failure Propagation

Errors in event publication, snapshot derivation, or consumer decoding need explicit reason codes and degraded-mode fallbacks. A broken state pipeline should degrade visibly, not silently.

### State Lifecycle Risks

Dual-write periods can introduce state skew between legacy artifacts and the new backbone. Cutover should favor canonical-state generation with compatibility readers, not long-lived duplicate sources of truth.

### API Surface Parity

Gateway inspection routes, TUI live-shell readers, and dashboard state consumers should all read the same semantic model even if transport/formats differ.

### Integration Test Scenarios

- Publish a sequence of runtime events and verify TUI and dashboard derive the same current state.
- Replay older artifact fixtures and verify compatibility adapters preserve expected operator summaries.
- Simulate malformed or missing event chunks and verify degradation is reason-coded and observable.

## SpecFlow Notes

### Primary Operator Flows

1. Inspect current runtime status from any operator surface and see the same answer.
2. Follow a mission from event history to current state without reconstructing it manually.
3. Diagnose degraded state using shared alert and reason-code semantics.

### Important Gaps to Resolve in Implementation

- Which state belongs in the canonical snapshot versus linked detail artifacts.
- Whether plan/tool/memory/rationale data are embedded or referenced.
- How much historical retention is required for local operator workflows.

### Default Planning Assumptions

- Canonical state consists of append-only events plus a derived snapshot.
- Consumers should prefer reading the new snapshot over reconstructing from old artifact sets.
- Legacy artifact compatibility is temporary, not permanent architecture.

## Acceptance Criteria

- [ ] A versioned canonical runtime state schema exists for operator-facing state.
- [ ] Runtime producers emit canonical events for plan, tool, approval, alert, recovery, and budget transitions.
- [ ] A reducer or equivalent derivation path produces a current-state snapshot from canonical events.
- [ ] TUI and dashboard can consume the canonical snapshot or shared adapter layer.
- [ ] Gateway inspection surfaces expose canonical state and/or event history.
- [ ] Compatibility tests cover legacy artifact replay and new canonical schema migration.
- [ ] Documentation and runbooks point to the canonical backbone as the primary state source.

## Success Metrics

- Operator surfaces no longer need separate artifact-specific logic to answer core status questions.
- Debugging runtime behavior starts from one canonical state source instead of multiple ad hoc logs.
- State drift defects between dashboard and TUI become rare and easy to diagnose.

## Dependencies & Risks

### Dependencies

- [ ] Mission domain design from Governed Mission Mode.
- [ ] Producer integration across coding-agent runtime, gateway, orchestrator, and dashboard paths.
- [ ] Artifact contract tests and migration fixtures.

### Risks

- State overreach can create a bloated schema that is hard to evolve.
- Migration may stall if too many consumers depend on legacy ad hoc files.
- Backward compatibility work can become open-ended without explicit cutoff criteria.

## Sources & References

- Source ideation: `docs/ideation/2026-03-23-autonomous-operator-mission-control-ideation.md`
- Current dashboard/live-shell artifact reader: `crates/tau-tui/src/lib.rs:620`
- TUI live mode entrypoint: `crates/tau-tui/src/main.rs:202`
- Gateway session/memory/UI telemetry surfaces: `docs/tau-coding-agent/code-map.md:96`
- Gateway operator routes: `docs/guides/gateway-api-reference.md:58`
- Runtime plan hooks: `crates/tau-coding-agent/src/runtime_loop.rs:1025`
- Structured plan model: `crates/tau-orchestrator/src/plan.rs:12`
- Execution report model: `crates/tau-orchestrator/src/plan_executor.rs:12`
- Existing roadmap context: `docs/planning/integration-gap-closure-plan.md`
