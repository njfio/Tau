# Issue 3796: Ops Routes Use Operator Shell Chrome

Status: Accepted
Priority: P2
Milestone: Tau Agent Harness UI hardening

## Problem

The live `/ops/harness` route now uses a custom operator console, but the other
left-rail routes still render the raw Leptos foundation scaffold. The routes
work, but they look and feel unfinished when an operator leaves the harness
surface for deploy, agents, chat, memory, tools, or safety.

## Scope

In scope:
- Add shared operator shell chrome for non-harness ops routes.
- Keep the existing harness-specific compact rail and mission-control overrides.
- Mark the active left-nav route so operators can tell where they are.
- Preserve theme, sidebar, and session context when operators use the left rail
  or shell breadcrumb to move between ops routes.
- Preserve selected harness proposal/history context when the active harness
  rail item is clicked from a harness history subroute.
- Keep internal contract-marker scaffolding out of the visible operator surface
  while preserving its machine-readable data markers.
- Keep the Tau Ops left rail scoped to first-class `/ops/*` routes; legacy
  dashboard and standalone webchat remain reachable as adapters but are not
  promoted in the ops rail.
- Verify the live `/ops/deploy` route in the in-app browser.
- Route-isolate Agent Fleet and Default Agent Detail so they do not ship hidden
  chat/session/harness/deploy payloads behind the agent page.

Out of scope:
- Rewriting individual route business functionality.
- Adding new gateway/channel integrations.
- Changing harness mission, benchmark, or self-improvement behavior.

## Acceptance Criteria

AC-1: Given a non-harness ops route renders, when the shell HTML is inspected,
then it exposes the shared operator-route visual contract and no longer presents
the foundation-shell subtitle or internal contract-marker headings as the
visible route experience.

AC-2: Given an operator opens a left-nav route, when the sidebar renders, then
exactly that route is marked as current, receives the active nav styling
contract, and each shell navigation target preserves the current theme, sidebar,
session context, plus active harness history context for the current harness
route.

AC-3: Given the live `/ops/deploy` route is loaded in Browser Use, when the DOM
and console are inspected, then the route exposes the operator-route contract and
console errors are zero.

AC-4: Given the Tau Ops left rail renders, when an operator scans navigation,
then the rail contains the first-class ops routes and omits legacy dashboard and
standalone webchat adapter links.

AC-5: Given an operator opens the Agent Fleet or Default Agent Detail route,
when the main operator shell renders, then the route exposes a visible
route-specific panel instead of an empty protected shell and preserves drill-down
or back navigation with the active shell context.

AC-6: Given an operator opens the Agent Fleet or Default Agent Detail route,
when the HTML is inspected, then non-agent route payloads are pruned from that
agent page and replaced by a deterministic `agent-route-pruned` marker.

## Conformance Cases

C-01 maps to AC-1: `tau-dashboard-ui` render tests assert global operator shell
style markers, route panel styling, hidden internal contract scaffolding, and
the absence of the old foundation subtitle on `/ops/deploy`.

C-02 maps to AC-2: `tau-dashboard-ui` render tests assert `/ops/deploy` and
`/ops/chat` each mark only their own nav link with `aria-current="page"` and
that shell navigation links carry the active theme/sidebar/session query; the
harness history render test asserts the active harness rail link also preserves
selected proposal, history filter, and audit ref.

C-03 maps to AC-3: Browser Use inspects the live localhost `/ops/deploy` route
and verifies the operator-route contract marker with zero console errors.

C-04 maps to AC-4: `tau-dashboard-ui` render tests assert the left rail contains
the expected ops route links and omits `/dashboard` and `/webchat`.

C-05 maps to AC-5: `tau-dashboard-ui` render tests and `tau-gateway` route tests
assert `/ops/agents` renders `tau-ops-agent-fleet-panel`,
`/ops/agents/default` renders `tau-ops-agent-detail-panel`, and both routes
expose context-preserving navigation between the fleet and default agent detail.

C-06 maps to AC-6: `tau-dashboard-ui` render tests and `tau-gateway` route
tests assert Agent Fleet and Default Agent Detail ship
`tau-ops-agent-route-payload-pruned` and omit hidden chat/session/harness/deploy
payload panels.

## Success Signals

- `cargo test -p tau-dashboard-ui functional_spec_3796_c01_non_harness_routes_use_operator_shell_chrome`
- `cargo test -p tau-dashboard-ui functional_spec_3796_c02_left_nav_marks_active_route`
- `cargo test -p tau-dashboard-ui regression_ops_internal_contract_sections_are_hidden_from_operator_surface`
- Browser Use DOM inspection of the live `/ops/deploy` route.
- Browser Use DOM inspection from a live harness history route shows left rail
  links preserving `theme`, `sidebar`, and `session`.
- Browser Use DOM inspection from a selected harness history record shows the
  active harness rail link preserving `proposal_id`, `view`, `audit_action`, and
  `audit_ref`.
- Browser Use DOM inspection of a live ops route shows no legacy dashboard or
  webchat links in the Tau Ops left rail.
- Browser Use DOM inspection of live `/ops/agents` and `/ops/agents/default`
  shows visible route-specific agent panels instead of a blank main region.
- Browser Use and HTTP inspection of live `/ops/agents/default` show the
  `agent-route-pruned` marker and no hidden non-agent payload panels.
