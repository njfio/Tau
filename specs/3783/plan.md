# Plan 3783: Right Review Pane Contained Proof Rows

## Approach

Add a scoped compact-density contract to the right review pane:

- Convert the learning queue into a two-column clipped-label grid so all four entries fit.
- Keep proposal detail at the existing first-viewport height but compress row gaps, text line height, and evidence-link chrome.
- Keep audit log at the existing first-viewport height but reduce row padding and line height so all four rows fit.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3783/`

## Risks and Mitigations

- Risk: Compact labels can truncate some explanatory text.
  Mitigation: Preserve full text in DOM and prioritize visible state/proof rows over verbose wrapping.

- Risk: Reducing evidence-link chrome could make the link less button-like.
  Mitigation: Scope the change to proposal proof links only; global action buttons remain unchanged.

## Interfaces

No Rust API, data, route, or wire-format changes. This is a rendered HTML/CSS contract.

## Verification

- Red/green conformance test for `functional_spec_3783`.
- Existing 377x and 378x harness regression tests.
- Full `tau-dashboard-ui` crate tests, fmt, clippy, gateway integration, and browser geometry.
