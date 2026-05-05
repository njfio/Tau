# Tasks: Issue #3762 - Harness compact dashboard readability

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for compact dashboard readability.
- [x] T2 (RED): add dashboard UI regression test for compact dashboard/table/proof markers.
- [x] T3 (GREEN): adjust compact desktop CSS for sidebar, KPI grid, mission table, and proof header.
- [x] T4 (VERIFY): run targeted/full tests, static checks, and browser geometry validation.

## Verification Evidence

- RED: temporary worktree at `22fd82d0` with only the new test patch ran `cargo test -p tau-dashboard-ui functional_spec_3762` -> failed on missing `data-compact-dashboard-breakpoint="1400px"`.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3762` -> 1 passed.
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37` -> 9 passed.
- Regression: `cargo test -p tau-dashboard-ui` -> 145 passed.
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit` -> 1 passed.
- Static: `cargo fmt --check -p tau-dashboard-ui` -> passed.
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings` -> passed.
- Browser Use: attempted to attach to the in-app browser, but the Browser Use runtime returned `No active Codex browser pane available`.
- Browser screenshot fallback: `/tmp/tau-harness-continue-after.png`; geometry check at 1371x967 reported document `scrollWidth=1371`, `clientWidth=1371`, dashboard `scrollWidth=361`, dashboard `clientWidth=361`, mission table `scrollWidth=339`, mission table `clientWidth=339`, review right edge `1344`, TUI right edge `1344`, console errors `[]`.
