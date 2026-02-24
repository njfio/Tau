# Tasks: Issue #3448 - M298 wave-1 E2E harness and ops dashboard conformance slice

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Conformance): add failing artifact/contract checks for wave-1 AC mapping and harness/dashboard expectations.
2. [x] T2 (GREEN, Spec/Docs): finalize artifact consistency and conformance mapping updates.
3. [x] T3 (RED, Integration): add failing E2E tests for gateway lifecycle + agent session flow using deterministic scripted LLM.
4. [x] T4 (GREEN, Implementation): implement wave-1 harness helpers and minimal runtime wiring to satisfy T3.
5. [x] T5 (RED, Functional/Integration): add failing ops dashboard live control/data conformance regressions.
6. [x] T6 (GREEN, Implementation): implement/fix dashboard/gateway behavior for live conformance.
7. [x] T7 (VERIFY, Regression): execute scoped fmt/clippy/tests and record RED/GREEN/REGRESSION evidence.
8. [x] T8 (VERIFY): update spec status to `Implemented` once all ACs pass.

## TDD Evidence
### RED
- Command: `cargo test -p tau-gateway integration_spec_3448_c02_tau_e2e_harness_runs_gateway_lifecycle_and_session_flow -- --nocapture`
- Result: compile failed as expected with unresolved `TauE2eHarness` and missing `scripted_gateway_response`.

### GREEN
- Command: `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway integration_spec_3448_ -- --nocapture`
- Result: passed (2/2):
  - `integration_spec_3448_c02_tau_e2e_harness_runs_gateway_lifecycle_and_session_flow`
  - `integration_spec_3448_c03_tau_e2e_harness_keeps_ops_control_and_dashboard_live_contracts`

### REGRESSION
- Command: `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway integration_ops_shell_control_action_form_submits_dashboard_mutation_and_redirects -- --nocapture`
- Result: passed.
- Command: `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway functional_spec_2810_c01_c02_c03_ops_shell_control_markers_reflect_dashboard_control_snapshot -- --nocapture`
- Result: passed.
- Command: `cargo fmt --check`
- Result: required formatting once (`cargo fmt --all`), then passed.
- Command: `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-gateway --tests --no-deps -- -D warnings`
- Result: passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | ✅ | Existing gateway unit coverage + helper usage in `gateway_openresponses::tests` |  |
| Property | N/A |  | No new parser/invariant surface introduced in wave-1 scope |
| Contract/DbC | N/A |  | No new public API contracts/annotations introduced |
| Snapshot | N/A |  | No new stable snapshot output introduced |
| Functional | ✅ | `integration_spec_3448_c03_tau_e2e_harness_keeps_ops_control_and_dashboard_live_contracts` |  |
| Conformance | ✅ | `integration_spec_3448_c02_*`, `integration_spec_3448_c03_*`, artifact existence checks (C-01) |  |
| Integration | ✅ | `integration_spec_3448_c02_tau_e2e_harness_runs_gateway_lifecycle_and_session_flow` |  |
| Fuzz | N/A |  | No untrusted parser/input surface changes in this issue |
| Mutation | N/A |  | Scope is test-harness/spec/docs; no production-path mutation target in-diff |
| Regression | ✅ | `integration_ops_shell_control_action_form_submits_dashboard_mutation_and_redirects`, `functional_spec_2810_c01_c02_c03_ops_shell_control_markers_reflect_dashboard_control_snapshot` |  |
| Performance | N/A |  | No runtime hot-path changes in production code |
