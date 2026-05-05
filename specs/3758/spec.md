# Spec: Issue #3758 - Mission harness operator UI usability pass

Status: Implemented

## Problem Statement

The `/ops/harness` route has the right functional primitives, but the current presentation is visually noisy, cramped, and hard to scan. Operators need the harness to look and behave like a deliberate mission-control workspace rather than a raw fixture dump.

## Scope

In scope:

- Rework the harness route styling to match the provided dark Tau Agent Harness template.
- Improve first-viewport hierarchy for dashboard, proof, self-improvement, and TUI companion panes.
- Add compact window chrome, status chips, progress bars, table containment, and responsive layout guards.
- Preserve existing element ids, forms, action routes, data markers, and tests.
- Add regression tests for the usability/design contract.

Out of scope:

- New backend state sources.
- New dependencies or UI framework migration.
- Replacing Leptos SSR with a client-side app.

## Acceptance Criteria

### AC-1 Operator layout matches the three-pane harness template

Given `/ops/harness` renders, when an operator opens the route, then the page exposes a dense three-window harness layout with dashboard, proof, self-improvement, and docked TUI companion regions.

### AC-2 Controls and status are usable, not raw fixture text

Given the harness has missions, gates, proposals, and benchmark state, when rendered, then mission status, verification gates, actions, benchmark proof source, and audit source have visible compact status affordances without changing their semantic data markers.

### AC-3 Responsive and overflow behavior is guarded

Given the route renders on narrower viewports or with wide table content, when layout contracts are inspected, then the harness has responsive collapse, table overflow containment, focus states, and fixed-density panel sizing markers.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | harness route context | render shell | panel carries mission-control design markers and window chrome |
| C-02 | AC-2 | Functional | default harness state | render shell | status chips, proof/audit markers, and action controls remain present |
| C-03 | AC-3 | Functional | rendered shell | inspect style contract | responsive, overflow, and focus rules exist |

## Success Metrics / Observable Signals

- Dashboard UI tests prove the design contract.
- Existing harness state-backed tests remain green.
- Browser screenshot smoke confirms the rendered route is non-blank and pane-based.
- No new dependency is introduced.
