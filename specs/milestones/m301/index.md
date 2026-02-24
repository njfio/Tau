# M301 - True RL productionization gate wave (promotion + rollback)

Status: Active

## Context
M301 adds deterministic promotion/rollback gate contracts to the true-RL
end-to-end harness so operator readiness decisions are explicit, auditable, and
fail-closed.

Primary sources:
- `crates/tau-trainer/src/rl_e2e.rs`
- `crates/tau-trainer/src/benchmark_significance.rs`
- `crates/tau-trainer/tests/rl_e2e_harness.rs`

## Issue Hierarchy
- Epic: #3460
- Story: #3461
- Task: #3462

## Scope
- Extend RL e2e artifact schema with checkpoint-promotion gate summary.
- Add rollback-required gate summary derived from deterministic checks.
- Add deterministic conformance tests for promotion/rollback outcomes.
- Add verification script for operator consumption of gate output contracts.

## Exit Criteria
- `specs/3462/spec.md` is implemented with AC-to-test evidence.
- RL e2e artifact JSON includes promotion and rollback gate structures.
- Conformance/regression selectors for allowed and blocked gates pass.
- Verification script passes and emits deterministic gate checks.
