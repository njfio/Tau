# Spec: Issue #3772 - Harness compact mission state visibility

Status: Implemented

## Problem Statement

The compact harness dashboard table now fits the first viewport, but the mission state and gate state are hidden in compact desktop mode because later table columns are suppressed. Operators need each active mission row to show the mission state and verification gate state next to the goal, without widening the table.

## Scope

In scope:

- Add inline mission state chips inside the compact goal cell for every active mission row.
- Add inline gate state chips inside the compact goal cell for every active mission row.
- Keep the existing compact hidden-column behavior so the dashboard does not regain horizontal overflow.
- Add scoped compact chip CSS for mission row metadata.

Out of scope:

- Changing mission names, counts, plan progress, budgets, memory hits, or artifact counts.
- Reintroducing hidden columns at the 1400px compact breakpoint.
- Changing proof pane or self-improvement pane layout.

## Acceptance Criteria

### AC-1 Mission state remains visible in compact rows

Given `/ops/harness` renders in compact desktop mode, when the active mission table is inspected, then every mission row exposes an inline state chip in the goal cell.

### AC-2 Gate state remains visible in compact rows

Given `/ops/harness` renders in compact desktop mode, when the active mission table is inspected, then every mission row exposes an inline gate chip in the goal cell.

### AC-3 Compact table width remains protected

Given `/ops/harness` renders in compact desktop mode, when style markers are inspected, then later table columns remain hidden and mission metadata chips use compact wrapping CSS.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | active mission table | inspect row markup | inline mission state chips are present |
| C-02 | AC-2 | Functional | active mission table | inspect row markup | inline mission gate chips are present |
| C-03 | AC-3 | Functional | compact mission table | inspect style markers | hidden-column and compact chip markers are present |

## Success Metrics / Observable Signals

- Dashboard UI tests prove mission state and gate state are visible in compact goal cells.
- Existing compact dashboard tests remain green.
- Browser geometry still reports no horizontal document overflow at the desktop preview width.
- No new dependency is introduced.
