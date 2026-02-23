# Tasks: Issue #3427 - RL fail-closed reason-code determinism

- [x] T1 (RED, Regression/Conformance): add failing tests for APO fail-closed deterministic reason codes (`apo_run_failed`, `apo_significance_failed`) and missing-runtime behavior.
- [x] T2 (GREEN, Implementation): normalize dynamic APO failure reason strings to deterministic contract values in live RL runtime.
- [x] T3 (REGRESSION, Functional): rerun targeted live RL APO selectors and adjacent conformance tests.
- [x] T4 (VERIFY, Conformance): capture AC -> test evidence and set issue artifact status to implemented.

## AC -> Conformance -> Tests
| AC | Conformance Case(s) | Test(s) |
|---|---|---|
| AC-1 | C-01, C-02 | `spec_c20_regression_live_apo_missing_runtime_reports_deterministic_reason_code`, `spec_c10_regression_live_apo_sample_thresholds_respect_min_and_hard_floor` |
| AC-2 | C-03 | `spec_c21_regression_live_apo_algorithm_failure_reports_deterministic_reason_code` |
| AC-3 | C-04 | `spec_c22_regression_live_apo_significance_failure_reports_deterministic_reason_code` |
| AC-4 | C-05 | `spec_c10_regression_live_apo_sample_thresholds_respect_min_and_hard_floor`, `spec_c11_regression_live_apo_rejects_non_significant_positive_delta`, `spec_c22_regression_live_apo_significance_failure_reports_deterministic_reason_code` |
| AC-5 | C-06 | Artifact set present under `specs/3427/{spec.md,plan.md,tasks.md}` |

## TDD Evidence
### RED
- `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c21_regression_live_apo_algorithm_failure_reports_deterministic_reason_code -- --test-threads=1 --nocapture`
- Result: failed with `report.reason_code=Some("apo_no_significant_improvement")` and `executed=true`, proving the test did not yet force algorithm failure and justifying the `FailingClient` harness update.

### GREEN
- `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c21_regression_live_apo_algorithm_failure_reports_deterministic_reason_code -- --test-threads=1`
- Result: `1 passed; 0 failed`.
- `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c2 -- --test-threads=1`
- Result: `4 passed; 0 failed` (covers `spec_c20`/`spec_c21`/`spec_c22` deterministic reason-code paths plus existing `spec_c20` calibration case).

### REGRESSION / VERIFY
- `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c10 -- --test-threads=1` -> `1 passed`.
- `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c11 -- --test-threads=1` -> `1 passed`.
- `cargo fmt --all -- --check` -> pass.
- `cargo clippy -p tau-coding-agent --tests -- -D warnings` -> pass.

## Tier Mapping
- Unit: live RL helper/runtime branches exercised in focused tests.
- Property: N/A (no new randomized invariant coverage in this slice).
- Contract/DbC: N/A (no DbC annotation changes).
- Snapshot: N/A.
- Functional: optimizer snapshot/APO report behavior.
- Conformance: C-01..C-06 mapped by test selectors and verify commands.
- Integration: N/A (in-crate runtime path; no cross-service integration added).
- Fuzz: N/A (no new untrusted parser input surface).
- Mutation: N/A (targeted deterministic contract hardening; no mutation gate required in this slice).
- Regression: existing APO adoption/non-adoption selectors.
- Performance: N/A (no performance-sensitive math changes).
