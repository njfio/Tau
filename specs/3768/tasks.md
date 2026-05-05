# Tasks: Issue #3768 - Harness proof DAG compact single row

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for compact proof DAG.
- [x] T2 (RED): add dashboard UI regression test for single-row DAG density.
- [x] T3 (GREEN): update proof DAG markup/CSS for compact desktop layout.
- [x] T4 (VERIFY): run targeted/full tests, static checks, gateway integration, and browser geometry validation.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3768` failed before implementation on the missing `data-proof-dag-density="single-row"` DAG marker.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3768` passed after the compact DAG CSS and marker were added.
- Regression: `cargo test -p tau-dashboard-ui functional_spec_376` passed with 9 harness UI tests.
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37` passed with 15 harness UI tests.
- Regression: `cargo test -p tau-dashboard-ui` passed with 151 tests.
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit` passed.
- Static: `cargo fmt --check -p tau-dashboard-ui` passed.
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings` passed.
- Static: `git diff --check` passed.
- Browser fallback: `/tmp/tau-harness-continue8-after.png` captured the rendered preview; geometry reported document `clientWidth=1371`, `scrollWidth=1371`, DAG `clientWidth=446`, DAG `scrollWidth=446`, one DAG row, no per-node overflow, no evidence-table overflow, and no console errors.
- Browser Use caveat: in-app Browser attach was attempted through the required `iab` runtime, but the runtime reported no active Codex browser pane available, so visual verification used the bundled-browser fallback.
