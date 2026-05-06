# Issue 3797: Channels Route Uses Operator Console Controls

Status: Implemented
Priority: P2
Milestone: Tau Agent Harness UI hardening

## Problem

The `/ops/channels` route inherits the shared operator shell, but the route body
still presents connector state as a plain table with unstyled action links. It is
functional enough for render contracts, but not usable as an operator surface.

## Scope

In scope:
- Add a channels-specific operator console contract.
- Surface online, offline, and degraded connector counts as first-scan cards.
- Present channel actions as bounded operator controls with disabled state.
- Keep the route as an adapter surface; do not add new channel integrations.
- Verify the live `/ops/channels` route in the in-app browser.

Out of scope:
- Changing channel gateway endpoints or connector behavior.
- Adding new channel providers.
- Reworking the harness page.

## Acceptance Criteria

AC-1: Given `/ops/channels` renders with connector rows, when the panel is
inspected, then it exposes channel-operator visual markers and first-scan KPI
cards for online, offline, and degraded connector counts.

AC-2: Given channel action links render, when the row is inspected, then actions
are grouped as bounded operator controls and disabled actions expose
`aria-disabled="true"`.

AC-3: Given the live `/ops/channels` route is loaded in Browser Use, when the DOM
and console are inspected, then the channel operator contract is present and
console errors are zero.

## Conformance Cases

C-01 maps to AC-1: `tau-dashboard-ui` render tests assert
`data-visual-contract="channel-operator-console"`, KPI card counts, and the
channel panel header contract.

C-02 maps to AC-2: `tau-dashboard-ui` render tests assert contained table
overflow, grouped action controls, `role="button"`, and disabled action
semantics.

C-03 maps to AC-3: Browser Use inspects the live localhost `/ops/channels` route
and verifies the channel operator contract with zero console errors.

## Success Signals

- `cargo test -p tau-dashboard-ui functional_spec_3797_c01_channels_route_exposes_operator_kpi_contract`
- `cargo test -p tau-dashboard-ui functional_spec_3797_c02_channels_route_groups_actions_as_controls`
- `cargo test -p tau-dashboard-ui`
- `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
- Browser Use DOM inspection of the live `/ops/channels` route.
