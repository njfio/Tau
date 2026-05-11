# Tasks: Issue #2939 - Deploy Agent wizard panel and conformance tests

## Ordered Tasks
1. [x] T1 (RED): add failing conformance tests for C-01..C-05 in `tau-dashboard-ui` render tests.
2. [x] T2 (GREEN): add route-aware render function and deploy panel markers for `/ops/deploy`.
3. [x] T3 (GREEN): keep baseline shell marker tests green via compatibility wrapper.
4. [x] T4 (REGRESSION): verify non-deploy routes omit deploy markers.
5. [x] T5 (VERIFY): run fmt, clippy, and scoped tests; update spec status to Implemented.
6. [x] T6 (REGRESSION): replace static deploy model catalog options with
   runtime-backed gateway model metadata.

## Tier Mapping
- Unit: render helper coverage for deploy/non-deploy route behavior.
- Property: N/A (static marker contracts).
- Contract/DbC: N/A (no contracts macro in module).
- Snapshot: N/A (marker asserts are explicit).
- Functional: deploy route markers present.
- Conformance: C-01..C-06 mapped to `spec_c0x_*` and regression tests.
- Integration: gateway route navigation test verifies `/ops/deploy` renders the
  runtime-backed model catalog through the server shell.
- Fuzz: N/A (no parser/untrusted input path).
- Mutation: N/A (UI marker scaffolding task; low algorithmic branching).
- Regression: non-deploy routes exclude deploy panel markers.
- Performance: N/A (no runtime-critical path change).

## Verification Evidence

- RED: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui regression_deploy_model_catalog_uses_gateway_runtime_model -- --nocapture`
  failed while `/ops/deploy` still rendered static `gpt-4.1-mini` /
  `gpt-4.1` model options.
- GREEN: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui regression_deploy_model_catalog_uses_gateway_runtime_model -- --nocapture`
  passed after the deploy model catalog was backed by gateway runtime model
  metadata.
- SCOPED: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui deploy -- --nocapture`
  passed all six deploy route tests.
- INTEGRATION: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway functional_spec_2794_c01_c02_c03_all_sidebar_ops_routes_return_shell_with_route_markers -- --nocapture`
  passed with `/ops/deploy` asserting gateway runtime model catalog markers.
- FULL UI: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui -- --nocapture`
  passed 204 tests plus doc tests.
- HYGIENE: `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`,
  `git diff --check`, and
  `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo build -p tau-coding-agent`
  passed.
- LIVE: restarted `tau-8795` from the rebuilt binary and verified
  `http://127.0.0.1:8795/gateway/status` reported `model:
  gpt-5.3-codex`, `service_status: running`, `auth mode:
  localhost-dev`, and `state_dir: .tau/gateway-live-demo`.
- BROWSER: `agent-browser` opened
  `http://127.0.0.1:8795/ops/deploy?theme=dark&sidebar=expanded&session=default`;
  the interactive snapshot showed Model Catalog option `gpt-5.3-codex`
  selected, and live HTML exposed `data-model-source="gateway-runtime"` with
  no stale `gpt-4.1-mini` option.
