# Tasks: Issue #3132 - ops channels action contracts

- [x] T1 (RED): Add UI + gateway conformance tests for C-01..C-04.
- [x] T2 (GREEN): Add deterministic channel action markers and enabled-state contracts in `/ops/channels` rows.
- [x] T3 (VERIFY): Run `spec_3132` + `spec_3128` regressions + fmt/clippy gates.
- [x] T4 (DOC): Update PR evidence and close issue/milestone artifacts.
- [x] T5 (REGRESSION): Require disabled channel actions to render with native
  `disabled` button semantics, not only `aria-disabled`.

## Follow-Up Evidence
### RED
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui functional_spec_3797_c02_channels_route_groups_actions_as_controls -- --nocapture`
  failed while disabled channel Login actions lacked native `disabled`.

### GREEN
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui functional_spec_3797_c02_channels_route_groups_actions_as_controls -- --nocapture`
  passed after native disabled semantics were added.
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway integration_spec_3132_c03_ops_channels_route_renders_channel_action_contracts -- --nocapture`
  passed with the same contract in the gateway route.
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui 3132 -- --nocapture`
  passed.
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui 3797 -- --nocapture`
  passed.
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway ops_tools_channels -- --nocapture`
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
- Live Browser and HTTP proof on `/ops/channels` confirmed open-channel Login
  buttons now render with `aria-disabled=true disabled`, while Logout and Probe
  remain enabled submit buttons.
