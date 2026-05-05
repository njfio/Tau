# Tasks: Issue #3766 - Harness compact navigation rail

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for compact harness navigation rail.
- [x] T2 (RED): add dashboard UI regression test for compact rail labels and width budget.
- [x] T3 (GREEN): add short labels and harness-route rail styling.
- [x] T4 (VERIFY): run targeted/full tests, static checks, gateway integration, and browser geometry validation.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3766` -> failed on missing `grid-template-columns: 76px minmax(0, 1fr);`.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3766` -> 1 passed.
- Focused regression: `cargo test -p tau-dashboard-ui functional_spec_376` -> 7 passed.
- Harness regression: `cargo test -p tau-dashboard-ui functional_spec_37` -> 13 passed.
- Full dashboard UI: `cargo test -p tau-dashboard-ui` -> 149 passed.
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit` -> 1 passed.
- Static: `cargo fmt --check -p tau-dashboard-ui` -> passed.
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings` -> passed.
- Browser Use: attempted to attach to the in-app browser, but the Browser Use runtime returned `No active Codex browser pane available`.
- Browser screenshot fallback: `/tmp/tau-harness-continue6-after.png`; geometry check at 1371x967 reported document `scrollWidth=1371`, `clientWidth=1371`, sidebar width `76`, harness panel width `1263`, and all compact navigation labels overflows `false`.
