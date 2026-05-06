# Issue 3796 Plan

## Approach

Add a shared ops-console shell contract in `tau-dashboard-ui` so every protected
route gets real layout, panel, control, table, and active-nav styling by default.
Keep the existing harness selectors route-scoped so the mission harness retains
its compact template.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`

## Risks / Mitigations

- Risk: Broad CSS could disturb harness layout.
  Mitigation: Keep harness overrides route-scoped and rerun harness render
  contract tests.
- Risk: Active nav markup could break existing route link assertions.
  Mitigation: Preserve route hrefs and add only additive attributes.

## Interfaces / Contracts

- Add `data-shell-quality="operator-route-parity"` to the root ops shell.
- Add active nav `aria-current="page"` for the current route and `"false"` for
  inactive route links.
- Add shared CSS selectors for `#tau-ops-layout`, `#tau-ops-sidebar`, active
  nav links, visible protected panels, controls, tables, and forms.
