# Tasks: Issue #3772 - Harness compact mission state visibility

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for compact mission state visibility.
- [x] T2 (RED): add dashboard UI regression test for inline mission state and gate chips.
- [x] T3 (GREEN): add compact row metadata markup and scoped chip CSS.
- [x] T4 (VERIFY): run targeted/full tests, static checks, gateway integration, and browser geometry validation.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3772` failed before implementation because the active mission section lacked `data-compact-mission-summary="status-and-gates"`.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3772` passed after inline state/gate chips and compact mission metadata CSS were added.
- Regression: `cargo test -p tau-dashboard-ui functional_spec_3762` passed.
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37` passed with 19 tests.
- Regression: `cargo test -p tau-dashboard-ui` passed with 155 tests and 0 doc-tests.
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit` passed.
- Static: `cargo fmt --check -p tau-dashboard-ui` passed.
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings` passed.
- Static: `git diff --check` passed.
- Browser fallback: `/tmp/tau-harness-continue12-after.png` confirms inline mission state/gate chips are visible in the first viewport.
- Browser geometry: 1371x967 viewport reported document `scrollWidth=1371`, mission table wrapper `scrollWidth=345` and `clientWidth=345`, hidden compact columns display `none`, state chips `Running, Verifying, Completed, Blocked, Running`, gate chips `3/5 gates, 2/5 gates, 5/5 gates, 1/5 gates, 2/5 gates`, chip gap `5px`, chip padding `2px 6px`, and console errors `[]`.
- Browser Use caveat: in-app Browser `iab` attach was attempted, but the runtime reported no active Codex browser pane in this thread.
