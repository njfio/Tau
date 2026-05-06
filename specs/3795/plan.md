# Issue 3795 Plan

## Approach

Keep the three-window harness grid for wide desktop viewports, but add an
intermediate responsive breakpoint for the in-app browser width. At that
breakpoint the page switches to two columns instead of allowing the review rail
to clip outside the shell.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`

## Risks

- A lower breakpoint can preserve the visual template but still clip on common
  browser widths.
- A single-column collapse at this width would be safe but unnecessarily
  reduce density.

## Mitigations

- Use a `1450px` breakpoint above the observed in-app browser viewport.
- Keep the existing `1180px` single-column collapse for narrower screens.
- Verify the live Browser Use route after rebuilding the gateway.

## Interfaces

No API, route, schema, or wire-format changes.
