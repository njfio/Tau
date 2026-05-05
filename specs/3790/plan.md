# Issue 3790 Plan

## Approach

Add a focused no-overflow contract to the Tool Evidence table and rebalance
only its compact visible columns. Allocate enough width to the Tool column for
`memory.search` / `memory.write`, slightly compact Plan Node, Runtime, and
Status columns, and leave the remaining width to Artifact.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`

## Risks

- Reducing supporting column widths could clip status or runtime values.
- Widening Tool too much could clip artifact paths.

## Mitigations

- Keep the total fixed-width budget roughly unchanged.
- Validate all visible Tool Evidence cells with browser geometry.

## Interfaces

No API, route, proof schema, or wire-format changes.
