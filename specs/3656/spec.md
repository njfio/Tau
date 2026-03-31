# Spec: Issue #3656 - Wire gateway mission loop into Tau action history and learning insights

Status: Reviewed

## Problem Statement
`#3655` gives the gateway a durable mission-supervisor record, but the outer
loop still does not learn from real tool outcomes. Tau already has an
`ActionHistoryStore`, `LearningInsight`, and Cortex bulletin composition, yet
the gateway mission path does not write tool results into action history or
inject distilled learning guidance into later mission attempts. The next slice
should connect the gateway mission loop to Tau's existing learning surfaces so
future requests benefit from prior tool failures and success-rate patterns.

## Scope
In scope:
- `crates/tau-gateway/src/gateway_openresponses/learning_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/3656/spec.md`
- `specs/3656/plan.md`
- `specs/3656/tasks.md`

Out of scope:
- changing the `tau-agent-core` inner tool loop contract
- replacing the existing Cortex background bulletin runtime
- new mission-control UI surfaces
- autonomous training or prompt-optimization pipelines

## Acceptance Criteria
### AC-1 Gateway mission requests persist tool outcomes into Tau action history
Given a gateway-backed mission request that executes one or more tools,
when the request completes,
then the gateway persists those tool outcomes to a gateway-local action-history
store using Tau's existing action-history schema with the linked mission/session
context and success/failure summaries.

### AC-2 Distilled learning insights are derived from gateway action history
Given gateway action-history records containing repeated tool failures or low
success-rate tools,
when the gateway distills learning context for a new mission request,
then it produces `LearningInsight` data using Tau's existing failure-pattern and
tool-success-rate logic rather than inventing a second learning format.

### AC-3 New mission attempts include learned guidance alongside existing memory bulletin context
Given gateway action-history records with actionable learning insights,
when a subsequent gateway mission request is executed,
then the system prompt includes the existing Cortex bulletin context plus a
learning-insights section derived from action history before model dispatch.

## Conformance Cases
- C-01 / AC-1 / Regression:
  execute a gateway request whose tool call fails, then verify the gateway-local
  action-history file contains a persisted tool-execution record for that
  mission/session.
- C-02 / AC-2 / Unit:
  seed gateway action-history with repeated failures and verify the distilled
  `LearningInsight` exposes the expected failing tool and declining success-rate
  entries.
- C-03 / AC-3 / Regression:
  run a first request that produces a repeated failing `bash` tool outcome, then
  run a second request and verify the captured system prompt includes a
  `## Learning Insights` section referencing the learned `bash` failure pattern.

## Success Metrics / Observable Signals
- Gateway mission runs leave behind reusable learning data in Tau's action
  history format
- Follow-up requests are primed with deterministic learned guidance before model
  dispatch
- The gateway reuses Tau learning primitives instead of inventing a parallel
  per-surface learning format

## Files To Touch
- `specs/3656/spec.md`
- `specs/3656/plan.md`
- `specs/3656/tasks.md`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/learning_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
