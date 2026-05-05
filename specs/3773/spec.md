# Spec: Issue #3773 - Harness TUI companion first-viewport fit

Status: Implemented

## Problem Statement

The harness TUI companion starts in the first desktop viewport, but its useful status output is clipped below the fold because the review pane consumes too much right-column height. Operators need the terminal companion to show the mission, status, call summary, and benchmark proof cue without scrolling.

## Scope

In scope:

- Add a first-viewport priority marker to the TUI companion.
- Reduce right-column review max height enough to leave room for the TUI companion.
- Make the TUI companion use a bounded box-model and compact preformatted output.
- Preserve existing TUI summary content and wrapping behavior.

Out of scope:

- Changing TUI summary text or command semantics.
- Changing dashboard, proof, benchmark, or self-improvement action ordering.
- Replacing the TUI companion with a different component.

## Acceptance Criteria

### AC-1 TUI companion is marked as first-viewport content

Given `/ops/harness` renders, when the TUI companion is inspected, then it exposes a first-viewport priority marker.

### AC-2 Right-column heights reserve room for the terminal companion

Given `/ops/harness` renders, when style markers are inspected, then the review pane and TUI companion use compact height bounds that fit together in the desktop viewport.

### AC-3 Terminal summary remains readable and wrapped

Given `/ops/harness` renders, when the TUI companion pre block is inspected, then it keeps the existing status summary content and uses compact wrapped output markers.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | TUI companion | inspect markup | first-viewport marker is present |
| C-02 | AC-2 | Functional | right-column panes | inspect style markers | review/TUI height bounds reserve space |
| C-03 | AC-3 | Functional | TUI companion pre | inspect content and CSS | status summary remains wrapped and compact |

## Success Metrics / Observable Signals

- Dashboard UI tests prove first-viewport TUI markers and compact height CSS exist.
- Browser geometry shows the TUI companion bottom is inside the 967px desktop viewport.
- Existing right-column action priority and harness layout tests remain green.
- No new dependency is introduced.
