# Tasks: Issue #3124 - ops job cancel action contracts

- [x] T1 (RED): Add UI + gateway conformance tests for C-01..C-04.
- [x] T2 (GREEN): Add cancel query parsing + deterministic cancel outcome resolution in gateway jobs fixtures.
- [x] T3 (GREEN): Add cancel action markers and cancel panel contracts in dashboard UI.
- [x] T4 (VERIFY): Run `spec_3124` + nearby regressions + fmt/clippy gates.
- [x] T5 (DOC): Update PR evidence and close issue/milestone artifacts.
- [x] T6 (REGRESSION): Render disabled terminal-job cancel affordances as
  non-link `Cancel unavailable` text while preserving deterministic markers.

## Evidence
### RED
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui functional_spec_3124_c01_c02_tools_route_renders_job_cancel_action_markers -- --nocapture`
  failed while completed/cancelled job cancel affordances still rendered as
  clickable anchors.

### GREEN
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui functional_spec_3124_c01_c02_tools_route_renders_job_cancel_action_markers -- --nocapture`
  passed after terminal cancel affordances rendered as disabled text.
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui 3124 -- --nocapture`
  passed.
- `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway 3124 -- --nocapture`
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
- Live Browser proof on `/ops/tools-jobs` confirmed `job-001` still exposes a
  `Cancel` link while terminal `job-002` and `job-003` render non-link
  `Cancel unavailable` text with `aria-disabled=true`.
