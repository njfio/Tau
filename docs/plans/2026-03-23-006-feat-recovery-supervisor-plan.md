---
title: feat: Add Recovery Supervisor
type: feat
status: active
date: 2026-03-23
---

# feat: Add Recovery Supervisor

## Overview

Promote failure detection, replanning, escalation, graceful termination, and circuit-breaker state into a first-class supervisory layer with visible operator controls. The Recovery Supervisor should make Tau's degraded-mode behavior understandable and steerable instead of implicit or buried in logs.

## Problem Statement / Motivation

The repository already includes a meaningful recovery substrate:

- `FailureDetector` with repeated-failure, loop, no-progress, and budget signals
- `RecoveryStrategy` with retry, alternative path, escalation, and graceful termination
- `CircuitBreaker` for provider protection
- `EscalationRequired` and `GracefulTermination` events in `tau-agent-core`

What is missing is a coherent supervisory experience around those primitives. Today the operator does not yet get a strong, centralized answer to: what went wrong, what did Tau infer, what is it proposing next, and what can I do about it right now?

## Proposed Solution

Build a Recovery Supervisor that:

1. Consolidates runtime failure signals and recovery decisions into one operator-facing state.
2. Distinguishes between automatic recovery, operator approval, and terminal failure.
3. Presents recovery options as explicit controls:
   - retry current step
   - replan remaining work
   - switch to degraded mode
   - request operator input
   - terminate gracefully
4. Publishes supervisor state into the runtime backbone so TUI and dashboard can render the same truth.
5. Audits all recovery decisions and operator interventions.

## Implementation Phases

### Phase 1: Recovery State Model

- Define a supervisory state model for active failure signals, suggested action, current recovery attempt, and escalation status.
- Map `FailureSignal`, `RecoveryStrategy`, and circuit-breaker state into that model.
- Add reason-coded transitions and audit events.

### Phase 2: Runtime Integration

- Emit supervisor state transitions during runtime execution.
- Attach recovery actions to mission and plan state.
- Ensure repeated failures and operator interventions are bounded and idempotent.

### Phase 3: Operator Controls

- Expose recovery state and controls in TUI and dashboard/operator APIs.
- Support approve retry, force replan, request guidance, or graceful termination actions.
- Surface recommended action and rationale together.

### Phase 4: Hardening

- Tune thresholds for repeated failure, no-progress, and budget exhaustion.
- Add deterministic tests for recovery state transitions and degraded-mode rendering.
- Document operator playbooks for common failure classes.

## Technical Considerations

- Recovery must be visible before it becomes automatic. Operators need to know when the system is self-correcting versus asking for intervention.
- Circuit breaker, recovery strategy, and failure detector currently exist as adjacent mechanisms; the supervisor should unify, not duplicate, them.
- Recovery loops need hard limits. A supervisor that endlessly retries is worse than one that escalates quickly and clearly.
- The supervisor should be compatible with both autonomous background runs and interactive operator-attached sessions.

## System-Wide Impact

### Interaction Graph

Runtime detects failure signals, supervisor maps them to recovery state, operator surface displays recommended action, operator or runtime executes recovery action, and resulting state is recorded into mission/runtime state and audit logs.

### Error & Failure Propagation

Supervisor state becomes the canonical place to understand whether a failure is transient, operator-blocked, or terminal. Raw errors can still exist, but supervisory classification should drive operator behavior.

### State Lifecycle Risks

If supervisor state is not tied to mission/plan state, retries or replans can produce duplicate actions or ambiguous progress. Recovery decisions must be attached to the same work object being supervised.

### API Surface Parity

TUI, dashboard, and gateway/operator APIs should expose the same recovery actions and reason codes.

### Integration Test Scenarios

- Repeated tool failure triggers a visible escalation after the configured retry path is exhausted.
- Conversation loop detection produces an alternative-approach recovery suggestion with operator-visible rationale.
- Budget exhaustion enters graceful-termination flow and preserves partial results with audit-visible cause.

## SpecFlow Notes

### Primary Operator Flows

1. See that a mission is degraded and understand why.
2. Review Tau's recommended next action.
3. Approve or override the recovery action.
4. Stop a failing mission cleanly while preserving useful artifacts.

### Important Gaps to Resolve in Implementation

- Which recovery actions can run automatically versus requiring operator approval.
- How many recovery attempts are allowed per step, mission, or failure class.
- Whether degraded-mode behavior changes budget/tool policy or only recovery messaging.

### Default Planning Assumptions

- Recovery suggestions are visible before operator action is requested.
- Escalation is preferred over silent looping.
- Graceful termination preserves a useful partial-results summary and audit trail.

## Acceptance Criteria

- [ ] Runtime emits a unified recovery-supervisor state derived from failure detection, circuit-breaker status, and recovery strategy selection.
- [ ] Recovery-supervisor state is attached to mission or equivalent work state and is audit-visible.
- [ ] Operator surfaces can display the current failure class, recommended action, and recovery attempt count.
- [ ] Operators can trigger retry, replan, request-input, degraded-mode, or graceful-termination actions where supported.
- [ ] Recovery loops are bounded with explicit thresholds and tested failure paths.
- [ ] Docs explain operator recovery workflows and reason codes for common failure classes.

## Success Metrics

- Operators can diagnose and respond to degraded runs without diving into low-level logs first.
- Repeat failure loops are cut off earlier and escalated more clearly.
- Recovery decisions become explainable and reviewable after the fact.

## Dependencies & Risks

### Dependencies

- [ ] Unified Runtime State Backbone to carry supervisor state consistently.
- [ ] Mission model or equivalent to anchor recovery state to work in progress.
- [ ] TUI/dashboard control surfaces for operator intervention.

### Risks

- Poor threshold tuning can create alert fatigue or timid behavior.
- Too many recovery options can overwhelm operators during failure.
- If recovery reason codes are inconsistent, the supervisor will look arbitrary.

## Sources & References

- Source ideation: `docs/ideation/2026-03-23-autonomous-operator-mission-control-ideation.md`
- Failure detection: `crates/tau-agent-core/src/failure_detector.rs`
- Recovery strategies: `crates/tau-agent-core/src/recovery.rs`
- Circuit breaker: `crates/tau-agent-core/src/circuit_breaker.rs`
- Recovery-related config and events: `crates/tau-agent-core/src/lib.rs:167`
- Escalation and graceful termination events: `crates/tau-agent-core/src/lib.rs:726`
- TUI status surface: `crates/tau-tui/src/interactive/ui_status.rs`
- Existing operator-shell alerts/actions: `crates/tau-tui/src/lib.rs:620`
- Recovery roadmap context: `docs/AGENT_IMPROVEMENTS_PLAN.md:973`
