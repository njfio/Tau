# Tasks: Issue #3765 - Harness compact evidence column priority

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for compact evidence column priority.
- [x] T2 (RED): add dashboard UI regression test for compact evidence call ID hiding and runtime nowrap.
- [x] T3 (GREEN): update compact proof evidence CSS.
- [x] T4 (VERIFY): run targeted/full tests, static checks, gateway integration, and browser geometry validation.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3765` -> failed on missing `data-compact-call-id-visibility="hidden-at-1400px"`.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_376` -> 6 passed, including `functional_spec_3765`.
- Harness regression: `cargo test -p tau-dashboard-ui functional_spec_37` -> 12 passed.
- Full dashboard UI: `cargo test -p tau-dashboard-ui` -> 148 passed.
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit` -> 1 passed.
- Static: `cargo fmt --check -p tau-dashboard-ui` -> passed.
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings` -> passed.
- Diff hygiene: `git diff --check` -> passed.
- Browser Use: attempted to attach to the in-app browser, but the Browser Use runtime returned `No active Codex browser pane available`.
- Browser screenshot fallback: `/tmp/tau-harness-continue5-after.png`; geometry check at 1371x967 reported document `scrollWidth=1371`, `clientWidth=1371`, tool evidence wrapper `scrollWidth=417`, `clientWidth=417`, call ID displays `none`, and all runtime cells overflows `false`.
