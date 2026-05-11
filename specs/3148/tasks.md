# Tasks: Issue #3148 - /ops/training status and control contracts

- [x] T1 (RED): Add failing conformance tests for C-01..C-05.
- [x] T2 (GREEN): Implement deterministic `/ops/training` status/rollout/optimizer/action contracts.
- [x] T3 (VERIFY): Run `spec_3148`, `spec_3144`, fmt, and scoped clippy.
- [x] T4 (DOC): Update issue/PR evidence and close milestone artifacts.
- [x] T5 (REGRESSION): Stop rendering pause/reset/export as inert GET links;
  keep them as disabled endpoint-marker-only controls until live action wiring
  exists.

## Follow-Up Evidence
### RED
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui functional_spec_3148_c03_training_route_renders_action_markers -- --nocapture`
  failed while training actions still rendered as enabled `/ops/training?action=...`
  links.

### GREEN
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui functional_spec_3148_c03_training_route_renders_action_markers -- --nocapture`
  passed after pause/reset/export rendered as disabled endpoint-marker-only
  buttons without inert GET hrefs.
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway integration_spec_3148_c04_ops_training_route_renders_training_contract_markers -- --nocapture`
  passed with the same server-rendered contract.
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui 3148 -- --nocapture`
  passed.
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway ops_config_training_safety -- --nocapture`
  passed.
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui -- --nocapture`
  passed.
- `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`
  passed.
- `git diff --check` passed.
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo build -p tau-coding-agent`
  passed.
- Live Browser and HTTP proof on `/ops/training` confirmed pause/reset/export
  are disabled buttons with `data-action-mode=endpoint-marker-only` and no inert
  `/ops/training?action=...` hrefs.
