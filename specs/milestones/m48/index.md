# M48 — Session Cost Tracking Rollups

Milestone: [GitHub milestone #48](https://github.com/njfio/Tau/milestone/48)

## Objective

Implement end-to-end per-session cost rollups so provider usage data (input/output/cached tokens and USD) is aggregated, persisted, and queryable at session scope.

## Scope

- Add session-level cost accumulator in the runtime usage pipeline.
- Persist rollup totals alongside session lifecycle state.
- Expose totals in diagnostics/status surfaces already used by operators.
- Add conformance and regression tests for accumulation correctness and idempotency.

## Out of Scope

- Billing export/reporting APIs.
- Organization/project-level cost aggregation.
- Provider-specific pricing catalog refresh logic.

## Linked Hierarchy

- Epic: #2302
- Story: #2303
- Task: #2304
- Subtask: #2305
