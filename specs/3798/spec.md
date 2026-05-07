# Issue 3798: Channels Route Runs Live Lifecycle Actions

Status: Implemented
Priority: P2
Milestone: Tau Agent Harness UI hardening

## Problem

The `/ops/channels` route can render connector health from multi-channel state,
but the live demo currently has no connector state. The route also renders
action controls that look operational while only linking back to the page with
query parameters. Operators need a real end-to-end path: visible connector
state, a POST action, persisted lifecycle state, and a refreshed route.

## Scope

In scope:
- Add an ops-shell POST action for channel lifecycle controls.
- Render channel actions as submit controls with explicit action forms.
- Preserve existing gateway `/gateway/channels/{channel}/lifecycle` API.
- Seed the live demo multi-channel state with realistic connector rows.
- Verify the live route, lifecycle API, persisted state, and browser surface.

Out of scope:
- Adding real provider credentials.
- Performing online provider probes against Telegram, Discord, or WhatsApp.
- Adding new channel providers.

## Acceptance Criteria

AC-1: Given `/ops/channels` renders connector rows, when an operator action is
submitted from the ops shell, then Tau invokes the multi-channel lifecycle
runtime and persists `.tau/multi-channel/security/channel-lifecycle.json`.

AC-2: Given channel action controls render, when inspected, then each channel
row exposes a POST form targeting the ops channel action endpoint with channel,
theme, sidebar, session, and clicked-button action fields.

AC-3: Given the live localhost harness is running, when connector state is
seeded and `/ops/channels` is loaded, then configured channel rows are visible
instead of fallback `none` state.

AC-4: Given live channel actions are exercised, when gateway status and browser
DOM are inspected, then connector state is present, lifecycle state is present,
and browser console errors are zero.

## Conformance Cases

C-01 maps to AC-1: gateway integration test posts `/ops/channels/action`,
asserts redirect markers, and verifies lifecycle state persistence.

C-02 maps to AC-2: dashboard render test asserts channel actions use a POST
form with shared hidden context fields and clicked submit-button action values.

C-03 maps to AC-3 and AC-4: live validation seeds `.tau/multi-channel` state,
exercises lifecycle actions, checks `/gateway/status`, and inspects the
in-app browser route.

## Success Signals

- `cargo test -p tau-dashboard-ui functional_spec_3798_c02_channels_route_posts_lifecycle_action_forms`
- `cargo test -p tau-gateway integration_spec_3798_c01_ops_channels_action_persists_lifecycle_state`
- Live `/gateway/status` reports `multi_channel.state_present=true` and
  `multi_channel.connectors.state_present=true`.
- Browser Use confirms `/ops/channels` has configured channel rows and no
  console errors.
