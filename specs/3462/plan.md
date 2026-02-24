# Plan: Issue #3462 - Wire RL e2e promotion and rollback gates

Status: Implemented

## Approach
1. Add RED tests in `crates/tau-trainer/tests/rl_e2e_harness.rs` for expected
   promotion/rollback gate contract fields and blocked rollback paths.
2. Extend `crates/tau-trainer/src/rl_e2e.rs` artifact structs and JSON encoding
   with deterministic gate summaries.
3. Reuse `benchmark_significance` policy/gate primitives to compute promotion
   decisions in harness flow.
4. Add deterministic rollback gate evaluation derived from check + promotion
   outcomes.
5. Add a verification script under `scripts/verify/` that enforces gate
   contract structure in emitted artifacts.
6. Run scoped verification, then update spec/tasks evidence and issue status.

## Affected Modules
- `crates/tau-trainer/src/rl_e2e.rs`
- `crates/tau-trainer/src/lib.rs`
- `crates/tau-trainer/tests/rl_e2e_harness.rs`
- `scripts/verify/` (new M301 script + script test)
- `specs/milestones/m301/index.md`
- `specs/3462/spec.md`
- `specs/3462/plan.md`
- `specs/3462/tasks.md`

## Risks / Mitigations
- Risk: gate logic drifts from benchmark significance rules.
  - Mitigation: use `evaluate_checkpoint_promotion_gate` directly.
- Risk: rollback decision semantics become ambiguous.
  - Mitigation: deterministic reason-code contract and explicit helper tests.
- Risk: script flakiness in local environments.
  - Mitigation: rely only on deterministic harness artifact generation and
    local JSON checks.

## Interfaces / Contracts
- RL artifact contract grows with:
  - promotion gate summary object
  - rollback gate summary object
- Existing fields remain backward-compatible for current consumers.

## Execution Summary
1. Added RED-first conformance requirements in `rl_e2e_harness` tests for:
   - `promotion_gate` and `rollback_gate` artifact fields,
   - deterministic rollback-required reason codes.
2. Extended `crates/tau-trainer/src/rl_e2e.rs` with:
   - `RlE2ePromotionGateSummary`,
   - `RlE2eRollbackGateSummary`,
   - `evaluate_rl_e2e_rollback_gate` helper,
   - deterministic significance/reproducibility-driven promotion gate wiring.
3. Updated harness checks to include:
   - `policy_improvement_significance`,
   - `checkpoint_promotion_gate`,
   - `rollback_gate`.
4. Exported new gate API/types via `crates/tau-trainer/src/lib.rs`.
5. Added M301 operator verification script and fail-closed script contract test:
   - `scripts/verify/m301-rl-promotion-rollback-gate.sh`
   - `scripts/verify/test-m301-rl-promotion-rollback-gate.sh`
6. Synced README true-RL gap row to reference new M301 gate verification.

## Verification Notes
- RED evidence: scoped test initially failed with unresolved rollback-gate API
  and missing artifact fields (`evaluate_rl_e2e_rollback_gate`,
  `promotion_gate`, `rollback_gate`).
- GREEN evidence:
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-trainer --test rl_e2e_harness -- --nocapture`
  - `bash scripts/verify/m301-rl-promotion-rollback-gate.sh`
  - `bash scripts/verify/test-m301-rl-promotion-rollback-gate.sh`
  all passed.
- Regression/gate evidence:
  - `cargo fmt --check` passed.
  - `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-trainer --tests --no-deps -- -D warnings` passed.
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-trainer -- --nocapture` passed (`107` unit tests + harness/bin selectors).
