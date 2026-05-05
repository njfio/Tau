# Plan 3782: Left Dashboard Table Overflow Removal

## Approach

Add a focused left-column table fit contract:

- Mark Active Missions and M334 benchmark panels as compact no-overflow table regions.
- Apply scoped CSS outside the existing viewport media query because the dashboard column remains narrow even on wide desktop viewports.
- Keep mission status and gate chips in the goal cell, so hiding secondary columns does not remove essential operator state.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3782/`

## Risks and Mitigations

- Risk: Hiding mission columns could remove useful context.
  Mitigation: Preserve acceptance and plan columns plus inline mission state and gate chips in the goal cell.

- Risk: Duplicating compact table rules could drift from responsive rules.
  Mitigation: Scope the new contract to panels declaring `data-left-table-fit="compact-no-overflow"`.

## Interfaces

No Rust API, route, data, or wire-format changes. This is a rendered HTML/CSS contract.

## Verification

- Red/green conformance test for `functional_spec_3782`.
- Existing harness regression filters.
- Full `tau-dashboard-ui` crate tests, fmt, clippy, gateway integration, and browser geometry.
