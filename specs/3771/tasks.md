# Tasks: Issue #3771 - Harness verification gates first-screen priority

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for first-screen verification gate priority.
- [x] T2 (RED): add dashboard UI regression test for gate ordering and compact chip markers.
- [x] T3 (GREEN): reorder proof secondary sections and compact gate/acceptance chips.
- [x] T4 (VERIFY): run targeted/full tests, static checks, gateway integration, and browser geometry validation.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3771` failed before implementation because verification gates rendered after memory summary.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3771` passed after promoting gates after acceptance.
- Regression: `cargo test -p tau-dashboard-ui functional_spec_3770` passed.
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37` passed with 18 tests.
- Regression: `cargo test -p tau-dashboard-ui` passed with 154 tests and 0 doc-tests.
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit` passed.
- Static: `cargo fmt --check -p tau-dashboard-ui` passed.
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings` passed.
- Static: `git diff --check` passed.
- Browser fallback: `/tmp/tau-harness-continue11-after.png` confirms verification gates share the first proof viewport row with acceptance and precede memory/artifacts.
- Browser Use caveat: in-app Browser `iab` attach was attempted, but the runtime reported no active Codex browser pane in this thread.
