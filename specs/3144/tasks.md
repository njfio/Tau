# Tasks: Issue #3144 - /ops/config profile and policy control contracts

- [x] T1 (RED): Add failing conformance tests for C-01..C-04.
- [x] T2 (GREEN): Implement `/ops/config` profile and policy deterministic contract controls.
- [x] T3 (VERIFY): Run `spec_3144`, `spec_3140`, fmt, and scoped clippy.
- [ ] T4 (DOC): Update issue status/PR evidence and close milestone artifacts.
- [x] T5 (REGRESSION): Replace stale static config model defaults with
  runtime-backed profile metadata from the gateway server config.

## Verification Evidence

- RED: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui regression_spec_3144_config_route_does_not_render_stale_static_model_profile -- --nocapture`
  failed while `/ops/config` still rendered the stale `gpt-4.1-mini` profile.
- GREEN: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui 3144 -- --nocapture`
  passed (4 tests).
- INTEGRATION: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway integration_spec_3144_c03_ops_config_route_renders_profile_policy_contract_markers -- --nocapture`
  passed with the gateway server model, system prompt length, and max turns in
  the rendered config profile.
- REGRESSION: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui -- --nocapture`
  passed (203 tests, 0 doc tests).
- STATIC: `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`
  and `git diff --check` passed.
- STATIC: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo build -p tau-coding-agent`
  passed.
- LIVE: Rebuilt `tau-coding-agent` running on `127.0.0.1:8795` reported
  `auth.mode=localhost-dev`, `model=gpt-5.3-codex`, `service=running`, and
  state dir `.tau/gateway-live-demo`.
- LIVE: Browser on `/ops/config?theme=dark&sidebar=expanded&session=default`
  found the visible Model combobox selected to `gpt-5.3-codex`, Max Turns `8`,
  and `No fallback models configured`; HTTP proof showed
  `tau-ops-config-profile-controls` with `data-config-source="gateway-runtime"`,
  `data-model-ref="gpt-5.3-codex"`, `data-system-prompt-chars="74"`, and
  `data-max-turns="8"`.
