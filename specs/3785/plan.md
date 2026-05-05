# Plan 3785: Review Queue Full-Label Readability

## Approach

Add a scoped readability contract for the Learning & Proposals queue:

- Mark the queue as a compact readable single-column list.
- Keep all four rows visible before Operator Actions.
- Use full-width rows with smaller but readable type so labels fit without ellipsis.
- Preserve the existing learning/proposal item DOM order and IDs.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3785/`

## Risks and Mitigations

- Risk: One-column queue may push later right-pane proof sections below the first viewport.
  Mitigation: Keep row height compact and verify the review pane plus TUI companion remain within the viewport.

- Risk: Smaller text can become hard to read.
  Mitigation: Use only a modest size reduction and keep full-width row contrast/borders.

## Interfaces

No Rust API, route, data, or wire-format changes. This is a rendered HTML/CSS contract.

## Verification

- Red/green conformance test for `functional_spec_3785`.
- Existing 377x and 378x harness regression tests.
- Full `tau-dashboard-ui` crate tests, fmt, clippy, gateway integration, and browser geometry.
