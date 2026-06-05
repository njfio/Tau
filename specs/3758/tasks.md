# Tasks: Issue #3758 - Supervised deploy/stop process lifecycle control

## Ordered Tasks

1. [x] T1 (RED): add failing process lifecycle tests for C-01, C-02, C-03, and C-05.
2. [x] T2 (GREEN): add deploy process supervisor trait and command-backed implementation.
3. [x] T3 (GREEN): wire configured supervisor into deploy/stop handlers and persisted state.
4. [x] T4 (REGRESSION): verify existing unauthorized, invalid, unknown, and discovery contracts remain green.
5. [x] T5 (GREEN): add cortex, quality, dependency graph, security cadence, key-rotation, and dashboard stack documentation.
6. [x] T6 (VERIFY): run scoped fmt/check/clippy/test commands and record evidence.
7. [x] T7 (GREEN): render persisted deploy process evidence in `/ops/deploy`.
8. [x] T8 (HARDEN): add JSON static args and graceful terminate-then-kill stop behavior for command supervisor.
9. [x] T9 (GREEN): wire `/ops/deploy` browser form posts to deploy/stop lifecycle handlers and validate through the shell.
10. [x] T10 (REGRESSION): add rapid same-agent redeploy/stop drill for C-12.
11. [x] T11 (REGRESSION): add Cortex fallback-readiness gate coverage for C-13.
12. [x] T12 (VERIFY): rerun release blocker checks (`cargo audit`; `cargo deny check advisories bans sources`) and focused gateway regressions.
13. [x] T13 (HARDEN): make heavy `/ops/deploy` render regressions carry their own stack budget so grouped `spec_3758` tests do not require `RUST_MIN_STACK`.
14. [x] T14 (REGRESSION): prove Cortex fallback readiness follows the latest chat event timestamp, not log-file order.

## Tier Mapping

| Tier | Status | Tests | N/A Why |
|---|---|---|---|
| Unit | ✅ | deploy runtime helpers; deploy process arg parsing | |
| Property | N/A | | no randomized invariant introduced in this process-lifecycle slice |
| Contract/DbC | N/A | | no `contracts` macro boundary added |
| Snapshot | N/A | | explicit JSON field assertions cover the payload |
| Functional | ✅ | `integration_spec_3758_c01_c02_c05_deploy_and_stop_spawn_and_terminate_configured_process`; `integration_spec_3758_c11_ops_deploy_form_spawns_and_stop_form_terminates_process`; `spec_3758_c09_deploy_route_renders_process_lifecycle_evidence` | |
| Conformance | ✅ | C-01..C-13 covered by tests/docs and verification commands below | |
| Integration | ✅ | HTTP deploy/stop with command supervisor | |
| Fuzz | N/A | | no parser/codec fuzz boundary changed |
| Mutation | N/A | | bounded endpoint/runtime slice; mutation gate deferred unless critical-path policy requires it |
| Regression | ✅ | `regression_spec_3758_c03_spawn_failure_returns_error_without_deploying_state`; `regression_spec_3758_c12_deploy_race_drill_replaces_running_child_and_idempotently_stops`; `regression_spec_2953_c03_c04_cortex_chat_provider_failure_uses_deterministic_fallback_and_reason_code`; `unit_load_cortex_status_report_uses_latest_chat_timestamp_for_fallback_gate`; `spec_2697` suite | |
| Performance | N/A | | no throughput/hot-path budget changed |

## Verification Evidence

- RELEASE BLOCKER: `cargo audit` passed after updating Wasmtime to `36.0.8`.
- RELEASE BLOCKER: `cargo deny check advisories bans sources` passed after
  updating `deny.toml` to the current cargo-deny advisory lint syntax. Duplicate
  crate findings remain warn-level under the existing bans policy.
- REGRESSION: `CARGO_INCREMENTAL=0
  CARGO_TARGET_DIR=/tmp/rust_pi-codex-issue-3758-verify-target cargo test -p
  tau-gateway cortex_status_report -- --nocapture` passed (`6 passed`) for
  missing-artifact, healthy, fallback, stale, missing-chat, and timestamp-order
  Cortex readiness gates.
- REGRESSION: `cargo test -p tau-gateway
  regression_spec_2953_c03_c04_cortex_chat_provider_failure_uses_deterministic_fallback_and_reason_code
  -- --nocapture` passed (`1 passed`) and confirmed `/cortex/status` stays
  `degraded`/`hold` after fallback output.
- REGRESSION: `cargo test -p tau-gateway
  regression_spec_3758_c12_deploy_race_drill_replaces_running_child_and_idempotently_stops
  -- --nocapture` passed (`1 passed`) for rapid same-agent redeploy and repeated
  stop behavior.
- CONFORMANCE: `CARGO_INCREMENTAL=0
  CARGO_TARGET_DIR=/tmp/rust_pi-codex-issue-3758-deploy-stop-process-lifecycle-fast-validate-full-target
  cargo test -p tau-gateway spec_3758 -- --nocapture` passed (`4 passed`)
  without `RUST_MIN_STACK` after moving heavy `/ops/deploy` render regressions
  onto a high-stack test runtime.
- STATIC: `cargo fmt --check`, `git diff --check`, and
  `CARGO_INCREMENTAL=0
  CARGO_TARGET_DIR=/tmp/rust_pi-codex-issue-3758-deploy-stop-process-lifecycle-fast-validate-full-target
  cargo clippy -p tau-gateway --tests -- -D warnings` passed.
- RELEASE-GRADE: `RUST_MIN_STACK=16777216 scripts/dev/fast-validate.sh --full`
  passed, including `cargo fmt --all -- --check`, workspace clippy, workspace
  tests, and doc-tests in an isolated target directory.
- RED/GREEN: `cargo test -p tau-gateway spec_3758 -- --nocapture` passed
  (`2 passed`) after adding process lifecycle tests and implementation.
- REGRESSION: `cargo test -p tau-gateway spec_2697 -- --nocapture` passed
  (`3 passed`) for existing deploy/stop endpoint contracts.
- FUNCTIONAL: `cargo test -p tau-dashboard-ui spec_3758_c09 -- --nocapture`
  passed (`1 passed`) for the deploy process lifecycle renderer without
  requiring the full shell stack.
- FUNCTIONAL/UI: `RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui
  deploy_route -- --nocapture` passed (`6 passed`) for deploy route shell
  form, stop control, process table, and contrast CSS markers.
- FUNCTIONAL/UI: `RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui
  spec_3758 -- --nocapture` passed (`4 passed`) for process lifecycle
  rendering and encoded row stop-form path segments.
- UNIT: `cargo test -p tau-gateway deploy_process -- --nocapture` passed
  (`3 passed`) for legacy whitespace args and JSON args preserving quoted
  argument boundaries.
- UNIT: `cargo test -p tau-gateway
  unit_collect_tau_ops_dashboard_deploy_snapshot_maps_process_rows --
  --nocapture` passed (`1 passed`) for operator-shell deploy row projection.
- CONFORMANCE: `RUST_MIN_STACK=16777216 cargo test -p tau-gateway spec_3758 --
  --nocapture` passed (`3 passed`) after wiring `/ops/deploy` process evidence
  and browser form lifecycle handlers.
- LIVE UI: local runtime launched with `TAU_GATEWAY_DEPLOY_PROCESS_PROGRAM=/bin/sh`
  and static loop args on `127.0.0.1:8797`; Playwright submitted the
  `/ops/deploy` form for `agent-ui-live-2`, observed `process_status=running`
  with pid `50573`, submitted the row stop form, and observed
  `process_status=stopped`, `process_stop_reason=operator_stop_request`, and a
  disabled stop button. Computed row colors were `rgb(219, 232, 239)` while
  running and `rgb(184, 203, 213)` after stop.
- REGRESSION: `RUST_MIN_STACK=16777216 cargo test -p tau-gateway --
  --nocapture` passed (`379 passed`, `1 ignored`). The same command without
  `RUST_MIN_STACK` overflowed in an existing ops chat shell render test before
  reaching completion.
- STATIC: `cargo fmt --check -p tau-gateway`, `cargo fmt --check -p
  tau-dashboard-ui`, `cargo fmt --check`, and `git diff --check` passed.
- STATIC: `cargo clippy -j1 -p tau-gateway -p tau-dashboard-ui --tests --
  -D warnings` passed after replacing deploy-runtime match/return blocks with
  `?`.
- STATIC: `cargo fmt --check` initially failed on pre-existing formatting drift
  in `tau-memory` and `tau-training-runner`; `cargo fmt -p tau-memory -p
  tau-training-runner` was applied, then `cargo fmt --check` passed.
- DOCS: `scripts/dev/crate-dependency-graph.sh --output-json
  tasks/reports/crate-dependency-graph.json --output-md
  tasks/reports/crate-dependency-graph.md --generated-at
  2026-05-17T00:00:00Z` generated `45` crates and `202` workspace edges.
- DOCS: `rg` checks confirmed advisory-only Cortex, Leptos SSR dashboard
  direction, deploy process supervisor environment variables, release freshness
  cadence, key-rotation runbook, and published crate graph counts.
