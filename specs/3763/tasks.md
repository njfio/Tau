# Tasks: Issue #3763 - Harness proof evidence and log wrapping

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for proof evidence and log wrapping.
- [x] T2 (RED): add dashboard UI regression test for compact proof evidence/log wrapping.
- [x] T3 (GREEN): adjust compact desktop CSS for proof table and preformatted logs.
- [x] T4 (VERIFY): run targeted/full tests, static checks, gateway integration, and browser geometry validation.

## Verification Evidence

- RED: temporary worktree at `201ad485` with only the new test patch ran `cargo test -p tau-dashboard-ui functional_spec_3763` -> failed on missing `data-compact-evidence-breakpoint="1400px"`.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3763` -> 1 passed.
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37` -> 10 passed.
- Regression: `cargo test -p tau-dashboard-ui` -> 146 passed.
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit` -> 1 passed.
- Static: `cargo fmt --check -p tau-dashboard-ui` -> passed.
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings` -> passed.
- Browser Use: attempted to attach to the in-app browser, but the Browser Use runtime returned `No active Codex browser pane available`.
- Browser screenshot fallback: `/tmp/tau-harness-continue3-after.png`; geometry check at 1371x967 reported document `scrollWidth=1371`, `clientWidth=1371`, tool evidence table `scrollWidth=417`, `clientWidth=417`, operator log `scrollWidth=417`, `clientWidth=417`, TUI pre `scrollWidth=357`, `clientWidth=357`, console errors `[]`.
