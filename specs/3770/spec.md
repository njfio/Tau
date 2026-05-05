# Spec: Issue #3770 - Harness operator log first-screen placement

Status: Implemented

## Problem Statement

The mission proof view shows tool evidence early, but the live operator log renders below secondary proof details and is not visible in the first desktop viewport. Operators need the live log near tool evidence because it explains what the autonomous run is doing now.

## Scope

In scope:

- Promote the operator log directly after tool execution evidence in the proof grid.
- Add durable first-screen log priority markers.
- Keep tool evidence as the first proof artifact.
- Compact the operator log pre block so secondary proof detail remains reachable without excessive scroll.

Out of scope:

- Changing log content, timestamps, or runtime data.
- Changing TUI companion behavior.
- Reordering the left dashboard or right self-improvement panes.

## Acceptance Criteria

### AC-1 Operator log is first-screen proof context

Given `/ops/harness` renders, when proof DOM order is inspected, then operator log appears after tool evidence and before secondary acceptance, gate, and artifact sections.

### AC-2 Tool evidence remains first

Given `/ops/harness` renders, when proof DOM order is inspected, then tool evidence remains before the operator log and all secondary proof sections.

### AC-3 Log height is compact and testable

Given `/ops/harness` renders, when style markers are inspected, then the operator log exposes first-screen priority and compact max-height markers.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | proof pane | inspect DOM order | operator log appears after tool evidence and before secondary proof detail |
| C-02 | AC-2 | Functional | proof pane | inspect DOM order | tool evidence remains first proof artifact |
| C-03 | AC-3 | Functional | operator log | inspect style and data markers | first-screen priority and compact height markers are present |

## Success Metrics / Observable Signals

- Dashboard UI tests prove first-screen operator log placement and compact style markers.
- Browser geometry shows the operator log top is visible in the first 967px desktop viewport.
- Existing proof evidence and static preview route tests remain green.
- No new dependency is introduced.
