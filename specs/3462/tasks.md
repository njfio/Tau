# Tasks: Issue #3462 - Wire RL e2e promotion and rollback gates

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Functional): add failing RL e2e harness tests asserting missing
   promotion/rollback gate contract fields.
2. [x] T2 (RED, Unit/Regression): add failing rollback helper tests for blocked
   and allowed gate outcomes.
3. [x] T3 (GREEN, Implementation): extend RL artifact schema and gate-evaluation
   logic in `rl_e2e.rs`.
4. [x] T4 (GREEN, Verification): add M301 verification script and script tests
   for gate contract enforcement.
5. [x] T5 (VERIFY): run scoped tests + fmt + clippy and capture evidence.
6. [x] T6 (VERIFY): mark spec implemented and complete tier matrix.

## TDD Evidence
### RED
- Command:
  `CARGO_TARGET_DIR=target-fast cargo test -p tau-trainer --test rl_e2e_harness -- --nocapture`
- Result (expected failure):
  unresolved `tau_trainer::evaluate_rl_e2e_rollback_gate` import and missing
  `promotion_gate` / `rollback_gate` fields on `RlE2eArtifact`.

### GREEN
- Command:
  `CARGO_TARGET_DIR=target-fast cargo test -p tau-trainer --test rl_e2e_harness -- --nocapture`
- Result: passed (`5` tests), including:
  - `spec_c03_rl_e2e_harness_emits_deterministic_artifact_with_rollout_gae_and_ppo_summaries`
  - `spec_c05_rl_e2e_rollback_gate_requires_rollback_when_gate_signals_failures`
  - `spec_c06_rl_e2e_rollback_gate_blocks_when_promotion_denied_without_failed_checks`
- Command:
  `bash scripts/verify/m301-rl-promotion-rollback-gate.sh`
- Result: passed (real harness run + gate contract checks).
- Command:
  `bash scripts/verify/test-m301-rl-promotion-rollback-gate.sh`
- Result: passed (pass path + fail-closed invalid-artifact path).

### REGRESSION
- `cargo fmt --check` passed.
- `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-trainer --tests --no-deps -- -D warnings` passed.
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-trainer -- --nocapture` passed.
- Scoped mutation campaign:
  `CARGO_TARGET_DIR=target-fast cargo mutants -p tau-trainer --file crates/tau-trainer/src/rl_e2e.rs -F evaluate_rl_e2e_rollback_gate --timeout 120 --minimum-test-timeout 20 -- -p tau-trainer --test rl_e2e_harness rollback_gate_ -- --nocapture`
  result: `4 mutants tested: 3 caught, 1 unviable` (unviable was function-replacement mutant that does not compile because `RlE2eRollbackGateSummary` intentionally has no `Default` implementation).

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | ✅ | `spec_c05_rl_e2e_rollback_gate_requires_rollback_when_gate_signals_failures`, `spec_c06_rl_e2e_rollback_gate_blocks_when_promotion_denied_without_failed_checks`, `unit_rl_e2e_export_summary_uses_stable_filename_contract` |  |
| Property | N/A |  | No randomized property surface added in this change |
| Contract/DbC | N/A |  | No `contracts`-based API annotations introduced |
| Snapshot | N/A |  | No snapshot artifact assertions introduced |
| Functional | ✅ | `spec_c03_rl_e2e_harness_emits_deterministic_artifact_with_rollout_gae_and_ppo_summaries` |  |
| Conformance | ✅ | RL harness selectors + `scripts/verify/m301-rl-promotion-rollback-gate.sh` + `scripts/verify/test-m301-rl-promotion-rollback-gate.sh` |  |
| Integration | ✅ | `spec_c03_rl_e2e_harness_emits_deterministic_artifact_with_rollout_gae_and_ppo_summaries` exercises trainer+runner+store+algorithm composition |  |
| Fuzz | N/A |  | No new untrusted parsing surface beyond existing serde contract checks |
| Mutation | ✅ | Scoped `cargo mutants` run on `evaluate_rl_e2e_rollback_gate` (`3` caught, `1` unviable compile-time mutant) |  |
| Regression | ✅ | `regression_spec_c04_rl_e2e_harness_rejects_output_path_that_is_a_file`, full `cargo test -p tau-trainer -- --nocapture` |  |
| Performance | N/A |  | No hotspot/budget target changed in this slice |
