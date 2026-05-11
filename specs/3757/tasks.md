# Tasks: Issue #3757 - State-backed harness benchmark and audit panels

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for state-backed harness rendering.
- [x] T2 (RED): add dashboard and gateway tests for state-backed benchmark and
  audit rendering.
- [x] T3 (GREEN): add harness snapshot structs and dynamic dashboard rendering.
- [x] T4 (GREEN): collect harness proof and audit state in the gateway shell.
- [x] T5 (VERIFY): run focused tests, fmt, clippy, and live smoke if needed.
- [x] T6 (GREEN): link state-backed tool evidence artifact cells to the
  harness artifact view.
- [x] T7 (GREEN): render a dedicated `view=history` audit summary with state
  source, proof-link count, route-backed action filters, latest action, and
  audit-log anchor.
- [x] T8 (GREEN): render route-backed selected audit row detail with inspect
  links and proof artifact continuity inside the history view.
- [x] T9 (GREEN): render capped inline selected proof artifact previews for
  safe `ops-harness/...` audit artifacts.
- [x] T10 (GREEN): upgrade safe harness artifact views with JSON metadata,
  top-level key proof, capped payload preview, raw artifact access, and
  context-preserving return links.
- [x] T11 (GREEN): promote the active history-route audit summary ahead of the
  default dashboard content.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3757` failed before
  implementation because `TauOpsDashboardHarnessSnapshot` did not exist.
- RED: `cargo test -p tau-gateway integration_spec_3757` failed before
  implementation because `/ops/harness` did not expose `data-proof-source`.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3757` passed
  (1 test).
- GREEN: `cargo test -p tau-gateway integration_spec_3757` passed
  (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui` passed (139 tests).
- REGRESSION: `cargo test -p tau-gateway` passed (351 tests, 1 ignored).
- REGRESSION: `cargo test -p tau-tui` passed (89 lib tests, 22 binary tests,
  5 demo smoke tests).
- STATIC: `cargo fmt --check -p tau-dashboard-ui -p tau-gateway` passed.
- STATIC: `cargo clippy -p tau-dashboard-ui -p tau-gateway --all-targets -- -D warnings`
  passed.
- GREEN: `cargo test -p tau-dashboard-ui regression_harness_tool_evidence_links_state_backed_proof_artifact`
  passed (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui harness` passed (51 tests).
- GREEN: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
  passed (1 test).
- REGRESSION: `cargo test -p tau-gateway ops_harness -- --test-threads=1`
  passed (6 tests).
- STATIC: `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `cargo build -p tau-coding-agent` passed.
- LIVE: Browser on `/ops/harness?view=history&audit_action=dry-run` found one
  state-backed history summary, the Dry Run filter selected, and 4 shown audit
  rows out of 20 total audit records.
- LIVE: Browser click on the Benchmark filter navigated to
  `/ops/harness?view=history&audit_action=run-benchmark` with the Benchmark
  filter selected and 1 shown audit row out of 20 total audit records.
- LIVE: Browser on `http://127.0.0.1:8795/ops/harness` found 5
  `data-tool-proof-artifact-href="true"` links and 5 state-backed tool rows.
- GREEN: `cargo test -p tau-dashboard-ui functional_harness_history_view_surfaces_state_audit_summary`
  passed (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui harness` passed (52 tests).
- REGRESSION: `cargo test -p tau-gateway ops_harness -- --test-threads=1`
  passed (6 tests).
- STATIC: `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `cargo build -p tau-coding-agent` passed.
- LIVE: Browser on `/ops/harness?view=history` found one history summary,
  `data-history-source="state"`, 4 audit rows, 2 proof links, and one audit
  anchor.
- GREEN: `cargo test -p tau-dashboard-ui functional_harness_history_view_surfaces_state_audit_summary`
  passed with route-backed history filter markers (1 test).
- GREEN: `cargo test -p tau-gateway unit_requested_harness_route_action_normalizes_supported_values`
  passed with supported and invalid `audit_action` parsing (1 test).
- GREEN: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
  passed with filtered `audit_action=run-benchmark` history proof (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui harness` passed (52 tests).
- REGRESSION: `cargo test -p tau-gateway ops_harness -- --test-threads=1`
  passed (6 tests).
- STATIC: `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- STATIC: `git diff --check` passed.
- BUILD: `cargo build -p tau-coding-agent` passed.
- T13: Hide the duplicate topbar route-action banner when `view=history`
  promotes the dedicated audit-history panel, including the CSS rule that makes
  hidden route-action markers actually non-visible in browser layout.
- GREEN: `cargo test -p tau-dashboard-ui functional_harness_history_view_surfaces_state_audit_summary -- --nocapture`
  passed with the history route-action marker hidden, CSS `[hidden]` selector
  present, and the primary history panel still visible (1 test).
- GREEN: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit -- --nocapture`
  passed with the hidden history route-action contract through the gateway
  route (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui harness -- --nocapture` passed
  (52 tests).
- REGRESSION: `cargo test -p tau-gateway ops_harness -- --test-threads=1 --nocapture`
  passed (6 tests).
- STATIC: `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`
  passed.
- STATIC: `git diff --check` passed.
- STATIC: `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `cargo build -p tau-coding-agent` passed.
- LIVE: Browser on `/ops/harness?view=history` against the restarted
  `127.0.0.1:8795` harness found
  `#tau-ops-harness-route-action[data-route-action-key="history"]` with
  `data-route-action-visible="false"`, `hidden`, and `isVisible=false`; the
  primary `#tau-ops-harness-history-view` remained visible with
  `data-history-route-priority="primary"`, source `state`, row count `4`, and
  exactly one visible `Applied History` mention.
- T14: Scope state-backed history rows to the selected proposal before applying
  the route-backed action filter, and clarify the history summary copy so the
  operator knows the rows are for the selected proposal.
- RED: Live Browser on
  `/ops/harness?proposal_id=PR-045&view=history&audit_action=apply` showed
  `data-history-selected-proposal="PR-045"` while the audit table still
  included a visible `PR-044` apply row.
- GREEN: `cargo test -p tau-dashboard-ui functional_harness_history_view_surfaces_state_audit_summary -- --nocapture`
  passed with selected-proposal history copy and proposal label markers (1
  test).
- GREEN: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit -- --nocapture`
  passed with a mixed-proposal audit regression proving PR-045 rows do not leak
  into a PR-044 history route (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui harness -- --nocapture` passed
  (52 tests).
- REGRESSION: `cargo test -p tau-gateway ops_harness -- --test-threads=1 --nocapture`
  passed (6 tests).
- STATIC: `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`
  passed.
- STATIC: `git diff --check` passed.
- STATIC: `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `cargo build -p tau-coding-agent` passed.
- LIVE: Browser on
  `/ops/harness?proposal_id=PR-045&view=history&audit_action=apply` against the
  restarted `127.0.0.1:8795` harness showed row count `3`, total count `17`,
  selected proposal `PR-045`, latest `Apply PR-045 Applied`, selected-proposal
  copy present, and no visible `PR-044` rows. The tab was then restored to
  `/ops/harness?proposal_id=PR-045&view=history`, which showed row count `4`,
  total count `17`, filter `all`, selected proposal `PR-045`, and no visible
  `PR-044` rows.
- GREEN: `cargo test -p tau-dashboard-ui functional_harness_history_view_surfaces_state_audit_summary`
  passed with context-preserving selected artifact links (1 test).
- GREEN: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
  passed with structured harness artifact view markers (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui harness` passed (52 tests).
- REGRESSION: `cargo test -p tau-gateway ops_harness -- --test-threads=1`
  passed (6 tests).
- STATIC: `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- STATIC: `cargo fmt --check -p tau-dashboard-ui -p tau-gateway` passed.
- STATIC: `git diff --check` passed.
- BUILD: `cargo build -p tau-coding-agent` passed.
- LIVE: Browser on `/ops/harness?view=history&audit_ref=1778419581966`
  found one selected proof link preserving `theme`, `sidebar`, `session`,
  `proposal_id`, `view=history`, and `audit_ref`.
- LIVE: Browser click opened
  `/ops/harness/artifacts/view/ops-harness/self-improvement/PR-045/dry-run-result.json`
  with `data-artifact-json="true"`, JSON kind `object`, 6 top-level keys,
  195 bytes, `truncated=false`, raw artifact access, and a history return link.
- GREEN: `cargo test -p tau-dashboard-ui functional_harness_history_view_surfaces_state_audit_summary -- --nocapture`
  passed with history-route priority before the default dashboard (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui harness -- --nocapture` passed
  (52 tests).
- GREEN: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit -- --nocapture`
  passed with state and filtered history priority markers (1 test).
- REGRESSION: `cargo test -p tau-gateway ops_harness -- --test-threads=1 --nocapture`
  passed (6 tests).
- STATIC: `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`
  passed.
- STATIC: `git diff --check` passed.
- STATIC: `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `cargo build -p tau-coding-agent` passed.
- LIVE: Browser on `/ops/harness?view=history` found one state-backed history
  panel with `data-history-route-priority="primary"` and confirmed the history
  section appears before the default dashboard in live HTML and visible text.
- LIVE: Browser on `/ops/harness?view=history` found 4 Inspect links; clicking
  an audit row navigated to `audit_ref=1778419581966`, selected exactly one
  matching audit row, and rendered the selected detail panel with proof artifact
  `ops-harness/self-improvement/PR-045/dry-run-result.json`.
- LIVE: Browser on `/ops/harness?view=history&audit_ref=1778419581966`
  rendered one selected proof preview with `loaded` status, 195 bytes shown,
  2048-byte cap, `truncated=false`, and preview text containing `proposal_id`
  plus `PR-045`.
- GREEN: `cargo test -p tau-dashboard-ui functional_harness_history_view_surfaces_state_audit_summary`
  passed with selected proof preview markers (1 test).
- GREEN: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
  passed with capped state-backed benchmark proof preview (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui harness` passed (52 tests).
- REGRESSION: `cargo test -p tau-gateway ops_harness -- --test-threads=1`
  passed (6 tests).
- STATIC: `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- STATIC: `git diff --check` passed.
- BUILD: `cargo build -p tau-coding-agent` passed.
- GREEN: `cargo test -p tau-dashboard-ui functional_harness_history_view_surfaces_state_audit_summary`
  passed with selected audit detail and inspect-link markers (1 test).
- GREEN: `cargo test -p tau-gateway unit_requested_harness_route_action_normalizes_supported_values`
  passed with supported `audit_ref` parsing and sanitization (1 test).
- GREEN: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
  passed with route-backed selected benchmark audit detail (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui harness` passed (52 tests).
- REGRESSION: `cargo test -p tau-gateway ops_harness -- --test-threads=1`
  passed (6 tests).
- STATIC: `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- STATIC: `git diff --check` passed.
- BUILD: `cargo build -p tau-coding-agent` passed.
