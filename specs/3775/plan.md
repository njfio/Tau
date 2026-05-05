# Plan 3775: Keep Canonical Benchmark Visible In Dashboard Viewport

## Approach

Constrain the Active Missions section with a desktop viewport budget and make
its table wrapper vertically scrollable. Add explicit dashboard markers for the
benchmark-visible budget and benchmark first-viewport anchor. This keeps mission
rows available while making the canonical autonomy benchmark visible without
requiring a full dashboard scroll.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`

## Risks and Mitigations

- Risk: Hiding mission rows behind internal scroll could reduce immediate
  mission visibility.
  Mitigation: Keep all rows in the DOM, preserve mission state chips, and use
  only a local scroll region for the mission table body area.
- Risk: Benchmark panel could still fall below smaller desktop viewports.
  Mitigation: Validate with a 1600x1000 viewport and preserve existing
  responsive collapse behavior for narrower screens.

## Interfaces

No route, endpoint, benchmark artifact, or schema change. This is a dashboard
layout and marker contract only.
