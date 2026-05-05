# Tasks: Issue #3764 - Harness self-improvement action priority

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for self-improvement action priority.
- [x] T2 (RED): add dashboard UI regression test for action placement and compact controls.
- [x] T3 (GREEN): move operator actions above policy and compact the action grid.
- [x] T4 (VERIFY): run targeted/full tests, static checks, gateway integration, and browser geometry validation.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3764` -> failed on `operator actions should be prioritized before conservative policy`.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3764` -> 1 passed.
- Focused regression: `cargo test -p tau-dashboard-ui functional_spec_376` -> 6 passed.
- Harness regression: `cargo test -p tau-dashboard-ui functional_spec_37` -> 12 passed.
- Full dashboard UI: `cargo test -p tau-dashboard-ui` -> 148 passed.
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit` -> 1 passed.
- Static: `cargo fmt --check -p tau-dashboard-ui` -> passed.
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings` -> passed.
- Diff hygiene: `git diff --check` -> passed.
- Browser Use: attempted to attach to the in-app browser, but the Browser Use runtime returned `No active Codex browser pane available`.
- Browser screenshot fallback: `/tmp/tau-harness-continue5-after.png`; geometry check at 1371x967 reported document `scrollWidth=1371`, `clientWidth=1371`, actions before policy `true`, policy before audit `true`, and actions fully inside review viewport `true`.
