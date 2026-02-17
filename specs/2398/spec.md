# Spec: Issue #2398 - Apply role model overrides at orchestrator dispatch time

Status: Accepted

## Problem Statement
Tau's plan-first orchestrator supports role profiles with an optional `model` hint, but routed
prompt attempts always dispatch with the agent baseline model. That prevents role-aware cost and
quality tuning for planner/delegated/review phases and leaves G15 model routing only partially
implemented.

## Acceptance Criteria

### AC-1 Routed attempts use route-role model overrides when configured
Given a route table role profile with `model` set,
When a routed orchestrator attempt executes for that role,
Then the underlying LLM request dispatches with that exact model id.

### AC-2 Routed attempts inherit the baseline model when role override is absent
Given a route table role profile without `model`,
When a routed orchestrator attempt executes,
Then the underlying LLM request dispatches with the agent baseline model.

### AC-3 Model override scope is limited to a single routed attempt
Given multiple routed attempts across planner/delegated/review phases,
When a role override is applied for one attempt,
Then subsequent attempts use their own override or baseline model without leakage.

### AC-4 Existing route/fallback behavior remains unchanged
Given routed orchestration with fallback and default route tables,
When this change is enabled,
Then route selection/fallback semantics and legacy default behavior remain unchanged.

## Scope

### In Scope
- Role model override propagation through routed prompt execution paths.
- Scoped model override helper in agent core for deterministic temporary dispatch override.
- Conformance/regression tests for override ordering and inherit fallback.

### Out of Scope
- New route-table schema fields.
- Prompt complexity/task-based automatic model selection.
- Non-orchestrator runtime model-routing policies.

## Conformance Cases

| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | Plan-first delegated run with route roles `planner/executor/reviewer` each defining `model` | Recorded request models match role attempt order and configured role model ids |
| C-02 | AC-2 | Integration | Plan-first routed run with role profiles missing `model` | Recorded request model remains equal to agent baseline model for those attempts |
| C-03 | AC-3 | Functional | Mixed route table with one overridden role and one inherited role | Override applies only to targeted attempt; later attempt uses expected non-leaked model |
| C-04 | AC-4 | Regression | Existing default-routed orchestrator run fixture | Prior routed-default parity test remains green unchanged |
| C-05 | AC-3 | Unit | Agent scoped-model helper around async closure | Baseline model restored after closure completion |

## Success Metrics / Observable Signals
- C-01..C-05 tests pass.
- Existing routed fallback/default behavior tests continue passing unchanged.
- No clippy or fmt regressions in touched crates.
