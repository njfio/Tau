# Tasks: Issue #3756 - Mission harness operator UI and TUI proof view

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for the harness UI adapter.
- [x] T2 (RED): add route, panel, benchmark, policy, and TUI expectations.
- [x] T3 (GREEN): add the `/ops/harness` dashboard route and template panels.
- [x] T4 (GREEN): wire `/ops/harness` through the gateway shell router.
- [x] T5 (GREEN): wire deterministic benchmark proof and proposal review actions.
- [x] T6 (GREEN): add the TUI harness summary panel.
- [x] T7 (VERIFY): run scoped UI/TUI/gateway tests, live browser/HTTP checks,
  fmt, and clippy.

## Verification Evidence

- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3756` passed
  (2 tests).
- GREEN: `cargo test -p tau-dashboard-ui regression_spec_3756_c03` passed
  (1 test).
- GREEN: `cargo test -p tau-tui functional_operator_shell_renderer_includes_harness_summary`
  passed (1 test).
- GREEN: `cargo test -p tau-gateway integration_spec_3756_c04` passed
  (1 test).
- GREEN: `cargo test -p tau-gateway integration_spec_3756_c05` passed
  (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui` passed (138 tests).
- REGRESSION: `cargo test -p tau-tui` passed (89 lib tests, 22 main tests,
  5 smoke tests).
- REGRESSION: `cargo test -p tau-gateway` passed (350 tests, 1 ignored).
- LIVE UI: Chrome loaded `http://127.0.0.1:18787/ops/harness` and the first
  viewport rendered the dark Mission Harness workspace with the left rail,
  dashboard, proof, self-improvement review, TUI companion, and disabled apply
  control visible.
- LIVE ACTION: clicking the visible Run Benchmark control redirected to
  `/ops/harness?benchmark_status=passed&benchmark_tasks=4`.
- LIVE HTTP: `jq` inspection of
  `/tmp/tau-harness-gateway-state/ops-harness/m334/latest.json` reported
  `benchmark_id=m334-tranche-one-autonomy`, `passed=true`, `task_count=4`, and
  `failure_count=0`.
- LIVE HTTP: `POST /ops/harness/proposals/PR-044/apply` returned `403`, while
  the diff route rendered `id="tau-ops-harness-diff"`.
- MANUAL: `cargo run -p tau-tui -- shell --width 96 --profile ops-west --no-color`
  rendered the HARNESS panel with M334, `run_8f3a2`, 4/4 passed, and
  `latest.json`.
- FORMAT: `cargo fmt --check -p tau-dashboard-ui -p tau-gateway -p tau-tui`
  passed.
- CLIPPY: `cargo clippy -p tau-dashboard-ui -p tau-gateway -p tau-tui --all-targets -- -D warnings`
  passed.
