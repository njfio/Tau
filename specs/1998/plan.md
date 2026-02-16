# Issue 1998 Plan

Status: Reviewed

## Approach

1. Add `M24RLGateExitDecision` model in `benchmark_artifact.rs` with
   machine-readable JSON projection.
2. Add evaluator:
   - `evaluate_m24_rl_gate_exit(bundle)`
   - derive deterministic reason codes from bundle pass signals and runbook refs
3. Add C-01..C-04 tests and one regression guard for runbook fail-closed behavior.

## Affected Areas

- `crates/tau-trainer/src/benchmark_artifact.rs`
- `specs/1998/spec.md`
- `specs/1998/plan.md`
- `specs/1998/tasks.md`

## Risks And Mitigations

- Risk: decision semantics drift from bundle section intent.
  - Mitigation: reason codes map 1:1 to section checks in evaluator tests.
- Risk: runbook checks become permissive.
  - Mitigation: explicit blank-string fail-closed tests.

## ADR

No dependency/protocol changes; ADR not required.
