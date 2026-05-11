# Issue 3797 Tasks

- [x] T1: Add failing render contract for channel KPI/operator markers.
- [x] T2: Add failing render contract for grouped action controls.
- [x] T3: Implement channels operator console markup and scoped styling.
- [x] T4: Run targeted and full `tau-dashboard-ui` validation.
- [x] T5: Reload the live route and verify Browser Use DOM markers.
- [x] T6: Commit and push.
- [x] T7: Regress disabled channel controls so unavailable actions render with
  both `aria-disabled` and native `disabled` button semantics.

## Follow-Up Evidence
- RED:
  `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui functional_spec_3797_c02_channels_route_groups_actions_as_controls -- --nocapture`
  failed while disabled Login controls only rendered `aria-disabled`.
- GREEN:
  `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui functional_spec_3797_c02_channels_route_groups_actions_as_controls -- --nocapture`
  passed after native disabled semantics were wired into channel action buttons.
- Gateway:
  `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway integration_spec_3132_c03_ops_channels_route_renders_channel_action_contracts -- --nocapture`
  passed with the same native-disabled marker in server-rendered HTML.
- Full validation:
  `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui -- --nocapture`,
  `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway ops_tools_channels -- --nocapture`,
  `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`,
  `git diff --check`,
  `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`,
  and `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo build -p tau-coding-agent`
  passed.
- Live Browser proof on `/ops/channels` confirmed Login is native-disabled for
  already-open channels while Logout and Probe remain enabled.
