# Issue 3791 Plan

## Approach

Add an explicit vertical no-clipping contract to the proposal detail card and
slightly increase only that card's compact max-height. Keep existing detail row
density, row text, and audit ordering intact.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`

## Risks

- Increasing the proposal detail height could push the audit log or TUI
  companion down.

## Mitigations

- Use the smallest max-height increase that removes hidden overflow.
- Validate the rendered preview geometry for proposal detail and document
  overflow.

## Interfaces

No API, route, proof schema, or wire-format changes.
