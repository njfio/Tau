# Tasks: Issue #3757 - State-backed harness benchmark and audit panels

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for state-backed harness rendering.
- [x] T2 (RED): add dashboard and gateway tests for state-backed benchmark and
  audit rendering.
- [x] T3 (GREEN): add harness snapshot structs and dynamic dashboard rendering.
- [x] T4 (GREEN): collect harness proof and audit state in the gateway shell.
- [x] T5 (VERIFY): run focused tests, fmt, clippy, and live smoke if needed.
- [x] T6 (GREEN): link state-backed tool evidence artifact cells to the
  harness artifact view.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3757` failed before
  implementation because `TauOpsDashboardHarnessSnapshot` did not exist.
- RED: `cargo test -p tau-gateway integration_spec_3757` failed before
  implementation because `/ops/harness` did not expose `data-proof-source`.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3757` passed
  (1 test).
- GREEN: `cargo test -p tau-gateway integration_spec_3757` passed
  (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui` passed (139 tests).
- REGRESSION: `cargo test -p tau-gateway` passed (351 tests, 1 ignored).
- REGRESSION: `cargo test -p tau-tui` passed (89 lib tests, 22 binary tests,
  5 demo smoke tests).
- STATIC: `cargo fmt --check -p tau-dashboard-ui -p tau-gateway` passed.
- STATIC: `cargo clippy -p tau-dashboard-ui -p tau-gateway --all-targets -- -D warnings`
  passed.
- GREEN: `cargo test -p tau-dashboard-ui regression_harness_tool_evidence_links_state_backed_proof_artifact`
  passed (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui harness` passed (51 tests).
- GREEN: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
  passed (1 test).
- REGRESSION: `cargo test -p tau-gateway ops_harness -- --test-threads=1`
  passed (6 tests).
- STATIC: `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `cargo build -p tau-coding-agent` passed.
- LIVE: Browser on `http://127.0.0.1:8795/ops/harness` found 5
  `data-tool-proof-artifact-href="true"` links and 5 state-backed tool rows.
