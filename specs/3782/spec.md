# Spec 3782: Left Dashboard Table Overflow Removal

Status: Reviewed

## Problem

The Tau Agent Harness left dashboard column still shows clipped table cells in the Active Missions and M334 benchmark sections. Partial numeric columns at the right edge make the operator surface look broken and force horizontal scanning in the first viewport.

## Acceptance Criteria

AC-1: Given `/ops/harness` is rendered in the desktop harness layout, when the Active Missions table is shown in the left dashboard column, then no mission table content extends beyond the visible table wrapper.

AC-2: Given `/ops/harness` is rendered in the desktop harness layout, when the M334 benchmark proof table is shown below Active Missions, then no benchmark table content extends beyond the visible table wrapper.

AC-3: Given the left dashboard uses compact evidence tables, when secondary mission columns are hidden to fit the column, then mission status and verification gates remain visible in the compact goal cell.

## Scope

In scope:
- Tau Agent Harness dashboard HTML/CSS table fit contract.
- Active Missions and M334 benchmark first-viewport table layout.
- Conformance coverage for left-column table overflow markers.

Out of scope:
- Changing mission or benchmark data.
- Backend benchmark execution.
- Gateway routing.

## Conformance Cases

C-01 maps AC-1: Render `/ops/harness` and assert Active Missions declares a no-horizontal-overflow table fit contract with compact mission columns.

C-02 maps AC-2: Render `/ops/harness` and assert the M334 benchmark panel declares the same left table fit contract.

C-03 maps AC-3: Render `/ops/harness` and assert compact goal-cell mission state and gate chips remain present after secondary columns are removed from the left-column view.

## Success Metrics

- `cargo test -p tau-dashboard-ui functional_spec_3782` passes.
- Browser geometry confirms Active Missions and benchmark tables stay within their wrappers.
- No horizontal document overflow is introduced.
