# Tasks: Issue #3766 - Harness compact navigation rail

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for compact harness navigation rail.
- [x] T2 (RED): add dashboard UI regression test for compact rail labels and width budget.
- [x] T3 (GREEN): add short labels and harness-route rail styling.
- [x] T4 (VERIFY): run targeted/full tests, static checks, gateway integration, and browser geometry validation.
- [x] T5 (REGRESSION): preserve shell context through compact rail links while
  keeping route destinations and compact labels intact.
- [x] T6 (REGRESSION): preserve selected harness history context through the
  active compact rail item.

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
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_37 -- --nocapture`
  passed with compact rail links preserving `theme`, `sidebar`, and `session`
  while keeping active-route state intact (45 tests).
- REGRESSION: `cargo test -p tau-dashboard-ui -- --nocapture` passed
  (194 tests).
- STATIC: `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`
  passed.
- STATIC: `git diff --check` passed.
- STATIC: `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `cargo build -p tau-coding-agent` passed.
- LIVE: Browser on
  `/ops/harness?proposal_id=PR-045&view=history` found compact rail links
  preserving `theme=dark`, `sidebar=expanded`, and `session=default`.
- GREEN: `cargo test -p tau-dashboard-ui functional_harness_history_view_surfaces_state_audit_summary -- --nocapture`
  passed with the active harness rail link preserving selected history context
  (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui functional_spec_37 -- --nocapture`
  passed (45 tests).
- REGRESSION: `cargo test -p tau-dashboard-ui -- --nocapture` passed
  (194 tests).
- STATIC: `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`
  passed.
- STATIC: `git diff --check` passed.
- STATIC: `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `cargo build -p tau-coding-agent` passed.
- LIVE: Browser on
  `/ops/harness?proposal_id=PR-045&view=history&audit_action=dry-run&audit_ref=1778419944988`
  found the active Mission Harness compact rail link preserving the selected
  proposal, history view, action filter, and audit ref.
