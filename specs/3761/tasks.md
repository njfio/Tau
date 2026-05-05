# Tasks: Issue #3761 - Harness desktop preview layout density

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for desktop preview layout density.
- [x] T2 (RED): add dashboard UI regression tests for desktop layout and bounded panes.
- [x] T3 (GREEN): adjust harness CSS breakpoints, pane minimums, and pane overflow bounds.
- [x] T4 (VERIFY): run targeted/full tests, static checks, and browser validation.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3761` initially failed because the desktop layout marker did not exist.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3761` -> 1 passed.
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37` -> 8 passed.
- Regression: `cargo test -p tau-dashboard-ui` -> 144 passed.
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit` -> 1 passed.
- Static: `cargo fmt --check -p tau-dashboard-ui` -> passed.
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings` -> passed.
- Browser Use: rendered `/tmp/tau-harness-after.html` at `file://`; DOM checks confirmed dashboard/proof/review/TUI/operator actions are visible and console log count is 0.
- Browser screenshot fallback: `/tmp/tau-harness-after-1371.png`; geometry check at 1371x967 reported `scrollWidth=1371`, `clientWidth=1371`, review right edge `1344`, TUI right edge `1344`, console errors `[]`.
