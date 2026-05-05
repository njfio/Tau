# Spec: Issue #3761 - Harness desktop preview layout density

Status: Implemented

## Problem Statement

The harness route has the correct panes and static-preview guards, but the desktop preview still collapses into a long single-column page at ordinary browser widths. Operators need the rendered `/ops/harness` preview to keep the dashboard, proof view, self-improvement review, and TUI companion visible as a mission-control workspace instead of forcing vertical scanning.

## Scope

In scope:

- Keep the three-window harness grid active at normal desktop preview widths.
- Move single-column collapse to a genuinely narrow breakpoint.
- Constrain tall harness windows with internal scrolling so proof and review panes remain in the first operator viewport.
- Ensure long review labels, proof metadata, and action text wrap inside their panes instead of widening the grid.
- Preserve existing IDs, data markers, forms, and static-preview guards.

Out of scope:

- New harness data sources.
- Client-side resizing or drag-to-resize.
- Replacing the existing Leptos SSR template.
- Changing gateway route behavior.

## Acceptance Criteria

### AC-1 Desktop preview keeps the three-window layout

Given `/ops/harness` is opened in the in-app browser at ordinary desktop width, when the route renders, then the harness panel keeps dashboard, proof, review, and TUI regions in the mission-control grid rather than collapsing to one column.

### AC-2 Narrow screens still collapse predictably

Given the route renders below the narrow breakpoint, when the style contract is inspected, then the harness falls back to a single-column order without losing any pane.

### AC-3 Tall panes do not push sibling panes below the fold

Given default harness data includes tables, proof details, proposal details, and TUI output, when the desktop layout renders, then the major panes have bounded viewport heights, internal overflow handling, and text wrapping that prevents horizontal pane expansion.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | harness route context | render shell | panel exposes desktop three-window layout markers |
| C-02 | AC-2 | Functional | rendered shell | inspect responsive style | single-column collapse is scoped to the narrow breakpoint |
| C-03 | AC-3 | Functional | rendered shell | inspect pane style | tall windows have viewport max-height and overflow handling |

## Success Metrics / Observable Signals

- Dashboard UI tests prove the desktop layout-density contract.
- Existing harness preview guards remain green.
- Browser Use DOM checks confirm the proof, review, dashboard, TUI, and operator actions render at the current preview width.
- Fallback browser screenshot/geometry validation confirms the panes fit inside a 1371px viewport without horizontal overflow.
- No new dependency is introduced.
