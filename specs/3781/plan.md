# Plan 3781: Active Missions Whole-Row Scroll Boundary

## Approach

Add a narrow dashboard contract for the Active Missions first-viewport budget:

- Mark the Active Missions section with a whole-row scroll-boundary contract.
- Keep the mission table wrapper as the scrollable region.
- Add a specific CSS override that ends the visible table at a complete mission row.
- Keep all five mission rows in the DOM for scrolling and state inspection.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3781/`

## Risks and Mitigations

- Risk: Tightening the mission table height could hide too much active work.
  Mitigation: Keep all rows in the DOM and expose a visible-row budget.

- Risk: Changing the existing 3775 viewport contract could regress the benchmark panel.
  Mitigation: Preserve the existing benchmark markers and add a more specific whole-row contract.

## Interfaces

No Rust API or wire-format changes. This is an HTML/CSS rendering contract only.

## Verification

- Red/green conformance test for `functional_spec_3781`.
- Existing 377x/378x harness tests.
- `tau-dashboard-ui` crate tests, fmt, clippy.
- Browser-generated static preview geometry.
