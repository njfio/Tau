# Plan 3784: Center Proof Evidence Containment

## Approach

Add scoped center-proof containment contracts:

- Mark Tool Execution Evidence as `compact-no-overflow`.
- Apply table-layout, ellipsis, and column width rules at the section level rather than only through viewport media queries.
- Mark Acceptance Criteria as `all-criteria-visible` and render the chips in a compact single-column grid.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3784/`

## Risks and Mitigations

- Risk: Compact evidence rows can hide full artifact paths visually.
  Mitigation: Keep artifact paths in the DOM and use ellipsis only for visual containment.

- Risk: Hiding call IDs removes one proof detail from the first screen.
  Mitigation: Preserve tool, plan node, runtime, status, and artifact, which are the operator-facing proof columns.

## Interfaces

No Rust API, route, data, or wire-format changes. This is a rendered HTML/CSS contract.

## Verification

- Red/green conformance test for `functional_spec_3784`.
- Existing 377x and 378x harness regression tests.
- Full `tau-dashboard-ui` crate tests, fmt, clippy, gateway integration, and browser geometry.
