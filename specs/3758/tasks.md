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

## Tier Mapping

| Tier | Status | Tests | N/A Why |
|---|---|---|---|
| Unit | ✅ | deploy runtime helpers; deploy process arg parsing | |
| Property | N/A | | no randomized invariant introduced in this process-lifecycle slice |
| Contract/DbC | N/A | | no `contracts` macro boundary added |
| Snapshot | N/A | | explicit JSON field assertions cover the payload |
| Functional | ✅ | `integration_spec_3758_c01_c02_c05_deploy_and_stop_spawn_and_terminate_configured_process`; `spec_3758_c09_deploy_route_renders_process_lifecycle_evidence` | |
| Conformance | ✅ | C-01..C-10 covered by tests/docs and verification commands below | |
| Integration | ✅ | HTTP deploy/stop with command supervisor | |
| Fuzz | N/A | | no parser/codec fuzz boundary changed |
| Mutation | N/A | | bounded endpoint/runtime slice; mutation gate deferred unless critical-path policy requires it |
| Regression | ✅ | `regression_spec_3758_c03_spawn_failure_returns_error_without_deploying_state`; `spec_2697` suite | |
| Performance | N/A | | no throughput/hot-path budget changed |

## Verification Evidence

- RED/GREEN: `cargo test -p tau-gateway spec_3758 -- --nocapture` passed
  (`2 passed`) after adding process lifecycle tests and implementation.
- REGRESSION: `cargo test -p tau-gateway spec_2697 -- --nocapture` passed
  (`3 passed`) for existing deploy/stop endpoint contracts.
- FUNCTIONAL: `cargo test -p tau-dashboard-ui spec_3758_c09 -- --nocapture`
  passed (`1 passed`) for the deploy process lifecycle renderer without
  requiring the full shell stack.
- UNIT: `cargo test -p tau-gateway deploy_process -- --nocapture` passed
  (`3 passed`) for legacy whitespace args and JSON args preserving quoted
  argument boundaries.
- UNIT: `cargo test -p tau-gateway
  unit_collect_tau_ops_dashboard_deploy_snapshot_maps_process_rows --
  --nocapture` passed (`1 passed`) for operator-shell deploy row projection.
- CONFORMANCE: `RUST_MIN_STACK=16777216 cargo test -p tau-gateway spec_3758 --
  --nocapture` passed (`2 passed`) after wiring `/ops/deploy` process evidence.
- REGRESSION: `RUST_MIN_STACK=16777216 cargo test -p tau-gateway --
  --nocapture` passed (`379 passed`, `1 ignored`). The same command without
  `RUST_MIN_STACK` overflowed in an existing ops chat shell render test before
  reaching completion.
- STATIC: `cargo fmt --check -p tau-gateway`, `cargo fmt --check -p
  tau-dashboard-ui`, `cargo fmt --check`, and `git diff --check` passed.
- STATIC: `cargo clippy -j1 -p tau-gateway -p tau-dashboard-ui --tests --
  -D warnings` passed.
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
