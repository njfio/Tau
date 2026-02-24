# Tasks: Issue #3466 - M302 fail-closed ops control-action outcomes

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Integration/Regression): add failing ops control-action form
   tests for missing/invalid/applied redirect marker contracts.
2. [x] T2 (RED, Functional): add failing UI marker test for command-center
   control-action outcome panel contracts.
3. [x] T3 (GREEN, Implementation): add query normalization and render markers
   for control-action outcome state.
4. [x] T4 (GREEN, Implementation): fail closed in ops control-action form
   handler with deterministic redirect marker contract.
5. [x] T5 (VERIFY): run scoped gateway + dashboard-ui tests and quality gates.
6. [x] T6 (VERIFY): update spec status/evidence and close issue with tier matrix.

## TDD Evidence
### RED
- Command:
  `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway unit_requested_control_action_reason_defaults_and_normalizes_values -- --nocapture`
- Result (expected failure before alias normalization):
  assertion failed with `left: "none"` and `right: "missing_action"`.

### GREEN
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway unit_requested_control_action_reason_defaults_and_normalizes_values -- --nocapture` passed.
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3466 -- --nocapture` passed:
  - `integration_spec_3466_c01_ops_control_action_missing_action_redirects_with_missing_marker`
  - `integration_spec_3466_c03_ops_control_action_form_submits_dashboard_mutation_and_redirects_with_applied_marker`
  - `regression_spec_3466_c02_ops_control_action_invalid_action_fails_closed_with_redirect_marker`
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3466 -- --nocapture` passed:
  - `functional_spec_3466_c04_control_action_status_panel_renders_marker_contracts`
  - `regression_spec_3466_c05_control_action_status_panel_defaults_to_idle_contract_markers`

### REGRESSION
- `cargo fmt --check` passed.
- `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-gateway -p tau-dashboard-ui --tests --no-deps -- -D warnings` passed.
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway requested_control_action -- --nocapture` passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | ✅ | `unit_requested_control_action_status_defaults_to_idle_and_normalizes_values`, `unit_requested_control_action_defaults_to_none_and_normalizes_values`, `unit_requested_control_action_reason_defaults_and_normalizes_values` |  |
| Property | N/A |  | No randomized invariant surface added in this slice |
| Contract/DbC | N/A |  | No `contracts` annotations introduced |
| Snapshot | N/A |  | No snapshot test surface used |
| Functional | ✅ | `functional_spec_3466_c04_control_action_status_panel_renders_marker_contracts` |  |
| Conformance | ✅ | `integration_spec_3466_c01_*`, `integration_spec_3466_c03_*`, `regression_spec_3466_c02_*` |  |
| Integration | ✅ | `cargo test -p tau-gateway 3466 -- --nocapture` (server + route + render loop) |  |
| Fuzz | N/A |  | No new untrusted parser surface requiring fuzz campaign |
| Mutation | N/A |  | Non-critical workflow slice; mutation campaign deferred |
| Regression | ✅ | `regression_spec_3466_c02_*`, `regression_spec_3466_c05_*` |  |
| Performance | N/A |  | No hotspot/perf budget path changed |
