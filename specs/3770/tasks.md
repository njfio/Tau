# Tasks: Issue #3770 - Harness operator log first-screen placement

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for first-screen operator log placement.
- [x] T2 (RED): add dashboard UI regression test for proof log order and compact markers.
- [x] T3 (GREEN): move operator log under tool evidence and cap its pre height.
- [x] T4 (VERIFY): run targeted/full tests, static checks, gateway integration, and browser geometry validation.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3770` failed before implementation because the operator log still rendered after acceptance/gates/artifacts.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3770` passed after the operator log was moved under tool evidence and compacted.
- Regression: `cargo test -p tau-dashboard-ui functional_spec_376` passed with 10 prior harness UI tests.
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37` passed with 17 harness UI tests.
- Regression: `cargo test -p tau-dashboard-ui` passed with 153 tests.
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit` passed.
- Static: `cargo fmt --check -p tau-dashboard-ui` passed.
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings` passed.
- Static: `git diff --check` passed.
- Browser fallback: `/tmp/tau-harness-continue10-after.png` captured the rendered preview; geometry reported `data-proof-grid-priority="evidence-log-first"`, evidence before log `true`, log before acceptance/gates/artifacts `true`, operator log top `y=590`, bottom `770`, viewport height `967`, log pre `max-height=118px`, document `clientWidth=1371`, document `scrollWidth=1371`, and no console errors.
- Browser Use caveat: in-app Browser attach was attempted through the required `iab` runtime after a fresh Node kernel reset, but initialization timed out, so visual verification used the bundled-browser fallback.
