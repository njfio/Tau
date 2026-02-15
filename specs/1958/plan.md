# Issue 1958 Plan

Status: Reviewed

## Approach

1. Add internal poll retry recovery context to `TrainingRunner::run`:
   - `consecutive_failures`
   - `accumulated_backoff_ms`
2. Thread recovery context into `process_once` / `process_dequeued` for the next
   processed rollout.
3. Emit metrics via tracer rewards before flush:
   - `runner.poll_retry_failures_before_rollout`
   - `runner.poll_retry_backoff_ms_before_rollout`
4. Reset recovery context after successful poll cycle (existing behavior retained).
5. Add tests for one-failure recovery, multi-failure deterministic totals, and
   clean-run no-noise behavior.

## Affected Areas

- `crates/tau-training-runner/src/lib.rs`
- `specs/1958/spec.md`
- `specs/1958/plan.md`
- `specs/1958/tasks.md`

## Risks And Mitigations

- Risk: metric emission could alter outcome logic.
  - Mitigation: emit additive reward spans only; do not alter status transitions.
- Risk: context reset semantics could drift.
  - Mitigation: assert deterministic values with conformance tests.

## ADR

No dependency/protocol changes; ADR not required.
