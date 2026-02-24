# Spec: Issue #3462 - Wire RL e2e promotion and rollback gates

Status: Implemented

## Problem Statement
`tau-trainer` deterministic RL e2e artifacts currently report rollout/GAE/PPO
summaries and numeric checks, but they do not explicitly declare whether
checkpoint promotion is allowed or whether rollback is required. This leaves an
operator-readiness gap in true-RL productionization.

## Scope
In scope:
- Add deterministic checkpoint-promotion gate summary to RL e2e artifact output.
- Add deterministic rollback-required gate summary to RL e2e artifact output.
- Add conformance tests for gate-allowed and gate-blocked paths.
- Add verification script that fails if gate contracts are missing or invalid.

Out of scope:
- Live policy-weight promotion into production runtime.
- External provider-dependent RL execution.
- Changes to training data schema or rollout execution engine.

## Acceptance Criteria
### AC-1 RL e2e artifact exposes promotion gate contract
Given deterministic RL harness execution,
when artifact JSON is generated,
then it includes a machine-readable promotion-gate summary with pass/fail and
reason-code fields.

### AC-2 RL e2e artifact exposes rollback gate contract
Given deterministic RL harness execution and gate checks,
when artifact JSON is generated,
then it includes rollback-required decision data with deterministic trigger
reason codes.

### AC-3 Conformance tests cover allowed and blocked gate decisions
Given RL e2e gate helpers,
when test selectors run,
then they assert both promotion-allowed and rollback-required outcomes
deterministically.

### AC-4 Operator verification script enforces gate contract presence
Given a generated RL harness artifact,
when verification script executes,
then it fails closed on missing/invalid gate fields and passes when fields are
present and consistent.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | deterministic RL harness run | artifact emitted | promotion gate object exists with decision/reason fields |
| C-02 | AC-2 | Functional | deterministic RL harness run | artifact emitted | rollback gate object exists with rollback decision and reason codes |
| C-03 | AC-3 | Unit/Regression | failing gate inputs | rollback helper evaluates | rollback-required path and reason codes are deterministic |
| C-04 | AC-3 | Unit/Functional | passing gate inputs | promotion/rollback helpers evaluate | promotion allowed and rollback not required |
| C-05 | AC-4 | Conformance | verification script + sample artifact | script runs | contract checks pass/fail deterministically |

## Success Metrics / Observable Signals
- RL harness artifact JSON includes explicit `promotion_gate` and
  `rollback_gate` objects.
- New conformance selectors in `tau-trainer` pass for both allowed and blocked
  decision paths.
- Verification script passes in valid mode and fails closed when gate fields are
  absent/invalid.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-trainer --test rl_e2e_harness spec_c03_rl_e2e_harness_emits_deterministic_artifact_with_rollout_gae_and_ppo_summaries -- --nocapture` validates `promotion_gate` fields in typed artifact and exported JSON. |
| AC-2 | ✅ | Same selector validates `rollback_gate`; `spec_c05_rl_e2e_rollback_gate_requires_rollback_when_gate_signals_failures` and `spec_c06_rl_e2e_rollback_gate_blocks_when_promotion_denied_without_failed_checks` assert deterministic rollback-trigger reason codes. |
| AC-3 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-trainer --test rl_e2e_harness -- --nocapture` passes gate-allowed and gate-blocked conformance selectors. |
| AC-4 | ✅ | `bash scripts/verify/m301-rl-promotion-rollback-gate.sh` (real harness run) and `bash scripts/verify/test-m301-rl-promotion-rollback-gate.sh` (pass + fail-closed contract checks) both pass. |
