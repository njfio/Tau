# Spec: Issue #3762 - Harness compact dashboard readability

Status: Implemented

## Problem Statement

The harness preview now keeps the three-window desktop layout, but the dashboard pane still clips its KPI cards and active mission table at ordinary desktop browser widths. Operators need the dashboard, proof metadata, and navigation rail to remain readable without horizontal scanning or broken words.

## Scope

In scope:

- Keep the harness desktop grid while making the dashboard pane readable at the current preview width.
- Prevent KPI cards and mission rows from clipping inside the dashboard window.
- Keep the active mission table structure while hiding low-priority columns at compact desktop widths.
- Keep proof metadata readable by stacking the proof header before values wrap into fragments.
- Prevent sidebar labels from splitting inside words.

Out of scope:

- Replacing the dashboard table with a new component.
- Adding client-side resizing or JavaScript layout logic.
- Changing gateway state, mission data, forms, or actions.

## Acceptance Criteria

### AC-1 Dashboard cards fit the compact desktop pane

Given `/ops/harness` renders at the current desktop preview width, when the dashboard pane is visible, then KPI cards use a two-column compact arrangement instead of clipping horizontally.

### AC-2 Mission table does not force dashboard horizontal scrolling

Given the active mission table renders in the dashboard pane, when compact desktop styles apply, then the table uses a fixed-width compact layout, wraps visible text, and hides low-priority columns after the plan column.

### AC-3 Proof metadata and sidebar labels remain readable

Given the proof view and route sidebar render at compact desktop width, when long labels are present, then proof metadata stacks without fragmented values and sidebar labels do not split inside words.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | harness dashboard route | inspect rendered CSS | compact KPI grid marker is present |
| C-02 | AC-2 | Functional | harness mission table | inspect rendered CSS | mission table is fixed-width and hides columns 4+ at compact desktop width |
| C-03 | AC-3 | Functional | harness proof/sidebar route | inspect rendered CSS | proof header stacks and sidebar word-splitting is disabled |

## Success Metrics / Observable Signals

- Dashboard UI tests prove the compact dashboard markers.
- Existing harness desktop layout tests remain green.
- Browser geometry validation reports no document horizontal overflow at 1371px.
- The dashboard mission table scroll width does not exceed its client width at 1371px.
- No new dependency is introduced.
