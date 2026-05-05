# Tasks: Issue #3758 - Mission harness operator UI usability pass

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for harness UI usability.
- [x] T2 (RED): add design-contract regression tests.
- [x] T3 (GREEN): rework harness scoped CSS and minimal markup contracts.
- [x] T4 (VERIFY): run tests, static checks, and browser screenshot smoke.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3758` failed because
  the harness route did not expose operator-console layout markers or
  focus/overflow/responsive style contracts.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3758` passed
  (2 tests).
- REGRESSION: `cargo test -p tau-dashboard-ui functional_spec_3756 &&
  cargo test -p tau-dashboard-ui functional_spec_3757` passed (3 tests).
- REGRESSION: `cargo test -p tau-dashboard-ui` passed (141 tests).
- REGRESSION: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
  passed (1 test).
- STATIC: `cargo fmt --check -p tau-dashboard-ui` passed.
- STATIC: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
  passed.
- VISUAL: Browser Use loaded `file:///tmp/tau-harness-after.html`, confirmed
  `data-layout-density="operator-console"`, `data-visual-contract="mission-control"`,
  4 table wrappers, 8 status chips, visible review/TUI panes, and
  approval-gated apply remained disabled.
