# Issue 3788 Plan

## Approach

Add an explicit KPI label fit contract to the harness KPI grid and scope CSS to
that contract. The CSS should compact KPI headings enough for the fixed
four-card dashboard row while restoring normal word-boundary wrapping so long
labels do not split inside words.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`

## Risks

- Reducing heading size too much could make KPI labels harder to scan.
- Removing `anywhere` wrapping globally would regress other dense proof tables.

## Mitigations

- Scope the change only to `#tau-ops-harness-kpi-grid`.
- Validate with targeted Rust tests and a desktop browser geometry check.

## Interfaces

No API, route, schema, or wire-format changes.
