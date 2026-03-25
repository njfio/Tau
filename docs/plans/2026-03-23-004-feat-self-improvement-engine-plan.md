---
title: feat: Enable Self-Improvement Engine by Default
type: feat
status: active
date: 2026-03-23
---

# feat: Enable Self-Improvement Engine by Default

## Overview

Make Tau visibly self-improving by default. Turn action history, session feedback, tool success/failure learning, and operator-visible recommendations into an always-on improvement loop with explicit inspection and override controls.

## Problem Statement / Motivation

The repository already contains learning-oriented primitives:

- `ActionHistoryStore` and `SessionFeedback` in `tau-memory`
- action-history configuration in `tau-agent-core`
- recovery and failure-detection primitives in `tau-agent-core`
- roadmap work in `docs/AGENT_IMPROVEMENTS_PLAN.md`

But the current defaults still keep this largely dormant. `action_history_enabled` is false by default, and the current operator surfaces do not make learned patterns or recommended behavior prominent. As a result, Tau can have learning infrastructure without feeling self-improving in day-to-day use.

## Proposed Solution

Enable a conservative, inspectable self-improvement loop:

1. Turn action-history collection on by default for supported runtime profiles.
2. Persist per-session feedback and runtime outcomes in a durable, queryable format.
3. Generate operator-visible recommendations from recent tool success rates, failure patterns, and recovery outcomes.
4. Feed learned recommendations back into:
   - tool preference and ordering
   - recovery hints
   - operator suggestions
   - mission postmortems and next-run guidance
5. Add explicit controls to view, clear, export, or disable learning behavior.

## Implementation Phases

### Phase 1: Harden Learning Data Path

- Confirm `ActionHistoryStore` persistence and retention behavior are production-safe enough for default use.
- Add explicit schema/versioning for stored learning artifacts.
- Capture session feedback and outcome summaries as part of normal runtime closeout.

### Phase 2: Runtime Integration

- Enable action history by default behind profile-aware configuration.
- Record tool usage, recovery actions, replans, and operator interventions consistently.
- Establish guardrails for low-signal and noisy data.

### Phase 3: Recommendation Layer

- Build recommendation summaries from success rates, latency, and common failure patterns.
- Distinguish between operator-facing recommendations and runtime-internal heuristics.
- Surface recommendations in mission summaries and post-run reviews.

### Phase 4: Operator Trust and Control

- Add inspection screens or panes for learned patterns.
- Add opt-out and reset controls.
- Explain when the runtime changed its preferred behavior due to learned evidence.

## Technical Considerations

- Learning without visibility becomes spooky behavior. Every adaptation that affects runtime decisions should be inspectable.
- The current `ActionHistoryStore` is lightweight; before default enablement, it needs a durability and scaling review suitable for normal use.
- Recommendations should be bounded and conservative. Avoid overfitting to small samples or recent anomalies.
- Learning data must respect privacy and safety boundaries, especially if prompts/tool inputs are summarized.

## System-Wide Impact

### Interaction Graph

Runtime actions emit learning records, session closeout writes feedback, recommendation engine derives learned guidance, operator surfaces present guidance, and future runs optionally consult the learned signals.

### Error & Failure Propagation

If learning storage or recommendation generation fails, runtime execution should continue in a clear degraded mode without corrupting normal task execution.

### State Lifecycle Risks

Incorrect retention or schema drift can make historical data untrustworthy. Reset, migration, and export semantics need to be explicit.

### API Surface Parity

Learned recommendations should be visible consistently across TUI, dashboard, and gateway inspection surfaces, not confined to one client.

### Integration Test Scenarios

- Record multiple sessions with divergent tool outcomes and verify stable success-rate recommendations.
- Trigger repeated failures and confirm the operator sees a recommendation and rationale before the next run.
- Disable learning and confirm runtime behavior falls back predictably without stale recommendations leaking through.

## SpecFlow Notes

### Primary Operator Flows

1. Finish a mission and review what the system learned.
2. Start a new mission and see recommended tools or warnings based on recent history.
3. Inspect why the system changed a preference.
4. Disable or reset learning when it becomes noisy or misleading.

### Important Gaps to Resolve in Implementation

- How much prompt/tool content can safely be summarized into stored history.
- What minimum evidence threshold is required before recommendations influence behavior.
- Whether learned preferences are global, profile-scoped, or mission-scoped.

### Default Planning Assumptions

- Learning is enabled by default but can be disabled per profile or globally.
- Recommendations are advisory first; automatic behavior changes should remain conservative.
- Operators can inspect both the learned signal and the reason a recommendation was produced.

## Acceptance Criteria

- [ ] Action history is enabled by default for supported profiles with documented override controls.
- [ ] Runtime records tool usage, replans, and recovery events into a durable learning store.
- [ ] Session feedback and outcome summaries are captured or derived at mission/session closeout.
- [ ] Learned recommendations are generated from recent history with explicit confidence or evidence thresholds.
- [ ] TUI, dashboard, or gateway inspection surfaces can show learned recommendations and failure patterns.
- [ ] Operators can clear, disable, or inspect learning state without manual file surgery.
- [ ] Tests cover retention, recommendation derivation, disable/reset behavior, and degraded-mode fallback.

## Success Metrics

- Repeated failure loops become less common because the system warns earlier and suggests better defaults.
- Operators can explain why Tau recommended a tool or approach from visible learned evidence.
- The system compounds useful behavior across sessions without feeling opaque.

## Dependencies & Risks

### Dependencies

- [ ] `tau-memory` learning storage hardening.
- [ ] Runtime instrumentation in `tau-agent-core`.
- [ ] Operator surfaces from TUI Mission Control or equivalent inspection UIs.

### Risks

- Noisy or low-quality data can create unhelpful recommendations.
- Implicit adaptation can erode trust if operators cannot inspect or override it.
- Data-retention and privacy concerns may constrain what can be stored by default.

## Sources & References

- Source ideation: `docs/ideation/2026-03-23-autonomous-operator-mission-control-ideation.md`
- Action history store: `crates/tau-memory/src/action_history.rs:99`
- Action history defaults: `crates/tau-memory/src/action_history.rs:80`
- Runtime config defaults: `crates/tau-agent-core/src/lib.rs:184`
- Runtime default disablement today: `crates/tau-agent-core/src/lib.rs:264`
- Recovery and escalation events: `crates/tau-agent-core/src/lib.rs:726`
- Learning roadmap: `docs/AGENT_IMPROVEMENTS_PLAN.md:861`
