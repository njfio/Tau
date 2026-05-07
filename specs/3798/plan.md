# Issue 3798 Plan

## Approach

Keep gateway/channel breadth as an adapter concern, but make the adapter surface
honest. Reuse the existing multi-channel lifecycle runtime and add only the
ops-shell glue needed for form submission. Seed the live demo with local state
files that match the existing multi-channel status schema so the browser route
can prove configured channels without external provider credentials.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `crates/tau-gateway/src/gateway_openresponses/endpoints.rs`
- `crates/tau-gateway/src/gateway_openresponses/server_bootstrap.rs`
- `crates/tau-gateway/src/gateway_openresponses/ops_dashboard_shell.rs`
- `crates/tau-gateway/src/gateway_openresponses/channel_telemetry_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/ops_shell_controls.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests/ops_tools_channels.rs`

## Risks / Mitigations

- Risk: Channel lifecycle API and ops-shell action flow diverge.
  Mitigation: Reuse the same lifecycle command builder and runtime function.
- Risk: External provider credentials are unavailable in local dev.
  Mitigation: Use offline probe mode and seed connector health state locally.
- Risk: Existing action marker tests depend on stable IDs.
  Mitigation: Preserve IDs and data-action markers on the new button controls.

## Interfaces / Contracts

- `POST /ops/channels/action`
- Form fields: `channel`, `action`, `theme`, `sidebar`, `session`
- Redirect query markers:
  `channel_action_status`, `channel_action`, `channel_action_channel`,
  `channel_action_reason`
- `.tau/multi-channel/state.json`
- `.tau/multi-channel/live-connectors-state.json`
- `.tau/multi-channel/security/channel-lifecycle.json`
