# Plan: Issue #2814 - Command-center timeline chart and range SSR markers

## Approach
1. Add RED integration tests for `/ops` timeline chart/range selector marker contracts.
2. Extend `tau-dashboard-ui` command-center snapshot and shell markup with timeline chart/range selector marker block.
3. Extend `ops_shell_controls` query parser for `range` with safe fallback defaults.
4. Map queue timeline point metadata + selected range into command-center shell context.
5. Re-run phase-1A..1H regressions and full validation gates.

## Affected Modules
- `specs/milestones/m137/index.md`
- `specs/2814/spec.md`
- `specs/2814/plan.md`
- `specs/2814/tasks.md`
- `crates/tau-gateway/src/gateway_openresponses/ops_shell_controls.rs`
- `crates/tau-gateway/src/gateway_openresponses/dashboard_status.rs`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `crates/tau-dashboard-ui/src/lib.rs`

## Risks and Mitigations
- Risk: route control URL assertions regress due new range links.
  - Mitigation: preserve existing theme/sidebar control URLs; add separate timeline range links.
- Risk: oversized gateway file growth.
  - Mitigation: keep mapping logic in `dashboard_status.rs` and minimal call-site changes.
- Risk: mutation escapes in range parsing logic.
  - Mitigation: add explicit conformance tests for valid + invalid range states.

## Interface and Contract Notes
- Extends `OpsShellControlsQuery` with `range` parsing helper.
- Extends `TauOpsDashboardCommandCenterSnapshot` with timeline range + chart metadata fields.
- No endpoint additions or protocol changes.
