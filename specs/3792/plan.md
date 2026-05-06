# Issue 3792 Plan

## Approach

Keep the existing desktop grid and make the proof footer contracts responsive
inside the already-supported `1400px` breakpoint. Use a smaller acceptance-chip
label size at that width and give the verification-gate section enough compact
height for all five gate chips.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`

## Risks

- Increasing the gate section height could push lower proof content down.
- Reducing acceptance label font size could make labels too small if applied
  globally.

## Mitigations

- Scope both adjustments to the in-app/narrow desktop breakpoint and explicit
  proof fit markers.
- Validate actual browser geometry at 1370x968 after rendering the static
  preview.

## Interfaces

No API, route, proof schema, or wire-format changes.
