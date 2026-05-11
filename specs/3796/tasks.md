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
- [x] T10: Remove legacy dashboard and standalone webchat adapter links from
  the Tau Ops left rail.
- [x] T11: Render visible Agent Fleet and Default Agent Detail route panels
  instead of leaving those first-class rail routes blank.

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
  and the then-present legacy dashboard/webchat adapter links carrying
  `theme=dark`, `sidebar=expanded`, `session=default`, and
  `data-preserves-shell-context="true"`. The adapter links were later removed
  from the Tau Ops rail in T10.
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
- RED: Live Browser on
  `/ops/login?theme=dark&sidebar=expanded&session=default` showed the Tau Ops
  left rail still linking to adapter surfaces `Legacy Dashboard` and `Webchat`,
  which route to standalone pages outside the operator shell.
- RED: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui functional_spec_2790_c01_sidebar_includes_15_ops_route_links -- --nocapture`
  failed before the adapter links were removed from the rail.
- GREEN: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui functional_spec_2790_c01_sidebar_includes_15_ops_route_links -- --nocapture`
  passed.
- REGRESSION: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui 2790 -- --nocapture`
  passed (3 tests).
- REGRESSION: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui 3796 -- --nocapture`
  passed (2 tests).
- REGRESSION: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway functional_spec_2790 -- --nocapture`
  passed (1 test).
- REGRESSION: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui -- --nocapture`
  passed (201 tests, 0 doc tests).
- STATIC: `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`
  and `git diff --check` passed.
- STATIC: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo build -p tau-coding-agent`
  passed.
- LIVE: Rebuilt `tau-coding-agent` running on `127.0.0.1:8795` reported
  `auth.mode=localhost-dev`, `model=gpt-5.3-codex`, and
  `service=running`; Browser on
  `/ops/login?theme=dark&sidebar=expanded&session=default` found zero
  `Legacy Dashboard` and zero `Webchat` links, `Operator Login` still marked
  current, first-class ops links preserving shell context, and the protected
  login payload still pruned.
- LIVE: HTTP proof showed `data-nav-item` count remained 15 while
  `Legacy Dashboard`, `tau-ops-nav-legacy-dashboard`, `/dashboard?`,
  `Webchat`, `tau-ops-nav-webchat`, and `/webchat?` were absent.
- RED: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui functional_spec_3796_c04_agent_routes_render_visible_operator_panels -- --nocapture`
  failed before the Agent Fleet and Agent Detail route panels existed.
- GREEN: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui functional_spec_3796_c04_agent_routes_render_visible_operator_panels -- --nocapture`
  passed after the route-specific agent panels were added.
- INTEGRATION: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway functional_spec_2794_c01_c02_c03_all_sidebar_ops_routes_return_shell_with_route_markers -- --nocapture`
  passed with live route marker checks for `/ops/agents` and
  `/ops/agents/default`.
- REGRESSION: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui 3796 -- --nocapture`
  passed (3 tests).
- REGRESSION: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui -- --nocapture`
  passed (202 tests, 0 doc tests).
- STATIC: `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`
  and `git diff --check` passed.
- STATIC: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo build -p tau-coding-agent`
  passed.
- LIVE: Rebuilt `tau-coding-agent` running on `127.0.0.1:8795` reported
  `auth.mode=localhost-dev`, `model=gpt-5.3-codex`, `service=running`, and
  state dir `.tau/gateway-live-demo`.
- LIVE: Browser on `/ops/agents?theme=dark&sidebar=expanded&session=default`
  found a visible `Agent Fleet` main heading, `Default Agent` summary, healthy
  runtime state, and `Open Default Agent` linking to
  `/ops/agents/default?theme=dark&sidebar=expanded&session=default`; HTTP proof
  showed `tau-ops-agent-fleet-panel` with `aria-hidden="false"` and
  `data-panel-visible="true"`.
- LIVE: Browser clicked through to
  `/ops/agents/default?theme=dark&sidebar=expanded&session=default` and found a
  visible `Agent Detail` main heading, `Default Agent` detail, active session
  `default`, healthy runtime state, and `Open Agent Fleet` linking back to
  `/ops/agents?theme=dark&sidebar=expanded&session=default`; HTTP proof showed
  `tau-ops-agent-detail-panel` with `aria-hidden="false"` and
  `data-panel-visible="true"`.
