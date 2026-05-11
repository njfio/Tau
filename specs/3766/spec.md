# Spec: Issue #3766 - Harness compact navigation rail

Status: Implemented

## Problem Statement

The harness preview now has readable mission, proof, and review panes, but the left navigation rail still uses full dashboard labels inside a narrow column. Several labels wrap awkwardly, which wastes vertical space and makes the harness feel less like the focused operator console shown in the design reference.

## Scope

In scope:

- Add short harness-only navigation labels for the left rail.
- Render the short labels only when the active route is `/ops/harness`.
- Reduce the harness route sidebar width and give the harness workspace the reclaimed width.
- Preserve all existing navigation item IDs, route destinations, and full labels
  for non-harness routes while carrying shell context through route links.
- Preserve selected harness subroute context when the active compact rail item
  points back into an already-selected harness history view.

Out of scope:

- Adding new icon assets or dependencies.
- Changing navigation destinations.
- Redesigning non-harness dashboard routes.
- Changing authentication or route-guard behavior.

## Acceptance Criteria

### AC-1 Harness route uses compact rail labels

Given `/ops/harness` renders, when the sidebar is inspected, then each harness rail anchor exposes a short label used by compact harness styling.

### AC-2 Navigation contracts are preserved

Given the compact rail renders, when IDs and hrefs are inspected, then existing
navigation IDs and route destinations remain intact and links preserve the
current theme, sidebar, session, and active harness history context where
applicable.

### AC-3 Compact rail gives space back to the harness workspace

Given the active route is harness, when CSS is inspected, then the layout uses a narrower navigation rail and the harness panel width budget is adjusted to match.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | harness route sidebar | inspect rendered HTML/CSS | compact rail labels and pseudo-label styling are present |
| C-02 | AC-2 | Functional | compact rail nav items | inspect rendered anchors | existing IDs/path destinations remain intact and shell/history context is preserved |
| C-03 | AC-3 | Functional | harness route layout | inspect CSS | grid and panel width use the compact rail budget |

## Success Metrics / Observable Signals

- Dashboard UI tests prove compact rail markers, preserved nav IDs, shell-context
  hrefs, active harness history hrefs, and adjusted width budget.
- Browser geometry shows no horizontal overflow at 1371px desktop preview width.
- Existing harness route tests remain green.
- No new dependency is introduced.
