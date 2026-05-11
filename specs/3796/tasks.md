# Issue 3796 Tasks

- [x] T1: Add failing render contracts for non-harness operator shell chrome.
- [x] T2: Add failing render contracts for active left-nav route state.
- [x] T3: Implement shared ops shell chrome and active nav attributes.
- [x] T4: Run targeted `tau-dashboard-ui` render tests and relevant harness regressions.
- [x] T5: Reload the live route and verify Browser Use DOM markers.
- [x] T6: Run final checks and commit.
- [x] T7: Hide internal contract-marker scaffolding from the live operator
  surface while preserving data-marker contracts.
- [x] T8: Preserve theme, sidebar, and session context through the shell left
  rail and breadcrumb navigation.
- [x] T9: Preserve selected harness history context through the active harness
  rail item.

## Verification Evidence

- GREEN: `cargo test -p tau-dashboard-ui regression_ops_internal_contract_sections_are_hidden_from_operator_surface -- --nocapture`
  passed (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui harness -- --nocapture` passed
  (52 tests).
- REGRESSION: `cargo test -p tau-dashboard-ui 3796 -- --nocapture` passed
  (2 tests).
- STATIC: `cargo fmt --check --package tau-dashboard-ui` passed.
- STATIC: `git diff --check` passed.
- STATIC: `cargo clippy -p tau-dashboard-ui -- -D warnings` passed.
- BUILD: `cargo build -p tau-coding-agent` passed.
- LIVE: Browser on `/ops/harness?view=history` found the accessibility,
  stream, and performance contract sections still present with
  `data-operator-visible="false"` and `hidden`, while their headings had
  zero role matches and the sections were not visible.
- RED: Live Browser on
  `/ops/harness?proposal_id=PR-045&view=history&audit_action=dry-run` showed
  the left rail and breadcrumb routes as bare `/ops/...` hrefs, dropping
  `theme`, `sidebar`, and `session`.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_37 -- --nocapture`
  passed with updated route chrome, harness, compact rail, and active-nav
  contracts (45 tests).
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_2790_c01_sidebar_includes_15_ops_route_links -- --nocapture`
  passed with all 15 route links preserving shell context (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui -- --nocapture` passed
  (194 tests).
- STATIC: `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`
  passed.
- STATIC: `git diff --check` passed.
- STATIC: `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `cargo build -p tau-coding-agent` passed.
- LIVE: Browser on
  `/ops/harness?proposal_id=PR-045&view=history` against the restarted
  `127.0.0.1:8795` harness found breadcrumb home, every left rail ops route,
  legacy dashboard, and webchat all carrying `theme=dark`, `sidebar=expanded`,
  `session=default`, and `data-preserves-shell-context="true"`.
- LIVE: Browser navigated via the Channels rail href to
  `/ops/channels?theme=dark&sidebar=expanded&session=default`, found one
  channels panel, and returned to the selected harness history URL.
- RED: Live Browser on
  `/ops/harness?proposal_id=PR-045&view=history` showed the active Mission
  Harness rail item preserving only shell context, so clicking it reset the
  selected proposal/history subroute.
- GREEN: `cargo test -p tau-dashboard-ui functional_harness_history_view_surfaces_state_audit_summary -- --nocapture`
  passed with the active harness rail href preserving selected proposal,
  history filter, and audit ref (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui functional_spec_37 -- --nocapture`
  passed after the compact rail and route guard expectations were updated
  (45 tests).
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
  against the restarted `127.0.0.1:8795` harness found the active Mission
  Harness rail href preserving `proposal_id=PR-045`, `view=history`,
  `audit_action=dry-run`, and `audit_ref=1778419944988`; navigating through
  that href kept the history view and selected audit detail open.
