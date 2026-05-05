# Issue 3789 Plan

## Approach

Derive an operator-readable display label from each benchmark category while
leaving the stable `data-category` contract unchanged. Add a scoped benchmark
category fit marker and CSS for the rendered category label so the compact
left-column table can wrap at spaces instead of overflowing raw underscore IDs.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`

## Risks

- External consumers may rely on `data-category`, so it must not change.
- Overly broad table CSS could affect mission or proof tables.

## Mitigations

- Keep `data-category` exactly as the source category.
- Scope display-label CSS to `#tau-ops-harness-benchmark-panel`.
- Validate with targeted tests and browser geometry.

## Interfaces

No route, API, schema, proof-artifact, or wire-format changes.
