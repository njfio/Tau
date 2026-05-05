# Tasks: Issue #3773 - Harness TUI companion first-viewport fit

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for TUI first-viewport fit.
- [x] T2 (RED): add dashboard UI regression test for TUI priority and compact bounds.
- [x] T3 (GREEN): tighten right-column/TUI height CSS and add the TUI priority marker.
- [x] T4 (VERIFY): run targeted/full tests, static checks, gateway integration, and browser geometry validation.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3773` failed before implementation because the TUI companion lacked `data-tui-priority="first-viewport-summary"`.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3773` passed after compact right-column bounds and the TUI priority marker were added.
- Regression: `cargo test -p tau-dashboard-ui functional_spec_3761` passed.
- Regression: `cargo test -p tau-dashboard-ui functional_spec_3764` passed.
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37` passed with 20 tests.
- Regression: `cargo test -p tau-dashboard-ui` passed with 156 tests and 0 doc-tests.
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit` passed.
- Static: `cargo fmt --check -p tau-dashboard-ui` passed.
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings` passed.
- Static: `git diff --check` passed.
- Browser fallback: `/tmp/tau-harness-continue13-after.png` confirms the TUI companion is fully visible in the first desktop viewport.
- Browser geometry: 1371x967 viewport reported document `scrollWidth=1371`, review bottom `705`, TUI box `y=742`/`bottom=942`, TUI pre `y=804`/`bottom=930`, `tuiBottomInsideViewport=true`, `tuiPreBottomInsideViewport=true`, `data-tui-priority=first-viewport-summary`, and console errors `[]`.
- Browser Use caveat: in-app Browser `iab` attach was attempted, but the runtime reported no active Codex browser pane in this thread.
