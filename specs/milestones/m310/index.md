# M310 - RL policy operations depth verification wave

Status: Active

## Context
M310 deepens true-RL delivery verification by adding one deterministic gate
that aggregates RL promotion/rollback/significance contracts plus runtime
audit-log fail-closed behavior into a single auditable report.

Primary sources:
- `scripts/verify/m301-rl-promotion-rollback-gate.sh`
- `scripts/demo/m24-rl-live-benchmark-proof.sh`
- `scripts/demo/rollback-drill-checklist.sh`
- `crates/tau-trainer/tests/rl_e2e_harness.rs`
- `crates/tau-trainer/src/benchmark_significance.rs`
- `crates/tau-runtime/src/observability_loggers_runtime.rs`

## Issue Hierarchy
- Epic: #3496
- Story: #3497
- Task: #3498

## Scope
- Add deterministic M310 RL policy-operations verification script and report.
- Add contract test with fail-closed required-step inventory checks.
- Map RL promotion/rollback/significance/runtime-audit coverage to selectors.
- Update README links with M310 verification entrypoint.

## Exit Criteria
- `specs/3498/spec.md` is `Implemented` with AC evidence.
- M310 script report includes all required RL policy-operations step IDs.
- Contract test fails closed on missing required-step IDs.
- README includes M310 verification entrypoint.
